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
                changed(entry) => Change(entry.get_text()),
                widget_name: "entry",
            },
            gtk::Label {
                text: &self.model.content,
                widget_name: "text_label",
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
            Increment => self.increment(),
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
                widget_name: "dec_button",
                clicked => Decrement,
            },
        }
    }

    fn increment(&mut self) {
        self.model.counter += 1;
    }
}

pub struct Model {
    counter: i32,
}

#[derive(Msg)]
pub enum Msg {
    TextChange(String),
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
            TextChange(text) => {
                println!("{}", text);
                self.model.counter += 1
            },
            Quit => gtk::main_quit(),
        }
    }


    view! {
        gtk::Window {
            gtk::Box {
                #[name="dec_button"]
                gtk::Button {
                    label: "Decrement",
                    clicked => counter1@Decrement,
                },
                #[name="counter1"]
                Counter {
                    Increment => counter2@Decrement,
                    Increment => log_increment(),
                },
                #[name="counter2"]
                Counter,
                #[name="text"]
                Text {
                    Change(_) => counter1@Increment,
                    Change(ref text) => TextChange(text.to_string()),
                },
                #[name="label"]
                gtk::Label {
                    text: &self.model.counter.to_string(),
                }
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn log_increment() {
    println!("Increment");
}

fn main() {
    Win::run(()).expect("Win::run failed");
}

#[cfg(test)]
mod tests {
    use gtk::{Button, Entry, Label, LabelExt};

    use gtk_test::{assert_text, click, enter_keys, find_child_by_name};

    use crate::Win;

    #[test]
    fn label_change() {
        let (_component, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let dec_button = &widgets.dec_button;
        let label1: Label = find_child_by_name(widgets.counter1.widget(), "label").expect("label1");
        let inc_button1: Button = find_child_by_name(widgets.counter1.widget(), "inc_button").expect("button1");
        let dec_button1: Button = find_child_by_name(widgets.counter1.widget(), "dec_button").expect("button1");
        let label2: Label = find_child_by_name(widgets.counter2.widget(), "label").expect("label2");
        let label = &widgets.label;
        let entry: Entry = find_child_by_name(widgets.text.widget(), "entry").expect("entry");
        let text_label: Label = find_child_by_name(widgets.text.widget(), "text_label").expect("label");

        assert_text!(label1, 0);

        click(dec_button);
        assert_text!(label1, -1);

        click(dec_button);
        assert_text!(label1, -2);

        assert_text!(label2, 0);

        click(&inc_button1);
        assert_text!(label1, -1);
        assert_text!(label2, -1);

        click(&inc_button1);
        assert_text!(label1, 0);
        assert_text!(label2, -2);

        click(&dec_button1);
        assert_text!(label1, -1);
        assert_text!(label2, -2);

        click(&dec_button1);
        assert_text!(label1, -2);
        assert_text!(label2, -2);

        assert_text!(label, 0);

        enter_keys(&entry, "t");
        assert_text!(label, 1);
        assert_text!(label1, -1);
        assert_text!(text_label, "t");

        enter_keys(&entry, "e");
        assert_text!(label, 2);
        assert_text!(label1, 0);
        assert_text!(text_label, "et");
    }
}
