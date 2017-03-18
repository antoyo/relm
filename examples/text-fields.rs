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

use gtk::{ContainerExt, EditableSignals, Entry, EntryExt, Label, WidgetExt, Window, WindowType};
use gtk::Orientation::Vertical;
use relm::{Relm, RemoteRelm, Widget};

use self::Msg::*;

#[derive(Clone)]
struct Model {
    content: String,
}

#[derive(Msg)]
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
    widgets: Widgets,
}

impl Win {
    fn view(relm: &RemoteRelm<Msg>) -> Widgets {
        let vbox = gtk::Box::new(Vertical, 0);

        let input = Entry::new();
        vbox.add(&input);

        let label = Label::new(None);
        vbox.add(&label);

        let window = Window::new(WindowType::Toplevel);

        window.add(&vbox);

        window.show_all();

        connect!(relm, input, connect_changed(_), Change);
        connect_no_inhibit!(relm, window, connect_delete_event(_, _), Quit);

        Widgets {
            input: input,
            label: label,
            window: window,
        }
    }
}

impl Widget<Msg> for Win {
    type Container = Window;
    type Model = Model;

    fn container(&self) -> &Self::Container {
        &self.widgets.window
    }

    fn new(relm: &RemoteRelm<Msg>) -> (Self, Model) {
        let widgets = Self::view(relm);
        let model = Model {
            content: String::new(),
        };
        let window = Win {
            widgets: widgets,
        };
        (window, model)
    }

    fn update(&mut self, event: Msg, model: &mut Model) {
        match event {
            Change => {
                model.content = self.widgets.input.get_text().unwrap().chars().rev().collect();
                self.widgets.label.set_text(&model.content);
            },
            Quit => gtk::main_quit(),
        }
    }
}

fn main() {
    Relm::run::<Win>().unwrap();
}
