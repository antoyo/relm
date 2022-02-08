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
    PackType,
    prelude::BoxExt,
    prelude::ButtonExt,
    prelude::LabelExt,
    prelude::OrientableExt,
    prelude::WidgetExt,
};
use gtk::Orientation::Vertical;
use gtk::WindowType::Popup;
use relm::Widget;
use relm_derive::{Msg, widget};

use self::Msg::*;

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
        gtk::Window(Popup) {
            gtk::Box {
                orientation: Vertical,
                #[name="label"]
                gtk::Label {
                    text: &self.model.counter.to_string(),
                },
                #[name="dec_button"]
                gtk::Button {
                    clicked => Decrement,
                    label: "-",
                },
                #[name="inc_button"]
                gtk::Button {
                    child: {
                        expand: false,
                        fill: true,
                        pack_type: PackType::Start,
                        padding: 10,
                        position: 0,
                    },
                    clicked => Increment,
                    label: "+",
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).expect("Win::run");
}

#[cfg(test)]
mod tests {
    use gtk::prelude::WidgetExt;

    use crate::Win;

    #[test]
    fn button_position() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let inc_button = &widgets.inc_button;
        let dec_button = &widgets.dec_button;
        let label = &widgets.label;

        let inc_allocation = inc_button.allocation();
        let dec_allocation = dec_button.allocation();
        let label_allocation = label.allocation();
        assert!(inc_allocation.y() < dec_allocation.y());
        // 10 is the padding.
        assert_eq!(
            dec_allocation.y(),
            inc_allocation.y() + inc_allocation.height() + 10 + label_allocation.height()
        );
    }
}
