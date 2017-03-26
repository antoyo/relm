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

#![feature(proc_macro)]

extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

use gtk::{
    Button,
    ButtonExt,
    ContainerExt,
    Inhibit,
    Label,
    OrientableExt,
    WidgetExt,
    Window,
};
use gtk::Orientation::Vertical;
use relm::{Relm, RemoteRelm, Widget};
use relm_attributes::widget;

use self::Msg::*;

#[derive(Clone)]
struct Model {
    counter: i32,
}

// TODO: does an attribute #[msg] would simplify the implementation?
#[derive(Msg)]
enum Msg {
    Decrement,
    Increment,
    Quit,
}

// TODO: automatically generate this struct?
struct Win {
    box1: gtk::Box,
    button1: Button,
    button2: Button,
    label1: Label,
    window1: Window,
}

#[widget]
impl Widget<Msg> for Win {
    fn model() -> Model {
        Model {
            counter: 0,
        }
    }

    fn update(&mut self, event: Msg, model: &mut Model) {
        match event {
            Decrement => model.counter -= 1,
            Increment => model.counter += 1,
            Quit => gtk::main_quit(),
        }
    }

    view! {
        Window {
            gtk::Box {
                orientation: Vertical,
                Button {
                    clicked => Increment,
                    label: "+",
                },
                Label {
                    text: &model.counter.to_string(),
                },
                Button {
                    clicked => Decrement,
                    label: "-",
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Relm::run::<Win>().unwrap();
}
