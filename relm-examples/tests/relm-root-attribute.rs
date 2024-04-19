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
    Inhibit,
    prelude::ButtonExt,
    prelude::LabelExt,
    prelude::OrientableExt,
    prelude::WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::Widget;
use relm_derive::{Msg, widget};

use self::Msg::*;

#[widget]
impl Widget for Button {
    fn model() {
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
    fn model() {
        
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

pub struct Model {
    visible: bool,
}

#[widget]
impl Widget for MyVBox {
    fn model() -> Model {
        Model {
            visible: true,
        }
    }

    fn update(&mut self, _event: ()) {
        self.model.visible = true;
    }

    view! {
        VBox {
            gtk::Button {
                widget_name: "inc_button",
                label: "+",
            },
            gtk::Label {
                widget_name: "label",
                text: "0",
            },
            Button {
                visible: self.model.visible,
                widget_name: "button",
            },
            gtk::Button {
                widget_name: "dec_button",
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
    fn model() {
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
    Win::run(()).expect("Win::run failed");
}

#[cfg(test)]
mod tests {
    use gtk::{Button, Label, prelude::WidgetExt};

    use gtk_test::find_child_by_name;

    use crate::Win;

    #[test]
    fn root_widget() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let vbox = &widgets.vbox;
        let inc_button: Button = find_child_by_name(vbox, "inc_button").expect("inc button");
        let label: Label = find_child_by_name(vbox, "label").expect("label");
        let button: Button = find_child_by_name(vbox, "button").expect("button");
        let dec_button: Button = find_child_by_name(vbox, "dec_button").expect("dec button");
        let inc_allocation = inc_button.allocation();
        let label_allocation = label.allocation();
        let button_allocation = button.allocation();
        let dec_button_allocation = dec_button.allocation();

        assert!(inc_allocation.y() < label_allocation.y());
        assert!(label_allocation.y() < button_allocation.y());
        assert!(button_allocation.y() < dec_button_allocation.y());
    }
}
