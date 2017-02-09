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
extern crate gtk;
extern crate tokio_core;

use std::cell::{Cell, RefCell};
use std::io::Error;
use std::rc::Rc;
use std::time::Duration;

use crossbeam::sync::MsQueue;
use futures::{Async, Future, Poll, Stream};
use futures::task::{self, Task};
use tokio_core::reactor::{self, Handle};

pub struct Core<M, W> {
    core: reactor::Core,
    quit_future: QuitFuture,
    stream: EventStream<M, Rc<W>>,
}

impl<M, W> Core<M, W> {
    pub fn new(widgets: W) -> Result<Self, Error> {
        Ok(Core {
            core: reactor::Core::new()?,
            quit_future: QuitFuture::new(),
            stream: EventStream::new(Rc::new(widgets)),
        })
    }

    pub fn handle(&self) -> Handle {
        self.core.handle()
    }

    pub fn quit_future(&self) -> &QuitFuture {
        &self.quit_future
    }

    pub fn run(&mut self) {
        while self.quit_future.poll() == Ok(Async::NotReady) {
            self.core.turn(Some(Duration::from_millis(10)));

            if gtk::events_pending() {
                gtk::main_iteration();
            }
        }
    }

    pub fn stream(&self) -> &EventStream<M, Rc<W>> {
        &self.stream
    }
}

#[derive(Clone)]
pub struct QuitFuture {
    quitted: Rc<Cell<bool>>,
}

impl QuitFuture {
    fn new() -> Self {
        QuitFuture {
            quitted: Rc::new(Cell::new(false)),
        }
    }

    pub fn quit(&self) {
        self.quitted.set(true);
    }
}

impl Future for QuitFuture {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if self.quitted.get() {
            Ok(Async::Ready(()))
        }
        else {
            Ok(Async::NotReady)
        }
    }
}

#[derive(Clone)]
pub struct EventStream<T, W: Clone> {
    events: Rc<MsQueue<T>>,
    task: Rc<RefCell<Option<Task>>>,
    widgets: W,
}

impl<T, W: Clone> EventStream<T, W> {
    fn new(widgets: W) -> Self {
        EventStream {
            events: Rc::new(MsQueue::new()),
            task: Rc::new(RefCell::new(None)),
            widgets: widgets,
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

impl<T, W: Clone> Stream for EventStream<T, W> {
    type Item = (T, W);
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.get_event() {
            Some(event) => {
                *self.task.borrow_mut() = None;
                Ok(Async::Ready(Some((event, self.widgets.clone()))))
            },
            None => {
                *self.task.borrow_mut() = Some(task::park());
                Ok(Async::NotReady)
            },
        }
    }
}
