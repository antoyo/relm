/*
 * Copyright (c) 2017-2020 Boucher, Antoni <bouanto@zoho.com>
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

use std::cell::RefCell;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};
use std::sync::mpsc::{self, Receiver, SendError};

use self::source::{SourceFuncs, new_source, source_get};

use glib::{
    MainContext,
    Source,
    SourceId,
};

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

struct ChannelData<MSG> {
    callback: Box<dyn FnMut(MSG)>,
    peeked_value: Option<MSG>,
    receiver: Receiver<MSG>,
}

/// A wrapper over a `std::sync::mpsc::Sender` to wakeup the glib event loop when sending a
/// message.
pub struct Sender<MSG> {
    sender: mpsc::Sender<MSG>,
}

impl<MSG> Clone for Sender<MSG> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<MSG> Sender<MSG> {
    /// Send a message and wakeup the event loop.
    pub fn send(&self, msg: MSG) -> Result<(), SendError<MSG>> {
        let result = self.sender.send(msg);
        let context = MainContext::default();
        context.wakeup();
        result
    }
}

/// A channel to send a message to a relm widget from another thread.
pub struct Channel<MSG> {
    _source: Source,
    _phantom: PhantomData<MSG>,
}

impl<MSG> Channel<MSG> {
    /// Create a new channel with a callback that will be called when a message is received.
    pub fn new<CALLBACK: FnMut(MSG) + 'static>(callback: CALLBACK) -> (Self, Sender<MSG>) {
        let (sender, receiver) = mpsc::channel();
        let source = new_source(RefCell::new(ChannelData {
            callback: Box::new(callback),
            peeked_value: None,
            receiver,
        }));
        let main_context = MainContext::default();
        source.attach(Some(&main_context));
        (Self {
            _source: source,
            _phantom: PhantomData,
        }, Sender {
            sender,
        })
    }
}

impl<MSG> SourceFuncs for RefCell<ChannelData<MSG>> {
    fn dispatch(&self) -> bool {
        // TODO: show errors.
        let msg = self.borrow_mut().peeked_value.take().or_else(|| {
            self.borrow().receiver.try_recv().ok()
        });
        if let Some(msg) = msg {
            let callback = &mut self.borrow_mut().callback;
            callback(msg);
        }
        true
    }

    fn prepare(&self) -> (bool, Option<u32>) {
        if self.borrow().peeked_value.is_some() {
            return (true, None);
        }
        let peek_val = self.borrow().receiver.try_recv().ok();
        self.borrow_mut().peeked_value = peek_val;
        (self.borrow().peeked_value.is_some(), None)
    }

}

struct _EventStream<MSG> {
    events: VecDeque<MSG>,
    locked: bool,
    observers: Vec<Rc<dyn Fn(&MSG)>>,
}

impl<MSG> SourceFuncs for SourceData<MSG> {
    fn dispatch(&self) -> bool {
        let event = self.stream.borrow_mut().events.pop_front();
        if let (Some(event), Some(callback)) = (event, self.callback.borrow_mut().as_mut()) {
            callback(event);
        }
        true
    }

    fn prepare(&self) -> (bool, Option<u32>) {
        (!self.stream.borrow().events.is_empty(), None)
    }

}

struct SourceData<MSG> {
    callback: Rc<RefCell<Option<Box<dyn FnMut(MSG)>>>>,
    stream: Rc<RefCell<_EventStream<MSG>>>,
}

impl<MSG> Drop for SourceData<MSG> {
    fn drop(&mut self) {
        println!("Drop SourceData");
    }
}

fn emit<MSG>(stream: &Rc<RefCell<_EventStream<MSG>>>, msg: MSG) {
    if !stream.borrow().locked {
        let len = stream.borrow().observers.len();
        for i in 0..len {
            let observer = stream.borrow().observers[i].clone();
            observer(&msg);
        }

        stream.borrow_mut().events.push_back(msg);
    }
}

/// A stream of messages to be used for widget/signal communication and inter-widget communication.
/// EventStream cannot be send to another thread. Use a `Channel` `Sender` instead.
pub struct EventStream<MSG> {
    source: Source,
    source_id: Option<SourceId>,
    _phantom: PhantomData<*mut MSG>,
}

/*impl<MSG> Clone for EventStream<MSG> {
    fn clone(&self) -> Self {
        EventStream {
            source_id: self.source_id,
            source: self.source.clone(),
            _phantom: PhantomData,
        }
    }
}*/

trait SourceDbg {
    fn print_ref_count(&self);
}

impl SourceDbg for Source {
    fn print_ref_count(&self) {
        unsafe {
            println!("Ref count: {}", (*self.to_glib_none().0).ref_count);
        }
    }
}

use glib::translate::ToGlibPtr;

impl<MSG> Drop for EventStream<MSG> {
    fn drop(&mut self) {
        println!("Drop EventStream");
        self.source.print_ref_count();
        Source::remove(self.source_id.take().expect("source id"));
        self.source.print_ref_count();
        self.close();
        self.source.print_ref_count();
    }
}


impl<MSG> EventStream<MSG> {
    fn get_callback(&self) -> Rc<RefCell<Option<Box<dyn FnMut(MSG)>>>> {
        source_get::<SourceData<MSG>>(&self.source).callback.clone()
    }

    fn get_stream(&self) -> Rc<RefCell<_EventStream<MSG>>> {
        source_get::<SourceData<MSG>>(&self.source).stream.clone()
    }
}

impl<MSG> EventStream<MSG> {
    /// Create a new event stream.
    pub fn new() -> Self {
        let event_stream: _EventStream<MSG> = _EventStream {
            events: VecDeque::new(),
            locked: false,
            observers: vec![],
        };
        let source = new_source(SourceData {
            callback: Rc::new(RefCell::new(None)),
            stream: Rc::new(RefCell::new(event_stream)),
        });
        let main_context = MainContext::default();
        let source_id = Some(source.attach(Some(&main_context)));
        EventStream {
            source,
            source_id,
            _phantom: PhantomData,
        }
    }

    /// Close the event stream, i.e. stop processing messages.
    pub fn close(&self) {
        println!("Destroy");
        self.source.destroy();
    }

    /// Create a Clone-able EventStream handle.
    pub fn downgrade(&self) -> StreamHandle<MSG> {
        StreamHandle::new(Rc::downgrade(&self.get_stream()))
    }

    /// Send the `event` message to the stream and the observers.
    pub fn emit(&self, event: MSG) {
        let stream = self.get_stream();
        if !stream.borrow().locked {
            let len = stream.borrow().observers.len();
            for i in 0..len {
                let observer = stream.borrow().observers[i].clone();
                observer(&event);
            }

            stream.borrow_mut().events.push_back(event);
        }
    }

    /// Lock the stream (don't emit message) until the `Lock` goes out of scope.
    pub fn lock(&self) -> Lock<MSG> {
        let stream = self.get_stream();
        stream.borrow_mut().locked = true;
        Lock {
            stream: self.get_stream().clone(),
        }
    }

    /// Add an observer to the event stream.
    /// This callback will be called every time a message is emmited.
    pub fn observe<CALLBACK: Fn(&MSG) + 'static>(&self, callback: CALLBACK) {
        let stream = self.get_stream();
        stream.borrow_mut().observers.push(Rc::new(callback));
    }

    /// Add a callback to the event stream.
    /// This is the main callback and received a owned version of the message, in contrast to
    /// observe().
    pub fn set_callback<CALLBACK: FnMut(MSG) + 'static>(&self, callback: CALLBACK) {
        let source_callback = self.get_callback();
        *source_callback.borrow_mut() = Some(Box::new(callback));
    }
}
