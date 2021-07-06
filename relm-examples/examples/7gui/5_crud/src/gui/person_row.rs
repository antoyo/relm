use crate::model::Person;

use gtk::LabelExt;use relm::{Relm, Widget};
use relm_derive::widget;

/// A `ListBoxRow` for the given `Person`.
#[widget]
impl Widget for PersonRow {
    fn model(_relm: &Relm<Self>, person: Person) -> String {
        format!("{}, {}", person.get_name(), person.get_surname())
    }

    fn update(&mut self, _: ()) {}

    view! {
        #[name="list_box"]
        gtk::ListBoxRow {
            gtk::Label {
                label: &self.model,
            },
        }
    }
}
