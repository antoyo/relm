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

use std::fmt::Display;

use gtk::{
    ButtonExt,
    Inhibit,
    LabelExt,
    OrientableExt,
    WidgetExt,
};
use gtk::Orientation::{Horizontal, Vertical};
use relm::Widget;
use relm_attributes::widget;

use self::CounterMsg::*;
use self::Msg::*;

pub trait IncDec {
    fn dec(&self) -> Self;
    fn inc(&self) -> Self;
}

impl IncDec for i32 {
    fn dec(&self) -> Self {
        *self - 1
    }

    fn inc(&self) -> Self {
        *self + 1
    }
}

pub struct Model<T> {
    counter: T,
}

#[derive(Msg)]
pub enum CounterMsg {
    Decrement,
    Increment,
}

#[widget]
impl<T: IncDec + Display> Widget for Counter<T> {
    fn model(value: T) -> Model<T> {
        Model {
            counter: value,
        }
    }

    fn update(&mut self, event: CounterMsg) {
        match event {
            Decrement => {
                self.model.counter = self.model.counter.dec();
            },
            Increment => {
                self.model.counter = self.model.counter.inc();
            },
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,
            gtk::Button {
                label: "+",
                name: "inc_button",
                clicked => Increment,
            },
            gtk::Label {
                name: "label",
                text: &self.model.counter.to_string(),
            },
            gtk::Button {
                label: "-",
                name: "dec_button",
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
    fn model(_: ()) -> () {
        ()
    }

    fn update(&mut self, event: Msg) {
        match event {
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Horizontal,
                #[name="counter1"]
                Counter<i32>(2),
                #[name="counter2"]
                Counter<i32>(3),
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
    use gtk::{Button, Label, LabelExt};

    use relm;
    use relm_test::{click, find_child_by_name};

    use Win;

    #[test]
    fn widget_position() {
        let (_component, widgets) = relm::init_test::<Win>(()).unwrap();
        let inc_button1: Button = find_child_by_name(widgets.counter1.widget(), "inc_button").expect("inc button");
        let dec_button1: Button = find_child_by_name(widgets.counter1.widget(), "dec_button").expect("dec button");
        let label1: Label = find_child_by_name(widgets.counter1.widget(), "label").expect("label");
        let inc_button2: Button = find_child_by_name(widgets.counter2.widget(), "inc_button").expect("inc button");
        let dec_button2: Button = find_child_by_name(widgets.counter2.widget(), "dec_button").expect("dec button");
        let label2: Label = find_child_by_name(widgets.counter2.widget(), "label").expect("label");

        assert_text!(label1, 2);

        click(&inc_button1);
        assert_text!(label1, 3);

        click(&inc_button1);
        assert_text!(label1, 4);

        click(&dec_button1);
        assert_text!(label1, 3);

        assert_text!(label2, 3);

        click(&inc_button2);
        assert_text!(label2, 4);

        click(&inc_button2);
        assert_text!(label2, 5);

        click(&dec_button2);
        assert_text!(label2, 4);
    }
}
