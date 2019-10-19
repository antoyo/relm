/*
 * Copyright (c) 2017-2019 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

//! Core primitive types for relm.
//!
//! The primary type is `EventStream`.

#![warn(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
)]

mod source;

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::mem;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SendError};

use glib::MainContext;
use slab::Slab;

static RUNNING: AtomicBool = AtomicBool::new(true);

/// Handle to a EventStream to emit messages.
pub struct StreamHandle<MSG> {
    stream: Weak<RefCell<_EventStream<MSG>>>,
}

impl<MSG> Clone for StreamHandle<MSG> {
    fn clone(&self) -> Self {
        Self {
            stream: self.stream.clone(),
        }
    }
}

impl<MSG> StreamHandle<MSG> {
    fn new(stream: Weak<RefCell<_EventStream<MSG>>>) -> Self {
        Self {
            stream: stream,
        }
    }

    /// Send the `event` message to the stream and the observers.
    pub fn emit(&self, msg: MSG) {
        if let Some(ref stream) = self.stream.upgrade() {
            emit(stream, msg);
        }
        else {
            panic!("Trying to call emit() on a dropped EventStream");
        }
    }

    /// Lock the stream (don't emit message) until the `Lock` goes out of scope.
    pub fn lock(&self) -> Lock<MSG> {
        if let Some(ref stream) = self.stream.upgrade() {
            stream.borrow_mut().locked = true;
            Lock {
                stream: stream.clone(),
            }
        }
        else {
            panic!("Trying to call lock() on a dropped EventStream");
        }
    }

    /// Add an observer to the event stream.
    /// This callback will be called every time a message is emmited.
    pub fn observe<CALLBACK: Fn(&MSG) + 'static>(&self, callback: CALLBACK) {
        if let Some(ref stream) = self.stream.upgrade() {
            stream.borrow_mut().observers.push(Rc::new(callback));
        }
        else {
            panic!("Trying to call observe() on a dropped EventStream");
        }
    }
}

/// A lock is used to temporarily stop emitting messages.
#[must_use]
pub struct Lock<MSG> {
    stream: Rc<RefCell<_EventStream<MSG>>>,
}

impl<MSG> Drop for Lock<MSG> {
    fn drop(&mut self) {
        self.stream.borrow_mut().locked = false;
    }
}

/// A wrapper over a `std::sync::mpsc::Sender` to wakeup the glib event loop when sending a
/// message.
pub struct Sender<MSG> {
    sender: mpsc::Sender<MSG>,
    wakeup_sender: glib::Sender<()>,
}

impl<MSG> Sender<MSG> {
    fn new(entry: Rc<Cell<usize>>, sender: mpsc::Sender<MSG>) -> Self {
        let (wakeup_sender, receiver) = MainContext::channel(glib::PRIORITY_DEFAULT);
        receiver.attach(None, move |_| {
            Loop::default().register(entry.get());
            MainContext::default().wakeup();
            glib::Continue(true)
        });
        Self {
            sender,
            wakeup_sender,
        }
    }
}

impl<MSG> Clone for Sender<MSG> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            wakeup_sender: self.wakeup_sender.clone(),
        }
    }
}

impl<MSG> Sender<MSG> {
    /// Send a message and wakeup the event loop.
    pub fn send(&self, msg: MSG) -> Result<(), SendError<MSG>> {
        let result = self.sender.send(msg);
        self.wakeup_sender.send(()).expect("wakeup sender");
        result
    }
}

/// A channel to send a message to a relm widget from another thread.
pub struct Channel<MSG> {
    callback: Box<dyn FnMut(MSG)>,
    entry: Rc<Cell<usize>>,
    receiver: Receiver<MSG>,
}

impl<MSG> Channel<MSG> {
    /// Create a new channel with a callback that will be called when a message is received.
    pub fn new<CALLBACK: FnMut(MSG) + 'static>(callback: CALLBACK) -> (Self, Sender<MSG>) {
        let (sender, receiver) = mpsc::channel();
        let entry = Rc::new(Cell::new(0));
        (Self {
            callback: Box::new(callback),
            entry: entry.clone(),
            receiver,
        }, Sender::new(entry, sender))
    }
}

impl<MSG> Handle for Channel<MSG> {
    fn handle(&mut self) {
        if let Ok(msg) = self.receiver.try_recv() {
            let callback = &mut self.callback;
            callback(msg);
        }
    }

    fn set_entry(&self, entry: usize) {
        self.entry.set(entry);
    }
}

struct _EventStream<MSG> {
    entry: usize,
    events: VecDeque<MSG>,
    locked: bool,
    observers: Vec<Rc<dyn Fn(&MSG)>>,
}

fn emit<MSG>(stream: &Rc<RefCell<_EventStream<MSG>>>, msg: MSG) {
    if !stream.borrow().locked {
        let len = stream.borrow().observers.len();
        for i in 0..len {
            let observer = stream.borrow().observers[i].clone();
            observer(&msg);
        }

        stream.borrow_mut().events.push_back(msg);
        Loop::default().register(stream.borrow().entry);
        MainContext::default().wakeup();
    }
}

/// A stream of messages to be used for widget/signal communication and inter-widget communication.
/// EventStream cannot be send to another thread. Use a `Channel` `Sender` instead.
pub struct EventStream<MSG> {
    callback: Option<Box<dyn FnMut(MSG)>>,
    stream: Rc<RefCell<_EventStream<MSG>>>,
}

impl<MSG> EventStream<MSG> {
    /// Create a new event stream.
    pub fn new() -> Self {
        let event_stream: _EventStream<MSG> = _EventStream {
            entry: 0,
            events: VecDeque::new(),
            locked: false,
            observers: vec![],
        };
        EventStream {
            callback: None,
            stream: Rc::new(RefCell::new(event_stream)),
        }
    }

    /// Create a Clone-able EventStream handle.
    pub fn downgrade(&self) -> StreamHandle<MSG> {
        StreamHandle::new(Rc::downgrade(&self.stream))
    }

    /// Send the `event` message to the stream and the observers.
    pub fn emit(&self, event: MSG) {
        emit(&self.stream, event);
    }

    /// Add an observer to the event stream.
    /// This callback will be called every time a message is emmited.
    pub fn observe<CALLBACK: Fn(&MSG) + 'static>(&self, callback: CALLBACK) {
        self.stream.borrow_mut().observers.push(Rc::new(callback));
    }

    /// Add a callback to the event stream.
    /// This is the main callback and received a owned version of the message, in contrast to
    /// observe().
    pub fn set_callback<CALLBACK: FnMut(MSG) + 'static>(&mut self, callback: CALLBACK) {
        self.callback = Some(Box::new(callback));
    }
}

pub trait Handle {
    fn handle(&mut self);
    fn set_entry(&self, entry: usize);
}

impl<MSG> Handle for EventStream<MSG> {
    fn handle(&mut self) {
        let event = self.stream.borrow_mut().events.pop_front();
        if let (Some(event), Some(callback)) = (event, self.callback.as_mut()) {
            callback(event);
        }
    }

    fn set_entry(&self, entry: usize) {
        self.stream.borrow_mut().entry = entry;
    }
}

/// Event loop wrapper to handle relm message system.
#[derive(Clone)]
pub struct Loop {
    context: MainContext,
    registered_entries: Rc<RefCell<Vec<usize>>>,
    streams: Rc<RefCell<Slab<Option<Box<dyn Handle>>>>>,
}

static mut MAIN_LOOP: usize = 0;

impl Loop {
    /// Get the default main loop.
    pub fn default() -> Self {
        if !gtk::is_initialized_main_thread() {
            if gtk::is_initialized() {
                panic!("GTK may only be used from the main thread.");
            }
            else {
                panic!("GTK has not been initialized. Call `gtk::init` first.");
            }
        }
        unsafe {
            if MAIN_LOOP == 0 {
                MAIN_LOOP = Box::into_raw(Box::new(Self::new())) as usize;
            }
            (&*(MAIN_LOOP as *const Self)).clone()
        }
    }

    fn new() -> Self {
        Self {
            context: MainContext::default(),
            registered_entries: Rc::new(RefCell::new(vec![])),
            streams: Rc::new(RefCell::new(Slab::new())),
        }
    }

    /// Add an EventStream to this event loop.
    /// The messages of every EventStream will be processed at each iteration.
    pub fn add_stream<HANDLE>(&self, stream: HANDLE) -> usize
    where HANDLE: Handle + 'static,
    {
        let mut streams = self.streams.borrow_mut();
        let entry = streams.vacant_entry();
        let key = entry.key();
        stream.set_entry(key);
        entry.insert(Some(Box::new(stream)));
        key
    }

    /// Check if any events are pending.
    pub fn events_pending(&self) -> bool {
        gtk::events_pending() || !self.registered_entries.borrow().is_empty()
    }

    /// Do one iteration of the event loop.
    pub fn iterate(&self, may_block: bool) {
        self.context.iteration(may_block);
        let registered_entries = mem::replace(&mut *self.registered_entries.borrow_mut(), vec![]);
        for entry in registered_entries {
            if let Some(ref stream) = self.streams.borrow().get(entry) {
                if stream.is_none() {
                    continue;
                }
            }
            else {
                continue;
            }
            let mut stream = mem::replace(&mut self.streams.borrow_mut()[entry], None);
            self.streams.borrow_mut()[entry] = None;
            if let Some(ref mut stream) = stream {
                stream.handle();
            }
            self.streams.borrow_mut()[entry] = stream;
        }
    }

    fn register(&self, entry: usize) {
        self.registered_entries.borrow_mut().push(entry);
    }

    /// Remove an event stream from the event loop.
    pub fn remove_stream(&self, entry: usize) {
        self.streams.borrow_mut().remove(entry);
    }

    /// Reserve an entry in the event loop.
    /// Call set_stream() with the returned entry afterwards.
    pub fn reserve(&self) -> usize {
        self.streams.borrow_mut().insert(None)
    }

    /// Set an EventStream at a specify entry.
    pub fn set_stream<HANDLE>(&self, entry: usize, stream: HANDLE)
    where HANDLE: Handle + 'static,
    {
        self.streams.borrow_mut()[entry] = Some(Box::new(stream));
    }

    /// End the main loop.
    pub fn quit() {
        RUNNING.store(false, Ordering::SeqCst);
        MainContext::default().wakeup();
    }

    pub(crate) fn run(&self) {
        while RUNNING.load(Ordering::SeqCst) {
            self.iterate(true);
        }
    }
}
