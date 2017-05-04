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

struct TextModel {
    content: String,
}

#[derive(Msg)]
enum TextMsg {
    Change(String),
}

struct Text {
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
            Change(text) => {
                self.model.content = text.chars().rev().collect();
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

        let input2 = input.clone();
        connect!(relm, input, connect_changed(_), Change(input2.get_text().unwrap()));

        Rc::new(RefCell::new(Text {
            label: label,
            model,
            vbox: vbox,
        }))
    }
}

struct CounterModel {
    counter: i32,
}

#[derive(Msg)]
enum CounterMsg {
    Decrement,
    Increment,
}

struct Counter {
    counter_label: Label,
    model: CounterModel,
    vbox: gtk::Box,
}

impl Widget for Counter {
    type Model = CounterModel;
    type ModelParam = ();
    type Msg = CounterMsg;
    type Root = gtk::Box;

    fn model(_: ()) -> CounterModel {
        CounterModel {
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

    fn view(relm: &Relm<Self>, model: CounterModel) -> Rc<RefCell<Self>> {
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
            counter_label: counter_label,
            model,
            vbox: vbox,
        }))
    }
}

struct Model {
    counter: i32,
}

#[derive(Msg)]
enum Msg {
    TextChange(String),
    Quit,
}

struct Win {
    counter1: Component<Counter>,
    counter2: Component<Counter>,
    label: Label,
    model: Model,
    text: Component<Text>,
    window: Window,
}

impl Widget for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;
    type Root = Window;

    fn model(_: ()) -> Model {
        Model {
            counter: 0,
        }
    }

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn update(&mut self, event: Msg) {
        match event {
            TextChange(text) => {
                println!("{}", text);
                self.model.counter += 1;
                self.label.set_text(&self.model.counter.to_string());
            },
            Quit => gtk::main_quit(),
        }
    }

    fn view(relm: &Relm<Self>, model: Model) -> Rc<RefCell<Self>> {
        let window = Window::new(WindowType::Toplevel);

        let hbox = gtk::Box::new(Horizontal, 0);

        let button = Button::new_with_label("Decrement");
        hbox.add(&button);

        let label = Label::new(None);

        let counter1 = hbox.add_widget::<Counter, _>(&relm, ());
        let counter2 = hbox.add_widget::<Counter, _>(&relm, ());
        let text = hbox.add_widget::<Text, _>(&relm, ());
        hbox.add(&label);

        let win = Rc::new(RefCell::new(Win {
            counter1,
            counter2,
            label: label,
            model,
            text,
            window: window,
        }));

        {
            let win_clone = Rc::downgrade(&win);
            let Win { ref counter1, ref counter2, ref text, ref window, .. } = *win.borrow();
            connect!(text@Change(text), relm, TextChange(text));
            connect!(text@Change(_), counter1, with win_clone win_clone.inc());
            connect!(counter1@Increment, counter2, Increment);
            connect!(button, connect_clicked(_), counter1, Decrement);

            window.add(&hbox);

            window.show_all();

            connect!(relm, window, connect_delete_event(_, _) (Some(Quit), Inhibit(false)));
        }

        win
    }
}

impl Win {
    fn inc(&self) -> CounterMsg {
        Increment
    }
}

fn main() {
    Win::run(()).unwrap();
}
