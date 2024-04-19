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
    EditableSignals,
    Inhibit,
    prelude::ButtonExt,
    prelude::EntryExt,
    prelude::LabelExt,
    prelude::OrientableExt,
    prelude::WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::Widget;
use relm_derive::{Msg, widget};

use self::CounterMsg::*;
use self::Msg::*;
use self::TextMsg::*;

pub struct TextModel {
    content: String,
}

#[derive(Msg)]
pub enum TextMsg {
    Change(glib::GString),
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
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,
            gtk::Entry {
                widget_name: "entry",
                changed(entry) => Change(entry.text()),
            },
            gtk::Label {
                widget_name: "label",
                text: &self.model.content,
            },
        }
    }
}

pub struct CounterModel {
    counter: i32,
}

#[derive(Msg)]
pub enum CounterMsg {
    Decrement,
    Increment,
}

#[widget]
impl Widget for Counter {
    fn model() -> CounterModel {
        CounterModel {
            counter: 0,
        }
    }

    fn update(&mut self, event: CounterMsg) {
        match event {
            Decrement => self.model.counter -= 1,
            Increment => self.model.counter += 1,
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,
            gtk::Button {
                label: "+",
                widget_name: "inc_button",
                clicked => Increment,
            },
            gtk::Label {
                widget_name: "label",
                text: &self.model.counter.to_string(),
            },
            gtk::Button {
                label: "-",
                clicked => Decrement,
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
            gtk::Box {
                #[name="counter1"]
                Counter,
                #[name="counter2"]
                Counter,
                #[name="text"]
                Text,
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
    use gtk::{Button, Entry, Label, prelude::LabelExt};

    use gtk_test::{assert_text, enter_keys, find_child_by_name};
    use relm_test::click;

    use crate::Win;

    #[test]
    fn model_params() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let counter1 = &widgets.counter1;
        let text = &widgets.text;
        let inc_button1: Button = find_child_by_name(counter1, "inc_button").expect("button");
        let label1: Label = find_child_by_name(counter1, "label").expect("label");
        let counter2 = &widgets.counter2;
        let inc_button2: Button = find_child_by_name(counter2, "inc_button").expect("button");
        let label2: Label = find_child_by_name(counter2, "label").expect("label");
        let entry: Entry = find_child_by_name(text, "entry").expect("entry");
        let text_label: Label = find_child_by_name(text, "label").expect("label");

        assert_text!(label1, 0);

        click(&inc_button1);
        assert_text!(label1, 1);

        assert_text!(label2, 0);

        click(&inc_button2);
        assert_text!(label2, 1);

        assert_text!(text_label, "");

        enter_keys(&entry, "test");
        assert_text!(text_label, "tset");
    }
}
