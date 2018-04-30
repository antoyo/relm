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

extern crate enigo;
extern crate gdk;
extern crate gtk;
extern crate relm_core;

use std::cell::RefCell;
use std::rc::Rc;

use enigo::{
    Enigo,
    KeyboardControllable,
    MouseButton,
    MouseControllable,
};
use gdk::{WindowExt, keyval_to_unicode};
use gdk::enums::key::{self, Key};
use gtk::{
    Bin,
    BinExt,
    Cast,
    Container,
    ContainerExt,
    Continue,
    EditableExt,
    Entry,
    IsA,
    Object,
    StaticType,
    Widget,
    WidgetExt,
    Window,
    test_widget_wait_for_draw,
};
use relm_core::EventStream;

#[macro_export]
macro_rules! assert_label {
    ($widget:expr, $string:expr) => {
        assert_eq!($widget.get_label().expect("get label"), $string.to_string());
    };
}

#[macro_export]
macro_rules! assert_text {
    ($widget:expr, $string:expr) => {
        assert_eq!($widget.get_text().expect("get text"), $string.to_string());
    };
}

/// Simulate a click on a widget.
pub fn click<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    wait_for_draw(widget, || {
        let allocation = widget.get_allocation();
        mouse_move(widget, allocation.width / 2, allocation.height / 2);
        let mut enigo = Enigo::new();
        enigo.mouse_click(MouseButton::Left);
        run_loop();
    });
}

/// Simulate a double-click on a widget.
pub fn double_click<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    click(widget);
    click(widget);
}

/// Move the mouse relative to the widget position.
pub fn mouse_move<W: IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W, x: i32, y: i32) {
    wait_for_draw(widget, || {
        let toplevel_window = widget.get_toplevel().and_then(|toplevel| toplevel.get_window());
        if let (Some(toplevel), Some(toplevel_window)) = (widget.get_toplevel(), toplevel_window) {
            let (_, window_x, window_y) = toplevel_window.get_origin();
            if let Some((x, y)) = widget.translate_coordinates(&toplevel, x, y) {
                let x = window_x + x;
                let y = window_y + y;
                let mut enigo = Enigo::new();
                enigo.mouse_move_to(x, y);
                run_loop();
            }
        }
    });
}

pub fn mouse_press<W: IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    wait_for_draw(widget, || {
        let allocation = widget.get_allocation();
        mouse_move(widget, allocation.width / 2, allocation.height / 2);
        let mut enigo = Enigo::new();
        enigo.mouse_down(MouseButton::Left);
        run_loop();
    });
}

pub fn mouse_release<W: IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    wait_for_draw(widget, || {
        let allocation = widget.get_allocation();
        mouse_move(widget, allocation.width / 2, allocation.height / 2);
        let mut enigo = Enigo::new();
        enigo.mouse_up(MouseButton::Left);
        run_loop();
    });
}

pub fn enter_key<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W, key: Key) {
    wait_for_draw(widget, || {
        focus(widget);
        let mut enigo = Enigo::new();
        enigo.key_click(gdk_key_to_enigo_key(key));
        run_loop();
    });
}

pub fn enter_keys<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W, text: &str) {
    wait_for_draw(widget, || {
        focus(widget);
        let mut enigo = Enigo::new();
        for char in text.chars() {
            enigo.key_sequence(&char.to_string());
            run_loop();
        }
    });
}

pub fn find_child_by_name<C: IsA<Widget>, W: Clone + IsA<Object> + IsA<Widget>>(parent: &W, name: &str) -> Option<C> {
    find_widget_by_name(parent, name)
        .and_then(|widget| widget.downcast().ok())
}

pub fn find_widget_by_name<W: Clone + IsA<Object> + IsA<Widget>>(parent: &W, name: &str) -> Option<Widget> {
    if let Ok(container) = parent.clone().dynamic_cast::<Container>() {
        for child in container.get_children() {
            if let Some(string) = child.get_name() {
                if string == name {
                    return Some(child);
                }
            }
            if let Some(widget) = find_widget_by_name(&child, name) {
                return Some(widget);
            }
        }
    }
    else if let Ok(bin) = parent.clone().dynamic_cast::<Bin>() {
        if let Some(child) = bin.get_child() {
            if let Some(string) = child.get_name() {
                if string == name {
                    return Some(child);
                }
            }
            if let Some(widget) = find_widget_by_name(&child, name) {
                return Some(widget);
            }
        }
    }
    None
}

pub fn focus<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    wait_for_draw(widget, || {
        if !widget.has_focus() {
            widget.grab_focus();
            if let Ok(entry) = widget.clone().dynamic_cast::<Entry>() {
                // Hack to make it work on Travis.
                // Should use grab_focus_without_selecting() instead.
                entry.set_position(-1);
            }
        }
    });
}

pub fn key_press<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W, key: Key) {
    wait_for_draw(widget, || {
        focus(widget);
        let mut enigo = Enigo::new();
        enigo.key_down(gdk_key_to_enigo_key(key));
        run_loop();
    });
}

pub fn key_release<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W, key: Key) {
    wait_for_draw(widget, || {
        focus(widget);
        let mut enigo = Enigo::new();
        enigo.key_up(gdk_key_to_enigo_key(key));
        run_loop();
    });
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
    while gtk::events_pending() {
        gtk::main_iteration();
    }
}

pub fn wait_for_draw<W: IsA<Object> + IsA<Widget> + WidgetExt, F: FnOnce()>(widget: &W, callback: F) {
    if widget.get_ancestor(Window::static_type()).is_none() {
        return;
    }
    test_widget_wait_for_draw(widget);
    callback();
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

fn gdk_key_to_enigo_key(key: Key) -> enigo::Key {
    use enigo::Key::*;
    match key {
        key::Return => Return,
        key::Tab => Tab,
        key::space => Space,
        key::BackSpace => Backspace,
        key::Escape => Escape,
        key::Super_L | key::Super_R => Super,
        key::Control_L | key::Control_R => Control,
        key::Shift_L | key::Shift_R => Shift,
        key::Shift_Lock => CapsLock,
        key::Alt_L | key::Alt_R => Alt,
        key::Option => Option,
        key::Home => Home,
        key::Page_Down => PageDown,
        key::Page_Up => PageUp,
        key::leftarrow => LeftArrow,
        key::rightarrow => RightArrow,
        key::downarrow => DownArrow,
        key::uparrow => UpArrow,
        key::F1 => F1,
        key::F2 => F2,
        key::F3 => F3,
        key::F4 => F4,
        key::F5 => F5,
        key::F6 => F6,
        key::F7 => F7,
        key::F8 => F8,
        key::F9 => F9,
        key::F10 => F10,
        key::F11 => F11,
        key::F12 => F12,
        _ => {
            if let Some(char) = keyval_to_unicode(key) {
                Layout(char)
            }
            else {
                Raw(key as u16)
            }
        },
    }
}
