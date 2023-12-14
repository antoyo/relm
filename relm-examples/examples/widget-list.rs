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

use std::sync::atomic::{AtomicUsize, Ordering};

use gtk::{
    prelude::ButtonExt,
    prelude::LabelExt,
    prelude::OrientableExt,
    prelude::WidgetExt,
};
use gtk::Orientation::{Horizontal, Vertical};
use relm::{Component, ContainerWidget, Widget};
use relm_derive::{Msg, widget};

use self::CounterMsg::*;
use self::Msg::*;
use glib::Propagation;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct CounterModel {
    counter: i32,
}

#[derive(Msg)]
pub enum CounterMsg {
    Decrement,
    Increment,
}

#[widget]
impl Widget for Counter {
    fn model() -> CounterModel {
        COUNTER.fetch_add(1, Ordering::SeqCst);
        CounterModel {
            counter: 0,
        }
    }

    fn update(&mut self, event: CounterMsg) {
        match event {
            Decrement => self.model.counter -= 1,
            Increment => self.model.counter += 1,
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,
            gtk::Button {
                label: "+",
                widget_name: "inc_button",
                clicked => Increment,
            },
            gtk::Label {
                label: "0",
                widget_name: "label",
                text: &self.model.counter.to_string(),
            },
            gtk::Button {
                label: "-",
                clicked => Decrement,
            },
        }
    }
}

impl Drop for Counter {
    fn drop(&mut self) {
        COUNTER.fetch_sub(1, Ordering::SeqCst);
    }
}

#[derive(Msg)]
pub enum Msg {
    Add,
    Quit,
    Remove,
}

pub struct Model {
    counters: Vec<Component<Counter>>,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            counters: vec![],
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Add => {
                let widget = self.widgets.hbox.add_widget::<Counter>(());
                self.model.counters.push(widget);
            },
            Quit => gtk::main_quit(),
            Remove => {
                if let Some(counter) = self.model.counters.pop() {
                    self.widgets.hbox.remove_widget(counter);
                }
            },
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                #[name="hbox"]
                gtk::Box {
                    orientation: Horizontal,
                },
                #[name="add_button"]
                gtk::Button {
                    label: "Add",
                    clicked => Add,
                },
                #[name="remove_button"]
                gtk::Button {
                    label: "Remove",
                    clicked => Remove,
                },
            },
            delete_event(_, _) => (Quit, Propagation::Proceed),
        }
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use gtk::{Button, prelude::ContainerExt, Label, prelude::LabelExt};

    use gtk_test::{assert_text, find_child_by_name};
    use relm_test::click;

    use crate::{Win, COUNTER};

    #[test]
    fn root_widget() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let hbox = &widgets.hbox;
        let add_button = &widgets.add_button;
        let remove_button = &widgets.remove_button;

        assert_eq!(hbox.children().len(), 0);
        assert_eq!(COUNTER.load(Ordering::SeqCst), 0);

        click(add_button);
        assert_eq!(hbox.children().len(), 1);
        assert_eq!(COUNTER.load(Ordering::SeqCst), 1);

        let widget1 = &hbox.children()[0];
        let inc_button1: Button = find_child_by_name(widget1, "inc_button").expect("inc button");
        let label1: Label = find_child_by_name(widget1, "label").expect("label");
        assert_text!(label1, 0);

        click(&inc_button1);
        assert_text!(label1, 1);

        click(add_button);
        assert_eq!(hbox.children().len(), 2);
        assert_eq!(COUNTER.load(Ordering::SeqCst), 2);

        let widget2 = &hbox.children()[1];
        let inc_button2: Button = find_child_by_name(widget2, "inc_button").expect("inc button");
        let label2: Label = find_child_by_name(widget2, "label").expect("label");
        assert_text!(label2, 0);

        click(&inc_button2);
        assert_text!(label2, 1);

        click(&inc_button1);
        assert_text!(label1, 2);

        click(add_button);
        assert_eq!(hbox.children().len(), 3);
        assert_eq!(COUNTER.load(Ordering::SeqCst), 3);

        let widget3 = &hbox.children()[2];
        let inc_button3: Button = find_child_by_name(widget3, "inc_button").expect("inc button");
        let label3: Label = find_child_by_name(widget3, "label").expect("label");
        assert_text!(label3, 0);

        click(&inc_button3);
        assert_text!(label3, 1);

        click(&inc_button2);
        assert_text!(label2, 2);

        click(&inc_button1);
        assert_text!(label1, 3);

        click(remove_button);
        assert_eq!(hbox.children().len(), 2);
        assert_eq!(COUNTER.load(Ordering::SeqCst), 2);

        click(&inc_button1);
        assert_text!(label1, 4);

        click(&inc_button2);
        assert_text!(label2, 3);

        click(remove_button);
        assert_eq!(hbox.children().len(), 1);
        assert_eq!(COUNTER.load(Ordering::SeqCst), 1);

        click(&inc_button1);
        assert_text!(label1, 5);

        click(remove_button);
        assert_eq!(hbox.children().len(), 0);
        assert_eq!(COUNTER.load(Ordering::SeqCst), 0);
    }
}
