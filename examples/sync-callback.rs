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

extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate tokio_core;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::{Dialog, DialogExt, Inhibit, WidgetExt, Window, WindowType, DIALOG_MODAL};
use relm::{Relm, Widget};

use self::Msg::*;

#[derive(Msg)]
enum Msg {
    Quit,
}

struct Win {
    window: Window,
}

impl Widget for Win {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;
    type Root = Window;

    fn model(_: &Relm<Self>, _: ()) -> () {
        ()
    }

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn update(&mut self, event: Msg) {
        match event {
            Quit => gtk::main_quit(),
        }
    }

    fn view(relm: &Relm<Self>, _model: Self::Model) -> Rc<RefCell<Self>> {
        let window = Window::new(WindowType::Toplevel);

        window.show_all();

        let parent = window.clone();
        connect!(relm, window, connect_delete_event(_, _) {
            let num = dialog(&parent);
            match num {
                1 => (Some(Quit), Inhibit(false)),
                _ => (None, Inhibit(true)),
            }
        });

        Rc::new(RefCell::new(Win {
            window: window,
        }))
    }
}

fn dialog(window: &Window) -> i32 {
    let buttons = &[("Yes", 1), ("No", 2)];
    let dialog = Dialog::new_with_buttons(Some("Quit?"), Some(window), DIALOG_MODAL, buttons);
    let result = dialog.run();
    dialog.destroy();
    result
}

fn main() {
    Win::run(()).unwrap();
}
