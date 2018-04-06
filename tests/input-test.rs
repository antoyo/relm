/*
 * Copyright (c) 2018 Boucher, Antoni <bouanto@zoho.com>
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

extern crate gdk;
extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate relm_test;

use gtk::{
    EditableSignals,
    EntryExt,
    Inhibit,
    LabelExt,
    OrientableExt,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::Widget;
use relm_attributes::widget;

use self::Msg::*;

pub struct Model {
    content: String,
}

#[derive(Msg)]
pub enum Msg {
    Change(String),
    Quit,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            content: String::new(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Change(text) => {
                self.model.content = text.chars().rev().collect();
            },
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                #[name = "entry"]
                gtk::Entry {
                    changed(entry) => Change(entry.get_text().unwrap()),
                    placeholder_text: "Text to reverse",
                },
                #[name = "entry2"]
                gtk::Entry { },
                #[name = "label"]
                gtk::Label {
                    text: &self.model.content,
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

#[cfg(test)]
mod tests {
    use gdk::enums::key;
    use gtk::{EntryExt, LabelExt};

    use relm;
    use relm_test::{
        enter_key,
        enter_keys,
        key_press,
        key_release,
    };

    use Win;

    #[test]
    fn label_change() {
        let (_component, widgets) = relm::init_test::<Win>(()).unwrap();
        let entry = &widgets.entry;
        let entry2 = &widgets.entry2;
        let label = &widgets.label;

        key_press(entry, key::A);
        assert_text!(label, "A");
        key_release(entry, key::A);
        assert_text!(label, "A");
        enter_key(entry, key::B);
        enter_key(entry2, key::C);
        assert_text!(label, "BA");
        assert_text!(entry2, "C");
        enter_keys(entry, "CD");
        assert_text!(label, "DCBA");
    }
}
