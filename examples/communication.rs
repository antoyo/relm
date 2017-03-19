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

use gtk::{Button, ButtonExt, ContainerExt, EditableSignals, Entry, EntryExt, Label, WidgetExt, Window, WindowType};
use gtk::Orientation::{Horizontal, Vertical};
use relm::{Component, ContainerWidget, Relm, RemoteRelm, Widget};

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

struct TextWidgets {
    input: Entry,
    label: Label,
    vbox: gtk::Box,
}

struct Text {
    widgets: TextWidgets,
}

impl Text {
    fn view(relm: &RemoteRelm<TextMsg>) -> TextWidgets {
        let vbox = gtk::Box::new(Vertical, 0);

        let input = Entry::new();
        vbox.add(&input);

        let label = Label::new(None);
        vbox.add(&label);

        connect!(relm, input, connect_changed(_), Change);

        TextWidgets {
            input: input,
            label: label,
            vbox: vbox,
        }
    }
}

impl Widget<TextMsg> for Text {
    type Container = gtk::Box;
    type Model = TextModel;

    fn container(&self) -> &Self::Container {
        &self.widgets.vbox
    }

    fn new(relm: RemoteRelm<TextMsg>) -> (Self, TextModel) {
        let widgets = Self::view(&relm);
        let widget = Text {
            widgets: widgets,
        };
        let model = TextModel {
            content: String::new(),
        };
        (widget, model)
    }

    fn update(&mut self, event: TextMsg, model: &mut TextModel) {
        match event {
            Change => {
                model.content = self.widgets.input.get_text().unwrap().chars().rev().collect();
                self.widgets.label.set_text(&model.content);
            },
        }
    }
}

#[derive(Clone)]
struct CounterModel {
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
    type Model = CounterModel;

    fn container(&self) -> &Self::Container {
        &self.widgets.vbox
    }

    fn new(relm: RemoteRelm<CounterMsg>) -> (Self, CounterModel) {
        let widgets = Self::view(&relm);
        let widget = Counter {
            widgets: widgets,
        };
        let model = CounterModel {
            counter: 0,
        };
        (widget, model)
    }

    fn update(&mut self, event: CounterMsg, model: &mut CounterModel) {
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

struct Model {
    counter: i32,
}

#[derive(Msg)]
enum Msg {
    TextChange,
    Quit,
}

struct Widgets {
    _counter1: Component<CounterModel, CounterMsg, gtk::Box>,
    _counter2: Component<CounterModel, CounterMsg, gtk::Box>,
    label: Label,
    _text: Component<TextModel, TextMsg, gtk::Box>,
    window: Window,
}

struct Win {
    widgets: Widgets,
}

impl Win {
    fn view(relm: &RemoteRelm<Msg>) -> Widgets {
        let window = Window::new(WindowType::Toplevel);

        let hbox = gtk::Box::new(Horizontal, 0);

        let button = Button::new_with_label("Decrement");
        hbox.add(&button);

        let counter1 = hbox.add_widget::<Counter, _, _>(relm);
        let counter2 = hbox.add_widget::<Counter, _, _>(relm);
        let text = hbox.add_widget::<Text, _, _>(relm);
        connect!(text, Change, relm, TextChange);
        connect!(text, Change, counter1, Increment);
        connect!(counter1, Increment, counter2, Decrement);
        connect!(button, connect_clicked(_), counter1, Decrement);

        let label = Label::new(None);
        hbox.add(&label);

        window.add(&hbox);

        window.show_all();

        connect_no_inhibit!(relm, window, connect_delete_event(_, _), Quit);

        Widgets {
            _counter1: counter1,
            _counter2: counter2,
            label: label,
            _text: text,
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

    fn new(relm: RemoteRelm<Msg>) -> (Self, Model) {
        let widgets = Self::view(&relm);
        let window = Win {
            widgets: widgets,
        };
        let model = Model {
            counter: 0,
        };
        (window, model)
    }

    fn update(&mut self, event: Msg, model: &mut Model) {
        match event {
            TextChange => {
                model.counter += 1;
                self.widgets.label.set_text(&model.counter.to_string());
            },
            Quit => gtk::main_quit(),
        }
    }
}

fn main() {
    Relm::run::<Win>().unwrap();
}
