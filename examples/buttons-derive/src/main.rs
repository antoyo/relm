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

extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

use gtk::{
    ButtonExt,
    Inhibit,
    OrientableExt,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::Relm;

#[derive(Clone)]
struct Model {
    counter: i32,
}

#[derive(Msg)]
enum Msg {
    Decrement,
    Increment,
    Quit,
}

#[derive(Widget)]
struct _Win {
    widget: impl_widget! {
        fn model() -> Model {
            Model {
                counter: 0,
            }
        }

        fn update(&mut self, event: Msg, model: &mut Model) {
            match event {
                Msg::Decrement => model.counter -= 1,
                Msg::Increment => model.counter += 1,
                Msg::Quit => gtk::main_quit(),
            }
        }

        view! {
            gtk::Window {
                gtk::Box {
                    orientation: Vertical,
                    gtk::Button {
                        clicked => Msg::Increment,
                        label: "+",
                    },
                    gtk::Label {
                        text: &model.counter.to_string(),
                    },
                    gtk::Button {
                        clicked => Msg::Decrement,
                        label: "-",
                    },
                },
                delete_event(_, _) => (Msg::Quit, Inhibit(false)),
            }
        }
    }
}

fn main() {
    Relm::run::<_WinWidgets>().unwrap();
}
