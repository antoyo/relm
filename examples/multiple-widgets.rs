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

use std::cell::RefCell;
use std::rc::Rc;

use gtk::{
    Button,
    ButtonExt,
    ContainerExt,
    EditableSignals,
    Entry,
    EntryExt,
    Inhibit,
    Label,
    WidgetExt,
    Window,
    WindowType,
};
use gtk::Orientation::{Horizontal, Vertical};
use relm::{Component, ContainerWidget, Relm, Widget};

use self::CounterMsg::*;
use self::Msg::*;
use self::TextMsg::*;

#[derive(Clone)]
struct TextModel {
    content: String,
}

#[derive(Msg)]
enum TextMsg {
    Change,
}

#[derive(Clone)]
struct Text {
    input: Entry,
    label: Label,
    model: TextModel,
    vbox: gtk::Box,
}

impl Widget for Text {
    type Model = TextModel;
    type ModelParam = ();
    type Msg = TextMsg;
    type Root = gtk::Box;

    fn model(_: ()) -> TextModel {
        TextModel {
            content: String::new(),
        }
    }

    fn root(&self) -> Self::Root {
        self.vbox.clone()
    }

    fn update(&mut self, event: TextMsg) {
        match event {
            Change => {
                self.model.content = self.input.get_text().unwrap().chars().rev().collect();
                self.label.set_text(&self.model.content);
            },
        }
    }

    fn view(relm: &Relm<Self>, model: TextModel) -> Rc<RefCell<Self>> {
        let vbox = gtk::Box::new(Vertical, 0);

        let input = Entry::new();
        vbox.add(&input);

        let label = Label::new(None);
        vbox.add(&label);

        connect!(relm, input, connect_changed(_), Change);

        Rc::new(RefCell::new(Text {
            input,
            label,
            model,
            vbox,
        }))
    }
}

#[derive(Clone)]
struct Model {
    counter: i32,
}

#[derive(Msg)]
enum CounterMsg {
    Decrement,
    Increment,
}

#[derive(Clone)]
struct Counter {
    counter_label: Label,
    model: Model,
    vbox: gtk::Box,
}

impl Widget for Counter {
    type Model = Model;
    type ModelParam = ();
    type Msg = CounterMsg;
    type Root = gtk::Box;

    fn model(_: ()) -> Model {
        Model {
            counter: 0,
        }
    }

    fn root(&self) -> Self::Root {
        self.vbox.clone()
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

    fn view(relm: &Relm<Self>, model: Model) -> Rc<RefCell<Self>> {
        let vbox = gtk::Box::new(Vertical, 0);

        let plus_button = Button::new_with_label("+");
        vbox.add(&plus_button);

        let counter_label = Label::new("0");
        vbox.add(&counter_label);

        let minus_button = Button::new_with_label("-");
        vbox.add(&minus_button);

        connect!(relm, plus_button, connect_clicked(_), Increment);
        connect!(relm, minus_button, connect_clicked(_), Decrement);

        Rc::new(RefCell::new(Counter {
            counter_label,
            model,
            vbox,
        }))
    }
}

#[derive(Msg)]
enum Msg {
    Quit,
}

#[derive(Clone)]
struct Win {
    _counter1: Component<Counter>,
    _counter2: Component<Counter>,
    _text: Component<Text>,
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

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn update(&mut self, event: Msg) {
        match event {
            Quit => gtk::main_quit(),
        }
    }

    fn view(relm: &Relm<Self>, _model: ()) -> Rc<RefCell<Win>> {
        let window = Window::new(WindowType::Toplevel);

        let hbox = gtk::Box::new(Horizontal, 0);

        let counter1 = hbox.add_widget::<Counter, _>(&relm, ());
        let counter2 = hbox.add_widget::<Counter, _>(&relm, ());
        let text = hbox.add_widget::<Text, _>(&relm, ());
        window.add(&hbox);

        window.show_all();

        connect!(relm, window, connect_delete_event(_, _) (Some(Quit), Inhibit(false)));

        Rc::new(RefCell::new(Win {
            _counter1: counter1,
            _counter2: counter2,
            _text: text,
            window: window,
        }))
    }
}

fn main() {
    Win::run(()).unwrap();
}
