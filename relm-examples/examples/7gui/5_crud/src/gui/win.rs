use crate::gui::person_list_box::{PersonListBox, PersonListBoxMsg};
use crate::model::Person;

use gtk::{
    BoxExt, ButtonExt, EditableSignals, EntryExt, Inhibit, OrientableExt, Orientation, WidgetExt,
};
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum WinMsg {
    /// Create a new person. This message is sent by the `Create` button.
    CreatePerson,
    /// Update the selected person. This message is sent by the `Update` button.
    UpdatePerson,
    /// Delete the selected person. This message is sent by the `Delete` button.
    DeletePerson,
    /// The selection in the `PersonListBox` has changed. This message is sent by the `person_list_box` component.
    UpdateSelected(Option<Person>),
    /// The filter has changed. This message is sent by the filter entry.
    FilterChanged,
    /// The window was closed.
    Quit,
}

pub struct WinModel {
    msg_stream: StreamHandle<WinMsg>,

    selected_person: Option<Person>,
}

#[widget]
impl Widget for Win {
    fn model(relm: &Relm<Self>, _: ()) -> WinModel {
        WinModel {
            msg_stream: relm.stream().clone(),
            selected_person: None,
        }
    }

    fn update(&mut self, event: WinMsg) {
        match event {
            WinMsg::CreatePerson => {
                let person = self.get_person();
                // `self.components` has all components of the widget referenced by name given in the `view!` macro.
                // You can send messages to the component using the `emit` function.
                // `self.compnents` and `self.widgets` are not the same. `self.widgets``refers to the `gtk::Widget` (or subclass).
                self.components
                    .person_list_box
                    .emit(PersonListBoxMsg::AddPerson(person))
            }
            WinMsg::UpdatePerson => {
                let person = self.get_person();
                self.components
                    .person_list_box
                    .emit(PersonListBoxMsg::UpdateSelected(person))
            }
            WinMsg::DeletePerson => self
                .components
                .person_list_box
                .emit(PersonListBoxMsg::DeleteSelected),
            WinMsg::UpdateSelected(person_opt) => {
                // Set the entry fields.
                if let Some(person) = &person_opt {
                    self.widgets.entry_name.set_text(&person.get_name());
                    self.widgets.entry_surname.set_text(&person.get_surname());
                } else {
                    self.widgets.entry_name.set_text("");
                    self.widgets.entry_surname.set_text("");
                }

                // Set the person in the model.
                self.model.selected_person = person_opt;
            }
            WinMsg::FilterChanged => {
                let filter = self.widgets.entry_filter.get_text();
                self.components
                    .person_list_box
                    .emit(PersonListBoxMsg::Filter(filter.to_string()));
            }
            // Quit the application
            WinMsg::Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                #[name="entry_filter"]
                gtk::Entry {
                    placeholder_text: Some("Filter"),
                    changed => WinMsg::FilterChanged,
                },
                orientation: Orientation::Vertical,
                spacing: 16,
                gtk::Box {
                    spacing: 16,
                    #[name="person_list_box"]
                    // Create a new `PersonListBox` with the stream of this widget as the argument.
                    PersonListBox(self.model.msg_stream.clone()) {
                        hexpand: true,
                    },
                    gtk::Box {
                        spacing: 16,
                        orientation: Orientation::Vertical,
                        #[name="entry_name"]
                        gtk::Entry {
                            placeholder_text: Some("Name"),
                        },
                        #[name="entry_surname"]
                        gtk::Entry {
                            placeholder_text: Some("Surname"),
                        }
                    }
                },
                gtk::Box {
                    spacing: 16,
                    gtk::Button {
                        label: "Create",
                        clicked => WinMsg::CreatePerson
                    },
                    gtk::Button {
                        label: "Update",
                        // This button will only be sensitive if a person is selected.
                        sensitive: self.model.selected_person.is_some(),
                        clicked => WinMsg::UpdatePerson
                    },
                    gtk::Button {
                        label: "Delete",
                        // This button will only be sensitive if a person is selected.
                        sensitive: self.model.selected_person.is_some(),
                        clicked => WinMsg::DeletePerson
                    },
                }
            },
            delete_event(_, _) => (WinMsg::Quit, Inhibit(false)),
        }
    }
}

impl Win {
    /// Get the person from the entries `entry_name` and `entry_surname`.
    fn get_person(&self) -> Person {
        let name = self.widgets.entry_name.get_text();
        let surname = self.widgets.entry_surname.get_text();

        Person::new(&name, &surname)
    }
}
