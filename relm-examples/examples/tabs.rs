/*
 * Copyright (c) 2017-2018 Boucher, Antoni <bouanto@zoho.com>
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
    ButtonExt,
    Inhibit,
    LabelExt,
    NotebookExt,
    WidgetExt,
};
use relm_derive::{widget, Msg};
use relm::Widget;

use self::Msg::*;

#[derive(Msg)]
pub enum Msg {
    Quit,
}

#[widget]
impl Widget for Win {
    fn model() -> () {
        ()
    }

    fn update(&mut self, event: Msg) {
        match event {
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            #[name="tabs"]
            gtk::Notebook {
                #[name="inc_button"]
                gtk::Button {
                    child: {
                        tab_label: Some("First Button"),
                    },
                    label: "Button",
                },
                #[name="label"]
                gtk::Label {
                    tab: {
                        label: Some(&gtk::Label::new(Some("Second page"))),
                    },
                    text: "Hello",
                },
                #[name="dec_button"]
                gtk::Button {
                    label: "Another Button",
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}

#[cfg(test)]
mod tests {
    use gtk::{Cast, Label, LabelExt, NotebookExt};
    use gtk_test::assert_text;

    use crate::Win;

    #[test]
    fn root_widget() {
        let (_component, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let tabs = &widgets.tabs;
        let inc_button = &widgets.inc_button;
        let label = &widgets.label;
        let dec_button = &widgets.dec_button;

        assert_eq!(tabs.get_tab_label_text(inc_button).expect("inc button label"), "First Button");
        let label_widget: Label = tabs.get_tab_label(label).expect("label widget").downcast::<Label>()
            .expect("downcast");
        assert_text!(label_widget, "Second page");
        assert_eq!(tabs.get_tab_label(dec_button), None);
        assert_eq!(tabs.get_tab_label_text(dec_button), None);
    }
}
