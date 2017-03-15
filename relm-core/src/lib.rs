/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
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

extern crate futures;
extern crate glib_itc;
extern crate gtk;
extern crate tokio_core;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;

use futures::{Async, IntoFuture, Poll, Stream};
use futures::task::{self, Task};
use glib_itc::Sender;
use tokio_core::reactor::{self, Handle};
pub use tokio_core::reactor::Remote;

pub struct Core { }

impl Core {
    pub fn run<F, R>(function: F)
        where F: FnOnce(&Handle) -> R + Send + 'static,
              R: IntoFuture<Item=(), Error=()>,
              R::Future: 'static,
    {
        let (sender, receiver) = channel();
        thread::spawn(move || {
            let mut core = reactor::Core::new().unwrap();
            sender.send(core.remote()).unwrap();
            loop {
                core.turn(None);
            }
        });

        let remote = receiver.recv().unwrap();
        remote.spawn(function);

        gtk::main();
    }
}

struct _EventStream<T> {
    events: VecDeque<T>,
    observers: Vec<Box<Fn(T) + Send>>,
    sender: Arc<Sender>,
    task: Option<Task>,
    ui_events: VecDeque<T>,
}

#[derive(Clone)]
pub struct EventStream<T> {
    stream: Arc<Mutex<_EventStream<T>>>,
}

impl<T: Clone + 'static> EventStream<T> {
    pub fn new(sender: Arc<Sender>) -> Self {
        EventStream {
            stream: Arc::new(Mutex::new(_EventStream {
                events: VecDeque::new(),
                observers: vec![],
                sender: sender,
                task: None,
                ui_events: VecDeque::new(),
            })),
        }
    }

    pub fn emit(&self, event: T) {
        let mut stream = self.stream.lock().unwrap();
        if let Some(ref task) = stream.task {
            task.unpark();
        }
        // TODO: try to avoid clone by sending a reference.
        stream.events.push_back(event.clone());

        for observer in &stream.observers {
            observer(event.clone());
        }
    }

    fn get_event(&self) -> Option<T> {
        self.stream.lock().unwrap().events.pop_front()
    }

    pub fn observe<F: Fn(T) + Send + 'static>(&self, callback: F) {
        self.stream.lock().unwrap().observers.push(Box::new(callback));
    }

    pub fn pop_ui_events(&self) -> Option<T> {
        self.stream.lock().unwrap().ui_events.pop_front()
    }
}

impl<T: Clone + 'static> Stream for EventStream<T> {
    type Item = T;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.get_event() {
            Some(event) => {
                let mut stream = self.stream.lock().unwrap();
                stream.task = None;
                stream.ui_events.push_back(event.clone());
                stream.sender.send();
                // TODO: try to avoid clone by sending a reference.
                Ok(Async::Ready(Some(event)))
            },
            None => {
                let mut stream = self.stream.lock().unwrap();
                stream.task = Some(task::park());
                Ok(Async::NotReady)
            },
        }
    }
}
