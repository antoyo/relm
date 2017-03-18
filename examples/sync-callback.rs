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

use gtk::{Dialog, DialogExt, Inhibit, WidgetExt, Window, WindowType, DIALOG_MODAL};
use relm::{Relm, RemoteRelm, Widget};

use self::Msg::*;

#[derive(Msg)]
enum Msg {
    Quit,
}

struct Widgets {
    window: Window,
}

struct Win {
    widgets: Widgets,
}

impl Win {
    fn view(relm: &RemoteRelm<Msg>) -> Widgets {
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

        Widgets {
            window: window,
        }
    }
}

impl Widget<Msg> for Win {
    type Container = Window;
    type Model = ();

    fn container(&self) -> &Self::Container {
        &self.widgets.window
    }

    fn new(relm: RemoteRelm<Msg>) -> (Self, ()) {
        let widgets = Self::view(&relm);
        let win = Win {
            widgets: widgets,
        };
        (win, ())
    }

    fn update(&mut self, event: Msg, _model: &mut ()) {
        match event {
            Quit => gtk::main_quit(),
        }
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
    Relm::run::<Win>().unwrap();
}
