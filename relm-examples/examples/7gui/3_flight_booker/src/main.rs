use std::convert::TryFrom;

use chrono::NaiveDate;
use gtk::prelude::*;
use gtk::{ButtonsType, DialogFlags, MessageDialog, MessageType};
use relm::Widget;
use relm_derive::{widget, Msg};

// The color to show if a date is invalid.
// TODO: uncomment when alternative to override_background_color is found
// const INVALID_BACKGROUND: RGBA = RGBA {
//     red: 1.0,
//     green: 0.0,
//     blue: 0.0,
//     alpha: 1.0,
// };

/// The date format used in the application.
const DATE_FORMAT: &str = "%d.%m.%Y";

/// The different flight types.
#[derive(Debug, PartialEq, Eq)]
pub enum FlightType {
    OneWay,
    Return,
}

/// Convert `FlightType` to `String`.
impl ToString for FlightType {
    fn to_string(&self) -> String {
        match self {
            FlightType::OneWay => "one-way flight".to_string(),
            FlightType::Return => "return flight".to_string(),
        }
    }
}

/// Convert `String` to `FlightType`
impl TryFrom<String> for FlightType {
    type Error = ();
    fn try_from(string: String) -> Result<Self, Self::Error> {
        let one_way_string = FlightType::OneWay.to_string();
        let return_string = FlightType::Return.to_string();

        if string == one_way_string {
            Ok(FlightType::OneWay)
        } else if string == return_string {
            Ok(FlightType::Return)
        } else {
            Err(())
        }
    }
}

/// The messages sent to the `Win` widget.
#[derive(Msg)]
pub enum Msg {
    /// The flight type in the combo box was changed.
    /// Has the new flight type.
    ChangedFlightType(FlightType),
    /// The starting date was changed.
    /// Has the new starting date.
    ChangedStartDate(Option<NaiveDate>),
    /// The ending date was changed.
    /// Has the new end date.
    ChangedEndDate(Option<NaiveDate>),
    /// Book the flight.
    Book,
    Quit,
}

/// The model the `Win` widget uses.
/// This will hold the flight type, the starting and ending dates and if the configuration is valid.
pub struct Model {
    flight_type: FlightType,
    starting_date: Option<NaiveDate>,
    ending_date: Option<NaiveDate>,
    valid: bool,
}

/// The widget containing the main window and the converter.
#[widget]
impl Widget for Win {
    /// Get the default model for the widget.
    /// The temperature will not be set.
    fn model() -> Model {
        let date = Some(NaiveDate::from_ymd(2021, 3, 20));
        Model {
            flight_type: FlightType::OneWay,
            starting_date: date.clone(),
            ending_date: date,
            valid: true,
        }
    }

    /// This will be called when a message was sent.
    fn update(&mut self, event: Msg) {
        match event {
            // Change the flight type.
            Msg::ChangedFlightType(flight_type) => {
                self.model.flight_type = flight_type;
                self.set_valid();
            }
            // Change the starting date.
            Msg::ChangedStartDate(date) => {
                self.model.starting_date = date;
                self.set_valid();

                // TODO: Find alternative to override_background_color
                // if date.is_none() {
                //     self.widgets
                //         .entry_starting_date;
                //         .override_background_color(StateFlags::NORMAL, Some(&INVALID_BACKGROUND));
                // } else {
                //     self.widgets
                //         .entry_starting_date
                //         .override_background_color(StateFlags::NORMAL, None);
                // }
            }
            // Change the ending date.
            Msg::ChangedEndDate(date) => {
                self.model.ending_date = date;
                self.set_valid();

                // TODO: Find alternative to override_background_color
                // if date.is_none() {
                //     self.widgets
                //         .entry_ending_date
                //         .override_background_color(StateFlags::NORMAL, Some(&INVALID_BACKGROUND));
                // } else {
                //     self.widgets
                //         .entry_ending_date
                //         .override_background_color(StateFlags::NORMAL, None);
                // }
            }
            // Book the flight.
            Msg::Book => {
                let message = self.get_message();
                let dialog = MessageDialog::new::<gtk::Window>(
                    Some(&self.widgets.window),
                    DialogFlags::empty(),
                    MessageType::Info,
                    ButtonsType::Ok,
                    &message,
                );

                // Show the dialog
                dialog.show();

                // Close the dialog after `Ok` has been pressed.
                dialog.connect_response(|dialog, _reponse| dialog.emit_close());
            }
            // Quit the application
            Msg::Quit => gtk::main_quit(),
        }
    }

