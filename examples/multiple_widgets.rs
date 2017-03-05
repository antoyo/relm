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
use gtk::{Button, ButtonExt, ContainerExt, EditableSignals, Entry, EntryExt, Label, WidgetExt, Window, WindowType};
use gtk::Orientation::{Horizontal, Vertical};
use relm::{AddWidget, EventStream, Handle, QuitFuture, Relm, UnitFuture, Widget};

use self::CounterMsg::*;
use self::Msg::*;
use self::TextMsg::*;

#[derive(Clone, Debug)]
struct TextModel {
    content: String,
}

#[derive(Clone)]
enum TextMsg {
    Change,
}

struct TextWidgets {
    input: Entry,
    label: Label,
    vbox: gtk::Box,
}

struct Text {
    model: TextModel,
    widgets: TextWidgets,
}

impl Text {
    fn view() -> TextWidgets {
        let vbox = gtk::Box::new(Vertical, 0);

        let input = Entry::new();
        vbox.add(&input);

        let label = Label::new(None);
        vbox.add(&label);

        TextWidgets {
            input: input,
            label: label,
            vbox: vbox,
        }
    }
}

impl Widget<TextMsg> for Text {
    type Container = gtk::Box;

    fn connect_events(&self, stream: &EventStream<TextMsg>) {
        connect!(stream, self.widgets.input, connect_changed(_), Change);
    }

    fn container(&self) -> &Self::Container {
        &self.widgets.vbox
    }

    fn new(_handle: Handle, _stream: EventStream<TextMsg>) -> Self {
        Text {
            model: TextModel {
                content: String::new(),
            },
            widgets: Self::view(),
        }
    }

    fn update(&mut self, event: TextMsg) -> UnitFuture {
        match event {
            Change => {
                self.model.content = self.widgets.input.get_text().unwrap().chars().rev().collect();
                self.widgets.label.set_text(&self.model.content);
            },
        }

        ok(()).boxed()
    }
}

#[derive(Clone, Debug)]
struct Model {
    counter: i32,
}

#[derive(Clone)]
enum CounterMsg {
    Decrement,
    Increment,
}

struct Counter {
    model: Model,
    widgets: CounterWidgets,
}

impl Counter {
    fn view() -> CounterWidgets {
        let vbox = gtk::Box::new(Vertical, 0);

        let plus_button = Button::new_with_label("+");
        vbox.add(&plus_button);

        let counter_label = Label::new(Some("0"));
        vbox.add(&counter_label);

        let minus_button = Button::new_with_label("-");
        vbox.add(&minus_button);

        CounterWidgets {
            counter_label: counter_label,
            minus_button: minus_button,
            plus_button: plus_button,
            vbox: vbox,
        }
    }
}

impl Widget<CounterMsg> for Counter {
    type Container = gtk::Box;

    fn connect_events(&self, stream: &EventStream<CounterMsg>) {
        connect!(stream, self.widgets.plus_button, connect_clicked(_), Increment);
        connect!(stream, self.widgets.minus_button, connect_clicked(_), Decrement);
    }

    fn container(&self) -> &Self::Container {
        &self.widgets.vbox
    }

    fn new(_handle: Handle, _stream: EventStream<CounterMsg>) -> Self {
        Counter {
            model: Model {
                counter: 0,
            },
            widgets: Self::view(),
        }
    }

    fn update(&mut self, event: CounterMsg) -> UnitFuture {
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
        }

        ok(()).boxed()
    }
}

struct CounterWidgets {
    counter_label: Label,
    minus_button: Button,
    plus_button: Button,
    vbox: gtk::Box,
}

#[derive(Clone)]
enum Msg {
    Quit,
}

struct Widgets {
    window: Window,
}

struct Win {
    widgets: Widgets,
}

impl Win {
    fn view(handle: Handle) -> Widgets {
        let window = Window::new(WindowType::Toplevel);

        let hbox = gtk::Box::new(Horizontal, 0);

        hbox.add_widget::<Counter, _>(&handle);
        hbox.add_widget::<Counter, _>(&handle);
        hbox.add_widget::<Text, _>(&handle);

        window.add(&hbox);

        window.show_all();

        Widgets {
            window: window,
        }
    }
}

impl Widget<Msg> for Win {
    type Container = Window;

    fn connect_events(&self, stream: &EventStream<Msg>) {
        connect_no_inhibit!(stream, self.widgets.window, connect_delete_event(_, _), Quit);
    }

    fn container(&self) -> &Self::Container {
        &self.widgets.window
    }

    fn new(handle: Handle, _stream: EventStream<Msg>) -> Self {
        Win {
            widgets: Self::view(handle),
        }
    }

    fn update(&mut self, event: Msg) -> UnitFuture {
        match event {
            Quit => QuitFuture.boxed(),
        }
    }
}

fn main() {
    Relm::run::<Win, _>().unwrap();
}
