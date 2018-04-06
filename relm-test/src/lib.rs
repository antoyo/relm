extern crate gdk;
extern crate gdk_sys;
extern crate glib;
extern crate glib_sys;
extern crate gtk;
extern crate relm_core;

use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use gdk::EventKey;
use gdk::enums::key::Key;
use gdk_sys::{GdkEventKey, GDK_KEY_PRESS, GDK_KEY_RELEASE};
use glib::translate::{FromGlibPtrFull, ToGlibPtr};
use gtk::{
    ButtonExt,
    Continue,
    IsA,
    Widget,
    WidgetExt,
    propagate_event,
};
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

pub fn key_press<W: IsA<Widget> + WidgetExt>(widget: &W, key: Key) {
    let mut event: GdkEventKey = unsafe { mem::zeroed() };
    event.type_ = GDK_KEY_PRESS;
    event.window = widget.get_window().expect("window").to_glib_none().0;
    event.send_event = 1;
    event.keyval = key;
    let mut event: EventKey = unsafe { FromGlibPtrFull::from_glib_full(&mut event as *mut _) };
    propagate_event(widget, &mut event);
    run_loop();
    mem::forget(event); // The event is allocated on the stack, hence we don't want to free it.
}

pub fn key_release<W: IsA<Widget> + WidgetExt>(widget: &W, key: Key) {
    let mut event: GdkEventKey = unsafe { mem::zeroed() };
    event.type_ = GDK_KEY_RELEASE;
    event.window = widget.get_window().expect("window").to_glib_none().0;
    event.send_event = 1;
    event.keyval = key;
    let mut event: EventKey = unsafe { FromGlibPtrFull::from_glib_full(&mut event as *mut _) };
    propagate_event(widget, &mut event);
    run_loop();
    mem::forget(event); // The event is allocated on the stack, hence we don't want to free it.
}

/// Wait for events the specified amount the milliseconds.
pub fn wait(ms: u32) {
    gtk::timeout_add(ms, || {
        gtk::main_quit();
        Continue(false)
    });
    gtk::main();
}

pub fn run_loop() {
    unsafe { glib_sys::g_usleep(1000) };
    while gtk::events_pending() {
        gtk::main_iteration();
    }
}

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
