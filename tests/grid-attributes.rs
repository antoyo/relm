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

#![feature(proc_macro)]

extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
extern crate relm_test;

use gtk::{
    ButtonExt,
    GridExt,
    Inhibit,
    WidgetExt,
};
use relm::Widget;
use relm_attributes::widget;

use self::Msg::*;

#[derive(Msg)]
pub enum Msg {
    Quit,
}

#[widget]
impl Widget for Win {
    fn model() {
    }

    fn update(&mut self, event: Msg) {
        match event {
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Grid {
                #[name="button7"]
                gtk::Button {
                    label: "7",
                    cell: {
                        left_attach: 0,
                        top_attach: 0,
                    },
                },
                #[name="button8"]
                gtk::Button {
                    label: "8",
                    cell: {
                        left_attach: 1,
                        top_attach: 0,
                    },
                },
                #[name="button9"]
                gtk::Button {
                    label: "9",
                    cell: {
                        left_attach: 2,
                        top_attach: 0,
                    },
                },
                #[name="button4"]
                gtk::Button {
                    label: "4",
                    cell: {
                        left_attach: 0,
                        top_attach: 1,
                    },
                },
                #[name="button5"]
                gtk::Button {
                    label: "5",
                    cell: {
                        left_attach: 1,
                        top_attach: 1,
                    },
                },
                #[name="button6"]
                gtk::Button {
                    label: "6",
                    cell: {
                        left_attach: 2,
                        top_attach: 1,
                    },
                },
                #[name="button1"]
                gtk::Button {
                    label: "1",
                    cell: {
                        left_attach: 0,
                        top_attach: 2,
                    },
                },
                #[name="button2"]
                gtk::Button {
                    label: "2",
                    cell: {
                        left_attach: 1,
                        top_attach: 2,
                    },
                },
                #[name="button3"]
                gtk::Button {
                    label: "3",
                    cell: {
                        left_attach: 2,
                        top_attach: 2,
                    },
                },
                #[name="button_plus_minus"]
                gtk::Button {
                    label: "+/-",
                    cell: {
                        left_attach: 0,
                        top_attach: 3,
                    },
                },
                #[name="button0"]
                gtk::Button {
                    label: "0",
                    cell: {
                        left_attach: 1,
                        top_attach: 3,
                    },
                },
                #[name="button_dot"]
                gtk::Button {
                    label: ".",
                    cell: {
                        left_attach: 2,
                        top_attach: 3,
                    },
                }
            },
            delete_event(_, _) => (Quit, Inhibit(false))
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}

#[cfg(test)]
mod tests {
    use gtk::WidgetExt;

    use relm;

    use Win;

    #[test]
    fn widget_position() {
        let (_component, widgets) = relm::init_test::<Win>(()).unwrap();
        let button7 = &widgets.button7;
        let button8 = &widgets.button8;
        let button9 = &widgets.button9;
        let button4 = &widgets.button4;
        let button5 = &widgets.button5;
        let button6 = &widgets.button6;
        let button1 = &widgets.button1;
        let button2 = &widgets.button2;
        let button3 = &widgets.button3;
        let button_plus_minus = &widgets.button_plus_minus;
        let button0 = &widgets.button0;
        let button_dot = &widgets.button_dot;

        let button7_allocation = button7.get_allocation();
        let button8_allocation = button8.get_allocation();
        let button9_allocation = button9.get_allocation();
        let button4_allocation = button4.get_allocation();
        let button5_allocation = button5.get_allocation();
        let button6_allocation = button6.get_allocation();
        let button1_allocation = button1.get_allocation();
        let button2_allocation = button2.get_allocation();
        let button3_allocation = button3.get_allocation();
        let button_pm_allocation = button_plus_minus.get_allocation();
        let button0_allocation = button0.get_allocation();
        let button_dot_allocation = button_dot.get_allocation();
        assert!(button7_allocation.x < button8_allocation.x);
        assert_eq!(button7_allocation.x, button4_allocation.x);
        assert!(button7_allocation.y < button4_allocation.y);
        assert!(button8_allocation.x < button9_allocation.x);
        assert_eq!(button8_allocation.x, button5_allocation.x);
        assert!(button8_allocation.y < button5_allocation.y);
        assert_eq!(button9_allocation.x, button6_allocation.x);
        assert!(button9_allocation.y < button6_allocation.y);

        assert!(button4_allocation.x < button5_allocation.x);
        assert_eq!(button4_allocation.x, button1_allocation.x);
        assert!(button4_allocation.y < button1_allocation.y);
        assert!(button5_allocation.x < button6_allocation.x);
        assert_eq!(button5_allocation.x, button2_allocation.x);
        assert!(button5_allocation.y < button2_allocation.y);
        assert_eq!(button6_allocation.x, button3_allocation.x);
        assert!(button6_allocation.y < button3_allocation.y);

        assert!(button1_allocation.x < button2_allocation.x);
        assert_eq!(button1_allocation.x, button_pm_allocation.x);
        assert!(button1_allocation.y < button_pm_allocation.y);
        assert!(button2_allocation.x < button3_allocation.x);
        assert_eq!(button2_allocation.x, button0_allocation.x);
        assert!(button2_allocation.y < button0_allocation.y);
        assert_eq!(button3_allocation.x, button_dot_allocation.x);
        assert!(button3_allocation.y < button_dot_allocation.y);
    }
}
