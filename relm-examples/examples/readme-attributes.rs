use gtk::prelude::*;
use gtk::Inhibit;
use gtk::Orientation::Vertical;

use relm::{Loop, Widget};
use relm_derive::{Msg, widget};

#[derive(Msg)]
pub enum Msg {
    Decrement,
    Increment,
    Quit,
}

pub struct Model {
    counter: u32,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            counter: 0,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            // A call to self.label1.set_text() is automatically inserted by the
            // attribute every time the model.counter attribute is updated.
            Msg::Decrement => self.model.counter -= 1,
            Msg::Increment => self.model.counter += 1,
            Msg::Quit => Loop::quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                gtk::Button {
                    // By default, an event with one paramater is assumed.
                    clicked => Msg::Increment,
                    // Hence, the previous line is equivalent to:
                    // clicked(_) => Increment,
                    label: "+",
                },
                gtk::Label {
                    // Bind the text property of this Label to the counter attribute
                    // of the model.
                    // Every time the counter attribute is updated, the text property
                    // will be updated too.
                    text: &self.model.counter.to_string(),
                },
                gtk::Button {
                    clicked => Msg::Decrement,
                    label: "-",
                },
            },
            // Use a tuple when you want to both send a message and return a value to
            // the GTK+ callback.
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}
