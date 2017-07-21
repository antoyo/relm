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

extern crate futures_glib;
extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

use std::cell::Cell;
use std::rc::Rc;

use gtk::{
    Inhibit,
    WidgetExt,
};
use relm::{Relm, Widget};
use relm_attributes::widget;

use self::Msg::*;

#[derive(Msg)]
pub enum Msg {
    Delete,
    Press,
    Release,
    Quit,
}

pub struct Model {
    press_count: Rc<Cell<i32>>,
    relm: Relm<Win>,
}

#[widget]
impl Widget for Win {
    fn model(relm: &Relm<Self>, _: ()) -> Model {
        Model {
            press_count: Rc::new(Cell::new(0)),
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Delete => {
                if self.model.press_count.get() <= 3 {
                    self.model.relm.stream().emit(Quit);
                }
            },
            Press => {
                self.model.press_count.set(self.model.press_count.get() + 1);
                println!("Press");
            },
            Release => {
                println!("Release");
            },
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            key_press_event(_, key) => (Press, Inhibit(false)),
            key_release_event(_, key) => (Release, Inhibit(false)),
            delete_event(_, _) with (press_count) => (Delete, inhibit_delete_event(&press_count)),
        }
    }
}

fn inhibit_delete_event(press_count: &Rc<Cell<i32>>) -> Inhibit {
    Inhibit(press_count.get() > 3)
}

fn main() {
    Win::run(()).unwrap();
}
