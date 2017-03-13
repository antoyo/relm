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
extern crate relm_core;
extern crate tokio_core;

use std::time::Duration;

use chrono::Local;
use futures::Stream;
use gtk::{Button, ButtonExt, ContainerExt, Inhibit, Label, WidgetExt, Window, WindowType};
use gtk::Orientation::Vertical;
use relm_core::{Core, EventStream, QuitFuture};
use tokio_core::reactor::Interval;

use self::Msg::*;

struct Widgets {
    clock_label: Label,
    counter_label: Label,
}

#[derive(Clone)]
enum Msg {
    Clock,
    Decrement,
    Increment,
    Quit,
}

fn main() {
    gtk::init().unwrap();

    let vbox = gtk::Box::new(Vertical, 0);

    let clock_label = Label::new(None);
    vbox.add(&clock_label);

    let plus_button = Button::new_with_label("+");
    vbox.add(&plus_button);

    let counter_label = Label::new(Some("0"));
    vbox.add(&counter_label);

    let widgets = Widgets {
        clock_label: clock_label,
        counter_label: counter_label,
    };

    let mut core = Core::new().unwrap();

    let window = Window::new(WindowType::Toplevel);
    window.add(&vbox);

    let stream = EventStream::new();

    {
        let stream = stream.clone();
        plus_button.connect_clicked(move |_| {
            stream.emit(Increment);
        });
    }

    let minus_button = Button::new_with_label("-");
    vbox.add(&minus_button);
    {
        let stream = stream.clone();
        minus_button.connect_clicked(move |_| {
            stream.emit(Decrement);
        });
    }

    window.show_all();

    {
        let stream = stream.clone();
        window.connect_delete_event(move |_, _| {
            stream.emit(Quit);
            Inhibit(false)
        });
    }

    let interval = {
        let interval = Interval::new(Duration::from_secs(1), &core.handle()).unwrap();
        let stream = stream.clone();
        interval.map_err(|_| ()).for_each(move |_| {
            stream.emit(Clock);
            Ok(())
        })
    };

    let event_future = {
        let stream = stream.clone();
        let handle = core.handle();
        stream.for_each(move |event| {
            fn adjust(label: &Label, delta: i32) {
                if let Some(text) = label.get_text() {
                    let num: i32 = text.parse().unwrap();
                    let result = num + delta;
                    label.set_text(&result.to_string());
                }
            }

            match event {
                Clock => {
                    let now = Local::now();
                    widgets.clock_label.set_text(&now.format("%H:%M:%S").to_string());
                },
                Decrement => adjust(&widgets.counter_label, -1),
                Increment => adjust(&widgets.counter_label, 1),
                Quit => handle.spawn(QuitFuture),
            }
            Ok(())
        })
    };

    let handle = core.handle();
    handle.spawn(event_future);
    handle.spawn(interval);

    core.run();
}
