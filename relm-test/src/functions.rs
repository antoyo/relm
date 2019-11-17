use std::cell::Cell;
use std::rc::Rc;

use enigo::{
    self,
    Enigo,
    KeyboardControllable,
    MouseButton,
    MouseControllable,
};
use gdk::{WindowExt, keyval_to_unicode};
use gdk::enums::key::{self, Key};
use gtk::{
    self,
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
};
use relm::Loop;

/// Simulate a click on a widget.
///
/// ## Warning!
///
/// Please note that the click will "fail" if the window isn't on top of all other windows (this
/// is a common issue on OSX). Don't forget to bring the button's window on top by using:
///
/// ```ignore
/// window.activate_focus();
/// ```
///
/// Example:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::{Button, ButtonExt};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let but = Button::new();
/// but.connect_clicked(|_| {
///     println!("clicked");
/// });
/// relm_test::click(&but);
/// # }
/// ```
pub fn click<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    wait_for_draw(widget, || {
        let allocation = widget.get_allocation();
        mouse_move(widget, allocation.width / 2, allocation.height / 2);
        let mut enigo = Enigo::new();
        enigo.mouse_click(MouseButton::Left);
        run_loop();
        //wait(500);
    });
}

/// Simulate a double-click on a widget.
///
/// ## Warning!
///
/// Please note that the double-click will "fail" if the window isn't on top of all other windows
/// (this is a common issue on OSX). Don't forget to bring the button's window on top by using:
///
/// ```ignore
/// window.activate_focus();
/// ```
///
/// Example:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::{FileChooserAction, FileChooserWidget, FileChooserExt};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let fcw = FileChooserWidget::new(FileChooserAction::Open);
/// fcw.connect_file_activated(|_| {
///     println!("double clicked");
/// });
/// relm_test::double_click(&fcw);
/// # }
/// ```
pub fn double_click<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    wait_for_draw(widget, || {
        let allocation = widget.get_allocation();
        mouse_move(widget, allocation.width / 2, allocation.height / 2);
        let mut enigo = Enigo::new();
        enigo.mouse_click(MouseButton::Left);
        run_loop();
        enigo.mouse_click(MouseButton::Left);
        run_loop();
        wait(500);
    });
}

/// Move the mouse relative to the widget position.
///
/// Example:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::Button;
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let but = Button::new();
/// relm_test::mouse_move(&but, 0, 0); // the mouse will be on the top-left corner of the button
/// # }
/// ```
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

/// Send a mouse press event to the given widget.
///
/// ## Warning!
///
/// Please note that the mouse-press event will "fail" if the window isn't on top of all other
/// windows (this is a common issue on OSX). Don't forget to bring the button's window on top
/// by using:
///
/// ```ignore
/// window.activate_focus();
/// ```
///
/// Example:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::{Entry, EntryExt};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let entry = Entry::new();
/// entry.connect_icon_press(|_, _, _| {
///     println!("pressed");
/// });
/// relm_test::mouse_press(&entry);
/// # }
/// ```
pub fn mouse_press<W: IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    wait_for_draw(widget, || {
        let allocation = widget.get_allocation();
        mouse_move(widget, allocation.width / 2, allocation.height / 2);
        let mut enigo = Enigo::new();
        enigo.mouse_down(MouseButton::Left);
        run_loop();
    });
}

/// Send a mouse release event to the given widget.
///
/// ## Warning!
///
/// Please note that the mouse-release event will "fail" if the window isn't on top of all other
/// windows (this is a common issue on OSX). Don't forget to bring the button's window on top
/// by using:
///
/// ```ignore
/// window.activate_focus();
/// ```
///
/// Example:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::{Entry, EntryExt};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let entry = Entry::new();
/// entry.connect_icon_release(|_, _, _| {
///     println!("released");
/// });
/// relm_test::mouse_release(&entry);
/// # }
/// ```
pub fn mouse_release<W: IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W) {
    wait_for_draw(widget, || {
        let allocation = widget.get_allocation();
        mouse_move(widget, allocation.width / 2, allocation.height / 2);
        let mut enigo = Enigo::new();
        enigo.mouse_up(MouseButton::Left);
        run_loop();
    });
}

