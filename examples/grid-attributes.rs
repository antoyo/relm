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
    ButtonExt,
    GridExt,
    Inhibit,
    LabelExt,
    WidgetExt,
};
use relm::Widget;
use relm_attributes::widget;

use self::Msg::*;

#[derive(Msg)]
pub enum Msg {
    Quit,
}

#[widget]
impl Widget for Win {
    fn model() -> () {
    }

    fn update(&mut self, event: Msg) {
        match event {
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Grid {
                gtk::Button {
                    label: "7",
                    cell: {
                        left_attach: 0,
                        top_attach: 0,
                    },
                },
                gtk::Button {
                    label: "8",
                    cell: {
                        left_attach: 1,
                        top_attach: 0,
                    },
                },
                gtk::Button {
                    label: "9",
                    cell: {
                        left_attach: 2,
                        top_attach: 0,
                    },
                },
                gtk::Button {
                    label: "4",
                    cell: {
                        left_attach: 0,
                        top_attach: 1,
                    },
                },
                gtk::Button {
                    label: "5",
                    cell: {
                        left_attach: 1,
                        top_attach: 1,
                    },
                },
                gtk::Button {
                    label: "6",
                    cell: {
                        left_attach: 2,
                        top_attach: 1,
                    },
                },
                gtk::Button {
                    label: "1",
                    cell: {
                        left_attach: 0,
                        top_attach: 2,
                    },
                },
                gtk::Button {
                    label: "2",
                    cell: {
                        left_attach: 1,
                        top_attach: 2,
                    },
                },
                gtk::Button {
                    label: "3",
                    cell: {
                        left_attach: 2,
                        top_attach: 2,
                    },
                },
                gtk::Button {
                    label: "+/-",
                    cell: {
                        left_attach: 0,
                        top_attach: 3,
                    },
                },
                gtk::Button {
                    label: "0",
                    cell: {
                        left_attach: 1,
                        top_attach: 3,
                    },
                },
                gtk::Button {
                    label: ".",
                    cell: {
                        left_attach: 2,
                        top_attach: 3,
                    },
                }
            },
            delete_event(_, _) => (Quit, Inhibit(false))
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}
