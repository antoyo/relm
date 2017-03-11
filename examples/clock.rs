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
extern crate futures;
extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate tokio_core;
extern crate tokio_timer;

use std::time::Duration;

use chrono::Local;
use gtk::{ContainerExt, Label, WidgetExt, Window, WindowType};
use relm::{QuitFuture, Relm, Widget};
use tokio_timer::Timer;

use self::Msg::*;

#[derive(Msg)]
enum Msg {
    Quit,
    Tick(()),
}

struct Widgets {
    label: Label,
    window: Window,
}

struct Win {
    relm: Relm<Msg>,
    widgets: Widgets,
}

impl Win {
    fn subscriptions(&self) {
        let timer = Timer::default();
        let stream = timer.interval(Duration::from_secs(1));
        self.relm.connect_exec(stream, Tick);
    }

    fn view(relm: &Relm<Msg>) -> Widgets {
        let label = Label::new(None);

        let window = Window::new(WindowType::Toplevel);

        window.add(&label);

        window.show_all();

        connect_no_inhibit!(relm, window, connect_delete_event(_, _), Quit);

        Widgets {
            label: label,
            window: window,
        }
    }
}

impl Widget<Msg> for Win {
    type Container = Window;

    fn container(&self) -> &Self::Container {
        &self.widgets.window
    }

    fn new(relm: Relm<Msg>) -> Self {
        let widgets = Self::view(&relm);
        let win = Win {
            relm: relm,
            widgets: widgets,
        };
        win.subscriptions();
        win
    }

    fn update(&mut self, event: Msg) {
        match event {
            Tick(()) => {
                let time = Local::now();
                self.widgets.label.set_text(&format!("{}", time.format("%H:%M:%S")));
            },
            Quit => self.relm.exec(QuitFuture),
        }
    }
}

fn main() {
    Relm::run::<Win>().unwrap();
}
