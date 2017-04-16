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

extern crate chrono;
extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate tokio_core;

use std::time::Duration;

use chrono::Local;
use gtk::{ContainerExt, Inhibit, Label, WidgetExt, Window, WindowType};
use relm::{Relm, RemoteRelm, Widget};
use tokio_core::reactor::Interval;

use self::Msg::*;

#[derive(Msg)]
enum Msg {
    Quit,
    Tick(()),
}

struct Win {
    label: Label,
    window: Window,
}

impl Widget for Win {
    type Container = Window;
    type Model = ();
    type Msg = Msg;

    fn container(&self) -> &Self::Container {
        &self.window
    }

    fn model() -> () {
        ()
    }

    fn subscriptions(relm: &Relm<Msg>) {
        let stream = Interval::new(Duration::from_secs(1), relm.handle()).unwrap();
        relm.connect_exec_ignore_err(stream, Tick);
    }

    fn update(&mut self, event: Msg, _model: &mut ()) {
        match event {
            Tick(()) => {
                let time = Local::now();
                self.label.set_text(&format!("{}", time.format("%H:%M:%S")));
            },
            Quit => gtk::main_quit(),
        }
    }

    fn view(relm: RemoteRelm<Msg>, _model: &Self::Model) -> Self {
        let label = Label::new(None);

        let window = Window::new(WindowType::Toplevel);

        window.add(&label);

        window.show_all();

        connect!(relm, window, connect_delete_event(_, _) (Some(Quit), Inhibit(false)));

        let mut win = Win {
            label: label,
            window: window,
        };

        win.update(Tick(()), &mut ());
        win
    }
}

fn main() {
    relm::run::<Win>().unwrap();
}
