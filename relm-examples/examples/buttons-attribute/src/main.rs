/*
 * Copyright (c) 2018-2020 Boucher, Antoni <bouanto@zoho.com>
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
    glib::Propagation,
    prelude::ButtonExt,
    prelude::LabelExt,
    prelude::OrientableExt,
    prelude::WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::{Relm, Widget, timeout};
use relm_derive::{Msg, widget};

use self::Msg::*;

// Define the structure of the model.
pub struct Model {
    counter: i32,
}

// The messages that can be sent to the update function.
#[derive(Msg)]
pub enum Msg {
    Decrement,
    Increment,
    Quit,
    Show,
}

#[widget]
impl Widget for Win {
    // The initial model.
    fn model(relm: &Relm<Self>, _: ()) -> Model {
        timeout(relm.stream(), 1000, || Show);
        Model {
            counter: 0,
        }
    }

    // Update the model according to the message received.
    fn update(&mut self, event: Msg) {
        match event {
            Decrement => self.model.counter -= 1,
            Increment => self.model.counter += 1,
            Quit => gtk::main_quit(),
            Show => self.widgets.dec_button.set_visible(true),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                // Set the orientation property of the Box.
                orientation: Vertical,
                // Create a Button inside the Box.
                gtk::Button {
                    // Send the message Increment when the button is clicked.
                    clicked => Increment,
                    // TODO: check if using two events of the same name work.
                    label: "+",
                },
                #[name="label"]
                gtk::Label {
                    // Bind the text property of the label to the counter attribute of the model.
                    text: &self.model.counter.to_string(),
                },
                #[name="dec_button"]
                gtk::Button {
                    clicked => Decrement,
                    label: "-",
                    visible: false,
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
    use gtk::prelude::LabelExt;

    use gtk_test::{assert_text, click};

    use crate::Win;

    #[test]
    fn label_change() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let dec_button = &widgets.dec_button;
        let label = &widgets.label;

        click(dec_button);
        assert_text!(label, -1);
        click(dec_button);
        assert_text!(label, -2);
    }
}
