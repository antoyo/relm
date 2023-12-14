/*
 * Copyright (c) 2018-2020 Boucher, Antoni <bouanto@zoho.com>
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
    EditableSignals,
    prelude::EntryExt,
    prelude::LabelExt,
    prelude::OrientableExt,
    prelude::WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::Widget;
use relm_derive::{Msg, widget};

use self::Msg::*;
use glib::Propagation;

pub struct Model {
    content: String,
}

#[derive(Msg)]
pub enum Msg {
    Change(glib::GString),
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
                    changed(entry) => Change(entry.text()),
                    placeholder_text: Some("Text to reverse"),
                },
                #[name = "entry2"]
                gtk::Entry { },
                #[name = "label"]
                gtk::Label {
                    text: &self.model.content,
                },
            },
            delete_event(_, _) => (Quit, Propagation::Proceed),
        }
    }
}

#[cfg(test)]
mod tests {
    use gdk::keys::constants as key;
    use gtk::prelude::{EntryExt, LabelExt};

    use gtk_test::{
        assert_text,
        enter_key,
        enter_keys,
        key_press,
        key_release,
    };

    use crate::Win;

    #[test]
    fn label_change() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let entry = &widgets.entry;
        let entry2 = &widgets.entry2;
        let label = &widgets.label;

        // TODO: add test with uppercase letter (shift) when this issue (https://github.com/enigo-rs/enigo/issues/49) is fixed.
        //key_press(entry, key::Shift_L);
        key_press(entry, key::a);
        assert_text!(label, "a");
        key_release(entry, key::a);
        assert_text!(label, "a");
        enter_key(entry, key::b);
        enter_key(entry2, key::c);
        assert_text!(label, "ba");
        assert_text!(entry2, "c");
        enter_keys(entry, "CD");
        //key_release(entry, key::Shift_L);
        assert_text!(label, "DCba");
    }
}
