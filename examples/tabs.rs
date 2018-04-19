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

extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
#[cfg_attr(test, macro_use)]
extern crate relm_test;

use gtk::{
    ButtonExt,
    Inhibit,
    LabelExt,
    NotebookExt,
    WidgetExt,
};
use relm::Widget;

use self::Msg::*;

#[derive(Msg)]
pub enum Msg {
    Quit,
}

relm_widget! {
    impl ::relm::Widget for Win {
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
                        label: "+",
                    },
                    #[name="label"]
                    gtk::Label {
                        tab: {
                            label: &gtk::Label::new("Second page"),
                        },
                        text: "0",
                    },
                    #[name="dec_button"]
                    gtk::Button {
                        label: "-",
                    },
                },
                delete_event(_, _) => (Quit, Inhibit(false)),
            }
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}

#[cfg(test)]
mod tests {
    use gtk::{Cast, Label, LabelExt, NotebookExt};

    use relm;

    use Win;

    #[test]
    fn root_widget() {
        let (_component, widgets) = relm::init_test::<Win>(()).unwrap();
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
