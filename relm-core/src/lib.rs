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
#[macro_use]
extern crate lazy_static;
extern crate gtk;
extern crate tokio_core;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::Error;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

use futures::{Async, Future, Poll, Stream};
use futures::task::{self, Task};
use tokio_core::reactor;
pub use tokio_core::reactor::Handle;

pub struct Core {
    core: reactor::Core,
}

impl Core {
    pub fn new() -> Result<Self, Error> {
        Ok(Core {
            core: reactor::Core::new()?,
        })
    }

    pub fn handle(&self) -> Handle {
        self.core.handle()
    }

    pub fn run(&mut self) {
        while !QUITTED.load(Relaxed) {
            self.core.turn(Some(Duration::from_millis(10)));

            if gtk::events_pending() {
                gtk::main_iteration();
            }
        }
    }
}

lazy_static! {
    static ref QUITTED: AtomicBool = AtomicBool::new(false);
}

pub struct QuitFuture;

impl Future for QuitFuture {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        QUITTED.store(true, Relaxed);
        Ok(Async::Ready(()))
    }
}

struct _EventStream<T> {
    events: VecDeque<T>,
    observers: Vec<Box<Fn(T)>>,
    task: Option<Task>,
}

#[derive(Clone)]
pub struct EventStream<T> {
    stream: Rc<RefCell<_EventStream<T>>>,
}

impl<T: Clone> EventStream<T> {
    pub fn new() -> Self {
        EventStream {
            stream: Rc::new(RefCell::new(_EventStream {
                events: VecDeque::new(),
                observers: vec![],
                task: None,
            })),
        }
    }

    pub fn emit(&self, event: T) {
        if let Some(ref task) = self.stream.borrow().task {
            task.unpark();
        }
        self.stream.borrow_mut().events.push_back(event.clone());

        for observer in &self.stream.borrow().observers {
            observer(event.clone());
        }
    }

    fn get_event(&self) -> Option<T> {
        self.stream.borrow_mut().events.pop_front()
    }

    pub fn observe<F: Fn(T) + 'static>(&self, callback: F) {
        self.stream.borrow_mut().observers.push(Box::new(callback));
    }
}

impl<T: Clone> Stream for EventStream<T> {
    type Item = T;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.get_event() {
            Some(event) => {
                self.stream.borrow_mut().task = None;
                Ok(Async::Ready(Some(event)))
            },
            None => {
                self.stream.borrow_mut().task = Some(task::park());
                Ok(Async::NotReady)
            },
        }
    }
}
