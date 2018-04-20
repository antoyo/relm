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

extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate relm_test;

use std::fmt::Display;
use std::marker::PhantomData;

use gtk::{
    Button,
    ButtonExt,
    ContainerExt,
    Inhibit,
    Label,
    LabelExt,
    WidgetExt,
    Window,
    WindowType,
};
use gtk::Orientation::{Horizontal, Vertical};
use relm::{
    Component,
    ContainerWidget,
    Relm,
    Update,
    Widget,
    WidgetTest,
};

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

impl IncDec for i64 {
    fn dec(&mut self) {
        *self -= 1;
    }

    fn inc(&mut self) {
        *self += 1;
    }
}

struct Model<S, T> {
    counter1: S,
    _counter2: T,
}

#[derive(Msg)]
enum CounterMsg {
    Decrement,
    Increment,
}

struct Counter<S, T> {
    counter_label: Label,
    model: Model<S, T>,
    vbox: gtk::Box,
    _phantom1: PhantomData<S>,
    _phantom2: PhantomData<T>,
}

impl<S: Clone + Display + IncDec, T: Clone + Display + IncDec> Update for Counter<S, T> {
    type Model = Model<S, T>;
    type ModelParam = (S, T);
    type Msg = CounterMsg;

    fn model(_: &Relm<Self>, (value1, value2): (S, T)) -> Self::Model {
        Model {
            counter1: value1,
            _counter2: value2,
        }
    }

    fn update(&mut self, event: CounterMsg) {
        let label = &self.counter_label;

        match event {
            Decrement => {
                self.model.counter1.dec();
                label.set_text(&self.model.counter1.to_string());
            },
            Increment => {
                self.model.counter1.inc();
                label.set_text(&self.model.counter1.to_string());
            },
        }
    }
}

impl<S: Clone + Display + IncDec, T: Clone + Display + IncDec> Widget for Counter<S, T> {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.vbox.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let vbox = gtk::Box::new(Vertical, 0);

        let plus_button = Button::new_with_label("+");
        vbox.add(&plus_button);

        let counter_label = Label::new(Some(model.counter1.to_string().as_ref()));
        counter_label.set_name("label");
        vbox.add(&counter_label);

        let minus_button = Button::new_with_label("-");
        vbox.add(&minus_button);

        connect!(relm, plus_button, connect_clicked(_), Increment);
        connect!(relm, minus_button, connect_clicked(_), Decrement);

        Counter {
            counter_label: counter_label,
            model,
            vbox: vbox,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }
}

#[derive(Msg)]
enum Msg {
    Quit,
}

#[derive(Clone)]
struct Win {
    counter1: Component<Counter<i32, i64>>,
    counter2: Component<Counter<i32, i64>>,
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
            Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, _model: ()) -> Win {
        let window = Window::new(WindowType::Toplevel);

        let hbox = gtk::Box::new(Horizontal, 0);

        let counter1 = hbox.add_widget::<Counter<i32, i64>>((2, 3));
        let counter2 = hbox.add_widget::<Counter<i32, i64>>((3, 4));
        window.add(&hbox);

        window.show_all();

        connect!(relm, window, connect_delete_event(_, _), return (Some(Quit), Inhibit(false)));

        Win {
            counter1: counter1,
            counter2: counter2,
            window: window,
        }
    }
}

impl WidgetTest for Win {
    type Widgets = Win;

    fn get_widgets(&self) -> Self::Widgets {
        self.clone()
    }
}

fn main() {
    Win::run(()).unwrap();
}

#[cfg(test)]
mod tests {
    use gtk::{Label, LabelExt};

    use relm;
    use relm_test::find_child_by_name;

    use Win;

    #[test]
    fn model_params() {
        let (_component, widgets) = relm::init_test::<Win>(()).unwrap();
        let label1: Label = find_child_by_name(widgets.counter1.widget(), "label").expect("label");
        let label2: Label = find_child_by_name(widgets.counter2.widget(), "label").expect("label");

        assert_text!(label1, 2);
        assert_text!(label2, 3);
    }
}
