/*
 * Copyright (c) 2017-2020 Boucher, Antoni <bouanto@zoho.com>
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
    ButtonExt,
    ContainerExt,
    EditableSignals,
    Entry,
    EntryExt,
    Inhibit,
    Label,
    LabelExt,
    WidgetExt,
    Window,
    WindowType,
};
use gtk::Orientation::{Horizontal, Vertical};
use relm::{
    connect,
    Component,
    ContainerWidget,
    Relm,
    Update,
    Widget,
    WidgetTest,
};
use relm_derive::Msg;

use self::CounterMsg::*;
use self::Msg::*;
use self::TextMsg::*;

struct TextModel {
    content: String,
}

#[derive(Msg)]
enum TextMsg {
    Change,
}

struct Text {
    input: Entry,
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
            Change => {
                self.model.content = self.input.get_text().chars().rev().collect();
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
        label.set_widget_name("label");
        vbox.add(&label);

        connect!(relm, input, connect_changed(_), Change);

        Text {
            input,
            label,
            model,
            vbox,
        }
    }
}

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
    type Model = Model;
    type ModelParam = ();
    type Msg = CounterMsg;

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

        let plus_button = Button::with_label("+");
        plus_button.set_widget_name("inc_button");
        vbox.add(&plus_button);

        let counter_label = Label::new(Some("0"));
        counter_label.set_widget_name("label");
        vbox.add(&counter_label);

        let minus_button = Button::with_label("-");
        vbox.add(&minus_button);

        connect!(relm, plus_button, connect_clicked(_), Increment);
        connect!(relm, minus_button, connect_clicked(_), Decrement);

        Counter {
            counter_label,
            model,
            vbox,
        }
    }
}

#[derive(Msg)]
enum Msg {
    Quit,
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
    text: gtk::Box,
    window: Window,
}

struct Win {
    _components: Components,
    widgets: Widgets,
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
            Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Self>, _model: ()) -> Win {
        let window = Window::new(WindowType::Toplevel);

        let hbox = gtk::Box::new(Horizontal, 0);

        let counter1 = hbox.add_widget::<Counter>(());
        let counter2 = hbox.add_widget::<Counter>(());
        let text = hbox.add_widget::<Text>(());
        window.add(&hbox);

        window.show_all();

        connect!(relm, window, connect_delete_event(_, _), return (Some(Quit), Inhibit(false)));

        Win {
            widgets: Widgets {
                counter1: counter1.widget().clone(),
                counter2: counter2.widget().clone(),
                text: text.widget().clone(),
                window,
            },
            _components: Components {
                _counter1: counter1,
                _counter2: counter2,
                _text: text,
            },
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
    use gtk::{Button, Entry, Label, LabelExt};

    use gtk_test::{assert_text, find_child_by_name};
    use relm_test::{click, enter_keys};

    use crate::Win;

    #[test]
    fn model_params() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let counter1 = &widgets.counter1;
        let text = &widgets.text;
        let inc_button1: Button = find_child_by_name(counter1, "inc_button").expect("button");
        let label1: Label = find_child_by_name(counter1, "label").expect("label");
        let counter2 = &widgets.counter2;
        let inc_button2: Button = find_child_by_name(counter2, "inc_button").expect("button");
        let label2: Label = find_child_by_name(counter2, "label").expect("label");
        let entry: Entry = find_child_by_name(text, "entry").expect("entry");
        let text_label: Label = find_child_by_name(text, "label").expect("label");

        assert_text!(label1, 0);

        click(&inc_button1);
        assert_text!(label1, 1);

        assert_text!(label2, 0);

        click(&inc_button2);
        assert_text!(label2, 1);

        assert_text!(text_label, "");

        enter_keys(&entry, "test");
        assert_text!(text_label, "tset");
    }
}
