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

use std::fmt::Display;

use gtk::{
    ButtonExt,
    Inhibit,
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

impl IncDec for i64 {
    fn dec(&self) -> Self {
        *self - 1
    }

    fn inc(&self) -> Self {
        *self + 1
    }
}

#[derive(Clone)]
pub struct Model<S, T> {
    counter1: S,
    counter2: T,
}

#[derive(Msg)]
pub enum CounterMsg {
    Decrement,
    Increment,
}

#[widget]
impl<S: Clone + Display + IncDec, T: Clone + Display + IncDec> Widget for Counter<S, T> {
    fn model((value1, value2): (S, T)) -> Model<S, T> {
        Model {
            counter1: value1,
            counter2: value2,
        }
    }

    fn update(&mut self, event: CounterMsg, model: &mut Self::Model) {
        match event {
            Decrement => {
                model.counter1 = model.counter1.dec();
            },
            Increment => {
                model.counter1 = model.counter1.inc();
            },
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,
            gtk::Button {
                label: "+",
                clicked => Increment,
            },
            gtk::Label {
                text: &model.counter1.to_string(),
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
    fn model(_: ()) -> () {
        ()
    }

    fn update(&mut self, event: Msg, _model: &mut ()) {
        match event {
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Horizontal,
                Counter<i32, i64>(2, 3),
                Counter<i32, i64>(3, 4),
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}
