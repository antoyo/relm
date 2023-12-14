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
    prelude::ButtonExt,
    prelude::LabelExt,
    prelude::OrientableExt,
    prelude::WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::Widget;
use relm_derive::{Msg, widget};

use self::Msg::*;
use glib::Propagation;

// Define the structure of the model.
pub struct Model {
    counter: i32,
    use_underline: bool,
}

// The messages that can be sent to the update function.
#[derive(Msg)]
pub enum Msg {
    Decrement,
    Increment,
    Quit,
}

#[widget]
impl Widget for Win {
    // The initial model.
    fn model() -> Model {
        Model {
            counter: 0,
            use_underline: true,
        }
    }

    // Update the model according to the message received.
    fn update(&mut self, event: Msg) {
        match event {
            Decrement => self.model.counter -= 1,
            Increment => self.model.counter += 1,
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                // Set the orientation property of the Box.
                orientation: Vertical,
                // Create a Button inside the Box.
                #[name="button"]
                gtk::Button({ label: "_inc", use_underline: self.model.use_underline }) {
                    // Send the message Increment when the button is clicked.
                    clicked => Increment,
                },
                gtk::Label {
                    // Bind the text property of the label to the counter attribute of the model.
                    text: &self.model.counter.to_string(),
                },
                gtk::Button {
                    clicked => Decrement,
                    label: "-",
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
    use gtk::prelude::ButtonExt;
    use gtk_test::assert_label;

    use crate::Win;

    #[test]
    fn button_position() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let button = &widgets.button;
        assert_label!(button, "_inc");
        assert!(button.uses_underline());
    }
}
