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

use gtk::{Button, ButtonExt, ContainerExt, Label, WidgetExt, Window, WindowType};
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
    // TODO: automatically add this type?
    type Container = Window;
    // TODO: and this one too?
    type Model = Model;

    fn model() -> Model {
        Model {
            counter: 0,
        }
    }

    fn update(&mut self, event: Msg, model: &mut Model) {
        let label = &self.label1;

        match event {
            Decrement => {
                model.counter -= 1;
                // TODO: add this from the attribute.
                label.set_text(&model.counter.to_string());
            },
            Increment => {
                model.counter += 1;
                label.set_text(&model.counter.to_string());
            },
            Quit => gtk::main_quit(),
        }
    }

    // TODO: provide default parameter for constructor (like Toplevel).
    // TODO: think about conditions and loops (widget-list).
    view! {
        // TODO: guess if it is a GTK+ widget or Relm widget by looking at the connected events?
        // This is to avoid having to write gtk::.
        // It can be disambiguate if needed by writing gtk::.
        // TODO: Toplevel is the default, so it should not be necessary with g_object_new().
        // TODO: to avoid having a list of initial attributes, use the g_object_new() function by
        // specifying the properties as strings.
        // To be able to do so, g_object_new() needs to accept not construct parameters.
        // Check if it is the case.
        gtk::Window(WindowType::Toplevel) {
            gtk::Box(Vertical, 0) {
                gtk::Button {
                    clicked => Increment,
                    label: "+",
                },
                // TODO: use model.counter instead of 0.
                gtk::Label("0") {},
                gtk::Button {
                    clicked => Decrement,
                    label: "-",
                },
            },
            delete_event(_, _) => Quit,
        }
    }
}

fn main() {
    Relm::run::<Win>().unwrap();
}
