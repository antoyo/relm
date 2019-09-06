/*
 * Copyright (c) 2018 Boucher, Antoni <bouanto@zoho.com>
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

use std::cell::RefCell;
use std::rc::Rc;

use relm::EventStream;

pub struct Observer<MSG> {
    result: Rc<RefCell<Option<MSG>>>,
}

impl<MSG: Clone + 'static> Observer<MSG> {
    pub fn new<F: Fn(&MSG) -> bool + 'static>(stream: &EventStream<MSG>, predicate: F) -> Self {
        let result = Rc::new(RefCell::new(None));
        let res = result.clone();
        stream.observe(move |msg| {
            if predicate(msg) {
                *res.borrow_mut() = Some(msg.clone());
            }
        });
        Self {
            result,
        }
    }

    pub fn wait(&self) -> MSG {
        loop {
            if let Ok(ref result) = self.result.try_borrow() {
                if result.is_some() {
                    break;
                }
            }
            gtk_test::run_loop();
        }
        self.result.borrow_mut().take()
            .expect("Message to take")
    }
}

#[macro_export]
macro_rules! relm_observer_new {
    ($component:expr, $pat:pat) => {
        $crate::Observer::new($component.stream(), |msg|
            if let $pat = msg {
                true
            }
            else {
                false
            }
        );
    };
}

#[macro_export]
macro_rules! relm_observer_wait {
    (let $($variant:ident)::*($name1:ident, $name2:ident $(,$rest:ident)*) = $observer:expr) => {
        let ($name1, $name2 $(, $rest)*) = {
            let msg = $observer.wait();
            if let $($variant)::*($name1, $name2 $(, $rest)*) = msg {
                ($name1, $name2 $(, $rest)*)
            }
            else {
                panic!("Wrong message type.");
            }
        };
    };
    (let $($variant:ident)::*($name:ident) = $observer:expr) => {
        let $name = {
            let msg = $observer.wait();
            if let $($variant)::*($name) = msg {
                $name
            }
            else {
                panic!("Wrong message type.");
            }
        };
    };
    (let $($variant:ident)::* = $observer:expr) => {
        let () = {
            let msg = $observer.wait();
            if let $($variant)::* = msg {
                ()
            }
            else {
                panic!("Wrong message type.");
            }
        };
    };
}
