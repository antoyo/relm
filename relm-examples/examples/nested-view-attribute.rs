/*
 * Copyright (c) 2020 Boucher, Antoni <bouanto@zoho.com>
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
    FrameExt,
    GtkMenuItemExt,
    Inhibit,
    LabelExt,
    OrientableExt,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::Widget;
use relm_derive::{Msg, widget};

use self::Msg::*;

pub struct Model {
    counter: i32,
}

#[derive(Msg)]
pub enum Msg {
    Click(f64, f64),
    Decrement,
    End,
    Increment,
    Move(f64, f64),
    Quit,
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
            Click(x, y) => println!("Clicked on {}, {}", x, y),
            Decrement => self.model.counter -= 1,
            End => println!("End"),
            Increment => self.model.counter += 1,
            Move(x, y) => println!("Moved to {}, {}", x, y),
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                gtk::MenuBar {
                    #[name="file_menu"]
                    gtk::MenuItem {
                        label: "File",
                        submenu: view! {
                            gtk::Menu {
                                #[name="increment_menu"]
                                gtk::MenuItem {
                                    label: "Increment",
                                    submenu: view! {
                                        gtk::Menu {
                                            #[name="inc_menu"]
                                            gtk::MenuItem {
                                                label: &("By one ".to_string() + &self.model.counter.to_string()),
                                                activate => Increment,
                                            },
                                            #[name="dec_menu"]
                                            gtk::MenuItem {
                                                label: &("By minus one ".to_string() + &self.model.counter.to_string()),
                                                activate => Decrement,
                                            },
                                        }
                                    },
                                },
                                gtk::MenuItem {
                                    label: "Quit",
                                    activate => Quit,
                                },
                            },
                        },
                    },
                },
                #[name="inc_button"]
                gtk::Button {
                    clicked => Increment,
                    label: "+",
                },
                #[name="label"]
                gtk::Label {
                    text: &self.model.counter.to_string(),
                },
                gtk::Frame {
                    label_widget: view! {
                        gtk::Label {
                            text: &self.model.counter.to_string(),
                        }
                    },
                    #[name="dec_button"]
                    gtk::Button {
                        clicked => Decrement,
                        label: "-",
                    },
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
    use gtk::{ButtonExt, GtkMenuItemExt, LabelExt};

    use gtk_test::{assert_label, assert_text};
    use relm_test::{click, mouse_move_to};

    use crate::Win;

    #[test]
    fn label_change() {
        let (_component, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let inc_button = &widgets.inc_button;
        let dec_button = &widgets.dec_button;
        let label = &widgets.label;
        let inc_menu = &widgets.inc_menu;
        let dec_menu = &widgets.dec_menu;
        let increment_menu = &widgets.increment_menu;
        let file_menu = &widgets.file_menu;

        assert_label!(inc_button, "+");
        assert_label!(dec_button, "-");

        let menu_click = || {
            click(file_menu);
            mouse_move_to(increment_menu);
            click(inc_menu);
        };

        assert_text!(label, 0);
        assert_label!(inc_menu, "By one 0");
        assert_label!(dec_menu, "By minus one 0");
        click(inc_button);
        assert_text!(label, 1);
        assert_label!(inc_menu, "By one 1");
        assert_label!(dec_menu, "By minus one 1");
        menu_click();
        assert_text!(label, 2);
        assert_label!(inc_menu, "By one 2");
        assert_label!(dec_menu, "By minus one 2");
        click(inc_button);
        assert_text!(label, 3);
        assert_label!(inc_menu, "By one 3");
        assert_label!(dec_menu, "By minus one 3");
        menu_click();
        assert_text!(label, 4);
        assert_label!(inc_menu, "By one 4");
        assert_label!(dec_menu, "By minus one 4");

        let menu_click = || {
            click(file_menu);
            mouse_move_to(increment_menu);
            click(dec_menu);
        };

        menu_click();
        assert_text!(label, 3);
        assert_label!(inc_menu, "By one 3");
        assert_label!(dec_menu, "By minus one 3");
        menu_click();
        assert_text!(label, 2);
        assert_label!(inc_menu, "By one 2");
        assert_label!(dec_menu, "By minus one 2");
        menu_click();
        assert_text!(label, 1);
        assert_label!(inc_menu, "By one 1");
        assert_label!(dec_menu, "By minus one 1");
        menu_click();
        assert_text!(label, 0);
        assert_label!(inc_menu, "By one 0");
        assert_label!(dec_menu, "By minus one 0");
        menu_click();
        assert_text!(label, -1);
        assert_label!(inc_menu, "By one -1");
        assert_label!(dec_menu, "By minus one -1");
    }
}
