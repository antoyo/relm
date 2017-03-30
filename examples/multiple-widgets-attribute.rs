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

extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

use gtk::{
    Button,
    ButtonExt,
    EditableSignals,
    Entry,
    EntryExt,
    Inhibit,
    Label,
    OrientableExt,
    WidgetExt,
    Window,
};
use gtk::Orientation::Vertical;
use relm::{Relm, RemoteRelm, Widget};
use relm_attributes::widget;

use self::CounterMsg::*;
use self::Msg::*;
use self::TextMsg::*;

#[derive(Clone)]
struct TextModel {
    content: String,
}

#[derive(Msg)]
enum TextMsg {
    Change(String),
}

#[widget]
impl Widget<TextMsg> for Text {
    fn model() -> TextModel {
        TextModel {
            content: String::new(),
        }
    }

    fn update(&mut self, event: TextMsg, model: &mut TextModel) {
        match event {
            Change(text) => model.content = text.chars().rev().collect(),
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,
            Entry {
                changed(entry) => Change(entry.get_text().unwrap()),
            },
            Label {
                text: &model.content,
            },
        }
    }
}

#[derive(Clone)]
struct CounterModel {
    counter: i32,
}

#[derive(Msg)]
enum CounterMsg {
    Decrement,
    Increment,
}

#[widget]
impl Widget<CounterMsg> for Counter {
    fn model() -> CounterModel {
        CounterModel {
            counter: 0,
        }
    }

    fn update(&mut self, event: CounterMsg, model: &mut CounterModel) {
        match event {
            Decrement => model.counter -= 1,
            Increment => model.counter += 1,
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,
            Button {
                label: "+",
                clicked => Increment,
            },
            Label {
                text: &model.counter.to_string(),
            },
            Button {
                label: "-",
                clicked => Decrement,
            },
        }
    }
}

#[derive(Msg)]
enum Msg {
    Quit,
}

#[widget]
impl Widget<Msg> for Win {
    fn model() -> () {
        ()
    }

    fn update(&mut self, event: Msg, _model: &mut ()) {
        match event {
            Quit => gtk::main_quit(),
        }
    }

    view! {
        Window {
            gtk::Box {
                Counter,
                Counter,
                Text,
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Relm::run::<Win>().unwrap();
}
