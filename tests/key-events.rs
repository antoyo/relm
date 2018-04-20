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

extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate relm_test;

use std::cell::Cell;
use std::rc::Rc;

use gtk::{
    ContainerExt,
    Entry,
    Inhibit,
    WidgetExt,
    Window,
    WindowType,
};
use relm::{Relm, Update, Widget, WidgetTest};

use self::Msg::*;

#[derive(Msg)]
pub enum Msg {
    KeyPress,
    Press,
    Release,
    Quit,
}

pub struct Model {
    press_count: Rc<Cell<i32>>,
}

#[derive(Clone)]
struct Widgets {
    entry: Entry,
    window: Window,
}

struct Win {
    model: Model,
    widgets: Widgets,
}

impl Update for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_relm: &Relm<Self>, _: ()) -> Model {
        Model {
            press_count: Rc::new(Cell::new(0)),
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
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Win>, model: Self::Model) -> Self {
        let window = Window::new(WindowType::Toplevel);
        let entry = Entry::new();
        window.add(&entry);

        let press_count = model.press_count.clone();
        let cloned_relm = relm.clone();
        connect!(relm, entry, connect_key_press_event(_, _), return (KeyPress, inhibit_press_event(&press_count, &cloned_relm)));

        window.show_all();

        connect!(relm, window, connect_key_press_event(_, _), return (Press, Inhibit(false)));
        connect!(relm, window, connect_key_release_event(_, _), return (Release, Inhibit(false)));
        connect!(relm, window, connect_delete_event(_, _), return (Quit, Inhibit(false)));

        Win {
            model,
            widgets: Widgets {
                entry,
                window,
            },
        }
    }
}

impl WidgetTest for Win {
    type Widgets = Widgets;

    fn get_widgets(&self) -> Self::Widgets {
        self.widgets.clone()
    }
}

fn inhibit_press_event(press_count: &Rc<Cell<i32>>, _relm: &Relm<Win>) -> Inhibit {
    Inhibit(press_count.get() > 3)
}

fn main() {
    Win::run(()).unwrap();
}

#[cfg(test)]
mod tests {
    use gtk::EntryExt;

    use relm;
    use relm_test::enter_keys;

    use Win;

    #[test]
    fn inhibit_event() {
        let (_component, widgets) = relm::init_test::<Win>(()).unwrap();
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
