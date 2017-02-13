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

extern crate futures;
extern crate gtk;
#[macro_use]
extern crate relm;

use futures::Future;
use futures::future::ok;
use gtk::{ContainerExt, EditableSignals, Entry, EntryExt, Label, WidgetExt, Window, WindowType};
use gtk::Orientation::Vertical;
use relm::{QuitFuture, Relm, UnitFuture, Widget};

use self::Msg::*;

#[derive(Clone, Debug)]
struct Model {
    content: String,
}

#[derive(Clone)]
enum Msg {
    Change,
    Quit,
}

struct Widgets {
    input: Entry,
    label: Label,
    window: Window,
}

struct Win {
    model: Model,
    widgets: Widgets,
}

impl Win {
    fn view() -> Widgets {
        let vbox = gtk::Box::new(Vertical, 0);

        let input = Entry::new();
        vbox.add(&input);

        let label = Label::new(None);
        vbox.add(&label);

        let window = Window::new(WindowType::Toplevel);

        window.add(&vbox);

        window.show_all();

        Widgets {
            input: input,
            label: label,
            window: window,
        }
    }
}

impl Widget<Msg> for Win {
    fn connect_events(&self, relm: &Relm<Msg>) {
        connect!(relm, self.widgets.input, connect_changed(_), Change);
        connect_no_inhibit!(relm, self.widgets.window, connect_delete_event(_, _), Quit);
    }

    fn new() -> Self {
        Win {
            model: Model {
                content: String::new(),
            },
            widgets: Self::view(),
        }
    }

    fn update(&mut self, event: Msg) -> UnitFuture {
        match event {
            Change => {
                self.model.content = self.widgets.input.get_text().unwrap().chars().rev().collect();
                self.widgets.label.set_text(&self.model.content);
            },
            Quit => return QuitFuture.boxed(),
        }

        ok(()).boxed()
    }
}

fn main() {
    Relm::run::<Win>().unwrap();
}
