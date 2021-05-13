/*
 * Copyright (c) 2017-2020 Boucher, Antoni <bouanto@zoho.com>
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
    BoxExt,
    ButtonExt,
    Inhibit,
    LabelExt,
    OrientableExt,
    WidgetExt,
};
use gtk::Orientation::{Horizontal, Vertical};
use relm::Widget;
use relm_derive::{Msg, widget};

use self::Msg::*;

#[widget]
impl Widget for MyFrame {
    fn model() -> () {
    }

    fn update(&mut self, _msg: ()) {
    }

    view! {
        #[container]
        gtk::Frame {
        }
    }
}

#[widget]
impl Widget for CenterButton {
    fn model() -> () {
    }

    fn update(&mut self, _msg: ()) {
    }

    view! {
        #[parent="center"]
        gtk::Button {
            label: "-",
        },
    }
}

#[widget]
impl Widget for Button {
    fn model() -> () {
    }

    fn update(&mut self, _msg: ()) {
    }

    view! {
        #[parent="right"]
        #[name="button"]
        gtk::Button {
            label: "+",
        },
    }
}

#[widget]
impl Widget for SplitBox {
    fn model() -> () {
        ()
    }

    fn update(&mut self, _event: Msg) {
    }

    view! {
        gtk::Box {
            orientation: Horizontal,
            // Specify where the widgets will be added in this container by default.
            #[container]
            gtk::Box {
                orientation: Vertical,
            },
            // Specify where the widgets will be added in this container when the child's parent id is
            // "center".
            #[container="center"]
            gtk::Frame {
            },
            #[container="right"]
            MyFrame {
                child: {
                    padding: 10,
                },
            }
        }
    }
}

pub struct Model {
    counter: i32,
}

#[derive(Msg)]
pub enum Msg {
    Decrement,
    Increment,
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
            Decrement => self.model.counter -= 1,
            Increment => self.model.counter += 1,
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            SplitBox {
                #[name="button1"]
                gtk::Button {
                    clicked => Increment,
                    label: "+",
                },
                #[name="label"]
                gtk::Label {
                    text: &self.model.counter.to_string(),
                },
                #[name="right_button"]
                Button {
                },
                #[name="center_button"]
                CenterButton {
                },
                #[name="button2"]
                gtk::Button {
                    clicked => Decrement,
                    label: "-",
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}

#[cfg(test)]
mod tests {
    use gtk::WidgetExt;

    use crate::Win;

    #[test]
    fn model_params() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let button1 = &widgets.button1;
        let label = &widgets.label;
        let button2 = &widgets.button2;
        let right_button = &widgets.right_button;
        let center_button = &widgets.center_button;

        let button1_allocation = button1.allocation();
        let label_allocation = label.allocation();
        let button2_allocation = button2.allocation();
        let right_allocation = right_button.allocation();
        let center_allocation = center_button.allocation();

        assert!(button1_allocation.y < label_allocation.y);
        assert!(label_allocation.y < button2_allocation.y);
        assert!(button1_allocation.x < center_allocation.x);
        assert!(center_allocation.x < right_allocation.x);
        assert!(center_allocation.y == right_allocation.y);
    }
}