/// Send a key event to the given widget.
///
/// ## Warning!
///
/// Please note that the enter-key event will "fail" if the window isn't on top of all other
/// windows (this is a common issue on OSX). Don't forget to bring the button's window on top
/// by using:
///
/// ```ignore
/// window.activate_focus();
/// ```
///
/// Example:
///
/// ```
/// extern crate gdk;
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::{Entry, EntryExt};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let entry = Entry::new();
/// entry.connect_preedit_changed(|_, _| {
///     println!("key entered");
/// });
/// relm_test::enter_key(&entry, gdk::enums::key::Agrave);
/// # }
/// ```
pub fn enter_key<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W, key: Key) {
    wait_for_draw(widget, || {
        focus(widget);
        let mut enigo = Enigo::new();
        enigo.key_click(gdk_key_to_enigo_key(key));
        run_loop();
    });
}

/// Send keys event to the given widget.
///
/// ## Warning!
///
/// Please note that the enter-key event will "fail" if the window isn't on top of all other
/// windows (this is a common issue on OSX). Don't forget to bring the button's window on top
/// by using:
///
/// ```ignore
/// window.activate_focus();
/// ```
///
/// Example:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::{Entry, EntryExt};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let entry = Entry::new();
/// entry.connect_preedit_changed(|_, _| {
///     println!("key entered");
/// });
/// relm_test::enter_keys(&entry, "A lot of keys!");
/// # }
/// ```
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

/// Returns the child element which has the given name.
///
/// Example:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::{Button, ContainerExt, WidgetExt, Window, WindowType};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let but = Button::new();
/// let w = Window::new(WindowType::Toplevel);
///
/// but.set_name("Button");
/// w.add(&but);
///
/// relm_test::find_child_by_name::<Button, Window>(&w, "Button").expect("failed to find child");
/// // Or even better:
/// let but: Button = relm_test::find_child_by_name(&w, "Button").expect("failed to find child");
/// # }
/// ```
pub fn find_child_by_name<C: IsA<Widget>, W: Clone + IsA<Object> + IsA<Widget>>(parent: &W, name: &str) -> Option<C> {
    find_widget_by_name(parent, name)
        .and_then(|widget| widget.downcast().ok())
}

/// Returns the child widget which has the given name.
///
/// Example:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::{Button, ContainerExt, WidgetExt, Window, WindowType};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let but = Button::new();
/// let w = Window::new(WindowType::Toplevel);
///
/// but.set_name("Button");
/// w.add(&but);
///
/// relm_test::find_widget_by_name(&w, "Button").unwrap();
/// # }
/// ```
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

/// Focus on the given widget.
///
/// Example:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::{Button, Inhibit, WidgetExt};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let but = Button::new();
///
/// but.connect_focus(|_, _| {
///     println!("focused!");
///     Inhibit(false)
/// });
/// relm_test::focus(&but);
/// # }
/// ```
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

/// Send a key press event to the given widget.
///
/// ## Warning!
///
/// Please note that the key-press event will "fail" if the window isn't on top of all other
/// windows (this is a common issue on OSX). Don't forget to bring the button's window on top
/// by using:
///
/// ```ignore
/// window.activate_focus();
/// ```
///
/// Example:
///
/// ```
/// extern crate gdk;
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::{Entry, Inhibit, WidgetExt};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let entry = Entry::new();
/// entry.connect_key_press_event(|_, _| {
///     println!("key pressed");
///     Inhibit(false)
/// });
/// relm_test::key_press(&entry, gdk::enums::key::Agrave);
/// # }
/// ```
pub fn key_press<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W, key: Key) {
    wait_for_draw(widget, || {
        focus(widget);
        let mut enigo = Enigo::new();
        enigo.key_down(gdk_key_to_enigo_key(key));
        run_loop();
    });
}

