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

#![deny(unused_doc_comments)]

use gtk::{Inhibit, prelude::LabelExt, prelude::WidgetExt};
use relm::Widget;
use relm_derive::{Msg, widget};

use self::Msg::*;

#[derive(Msg)]
pub enum LabelMsg {
}

pub struct LabelModel {
    counter: i32,
}

#[widget]
impl Widget for Label {
    fn init_view(&mut self) {
        self.widgets.label.set_text("Test");
    }

    fn model() -> LabelModel {
        LabelModel {
            counter: 0,
        }
    }

    fn update(&mut self, _event: LabelMsg) {
        self.widgets.label.set_text("");
    }

    view! {
        #[name="label"]
        gtk::Label {
            text: &self.model.counter.to_string(),
            visible: false,
        }
    }
}

#[derive(Msg)]
pub enum Msg {
    /// User requests application quit.
    Quit,
}

#[widget]
impl Widget for Win {
    fn model() {
    }

    fn update(&mut self, event: Msg) {
        match event {
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            #[name="label"]
            Label,
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}

#[cfg(test)]
mod tests {
    use gtk::prelude::LabelExt;
    use gtk_test::assert_text;

    use crate::Win;

    #[test]
    fn root_widget() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let label = &widgets.label;

        assert_text!(label, "Test");
    }
}
