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

use gtk::{
    Button,
    ButtonExt,
    ContainerExt,
    Inhibit,
    Label,
    WidgetExt,
    Window,
    WindowType,
};
use gtk::Orientation::{Horizontal, Vertical};
use relm::{Component, ContainerWidget, Relm, Update, Widget};

use self::CounterMsg::*;
use self::Msg::*;

struct Model {
    counter: i32,
}

#[derive(Msg)]
enum CounterMsg {
    Decrement,
    Increment,
}

struct Counter {
    counter_label: Label,
    model: Model,
    vbox: gtk::Box,
}

impl Update for Counter {
    type Msg = CounterMsg;
    type Model = Model;
    type ModelParam = ();

    fn model(_: &Relm<Self>, _: ()) -> Model {
        Model {
            counter: 0,
        }
    }

    fn update(&mut self, event: CounterMsg) {
        let label = &self.counter_label;

        match event {
            Decrement => {
                self.model.counter -= 1;
                label.set_text(&self.model.counter.to_string());
            },
            Increment => {
                self.model.counter += 1;
                label.set_text(&self.model.counter.to_string());
            },
        }
    }
}

impl Widget for Counter {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.vbox.clone()
    }

    fn view(relm: &Relm<Self>, model: Model) -> Self {
        let vbox = gtk::Box::new(Vertical, 0);

        let plus_button = Button::new_with_label("+");
        vbox.add(&plus_button);

        let counter_label = Label::new("0");
        vbox.add(&counter_label);

        let minus_button = Button::new_with_label("-");
        vbox.add(&minus_button);

        vbox.show_all();

        connect!(relm, plus_button, connect_clicked(_), Increment);
        connect!(relm, minus_button, connect_clicked(_), Decrement);

        Counter {
            counter_label: counter_label,
            model,
            vbox: vbox,
        }
    }
}

#[derive(Msg)]
enum Msg {
    Add,
    Quit,
    Remove,
}

struct Win {
    counters: Vec<Component<Counter>>,
    hbox: gtk::Box,
    relm: Relm<Win>,
    window: Window,
}

impl Update for Win {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> () {
        ()
    }

    fn update(&mut self, event: Msg) {
        match event {
            Add => {
                let widget = self.hbox.add_widget::<Counter, _>(&self.relm, ());
                self.counters.push(widget);
            },
            Quit => gtk::main_quit(),
            Remove => {
                if let Some(counter) = self.counters.pop() {
                    self.hbox.remove_widget(counter);
                }
            },
        }
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, _model: ()) -> Self {
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
        connect!(relm, window, connect_delete_event(_, _), return (Some(Quit), Inhibit(false)));

        Win {
            counters: vec![],
            hbox: hbox,
            relm: relm.clone(),
            window: window,
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}
