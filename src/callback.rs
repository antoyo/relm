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

//! Utility to transform a synchronous callback into an asynchronous one.

use std::cell::Cell;

use futures::{Async, AsyncSink, Sink};
use futures::unsync::mpsc::{Receiver, Sender};
use futures_glib::MainLoop;

macro_rules! unexpected_error {
    ($data:expr) => {
        panic!("Unexpected error while sending the default return value of the callback: {:?}", $data)
    };
}

/// Struct use to set the return value of a synchronous callback.
pub struct Resolver<T: Default> {
    lp: MainLoop,
    send_done: Cell<bool>,
    tx: Sender<T>,
}

impl<T: Default> Drop for Resolver<T> {
    fn drop(&mut self) {
        if !self.send_done.get() {
            self.send(Default::default());
        }
    }
}

impl<T: Default> Resolver<T> {
    #[doc(hidden)]
    pub fn channel(lp: MainLoop) -> (Self, Receiver<T>) {
        let (tx, rx) = ::futures::unsync::mpsc::channel(1);
        (Resolver::new(lp, tx), rx)
    }

    fn new(lp: MainLoop, tx: Sender<T>) -> Self {
        Resolver {
            lp,
            send_done: Cell::new(false),
            tx,
        }
    }

    /// Duplicate a channel.
    pub fn dup(&self) -> Self {
        self.send_done.set(true);
        Resolver::new(self.lp.clone(), self.tx.clone())
    }

    /// Set the return value of a synchronous callback in an asynchronous fashion.
    pub fn resolve(&mut self, value: T) {
        self.send(value);
    }

    fn send(&mut self, value: T) {
        if !self.send_done.get() {
            match self.tx.start_send(value) {
                Ok(AsyncSink::Ready) => {
                    match self.tx.poll_complete() {
                        Ok(result) => assert_eq!(result, Async::Ready(())),
                        Err(error) => unexpected_error!(error),
                    }
                },
                Ok(AsyncSink::NotReady(_)) => unexpected_error!("not ready"),
                Err(error) => unexpected_error!(error),
            }
            self.send_done.set(true);
            self.lp.quit();
        }
        else {
            warn!("Attempted to resolve a value for the second time on this Resolver");
        }
    }
}
