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
use gtk::{Button, ButtonExt, ContainerExt, Label, WidgetExt, Window, WindowType};
use gtk::Orientation::Vertical;
use relm::{Handle, QuitFuture, Relm, UnitFuture, Widget};

use self::Msg::*;

#[derive(Clone, Debug)]
struct Model {
    counter: i32,
}

#[derive(Clone)]
enum Msg {
    Decrement,
    Increment,
    Quit,
}

struct Widgets {
    counter_label: Label,
    minus_button: Button,
    plus_button: Button,
    window: Window,
}

struct Win {
    model: Model,
    widgets: Widgets,
}

impl Win {
    // TODO: create an attribute (or procedural macro) to have the ability to generate a view from
    // a declarative structure.
    fn view() -> Widgets {
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

        Widgets {
            counter_label: counter_label,
            minus_button: minus_button,
            plus_button: plus_button,
            window: window,
        }
    }
}

impl Widget<Msg> for Win {
    fn connect_events(&self, relm: &Relm<Msg>) {
        connect!(relm, self.widgets.plus_button, connect_clicked(_), Increment);
        connect!(relm, self.widgets.minus_button, connect_clicked(_), Decrement);
        connect_no_inhibit!(relm, self.widgets.window, connect_delete_event(_, _), Quit);
    }

    fn new() -> Self {
        Win {
            model: Model {
                counter: 0,
            },
            widgets: Self::view(),
        }
    }

    fn update(&mut self, event: Msg, _handle: Handle) -> UnitFuture {
        let label = &self.widgets.counter_label;

        match event {
            Decrement => {
                self.model.counter -= 1;
                label.set_text(&self.model.counter.to_string());
            },
            Increment => {
                self.model.counter += 1;
                label.set_text(&self.model.counter.to_string());
            },
            Quit => return QuitFuture.boxed(),
        }

        ok(()).boxed()
    }
}

fn main() {
    Relm::run::<Win>().unwrap();
}
