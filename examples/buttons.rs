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

use gtk::{Button, ButtonExt, ContainerExt, Label, WidgetExt, Window, WindowType};
use gtk::Orientation::Vertical;
use relm::{Relm, RemoteRelm, Widget};

use self::Msg::*;

#[derive(Clone)]
struct Model {
    counter: i32,
}

#[derive(Msg)]
enum Msg {
    Decrement,
    Increment,
    Quit,
}

struct Win {
    counter_label: Label,
    window: Window,
}

impl Widget<Msg> for Win {
    type Container = Window;
    type Model = Model;

    fn container(&self) -> &Self::Container {
        &self.window
    }

    fn model() -> Model {
        Model {
            counter: 0,
        }
    }

    fn update(&mut self, event: Msg, model: &mut Model) {
        let label = &self.counter_label;

        match event {
            Decrement => {
                model.counter -= 1;
                label.set_text(&model.counter.to_string());
            },
            Increment => {
                model.counter += 1;
                label.set_text(&model.counter.to_string());
            },
            Quit => gtk::main_quit(),
        }
    }

    // TODO: create an attribute (or procedural macro) to have the ability to generate a view from
    // a declarative structure.
    fn view(relm: RemoteRelm<Msg>, _model: &Self::Model) -> Self {
        let vbox = gtk::Box::new(Vertical, 0);

        let plus_button = Button::new_with_label("+");
        vbox.add(&plus_button);

        let counter_label = Label::new(Some("0"));
        vbox.add(&counter_label);

        let minus_button = Button::new_with_label("-");
        vbox.add(&minus_button);

        let window = Window::new(WindowType::Toplevel);

        window.add(&vbox);

        window.show_all();

        connect!(relm, plus_button, connect_clicked(_), Increment);
        connect!(relm, minus_button, connect_clicked(_), Decrement);
        connect_no_inhibit!(relm, window, connect_delete_event(_, _), Quit);

        Win {
            counter_label: counter_label,
            window: window,
        }
    }
}

fn main() {
    Relm::run::<Win>().unwrap();
}
