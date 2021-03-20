use gtk::{EditableSignals, EntryExt, Inhibit, LabelExt, WidgetExt};
use relm::Widget;
use relm_derive::{widget, Msg};

/// The messages sent to the `Win` widget.
#[derive(Msg)]
pub enum Msg {
    /// The celsius was changed.
    /// Will give the new temperature in celsius.
    ChangedCelsius(String),
    /// The celsius was changed.
    /// Will give the new temperature in fahrenheit.
    ChangedFahrenheit(String),
    Quit,
}

/// The model the `Win` widget uses.
/// This is current temperature in celsius.
/// When uninitialized the temperature will be `None`.
pub struct Model {
    temp_celsius: String,
    temp_fahrenheit: String,
}

/// The widget containing the main window and the converter.
#[widget]
impl Widget for Win {
    /// Get the default model for the widget.
    /// The temperature will not be set.
    fn model() -> Model {
        Model {
            temp_celsius: "".to_string(),
            temp_fahrenheit: "".to_string(),
        }
    }

    /// This will be called when a message was sent.
    fn update(&mut self, event: Msg) {
        match event {
            // The celsius input was changed.
            Msg::ChangedCelsius(celsius) => {
                if self.model.temp_celsius != celsius {
                    self.model.temp_celsius = celsius.clone();
                }

                if let Ok(temp) = celsius.parse::<i64>() {
                    let fahrenheit = (temp as f64) * 9.0 / 5.0 + 32.0;
                    self.model.temp_fahrenheit = format!("{:?}", fahrenheit as i64);
                }
            }
            // The celsius input was changed.
            Msg::ChangedFahrenheit(fahrenheit) => {
                if self.model.temp_fahrenheit != fahrenheit {
                    self.model.temp_fahrenheit = fahrenheit.clone();
                }

                if let Ok(temp) = fahrenheit.parse::<i64>() {
                    let celsius = ((temp as f64) - 32.0) * 5.0 / 9.0;
                    self.model.temp_celsius = format!("{:?}", celsius as i64);
                }
            }
            // Quit the application
            Msg::Quit => gtk::main_quit(),
        }
    }

    // This macro builds the application.
    view! {
        gtk::Window {
            gtk::Box {
                // The entry box for the temperature in celsius.
                gtk::Entry {
                    // This will be called when the entry changes.
                    changed(entry) => {
                        // Get the text from the entry
                        let text = entry.get_text().to_string();
                        Msg::ChangedCelsius(text)
                    },
                    text: &self.model.temp_celsius,
                },
                // The label only showing text.
                gtk::Label {
                    label: "Celsius = "
                },
                // The entry box for the temperature in fahrenheit.
                gtk::Entry {
                    // This will be called when the entry changes.
                    changed(entry) => {
                        // Get the text from the entry
                        let text = entry.get_text().to_string();
                        Msg::ChangedFahrenheit(text)
                    },
                    text: &self.model.temp_fahrenheit,
                },
                // The label showing the text.
                gtk::Label {
                    label: "Fahrenheit"
                },
            },
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}

fn main() {
    // Run the application.
    Win::run(()).expect("Win::run failed");
}
