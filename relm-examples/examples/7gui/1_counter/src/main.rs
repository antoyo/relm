use gtk::{ButtonExt, Inhibit, LabelExt, WidgetExt};
use relm::Widget;
use relm_derive::{widget, Msg};

/// The messages sent to the `Win` widget.
#[derive(Msg)]
pub enum Msg {
    Increment,
    Quit,
}

/// The model the `Win` widget uses.
/// This is current count of the counter.
pub struct Model {
    counter: u64,
}

/// The widget containing the main window and the counter.
#[widget]
impl Widget for Win {
    /// Get the default model for the widget.
    /// The counter will start at `0`.
    fn model() -> Model {
        Model { counter: 0 }
    }

    /// This will be called when a message was sent.
    fn update(&mut self, event: Msg) {
        match event {
            // Increment the counter
            Msg::Increment => {
                self.model.counter += 1;
            }
            // Quit the application
            Msg::Quit => gtk::main_quit(),
        }
    }

    // This macro builds the application.
    view! {
        gtk::Window {
            gtk::Box {
                // The label showing the text.
                gtk::Label {
                    // The text in the label. Will be updated when `self.model.counter` changed.
                    label: &self.model.counter.to_string()
                },

                // The button to increment the counter.
                gtk::Button {
                    label: "Count",
                    // Clicking the button will send the `Increment` message to the `Win` widget.
                    clicked => Msg::Increment
                }
            },
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}

fn main() {
    // Run the application.
    Win::run(()).expect("Win::run failed");
}