    /// This method will be called when the widget has been constructed.
    /// It is mostly used to manipulate the widget where the `view!` macro is not powerfull enough.
    fn init_view(&mut self) {
        // Add the flight types to the combo box and set `FlightType::OneWay` to be active.
        let combo_box_flight_type = &mut self.widgets.combo_box_flight_type;

        combo_box_flight_type.append(Some("row_one_way"), &FlightType::OneWay.to_string());
        combo_box_flight_type.append(Some("row_return"), &FlightType::Return.to_string());

        combo_box_flight_type.set_active_id(Some("row_one_way"));

        // Set the initial texts of the entries.
        self.widgets.entry_starting_date.set_text(
            &self
                .model
                .starting_date
                .unwrap()
                .format(DATE_FORMAT)
                .to_string(),
        );
        self.widgets.entry_ending_date.set_text(
            &self
                .model
                .ending_date
                .unwrap()
                .format(DATE_FORMAT)
                .to_string(),
        );
    }

    // This macro builds the application.
    view! {
        #[name="window"]
        gtk::Window {
            gtk::Box {
                orientation: gtk::Orientation::Vertical,

                // Give the combo box a name so you can access it using `self.widgets.combo_box_flight_type`.
                #[name="combo_box_flight_type"]
                gtk::ComboBoxText {
                    changed(combo_box) => {
                        let selected = combo_box.active_text();
                        Msg::ChangedFlightType(FlightType::try_from(selected.unwrap().as_str().to_string()).unwrap())
                    }
                },

                // The entry for starting date
                #[name="entry_starting_date"]
                gtk::Entry {
                    changed(entry) => {
                        let text = entry.text();

                        Msg::ChangedStartDate(str_to_date(&text))
                    },

                    placeholder_text: Some("Starting date")
                },

                // The entry for ending date
                #[name="entry_ending_date"]
                gtk::Entry {
                    changed(entry) => {
                        let text = entry.text();

                        Msg::ChangedEndDate(str_to_date(&text))
                    },

                    placeholder_text: Some("Ending date"),
                    sensitive: self.model.flight_type == FlightType::Return
                },

                // The button to book the flight.
                // This button is disabled manually in `set_valid`.
                #[name="button_book"]
                gtk::Button {
                    label: "Book",
                    clicked => Msg::Book,
                }
            },
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}

impl Win {
    /// Set `self.model.valid`.
    /// Will also disable the `self.widgets.button_book` when needed.
    fn set_valid(&mut self) {
        let model = &mut self.model;
        match model.flight_type {
            FlightType::OneWay => model.valid = model.starting_date.is_some(),
            FlightType::Return => {
                model.valid = model.starting_date.is_some()
                    && model.ending_date.is_some()
                    && model.starting_date.unwrap() <= model.ending_date.unwrap();
            }
        }

        // A alternative way to set attributes of widgets.
        self.widgets.button_book.set_sensitive(model.valid);
    }

    /// Get the message that will be shown when booking.
    fn get_message(&self) -> String {
        let message: String;

        match self.model.flight_type {
            FlightType::OneWay => {
                message = format!(
                    "You have booked a one-way-flight on {}.",
                    self.model
                        .starting_date
                        .unwrap()
                        .format(DATE_FORMAT)
                        .to_string()
                )
            }
            FlightType::Return => {
                message = format!(
                    "You have booked a flight on {} with return date {}.",
                    self.model
                        .starting_date
                        .unwrap()
                        .format(DATE_FORMAT)
                        .to_string(),
                    self.model
                        .ending_date
                        .unwrap()
                        .format(DATE_FORMAT)
                        .to_string()
                )
            }
        }

        message
    }
}

/// Convert a string to a date using `DATE_FORMAT` as the format.
fn str_to_date(string: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(string, "%d.%m.%Y").ok()
}

fn main() {
    // Run the application.
    Win::run(()).expect("Win::run failed");
}
