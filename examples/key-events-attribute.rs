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
    Inhibit,
    WidgetExt,
};
use relm::Widget;
use relm_attributes::widget;

use self::Msg::*;

#[derive(Msg)]
pub enum Msg {
    Press,
    Release,
    Quit,
}

#[derive(Clone)]
pub struct Model {
    press_count: i32,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            press_count: 0,
        }
    }

    fn update(&mut self, event: Msg, model: &mut Model) {
        match event {
            Press => {
                model.press_count += 1;
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
            delete_event(_, _) with model => return self.quit(model),
        }
    }
}

impl Win {
    fn quit(&self, model: &mut Model) -> (Option<Msg>, Inhibit) {
        if model.press_count > 3 {
            (None, Inhibit(true))
        }
        else {
            (Some(Quit), Inhibit(false))
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}
