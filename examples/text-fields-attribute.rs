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

extern crate gdk;
extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate relm_test;

use gtk::prelude::*;
use gtk::Orientation::Vertical;
use relm::Widget;
use relm_attributes::widget;

use self::Msg::*;

pub struct Model {
    content: String,
}

#[derive(Msg)]
pub enum Msg {
    Change(String, usize),
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
            Change(text, len) => {
                self.model.content = text.chars().rev().collect();
                self.model.content += &format!(" ({})", len);
            },
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                #[name="entry"]
                gtk::Entry {
                    changed(entry) => {
                        let text = entry.get_text().unwrap();
                        let len = text.len();
                        Change(text, len)
                    },
                    placeholder_text: "Text to reverse",
                },
                #[name="label"]
                gtk::Label {
                    text: &self.model.content,
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}

#[cfg(test)]
mod tests {
    use gdk::enums::key;
    use gtk::prelude::*;

    use relm;
    use relm_test::{enter_key, enter_keys};

    use Win;

    #[test]
    fn label_change() {
        let (_component, widgets) = relm::init_test::<Win>(()).unwrap();
        let entry = &widgets.entry;
        let label = &widgets.label;

        assert_text!(label, "");

        enter_keys(entry, "test");
        assert_text!(label, "tset (4)");

        enter_key(entry, key::BackSpace);
        assert_text!(label, "set (3)");

        enter_key(entry, key::Home);
        //enter_key(entry, key::Delete); // TODO: when supported by enigo.
        enter_keys(entry, "a");
        assert_text!(label, "seta (4)");

        enter_key(entry, key::End);
        enter_keys(entry, "a");
        assert_text!(label, "aseta (5)");
    }
}
