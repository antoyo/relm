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

use std::fmt::Display;

use gtk::{
    Button,
    Label,
    Window,
    WindowType,
    prelude::ButtonExt,
    prelude::ContainerExt,
    prelude::LabelExt,
    prelude::WidgetExt,
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
use glib::Propagation;

trait IncDec {
    fn dec(&mut self);
    fn identity() -> Self;
    fn inc(&mut self);
}

impl IncDec for i32 {
    fn dec(&mut self) {
        *self -= 1;
    }

    fn identity() -> Self {
        1
    }

    fn inc(&mut self) {
        *self += 1;
    }
}

struct Model<T> {
    counter: T,
}

#[derive(Msg)]
enum CounterMsg<T> {
    Decrement,
    Increment(T),
}

struct Counter<T> {
    counter_label: Label,
    model: Model<T>,
    vbox: gtk::Box,
}

impl<T: Clone + IncDec + Display + 'static> Update for Counter<T> {
    type Model = Model<T>;
    type ModelParam = T;
    type Msg = CounterMsg<T>;

    fn model(_: &Relm<Self>, value: T) -> Self::Model {
        Model {
            counter: value,
        }
    }

    fn update(&mut self, event: CounterMsg<T>) {
        let label = &self.counter_label;

        match event {
            Decrement => {
                self.model.counter.dec();
                label.set_text(&self.model.counter.to_string());
            },
            Increment(_) => {
                self.model.counter.inc();
                label.set_text(&self.model.counter.to_string());
            },
        }
    }
}

impl<T: Clone + IncDec + Display + 'static> Widget for Counter<T> {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.vbox.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let vbox = gtk::Box::new(Vertical, 0);

        let plus_button = Button::with_label("+");
        plus_button.set_widget_name("inc_button");
        vbox.add(&plus_button);

        let counter_label = Label::new(Some(model.counter.to_string().as_ref()));
        counter_label.set_widget_name("label");
        vbox.add(&counter_label);

        let minus_button = Button::with_label("-");
        minus_button.set_widget_name("dec_button");
        vbox.add(&minus_button);

        connect!(relm, plus_button, connect_clicked(_), Increment(T::identity()));
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
    Quit,
}

struct Components {
    _counter1: Component<Counter<i32>>,
    _counter2: Component<Counter<i32>>,
}

#[derive(Clone)]
struct Widgets {
    counter1: gtk::Box,
    counter2: gtk::Box,
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

        let counter1 = hbox.add_widget::<Counter<i32>>(2);
        let counter2 = hbox.add_widget::<Counter<i32>>(3);
        window.add(&hbox);

        window.show_all();

        connect!(relm, window, connect_delete_event(_, _), return (Some(Quit), Propagation::Proceed));

        Win {
            widgets: Widgets {
                counter1: counter1.widget().clone(),
                counter2: counter2.widget().clone(),
                window: window,
            },
            _components: Components {
                _counter1: counter1,
                _counter2: counter2,
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
    use gtk::{Button, Label, prelude::LabelExt};

    use gtk_test::{assert_text, click, find_child_by_name};

    use crate::Win;

    #[test]
    fn widget_position() {
        let (_components, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let inc_button1: Button = find_child_by_name(&widgets.counter1, "inc_button").expect("inc button");
        let dec_button1: Button = find_child_by_name(&widgets.counter1, "dec_button").expect("dec button");
        let label1: Label = find_child_by_name(&widgets.counter1, "label").expect("label");
        let inc_button2: Button = find_child_by_name(&widgets.counter2, "inc_button").expect("inc button");
        let dec_button2: Button = find_child_by_name(&widgets.counter2, "dec_button").expect("dec button");
        let label2: Label = find_child_by_name(&widgets.counter2, "label").expect("label");

        assert_text!(label1, 2);

        click(&inc_button1);
        assert_text!(label1, 3);

        click(&inc_button1);
        assert_text!(label1, 4);

        click(&dec_button1);
        assert_text!(label1, 3);

        assert_text!(label2, 3);

        click(&inc_button2);
        assert_text!(label2, 4);

        click(&inc_button2);
        assert_text!(label2, 5);

        click(&dec_button2);
        assert_text!(label2, 4);
    }
}
