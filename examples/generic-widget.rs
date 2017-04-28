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

use std::fmt::Display;
use std::marker::PhantomData;

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
use relm::{Component, ContainerWidget, RemoteRelm, Widget};

use self::CounterMsg::*;
use self::Msg::*;

trait IncDec {
    fn dec(&mut self);
    fn inc(&mut self);
}

impl IncDec for i32 {
    fn dec(&mut self) {
        *self -= 1;
    }

    fn inc(&mut self) {
        *self += 1;
    }
}

#[derive(Clone)]
struct Model<T> {
    counter: T,
}

#[derive(Msg)]
enum CounterMsg {
    Decrement,
    Increment,
}

#[derive(Clone)]
struct Counter<T> {
    counter_label: Label,
    vbox: gtk::Box,
    _phantom: PhantomData<T>,
}

impl<T: Clone + IncDec + Display> Widget for Counter<T> {
    type Model = Model<T>;
    type ModelParam = T;
    type Msg = CounterMsg;
    type Root = gtk::Box;

    fn model(value: T) -> Self::Model {
        Model {
            counter: value,
        }
    }

    fn root(&self) -> &Self::Root {
        &self.vbox
    }

    fn update(&mut self, event: CounterMsg, model: &mut Self::Model) {
        let label = &self.counter_label;

        match event {
            Decrement => {
                model.counter.dec();
                label.set_text(&model.counter.to_string());
            },
            Increment => {
                model.counter.inc();
                label.set_text(&model.counter.to_string());
            },
        }
    }

    fn view(relm: &RemoteRelm<Self>, model: &Self::Model) -> Self {
        let vbox = gtk::Box::new(Vertical, 0);

        let plus_button = Button::new_with_label("+");
        vbox.add(&plus_button);

        let counter_label = Label::new(Some(model.counter.to_string().as_ref()));
        vbox.add(&counter_label);

        let minus_button = Button::new_with_label("-");
        vbox.add(&minus_button);

        connect!(relm, plus_button, connect_clicked(_), Increment);
        connect!(relm, minus_button, connect_clicked(_), Decrement);

        Counter {
            counter_label: counter_label,
            vbox: vbox,
            _phantom: PhantomData,
        }
    }
}

#[derive(Msg)]
enum Msg {
    Quit,
}

#[derive(Clone)]
struct Win {
    _counter1: Component<Counter<i32>>,
    _counter2: Component<Counter<i32>>,
    window: Window,
}

impl Widget for Win {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;
    type Root = Window;

    fn model(_: ()) -> () {
        ()
    }

    fn root(&self) -> &Self::Root {
        &self.window
    }

    fn update(&mut self, event: Msg, _model: &mut ()) {
        match event {
            Quit => gtk::main_quit(),
        }
    }

    fn view(relm: &RemoteRelm<Self>, _model: &()) -> Win {
        let window = Window::new(WindowType::Toplevel);

        let hbox = gtk::Box::new(Horizontal, 0);

        let counter1 = hbox.add_widget::<Counter<i32>, _>(&relm, 2);
        let counter2 = hbox.add_widget::<Counter<i32>, _>(&relm, 3);
        window.add(&hbox);

        window.show_all();

        connect!(relm, window, connect_delete_event(_, _) (Some(Quit), Inhibit(false)));

        Win {
            _counter1: counter1,
            _counter2: counter2,
            window: window,
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}
