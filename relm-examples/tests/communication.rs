/*
 * Copyright (c) 2017-2018 Boucher, Antoni <bouanto@zoho.com>
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

use gtk::{
    Button,
    EditableSignals,
    Entry,
    Label,
    Window,
    WindowType,
    prelude::ButtonExt,
    prelude::ContainerExt,
    prelude::EntryExt,
    prelude::LabelExt,
    prelude::WidgetExt,
};
use gtk::Orientation::{Horizontal, Vertical};
use relm_derive::Msg;
use relm::{
    connect,
    Component,
    ContainerWidget,
    Relm,
    Update,
    Widget,
    WidgetTest,
};

use self::CounterMsg::*;
use self::Msg::*;
use self::TextMsg::*;
use glib::Propagation;

struct TextModel {
    content: String,
}

#[derive(Msg)]
enum TextMsg {
    Change(glib::GString),
}

struct Text {
    label: Label,
    model: TextModel,
    vbox: gtk::Box,
}

impl Update for Text {
    type Model = TextModel;
    type ModelParam = ();
    type Msg = TextMsg;

    fn model(_: &Relm<Self>, _: ()) -> TextModel {
        TextModel {
            content: String::new(),
        }
    }

    fn update(&mut self, event: TextMsg) {
        match event {
            Change(text) => {
                self.model.content = text.chars().rev().collect();
                self.label.set_text(&self.model.content);
            },
        }
    }
}

impl Widget for Text {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.vbox.clone()
    }

    fn view(relm: &Relm<Self>, model: TextModel) -> Self {
        let vbox = gtk::Box::new(Vertical, 0);

        let input = Entry::new();
        input.set_widget_name("entry");
        vbox.add(&input);

        let label = Label::new(None);
        label.set_widget_name("text_label");
        vbox.add(&label);

        let input2 = input.clone();
        connect!(relm, input, connect_changed(_), Change(input2.text()));

        Text {
            label: label,
            model,
            vbox: vbox,
        }
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

impl Update for Counter {
    type Model = CounterModel;
    type ModelParam = ();
    type Msg = CounterMsg;

    fn model(_: &Relm<Self>, _: ()) -> CounterModel {
        CounterModel {
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

    fn view(relm: &Relm<Self>, model: CounterModel) -> Self {
        let vbox = gtk::Box::new(Vertical, 0);

        let plus_button = Button::with_label("+");
        plus_button.set_widget_name("inc_button");
        vbox.add(&plus_button);

        let counter_label = Label::new(Some("0"));
        counter_label.set_widget_name("label");
        vbox.add(&counter_label);

        let minus_button = Button::with_label("-");
        minus_button.set_widget_name("dec_button");
        vbox.add(&minus_button);

        connect!(relm, plus_button, connect_clicked(_), Increment);
        connect!(relm, minus_button, connect_clicked(_), Decrement);

        Counter {
            counter_label: counter_label,
            model,
            vbox: vbox,
        }
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
    _components: Components,
    model: Model,
    widgets: Widgets,
}

struct Components {
    _counter1: Component<Counter>,
    _counter2: Component<Counter>,
    _text: Component<Text>,
}

#[derive(Clone)]
struct Widgets {
    counter1: gtk::Box,
    counter2: gtk::Box,
    dec_button: Button,
    label: Label,
    text: gtk::Box,
    window: Window,
}

impl Update for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> Model {
        Model {
            counter: 0,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            TextChange(text) => {
                println!("{}", text);
                self.model.counter += 1;
                self.widgets.label.set_text(&self.model.counter.to_string());
            },
            Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Model) -> Self {
        let window = Window::new(WindowType::Toplevel);

        let hbox = gtk::Box::new(Horizontal, 0);

        let dec_button = Button::with_label("Decrement");
        hbox.add(&dec_button);

        let label = Label::new(None);

        let counter1 = hbox.add_widget::<Counter>(());
        let counter2 = hbox.add_widget::<Counter>(());
        let text = hbox.add_widget::<Text>(());
        hbox.add(&label);

        connect!(text@Change(ref text), relm, TextChange(text.to_string()));
        connect!(text@Change(_), counter1, Increment);
        connect!(counter1@Increment, counter2, Decrement);
        connect!(dec_button, connect_clicked(_), counter1, Decrement);

        window.add(&hbox);

        window.show_all();

        connect!(relm, window, connect_delete_event(_, _), return (Some(Quit), Propagation::Proceed));

        Win {
            widgets: Widgets {
                counter1: counter1.widget().clone(),
                counter2: counter2.widget().clone(),
                dec_button,
                label,
                text: text.widget().clone(),
                window,
            },
            _components: Components {
                _counter1: counter1,
                _counter2: counter2,
                _text: text,
            },
            model,
        }
    }
}

impl WidgetTest for Win {
    type Streams = ();

    fn get_streams(&self) -> Self::Streams {
    }

    type Widgets = Widgets;

    fn get_widgets(&self) -> Self::Widgets {
        self.widgets.clone()
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}

#[cfg(test)]
mod tests {
    use gtk::{Button, Entry, Label, prelude::LabelExt};

    use gtk_test::{assert_text, click, enter_keys, find_child_by_name};

    use crate::Win;

    #[test]
    fn label_change() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let dec_button = &widgets.dec_button;
        let label1: Label = find_child_by_name(&widgets.counter1, "label").expect("label1");
        let inc_button1: Button = find_child_by_name(&widgets.counter1, "inc_button").expect("button1");
        let dec_button1: Button = find_child_by_name(&widgets.counter1, "dec_button").expect("button1");
        let label2: Label = find_child_by_name(&widgets.counter2, "label").expect("label2");
        let label = &widgets.label;
        let entry: Entry = find_child_by_name(&widgets.text, "entry").expect("entry");
        let text_label: Label = find_child_by_name(&widgets.text, "text_label").expect("label");

        assert_text!(label1, 0);

        click(dec_button);
        assert_text!(label1, -1);

        click(dec_button);
        assert_text!(label1, -2);

        assert_text!(label2, 0);

        click(&inc_button1);
        assert_text!(label1, -1);
        assert_text!(label2, -1);

        click(&inc_button1);
        assert_text!(label1, 0);
        assert_text!(label2, -2);

        click(&dec_button1);
        assert_text!(label1, -1);
        assert_text!(label2, -2);

        click(&dec_button1);
        assert_text!(label1, -2);
        assert_text!(label2, -2);

        assert_text!(label, "");

        enter_keys(&entry, "t");
        assert_text!(label, 1);
        assert_text!(label1, -1);
        assert_text!(text_label, "t");

        enter_keys(&entry, "e");
        assert_text!(label, 2);
        assert_text!(label1, 0);
        assert_text!(text_label, "et");
    }
}
