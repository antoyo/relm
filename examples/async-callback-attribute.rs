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

// TODO: update to be on par with the example async-callback.

#![feature(proc_macro)]

extern crate futures;
extern crate futures_glib;
extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

use gtk::{
    ButtonExt,
    Inhibit,
    OrientableExt,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::{Resolver, Widget};
use relm_attributes::widget;

use self::Msg::*;

// Define the structure of the model.
pub struct Model {
    counter: i32,
}

// The messages that can be sent to the update function.
#[derive(Msg)]
pub enum Msg {
    Decrement,
    Delete(Resolver<Inhibit>),
    Increment,
}

#[widget]
impl Widget for Win {
    // The initial model.
    fn model() -> Model {
        Model {
            counter: 0,
        }
    }

    // Update the model according to the message received.
    fn update(&mut self, event: Msg) {
        match event {
            Decrement => self.model.counter -= 1,
            Delete(mut resolver) => resolver.resolve(Inhibit(false)),
            Increment => self.model.counter += 1,
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                // Set the orientation property of the Box.
                orientation: Vertical,
                // Create a Button inside the Box.
                gtk::Button {
                    // Send the message Increment when the button is clicked.
                    clicked => Increment,
                    label: "+",
                },
                gtk::Label {
                    // Bind the text property of the label to the counter attribute of the model.
                    text: &self.model.counter.to_string(),
                },
                gtk::Button {
                    clicked => Decrement,
                    label: "-",
                },
            },
            #[async] delete_event(_, _) => Delete,
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}
