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

extern crate gdk;
extern crate gdk_sys;
extern crate glib;
extern crate glib_sys;
extern crate gtk;
extern crate relm_core;

use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use gdk::{
    EventButton,
    EventKey,
    EventMotion,
    unicode_to_keyval,
};
use gdk::enums::key::Key;
use gdk_sys::{
    GdkEventButton,
    GdkEventKey,
    GdkEventMotion,
    GDK_BUTTON_PRIMARY,
    GDK_BUTTON_PRESS,
    GDK_BUTTON_RELEASE,
    GDK_KEY_PRESS,
    GDK_KEY_RELEASE,
    GDK_MOTION_NOTIFY,
};
use glib::translate::{FromGlibPtrFull, ToGlibPtr};
use gtk::{
    Button,
    ButtonExt,
    Cast,
    Continue,
    Entry,
    EntryExt,
    IsA,
    MenuItem,
    MenuItemExt,
    Object,
    ToolButton,
    ToolButtonExt,
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

/// Simulate a click on a widget.
pub fn click<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    if let Ok(menu_item) = widget.clone().dynamic_cast::<MenuItem>() {
        menu_item.emit_activate();
    }
    else {
        mouse_press(widget);
        mouse_release(widget);
        if let Ok(button) = widget.clone().dynamic_cast::<Button>() {
            button.clicked();
        }
        else if let Ok(tool_button) = widget.clone().dynamic_cast::<ToolButton>() {
            tool_button.emit_clicked();
        }
    }
    run_loop();
}

pub fn mouse_move<W: IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W, x: u32, y: u32) {
    let mut event: GdkEventMotion = unsafe { mem::zeroed() };
    event.type_ = GDK_MOTION_NOTIFY;
    if let Some(window) = widget.get_window() {
        event.window = window.to_glib_none().0;
    }
    event.send_event = 1;
    event.x_root = x as f64;
    event.y_root = y as f64;
    let mut event: EventMotion = unsafe { FromGlibPtrFull::from_glib_full(&mut event as *mut _) };
    propagate_event(widget, &mut event);
    run_loop();
    mem::forget(event); // The event is allocated on the stack, hence we don't want to free it.
}

pub fn mouse_press<W: IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    let mut event: GdkEventButton = unsafe { mem::zeroed() };
    event.type_ = GDK_BUTTON_PRESS;
    if let Some(window) = widget.get_window() {
        event.window = window.to_glib_none().0;
    }
    event.send_event = 1;
    event.button = GDK_BUTTON_PRIMARY as u32;
    let mut event: EventButton = unsafe { FromGlibPtrFull::from_glib_full(&mut event as *mut _) };
    propagate_event(widget, &mut event);
    run_loop();
    mem::forget(event); // The event is allocated on the stack, hence we don't want to free it.
}

pub fn mouse_release<W: IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    let mut event: GdkEventButton = unsafe { mem::zeroed() };
    event.type_ = GDK_BUTTON_RELEASE;
    if let Some(window) = widget.get_window() {
        event.window = window.to_glib_none().0;
    }
    event.send_event = 1;
    event.button = GDK_BUTTON_PRIMARY as u32;
    let mut event: EventButton = unsafe { FromGlibPtrFull::from_glib_full(&mut event as *mut _) };
    propagate_event(widget, &mut event);
    run_loop();
    mem::forget(event); // The event is allocated on the stack, hence we don't want to free it.
}

pub fn enter_key<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W, key: Key) {
    key_press(widget, key);
    key_release(widget, key);
}

pub fn enter_keys<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W, text: &str) {
    for char in text.chars() {
        let key = unicode_to_keyval(char as u32);
        enter_key(widget, key);
    }
}

pub fn focus<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    if let Ok(entry) = widget.clone().dynamic_cast::<Entry>() {
        entry.grab_focus_without_selecting();
    }
    else {
        widget.grab_focus();
    }
}

pub fn key_press<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W, key: Key) {
    focus(widget);
    let mut event: GdkEventKey = unsafe { mem::zeroed() };
    event.type_ = GDK_KEY_PRESS;
    if let Some(window) = widget.get_window() {
        event.window = window.to_glib_none().0;
    }
    event.send_event = 1;
    event.keyval = key;
    let mut event: EventKey = unsafe { FromGlibPtrFull::from_glib_full(&mut event as *mut _) };
    propagate_event(widget, &mut event);
    run_loop();
    mem::forget(event); // The event is allocated on the stack, hence we don't want to free it.
}

pub fn key_release<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W, key: Key) {
    focus(widget);
    let mut event: GdkEventKey = unsafe { mem::zeroed() };
    event.type_ = GDK_KEY_RELEASE;
    if let Some(window) = widget.get_window() {
        event.window = window.to_glib_none().0;
    }
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
