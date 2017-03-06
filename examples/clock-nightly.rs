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

#![feature(fn_traits, unboxed_closures)]

extern crate chrono;
extern crate futures;
#[macro_use]
extern crate fns_derive;
extern crate gtk;
#[macro_use]
extern crate relm;
extern crate tokio_core;
extern crate tokio_timer;

use std::time::Duration;

use chrono::Local;
use futures::Future;
use futures::future::ok;
use gtk::{ContainerExt, Label, WidgetExt, Window, WindowType};
use relm::{EventStream, Handle, QuitFuture, Relm, UnitFuture, Widget, connect};
use tokio_timer::Timer;

use self::Msg::*;

#[derive(Clone, Fns)]
enum Msg {
    Quit,
    Tick,
}

struct Widgets {
    label: Label,
    window: Window,
}

struct Win {
    stream: EventStream<Msg>,
    widgets: Widgets,
}

impl Win {
    fn view() -> Widgets {
        let label = Label::new(None);

        let window = Window::new(WindowType::Toplevel);

        window.add(&label);

        window.show_all();

        Widgets {
            label: label,
            window: window,
        }
    }
}

impl Widget<Msg> for Win {
    type Container = Window;

    fn connect_events(&self, stream: &EventStream<Msg>) {
        connect_no_inhibit!(stream, self.widgets.window, connect_delete_event(_, _), Quit);
    }

    fn container(&self) -> &Self::Container {
        &self.widgets.window
    }

    fn new(_handle: Handle, stream: EventStream<Msg>) -> Self {
        let widgets = Self::view();
        Win {
            stream: stream,
            widgets: widgets,
        }
    }

    fn subscriptions(&self) -> Vec<UnitFuture> {
        let timer = Timer::default();
        let stream = timer.interval(Duration::from_secs(1));
        let clock_stream = connect(stream, Tick, &self.stream);
        vec![clock_stream]
    }

    fn update(&mut self, event: Msg) -> UnitFuture {
        match event {
            Tick => {
                let time = Local::now();
                self.widgets.label.set_text(&format!("{}", time.format("%H:%M:%S")));
            },
            Quit => return QuitFuture.boxed(),
        }

        ok(()).boxed()
    }
}

fn main() {
    Relm::run::<Win, _>().unwrap();
}
