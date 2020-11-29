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
use gtk::Orientation::{Horizontal, Vertical};
use relm::Widget;
use relm_derive::{Msg, widget};

use self::Msg::*;
use self::CounterMsg::SetIncrement;

pub struct CounterModel {
    counter: i32,
}

#[derive(Msg)]
pub enum CounterMsg {
    Decrement,
    Increment,
    SetIncrement(i32),
}

#[widget]
impl Widget for Counter {
    fn model() -> CounterModel {
        CounterModel {
            counter: 0,
        }
    }

    fn update(&mut self, event: CounterMsg) {
        match event {
            CounterMsg::Decrement => self.model.counter -= 1,
            CounterMsg::Increment => self.model.counter += 1,
            SetIncrement(value) => self.model.counter = value,
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,
            gtk::Button {
                label: "+",
                widget_name: "inc_button",
                clicked => CounterMsg::Increment,
            },
            gtk::Label {
                widget_name: "label",
                text: &self.model.counter.to_string(),
            },
            gtk::Button {
                label: "-",
                clicked => CounterMsg::Decrement,
            },
        }
    }
}

#[widget]
impl Widget for HBox {
    fn model() -> () {
        ()
    }

    fn update(&mut self, _event: ()) {
    }

    view! {
        #[container]
        gtk::Box {
            orientation: Horizontal,
        }
    }
}

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
                        HBox {
                            gtk::Frame {
                                label_widget: view! {
                                    #[name="counter"]
                                    Counter {
                                        SetIncrement: self.model.counter,
                                    },
                                }
                            }
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
    use gtk::{Button, ButtonExt, GtkMenuItemExt, Label, LabelExt};

    use gtk_test::{assert_label, assert_text, find_child_by_name};
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

        let counter = &widgets.counter;
        let counter_inc_button: Button = find_child_by_name(counter.widget(), "inc_button").expect("button");
        let counter_label: Label = find_child_by_name(counter.widget(), "label").expect("label");

        assert_label!(inc_button, "+");
        assert_label!(dec_button, "-");

        let menu_click = || {
            click(file_menu);
            mouse_move_to(increment_menu);
            click(inc_menu);
        };

        assert_text!(label, 0);
        assert_text!(counter_label, 0);
        assert_label!(inc_menu, "By one 0");
        assert_label!(dec_menu, "By minus one 0");
        click(inc_button);
        assert_text!(label, 1);
        assert_text!(counter_label, 1);
        assert_label!(inc_menu, "By one 1");
        assert_label!(dec_menu, "By minus one 1");
        menu_click();
        assert_text!(label, 2);
        assert_text!(counter_label, 2);
        assert_label!(inc_menu, "By one 2");
        assert_label!(dec_menu, "By minus one 2");
        click(inc_button);
        assert_text!(label, 3);
        assert_text!(counter_label, 3);
        assert_label!(inc_menu, "By one 3");
        assert_label!(dec_menu, "By minus one 3");

        click(&counter_inc_button);
        assert_text!(label, 3);
        assert_text!(counter_label, 4);
        assert_label!(inc_menu, "By one 3");
        assert_label!(dec_menu, "By minus one 3");

        menu_click();
        assert_text!(label, 4);
        assert_text!(counter_label, 4);
        assert_label!(inc_menu, "By one 4");
        assert_label!(dec_menu, "By minus one 4");

        let menu_click = || {
            click(file_menu);
            mouse_move_to(increment_menu);
            click(dec_menu);
        };

        menu_click();
        assert_text!(label, 3);
        assert_text!(counter_label, 3);
        assert_label!(inc_menu, "By one 3");
        assert_label!(dec_menu, "By minus one 3");
        menu_click();
        assert_text!(label, 2);
        assert_text!(counter_label, 2);
        assert_label!(inc_menu, "By one 2");
        assert_label!(dec_menu, "By minus one 2");
        menu_click();
        assert_text!(label, 1);
        assert_text!(counter_label, 1);
        assert_label!(inc_menu, "By one 1");
        assert_label!(dec_menu, "By minus one 1");
        menu_click();
        assert_text!(label, 0);
        assert_text!(counter_label, 0);
        assert_label!(inc_menu, "By one 0");
        assert_label!(dec_menu, "By minus one 0");
        menu_click();
        assert_text!(label, -1);
        assert_text!(counter_label, -1);
        assert_label!(inc_menu, "By one -1");
        assert_label!(dec_menu, "By minus one -1");
    }
}
