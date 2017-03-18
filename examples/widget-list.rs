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
use gtk::Orientation::{Horizontal, Vertical};
use relm::{Component, ContainerWidget, Relm, RemoteRelm, Widget};

use self::CounterMsg::*;
use self::Msg::*;

#[derive(Clone)]
struct Model {
    counter: i32,
}

#[derive(Msg)]
enum CounterMsg {
    Decrement,
    Increment,
}

struct Counter {
    widgets: CounterWidgets,
}

impl Counter {
    fn view(relm: &RemoteRelm<CounterMsg>) -> CounterWidgets {
        let vbox = gtk::Box::new(Vertical, 0);

        let plus_button = Button::new_with_label("+");
        vbox.add(&plus_button);

        let counter_label = Label::new(Some("0"));
        vbox.add(&counter_label);

        let minus_button = Button::new_with_label("-");
        vbox.add(&minus_button);

        connect!(relm, plus_button, connect_clicked(_), Increment);
        connect!(relm, minus_button, connect_clicked(_), Decrement);

        CounterWidgets {
            counter_label: counter_label,
            vbox: vbox,
        }
    }
}

impl Widget<CounterMsg> for Counter {
    type Container = gtk::Box;
    type Model = Model;

    fn container(&self) -> &Self::Container {
        &self.widgets.vbox
    }

    fn new(relm: RemoteRelm<CounterMsg>) -> (Self, Model) {
        let widgets = Self::view(&relm);
        let model = Model {
            counter: 0,
        };
        let widget = Counter {
            widgets: widgets,
        };
        (widget, model)
    }

    fn update(&mut self, event: CounterMsg, model: &mut Model) {
        let label = &self.widgets.counter_label;

        match event {
            Decrement => {
                model.counter -= 1;
                label.set_text(&model.counter.to_string());
            },
            Increment => {
                model.counter += 1;
                label.set_text(&model.counter.to_string());
            },
        }
    }
}

struct CounterWidgets {
    counter_label: Label,
    vbox: gtk::Box,
}

#[derive(Msg)]
enum Msg {
    Add,
    Quit,
    Remove,
}

struct Widgets {
    counters: Vec<Component<Model, CounterMsg, gtk::Box>>,
    hbox: gtk::Box,
    window: Window,
}

struct Win {
    relm: RemoteRelm<Msg>,
    widgets: Widgets,
}

impl Win {
    fn view(relm: &RemoteRelm<Msg>) -> Widgets {
        let window = Window::new(WindowType::Toplevel);

        let vbox = gtk::Box::new(Vertical, 0);
        let add_button = Button::new_with_label("Add");
        let remove_button = Button::new_with_label("Remove");

        let hbox = gtk::Box::new(Horizontal, 0);
        vbox.add(&hbox);

        vbox.add(&add_button);
        vbox.add(&remove_button);

        window.add(&vbox);

        window.show_all();

        connect!(relm, add_button, connect_clicked(_), Add);
        connect!(relm, remove_button, connect_clicked(_), Remove);
        connect_no_inhibit!(relm, window, connect_delete_event(_, _), Quit);

        Widgets {
            counters: vec![],
            hbox: hbox,
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
        let window = Win {
            relm: relm,
            widgets: widgets,
        };
        (window, ())
    }

    fn update(&mut self, event: Msg, _model: &mut ()) {
        match event {
            Add => {
                let widget = self.widgets.hbox.add_widget::<Counter, _, _>(&self.relm);
                self.widgets.counters.push(widget);
            },
            Quit => gtk::main_quit(),
            Remove => {
                if let Some(counter) = self.widgets.counters.pop() {
                    self.widgets.hbox.remove_widget(counter);
                }
            },
        }
    }
}

fn main() {
    Relm::run::<Win>().unwrap();
}
