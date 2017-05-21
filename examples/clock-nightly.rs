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

#![feature(fn_traits, unboxed_closures)]

extern crate chrono;
extern crate futures_glib;
extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use chrono::Local;
use futures_glib::Interval;
use gtk::{ContainerExt, Inhibit, Label, WidgetExt, Window, WindowType};
use relm::{Relm, Update, Widget};

use self::Msg::*;

#[derive(SimpleMsg)]
enum Msg {
    Quit,
    Tick,
}

struct Win {
    label: Label,
    window: Window,
}

impl Update for Win {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> () {
        ()
    }

    fn subscriptions(relm: &Relm<Self>) {
        let stream = Interval::new(Duration::from_secs(1));
        relm.connect_exec_ignore_err(stream, Tick);
    }

    fn update(&mut self, event: Msg) {
        match event {
            Tick => {
                let time = Local::now();
                self.label.set_text(&format!("{}", time.format("%H:%M:%S")));
            },
            Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, _model: Self::Model) -> Rc<RefCell<Self>> {
        let label = Label::new(None);

        let window = Window::new(WindowType::Toplevel);

        window.add(&label);

        window.show_all();

        connect!(relm, window, connect_delete_event(_, _), return (Some(Quit), Inhibit(false)));

        let mut win = Win {
            label: label,
            window: window,
        };
        win.update(Tick);
        Rc::new(RefCell::new(win))
    }
}

fn main() {
    Win::run(()).unwrap();
}
