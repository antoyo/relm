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
#[macro_use]
extern crate relm_test;

use gtk::{
    ButtonExt,
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

use Msg::*;
use TextMsg::*;

pub struct TextModel {
    content: String,
}

#[derive(Msg)]
pub enum TextMsg {
    Change(String),
    SetText(String),
}

#[widget]
impl Widget for Text {
    fn model() -> TextModel {
        TextModel {
            content: String::new(),
        }
    }

    fn update(&mut self, event: TextMsg) {
        match event {
            Change(text) => self.model.content = text.chars().rev().collect(),
            SetText(text) => self.text_entry.set_text(&text),
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,
            #[name="text_entry"]
            gtk::Entry {
                name: "text_entry",
                changed(entry) => Change(entry.get_text().unwrap()),
            },
            gtk::Label {
                text: &self.model.content,
            },
        }
    }
}

pub struct Model {
    text: String,
}

#[derive(Msg)]
pub enum Msg {
    Reset,
    Quit,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            text: "Test".to_string(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Reset => self.model.text = String::new(),
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                #[name="button"]
                gtk::Button {
                    clicked => Reset,
                    label: "Reset",
                },
                #[name="text"]
                Text {
                    // Send the message SetText(self.model.text.clone()) at initialization and when
                    // the model attribute is updated.
                    SetText: self.model.text.clone(),
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
    use gtk::{Entry, EntryExt};

    use relm;
    use relm_test::{click, find_child_by_name, wait};

    use Win;

    #[test]
    fn root_widget() {
        let (_component, widgets) = relm::init_test::<Win>(()).unwrap();
        let button = &widgets.button;
        let entry: Entry = find_child_by_name(widgets.text.widget(), "text_entry").expect("entry");

        wait(200);

        assert_text!(entry, "Test");

        click(button);

        assert_text!(entry, "");
    }
}
