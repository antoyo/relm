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

use std::cell::Cell;
use std::rc::Rc;

use gtk::prelude::WidgetExt;
use relm::{Relm, Widget};
use relm_derive::{Msg, widget};

use self::Msg::*;
use glib::Propagation;

#[derive(Msg)]
pub enum Msg {
    KeyPress,
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
            KeyPress => (),
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
        #[name="window"]
        gtk::Window {
            gtk::Box {
                #[name="entry"]
                gtk::Entry {
                    key_press_event(_, key) with (press_count, relm) => (KeyPress, inhibit_press_event(&press_count, &relm)),
                },
            },
            key_press_event(_, key) => (Press, Propagation::Proceed),
            key_release_event(_, key) => (Release, Propagation::Proceed),
            delete_event(_, _) => (Quit, Propagation::Proceed),
        }
    }
}

fn inhibit_press_event(press_count: &Rc<Cell<i32>>, _relm: &Relm<Win>) -> Propagation {
    if press_count.get() > 3 {
        Propagation::Stop
    } else {
        Propagation::Proceed
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}

#[cfg(test)]
mod tests {
    use gtk::prelude::EntryExt;

    use gtk_test::{assert_text, enter_keys};

    use crate::Win;

    #[test]
    fn inhibit_event() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let entry = &widgets.entry;

        enter_keys(entry, "a");
        assert_text!(entry, "a");

        enter_keys(entry, "b");
        assert_text!(entry, "ab");

        enter_keys(entry, "c");
        assert_text!(entry, "abc");

        enter_keys(entry, "d");
        assert_text!(entry, "abcd");

        enter_keys(entry, "e");
        assert_text!(entry, "abcd");

        enter_keys(entry, "f");
        assert_text!(entry, "abcd");
    }
}