/// Send a key release event to the given widget.
///
/// ## Warning!
///
/// Please note that the key-release event will "fail" if the window isn't on top of all other
/// windows (this is a common issue on OSX). Don't forget to bring the button's window on top
/// by using:
///
/// ```ignore
/// window.activate_focus();
/// ```
///
/// Example:
///
/// ```
/// extern crate gdk;
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::{Entry, Inhibit, WidgetExt};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let entry = Entry::new();
/// entry.connect_key_release_event(|_, _| {
///     println!("key released");
///     Inhibit(false)
/// });
/// relm_test::key_release(&entry, gdk::enums::key::Agrave);
/// # }
/// ```
pub fn key_release<W: Clone + IsA<Object> + IsA<Widget> + WidgetExt>(widget: &W, key: Key) {
    wait_for_draw(widget, || {
        focus(widget);
        let mut enigo = Enigo::new();
        enigo.key_up(gdk_key_to_enigo_key(key));
        run_loop();
    });
}

/// Wait for events the specified amount the milliseconds.
///
/// Very convenient when you need GTK to update the UI to let it process some events.
///
/// Example:
///
/// ```
/// extern crate gtk;
/// extern crate relm_test;
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// relm_test::wait(1000); // wait for a second
/// # }
/// ```
pub fn wait(ms: u32) {
    let running = Rc::new(Cell::new(true));
    let run = running.clone();
    gtk::timeout_add(ms, move || {
        run.set(false);
        Continue(false)
    });
    let event_loop = Loop::default();
    while running.get() {
        event_loop.iterate(false);
    }
}

/// Process all pending events and then return.
///
/// This function is called in all functions related to events handling (like `key_release` for
/// example).
///
/// Example:
///
/// ```
/// extern crate gtk;
/// extern crate relm_test;
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// relm_test::run_loop();
/// # }
/// ```
pub fn run_loop() {
    let event_loop = Loop::default();
    while event_loop.events_pending() {
        event_loop.iterate(false);
    }
}

/// Wait until the given condition returns `true`.
///
/// Example:
///
/// ```
/// extern crate gtk;
/// extern crate relm_test;
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let mut x = 0;
///
/// relm_test::wait_until_done(move || {
///     x += 1;
///     x > 10
/// });
/// # }
/// ```
pub fn wait_until_done<F: FnMut() -> bool>(mut f: F) {
    while !f() {
        run_loop();
    }
}

/// Wait for a widget to be drawn.
///
/// Example:
///
/// ```
/// extern crate gtk;
/// extern crate relm_test;
///
/// use gtk::{WidgetExt, Window, WindowType};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let mut w = Window::new(WindowType::Toplevel);
///
/// w.show_all();
/// relm_test::wait_for_draw(&w, || {
///     println!("drawn!");
/// });
/// # }
/// ```
pub fn wait_for_draw<W, F: FnOnce()>(widget: &W, callback: F)
where W: IsA<Object> + IsA<Widget> + WidgetExt {
    if widget.get_ancestor(Window::static_type()).is_none() {
        return;
    }

    // NOTE: this tick callback technique to wait for a widget to be drawn comes from the
    // implementation of gtk_test_widget_wait_for_draw.
    let running = Rc::new(Cell::new(true));
    let run = running.clone();
    widget.add_tick_callback(move |_, _| {
        run.set(false);
        false
    });
    let event_loop = Loop::default();
    while running.get() {
        event_loop.iterate(false);
    }
    callback();
}

fn gdk_key_to_enigo_key(key: Key) -> enigo::Key {
    use enigo::Key::*;
    match key {
        key::Return => Return,
        key::Tab => Tab,
        key::space => Space,
        key::BackSpace => Backspace,
        key::Escape => Escape,
        key::Super_L | key::Super_R => Meta,
        key::Control_L | key::Control_R => Control,
        key::Shift_L | key::Shift_R => Shift,
        key::Shift_Lock => CapsLock,
        key::Alt_L | key::Alt_R => Alt,
        key::Option => Option,
        key::End => End,
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
