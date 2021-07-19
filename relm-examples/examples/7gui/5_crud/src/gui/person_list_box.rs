use crate::gui::win::WinMsg;
use crate::model::{Person, PersonList};

use gtk::prelude::*;
use gtk::Label;
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum PersonListBoxMsg {
    AddPerson(Person),
    DeleteSelected,
    UpdateSelected(Person),
    Filter(String),
    SelectionChanged(Option<usize>),
}

pub struct PersonListBoxModel {
    persons: PersonList,
    filtered_persons: PersonList,
    filter: String,

    selected: Option<Person>,

    /// The stream for the window will be used to send signals to the window.
    win_stream: StreamHandle<WinMsg>,
}

#[widget]
impl Widget for PersonListBox {
    fn model(_relm: &Relm<Self>, win_stream: StreamHandle<WinMsg>) -> PersonListBoxModel {
        PersonListBoxModel {
            persons: PersonList::new(),
            filtered_persons: PersonList::new(),
            filter: "".to_string(),

            selected: None,

            win_stream,
        }
    }

    fn update(&mut self, event: PersonListBoxMsg) {
        match event {
            PersonListBoxMsg::AddPerson(person) => self.add_person(person),
            PersonListBoxMsg::DeleteSelected => self.delete_selected(),
            PersonListBoxMsg::UpdateSelected(person) => {
                self.delete_selected();
                self.add_person(person);
            }
            PersonListBoxMsg::Filter(filter) => {
                self.model.filter = filter.clone();
                self.update_filter();
                self.update_list_box();
            }
            PersonListBoxMsg::SelectionChanged(index) => {
                if let Some(idx) = index {
                    let person = &self.model.filtered_persons[idx];
                    self.model.selected = Some(person.clone());
                } else {
                    self.model.selected = None;
                }

                // Send the update to the parent widget.
                self.model
                    .win_stream
                    .emit(WinMsg::UpdateSelected(self.model.selected.clone()));
            }
        }
    }

    view! {
        #[name="list_box"]
        gtk::ListBox {
            selected_rows_changed(listbox) => {
                let selected = listbox.selected_row();
                if let Some(row) = selected {
                    let index = listbox
                        .children()
                        .iter()
                        .position(|r| r == &row)
                        .unwrap();
                    PersonListBoxMsg::SelectionChanged(Some(index))
                } else {
                    PersonListBoxMsg::SelectionChanged(None)
                }
            }
        }
    }
}

impl PersonListBox {
    /// Updates the list box to represent the persons in `self.model.filtered_persons`.
    fn update_list_box(&self) {
        // Remove all rows from the list box.
        let list_box = &self.widgets.list_box;
        let list_box_clone = list_box.clone();
        list_box.foreach(|c| list_box_clone.remove(c));

        // Add all persons in the filter.
        for person in self.model.filtered_persons.get_all() {
            // let _ = list_box.add_widget::<PersonRow>(person.clone());
            let label = Label::new(Some(&format!(
                "{}, {}",
                person.get_name(),
                person.get_surname()
            )));
            label.show();
            list_box.add(&label);
        }
    }

    /// Reset `self.model.filtered_persons` from `self.model.persons`.
    fn update_filter(&mut self) {
        self.model.filtered_persons = self.model.persons.clone().filter(&self.model.filter);
    }

    /// Add the given `Person` the the widget.
    /// Will update the list box and the filter.
    fn add_person(&mut self, person: Person) {
        self.model.persons.add(person);
        self.update_filter();
        self.update_list_box();
    }

    /// Delete the selected `Person` from the widget.
    /// Will update the list box and the filter.
    fn delete_selected(&mut self) {
        if let Some(person) = &self.model.selected {
            self.model.persons.remove(person.clone());
            self.update_filter();
            self.update_list_box();
        }
    }
}
