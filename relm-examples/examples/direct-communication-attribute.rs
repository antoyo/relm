/*
 * Copyright (c) 2021 Boucher, Antoni <bouanto@zoho.com>
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
    prelude::ButtonExt,
    prelude::EntryExt,
    prelude::LabelExt,
    prelude::OrientableExt,
    prelude::WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{Msg, widget};

use self::CounterMsg::*;
use self::Msg::*;
use self::TextMsg::*;
use glib::Propagation;

pub struct TextModel {
    content: String,
    win_stream: StreamHandle<Msg>,
}

#[derive(Msg)]
pub enum TextMsg {
    Change(glib::GString),
}

#[widget]
impl Widget for Text {
    fn model(win_stream: StreamHandle<Msg>) -> TextModel {
        TextModel {
            content: String::new(),
            win_stream,
        }
    }

    fn update(&mut self, event: TextMsg) {
        match event {
            Change(text) => {
                self.model.content = text.chars().rev().collect();
                self.model.win_stream.emit(TextChange(text.to_string()));
            },
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,
            gtk::Entry {
                changed(entry) => Change(entry.text()),
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
    relm: Relm<Win>,
}

#[derive(Msg)]
pub enum Msg {
    TextChange(String),
    Quit,
}

#[widget]
impl Widget for Win {
    fn model(relm: &Relm<Self>, _: ()) -> Model {
        Model {
            counter: 0,
            relm: relm.clone(),
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
                Text(self.model.relm.stream().clone()) {
                    Change(_) => counter1@Increment,
                },
                #[name="label"]
                gtk::Label {
                    text: &self.model.counter.to_string(),
                }
            },
            delete_event(_, _) => (Quit, Propagation::Proceed),
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
    use gtk::{Button, Entry, Label, prelude::LabelExt};

    use gtk_test::{assert_text, find_child_by_name};
    use relm_test::{click, enter_keys};

    use crate::Win;

    #[test]
    fn label_change() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let dec_button = &widgets.dec_button;
        let label1: Label = find_child_by_name(&widgets.counter1, "label").expect("label1");
        let inc_button1: Button = find_child_by_name(&widgets.counter1, "inc_button").expect("button1");
        let dec_button1: Button = find_child_by_name(&widgets.counter1, "dec_button").expect("button1");
        let label2: Label = find_child_by_name(&widgets.counter2, "label").expect("label2");
        let label = &widgets.label;
        let entry: Entry = find_child_by_name(&widgets.text, "entry").expect("entry");
        let text_label: Label = find_child_by_name(&widgets.text, "text_label").expect("label");

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
