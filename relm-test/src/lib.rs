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

// TODO: still depend on gtk-test?

#![warn(missing_docs)]

//! Crate to test UI interactions with [gtk-rs] crates.
//!
//! [gtk-rs]: https://gtk-rs.org
//!
//! Small example:
//!
//! ```
//! extern crate gtk;
//! #[macro_use]
//! extern crate relm_test;
//!
//! use gtk::{ButtonExt, ContainerExt, GtkWindowExt, LabelExt, WidgetExt};
//!
//! # fn main() {
//! gtk::init().expect("GTK init failed");
//!
//! let win = gtk::Window::new(gtk::WindowType::Toplevel);
//! let but = gtk::Button::new();
//!
//! but.set_label(""); // Otherwise, assert_label! call will fail.
//! but.connect_clicked(|b| {
//!     b.set_label("clicked!");
//! });
//!
//! win.add(&but);
//! win.show_all();
//! win.activate_focus(); // Very important, otherwise tests will fail on OSX!
//!
//! assert_label!(but, "");
//! relm_test::click(&but);
//! relm_test::wait(1000); // To be sure that GTK has updated the label's text.
//! assert_label!(but, "clicked!");
//! # }
//! ```

extern crate enigo;
extern crate gdk;
extern crate gtk;
extern crate relm;

mod macros;

mod functions;
mod observer;

pub use functions::*;
pub use observer::{Observer, RelmObserver};
