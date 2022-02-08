/*
 * Copyright (c) 2018 Boucher, Antoni <bouanto@zoho.com>
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
    EditableSignals,
    Inhibit,
    prelude::ButtonExt,
    prelude::EntryExt,
    prelude::LabelExt,
    prelude::OrientableExt,
    prelude::WidgetExt,
};
use gtk::Orientation::{Horizontal, Vertical};
use relm::{Relm, Widget};
use relm_derive::{Msg, widget};

use self::Msg::*;

pub struct Model {
    left_text: String,
    relm: Relm<Win>,
    right_text: String,
    text: String,
}

#[derive(Clone, Msg)]
pub enum Msg {
    Cancel,
    Concat,
    DataAvailable(String),
    DataCleared,
    LeftChanged(String),
    RightChanged(String),
    Quit,
}

#[widget]
impl Widget for Win {
    fn model(relm: &Relm<Self>, (): ()) -> Model {
        Model {
            left_text: String::new(),
            right_text: String::new(),
            relm: relm.clone(),
            text: String::new(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Cancel => {
                self.model.left_text = String::new();
                self.model.right_text = String::new();
                self.model.text = String::new();
                self.model.relm.stream().emit(DataCleared);
            },
            Concat => {
                self.model.text = format!("{}{}", self.model.left_text, self.model.right_text);
                self.model.relm.stream().emit(DataAvailable(self.model.text.clone()));
            },
            // To be listened to by the user.
            DataAvailable(_) | DataCleared => (),
            LeftChanged(text) => self.model.left_text = text,
            RightChanged(text) => self.model.right_text = text,
            Quit => gtk::main_quit(),
        }
    }

    view! {
        #[name="window"]
        gtk::Window {
            gtk::Box {
                gtk::Box {
                    #[name="left_entry"]
                    gtk::Entry {
                        text: &self.model.left_text,
                        changed(entry) => LeftChanged(entry.text().to_string()),
                    },
                    #[name="right_entry"]
                    gtk::Entry {
                        text: &self.model.right_text,
                        changed(entry) => RightChanged(entry.text().to_string()),
                    },
                    orientation: Horizontal,
                },
                gtk::ButtonBox {
                    #[name="concat_button"]
                    gtk::Button {
                        clicked => Concat,
                        label: "Concat",
                    },
                    #[name="cancel_button"]
                    gtk::Button {
                        clicked => Cancel,
                        label: "Cancel",
                    },
                    orientation: Horizontal,
                },
                orientation: Vertical,
                #[name="label"]
                gtk::Label {
                    label: &self.model.text,
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

#[cfg(test)]
mod tests {
    use gdk::keys::constants as key;
    use gtk::prelude::{
        EntryExt,
        GtkWindowExt,
        LabelExt,
        WidgetExt,
    };

    use gtk_test::{
        assert_text,
        focus,
    };
    use relm_test::{
        enter_key,
        enter_keys,
        relm_observer_new,
        relm_observer_wait,
    };

    use crate::Msg::{DataAvailable, DataCleared};
    use crate::Win;

    #[test]
    fn label_change() {
        let (component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let cancel_button = &widgets.cancel_button;
        let concat_button = &widgets.concat_button;
        let label = &widgets.label;
        let left_entry = &widgets.left_entry;
        let right_entry = &widgets.right_entry;
        let window = &widgets.window;

        let available_observer = relm_observer_new!(component, DataAvailable(_));
        let cleared_observer = relm_observer_new!(component, DataCleared);

        assert_text!(label, "");
        enter_keys(&window.focused_widget().expect("focused widget"), "left");
        enter_key(window, key::Tab);
        assert!(right_entry.has_focus());

        enter_keys(&window.focused_widget().expect("focused widget"), "right");
        enter_key(window, key::Tab);
        assert!(concat_button.has_focus());
        enter_key(
            &window.focused_widget().expect("focused widget"),
            key::space,
        );
        assert_text!(label, "leftright");

        enter_key(window, key::Tab);
        assert!(cancel_button.has_focus());
        enter_key(
            &window.focused_widget().expect("focused widget"),
            key::space,
        );
        assert_text!(label, "");
        assert_text!(left_entry, "");
        assert_text!(right_entry, "");

        focus(left_entry);
        assert!(left_entry.has_focus());
        focus(right_entry);
        assert!(right_entry.has_focus());

        relm_observer_wait!(let DataAvailable(text) = available_observer);
        assert_eq!(text, "leftright");

        relm_observer_wait!(let DataCleared = cleared_observer);
    }
}
