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

extern crate crossbeam;
extern crate futures;
#[macro_use]
extern crate lazy_static;
extern crate gtk;
extern crate tokio_core;

use std::cell::RefCell;
use std::io::Error;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

use crossbeam::sync::MsQueue;
use futures::{Async, Future, Poll, Stream};
use futures::task::{self, Task};
use tokio_core::reactor;
pub use tokio_core::reactor::Handle;

pub struct Core<M> {
    core: reactor::Core,
    stream: EventStream<M>,
}

impl<M> Core<M> {
    pub fn new() -> Result<Self, Error> {
        Ok(Core {
            core: reactor::Core::new()?,
            stream: EventStream::new(),
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

    pub fn stream(&self) -> &EventStream<M> {
        &self.stream
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

#[derive(Clone)]
pub struct EventStream<T> {
    events: Rc<MsQueue<T>>,
    task: Rc<RefCell<Option<Task>>>,
}

impl<T> EventStream<T> {
    fn new() -> Self {
        EventStream {
            events: Rc::new(MsQueue::new()),
            task: Rc::new(RefCell::new(None)),
        }
    }

    pub fn emit(&self, event: T) {
        if let Some(ref task) = *self.task.borrow() {
            task.unpark();
        }
        self.events.push(event);
    }

    fn get_event(&self) -> Option<T> {
        self.events.try_pop()
    }
}

impl<T> Stream for EventStream<T> {
    type Item = T;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.get_event() {
            Some(event) => {
                *self.task.borrow_mut() = None;
                Ok(Async::Ready(Some(event)))
            },
            None => {
                *self.task.borrow_mut() = Some(task::park());
                Ok(Async::NotReady)
            },
        }
    }
}
