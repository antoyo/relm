/*
 * Copyright (c) 2020 Boucher, Antoni <bouanto@zoho.com>
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

use gtk::{
    ButtonExt,
    FrameExt,
    GtkMenuItemExt,
    Inhibit,
    LabelExt,
    OrientableExt,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::Widget;
use relm_derive::{Msg, widget};

use self::Msg::*;

pub struct Model {
    counter: i32,
}

#[derive(Msg)]
pub enum Msg {
    Click(f64, f64),
    Decrement,
    End,
    Increment,
    Move(f64, f64),
    Quit,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            counter: 0,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Click(x, y) => println!("Clicked on {}, {}", x, y),
            Decrement => self.model.counter -= 1,
            End => println!("End"),
            Increment => self.model.counter += 1,
            Move(x, y) => println!("Moved to {}, {}", x, y),
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                gtk::MenuBar {
                    gtk::MenuItem {
                        label: "File",
                        submenu: view! {
                            gtk::Menu {
                                gtk::MenuItem {
                                    label: "Increment",
                                    submenu: view! {
                                        gtk::Menu {
                                            gtk::MenuItem {
                                                label: &("By one ".to_string() + &self.model.counter.to_string()),
                                                activate => Increment,
                                            },
                                            gtk::MenuItem {
                                                label: &("By minus one ".to_string() + &self.model.counter.to_string()),
                                                activate => Decrement,
                                            },
                                        }
                                    },
                                },
                                gtk::MenuItem {
                                    label: "Quit",
                                    activate => Quit,
                                },
                            },
                        },
                    },
                },
                #[name="inc_button"]
                gtk::Button {
                    clicked => Increment,
                    label: "+",
                },
                gtk::Label {
                    text: &self.model.counter.to_string(),
                },
                gtk::Frame {
                    label_widget: view! {
                        gtk::Label {
                            text: &self.model.counter.to_string(),
                        }
                    },
                    #[name="dec_button"]
                    gtk::Button {
                        clicked => Decrement,
                        label: "-",
                    },
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}
