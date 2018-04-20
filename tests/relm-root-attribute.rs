/*
 * Copyright (c) 2017-2018 Boucher, Antoni <bouanto@zoho.com>
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
extern crate relm_test;

use gtk::{
    ButtonExt,
    Inhibit,
    LabelExt,
    OrientableExt,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::Widget;
use relm_attributes::widget;

use self::Msg::*;

#[widget]
impl Widget for Button {
    fn model() -> () {
    }

    fn update(&mut self, _msg: ()) {
    }

    view! {
        gtk::Button {
            label: "+",
        },
    }
}

#[widget]
impl Widget for VBox {
    fn model() -> () {
        ()
    }

    fn update(&mut self, _event: Msg) {
    }

    view! {
        gtk::EventBox {
            #[container]
            gtk::Box {
                orientation: Vertical,
            }
        }
    }
}

#[widget]
impl Widget for MyVBox {
    fn model() -> () {
    }

    fn update(&mut self, _event: ()) {
    }

    view! {
        VBox {
            gtk::Button {
                name: "inc_button",
                label: "+",
            },
            gtk::Label {
                name: "label",
                text: "0",
            },
            Button {
                name: "button",
            },
            gtk::Button {
                name: "dec_button",
                label: "-",
            },
        }
    }
}

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
            #[name="vbox"]
            MyVBox,
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}

#[cfg(test)]
mod tests {
    use gtk::{Button, Label, WidgetExt};

    use relm;
    use relm_test::find_child_by_name;

    use Win;

    #[test]
    fn root_widget() {
        let (_component, widgets) = relm::init_test::<Win>(()).unwrap();
        let vbox = &widgets.vbox;
        let inc_button: Button = find_child_by_name(vbox.widget(), "inc_button").expect("inc button");
        let label: Label = find_child_by_name(vbox.widget(), "label").expect("label");
        let button: Button = find_child_by_name(vbox.widget(), "button").expect("button");
        let dec_button: Button = find_child_by_name(vbox.widget(), "dec_button").expect("dec button");
        let inc_allocation = inc_button.get_allocation();
        let label_allocation = label.get_allocation();
        let button_allocation = button.get_allocation();
        let dec_button_allocation = dec_button.get_allocation();

        assert!(inc_allocation.y < label_allocation.y);
        assert!(label_allocation.y < button_allocation.y);
        assert!(button_allocation.y < dec_button_allocation.y);
    }
}
