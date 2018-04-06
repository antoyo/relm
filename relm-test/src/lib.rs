extern crate glib_sys;
extern crate gtk;
extern crate relm_core;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use gtk::ButtonExt;
use relm_core::EventStream;

#[macro_export]
macro_rules! assert_text {
    ($widget:expr, $string:expr) => {
        assert_eq!($widget.get_text().unwrap(), $string.to_string());
    };
}

/// Simulate a click on a button.
pub fn click<B: ButtonExt>(button: &B) {
    // TODO: look at how this is implemented to support other widgets.
    button.clicked();
    run_loop();
}

pub fn run_loop() {
    unsafe { glib_sys::g_usleep(1000) };
    while gtk::events_pending() {
        gtk::main_iteration();
    }
}

pub struct Observer<MSG> {
    loop_running: Rc<AtomicBool>,
    result: Rc<RefCell<Option<MSG>>>,
}

impl<MSG: Clone + 'static> Observer<MSG> {
    pub fn new<F: Fn(&MSG) -> bool + 'static>(stream: &EventStream<MSG>, predicate: F) -> Self {
        let result = Rc::new(RefCell::new(None));
        let loop_running = Rc::new(AtomicBool::new(true));
        let res = result.clone();
        let running = loop_running.clone();
        stream.observe(move |msg| {
            if predicate(msg) {
                running.store(false, Ordering::SeqCst);
                *res.borrow_mut() = Some(msg.clone());
            }
        });
        Self {
            loop_running,
            result,
        }
    }

    pub fn wait(&self) -> MSG {
        while self.loop_running.load(Ordering::SeqCst) {
            run_loop();
        }
        self.result.borrow_mut().take()
            .expect("Message to take")
    }
}

#[macro_export]
macro_rules! observer_new {
    ($component:expr, $pat:pat) => {
        Observer::new($component.stream(), |msg|
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
macro_rules! observer_wait {
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
}
