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

use gdk::EventType::DoubleButtonPress;
use gtk::{
    ButtonExt,
    Inhibit,
    LabelExt,
    Menu,
    MenuItem,
    MenuShellExt,
    OrientableExt,
    ToolButtonExt,
    GtkMenuItemExt,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::{connect, Relm, Widget, timeout};
use relm_derive::{Msg, widget};

use self::Msg::*;
use self::LabelMsg::*;

pub struct LabelModel {
    text: String,
}

#[derive(Clone, Msg)]
pub enum LabelMsg {
    Click,
    DblClick,
    Text(String),
}

#[widget]
impl Widget for ClickableLabel {
    fn model() -> LabelModel {
        LabelModel {
            text: String::new(),
        }
    }

    fn update(&mut self, event: LabelMsg) {
        match event {
            // To be listened to by the user.
            Click | DblClick => (),
            Text(text) => self.model.text = text,
        }
    }

    view! {
        gtk::EventBox {
            button_press_event(_, event) => ({
                if event.get_event_type() == DoubleButtonPress {
                    DblClick
                }
                else {
                    Click
                }
            }, Inhibit(false)),
            #[name="label"]
            gtk::Label {
                widget_name: "label",
                text: &self.model.text,
            },
        },
    }
}

#[derive(Clone)]
pub struct Model {
    counter: i32,
    inc_text: String,
    relm: Relm<Win>,
    text: String,
}

#[derive(Clone, Msg)]
pub enum Msg {
    Decrement,
    DoubleClick,
    FiveInc,
    GetModel,
    Increment,
    RecvModel(Model),
    Quit,
    TwoInc(i32, i32),
    UpdateText,
    UpdateTextNow,
}

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        let menu = Menu::new();
        let inc = MenuItem::new_with_label("Increment");
        connect!(self.model.relm, inc, connect_activate(_), Increment);
        menu.append(&inc);
        self.menu_action.set_submenu(Some(&menu));
        self.menu_bar.show_all();
    }

    fn model(relm: &Relm<Self>, _: ()) -> Model {
        Model {
            counter: 0,
            inc_text: "Increment".to_string(),
            relm: relm.clone(),
            text: String::new(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Decrement => self.model.counter -= 1,
            DoubleClick => self.model.inc_text = "Double click".to_string(),
            // To be listened to by the user.
            FiveInc => (),
            GetModel => self.model.relm.stream().emit(RecvModel(self.model.clone())),
            Increment => {
                self.model.counter += 1;
                if self.model.counter == 2 {
                    self.model.relm.stream().emit(TwoInc(1, 2));
                }
                if self.model.counter == 5 {
                    self.model.relm.stream().emit(FiveInc);
                }
            },
            // To be listened to by the user.
            RecvModel(_) => (),
            Quit => gtk::main_quit(),
            // To be listened to by the user.
            TwoInc(_, _) => (),
            UpdateText => timeout(self.model.relm.stream(), 100, || UpdateTextNow),
            UpdateTextNow => self.model.text = "Updated text".to_string(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                #[name="menu_bar"]
                gtk::MenuBar {
                    #[name="menu_action"]
                    gtk::MenuItem {
                        label: "Action",
                    },
                },
                gtk::Toolbar {
                    #[name="inc_tool_button"]
                    gtk::ToolButton {
                        label: Some("Increment"),
                        clicked => Increment,
                    },
                },
                orientation: Vertical,
                #[name="inc_button"]
                gtk::Button {
                    clicked => Increment,
                    label: "+",
                },
                #[name="label"]
                gtk::Label {
                    text: &self.model.counter.to_string(),
                },
                #[name="dec_button"]
                gtk::Button {
                    clicked => Decrement,
                    label: "-",
                },
                #[name="text"]
                gtk::Label {
                    text: &self.model.text,
                },
                #[name="update_button"]
                gtk::Button {
                    clicked => UpdateText,
                    label: "Update text",
                },
                #[name="inc_label"]
                ClickableLabel {
                    Click => Increment,
                    DblClick => DoubleClick,
                    Text: self.model.inc_text.clone(),
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

#[cfg(test)]
mod tests {
    use glib::Cast;
    use gtk::{
        ContainerExt,
        LabelExt,
        Menu,
        MenuItem,
        GtkMenuItemExt,
    };

    use gtk_test::{
        assert_text,
        click,
        double_click,
        find_widget_by_name,
        wait,
    };
    use relm_test::{
        Observer,
        relm_observer_new,
        relm_observer_wait,
    };

    use crate::Msg::{FiveInc, GetModel, RecvModel, TwoInc};
    use crate::LabelMsg::Text;
    use crate::Win;

    #[test]
    fn label_change() {
        let (component, widgets) = relm::init_test::<Win>(()).expect("init relm test");
        let inc_button = &widgets.inc_button;
        let dec_button = &widgets.dec_button;
        let update_button = &widgets.update_button;
        let inc_tool_button = &widgets.inc_tool_button;
        let inc_label = &widgets.inc_label;

        // Observe for messages.
        let observer = Observer::new(component.stream(), |msg|
            if let FiveInc = msg {
                true
            }
            else {
                false
            }
        );
        let label_observer = relm_observer_new!(inc_label, Text(_));

        // Shortcut for the previous call to Observer::new().
        let two_observer = relm_observer_new!(component, TwoInc(_, _));

        let model_observer = Observer::new(component.stream(), |msg|
            if let RecvModel(_) = msg {
                true
            }
            else {
                false
            }
        );

        assert_text!(widgets.label, 0);
        click(inc_button);
        assert_text!(widgets.label, 1);
        click(inc_button);
        assert_text!(widgets.label, 2);

        // Shortcut for the call to wait() below.
        relm_observer_wait!(let TwoInc(one, two) = two_observer);
        assert_eq!(one, 1);
        assert_eq!(two, 2);

        click(dec_button);
        assert_text!(widgets.label, 1);
        click(inc_button);
        assert_text!(widgets.label, 2);

        relm_observer_wait!(let TwoInc(one, two) = two_observer);
        assert_eq!(one, 1);
        assert_eq!(two, 2);

        click(dec_button);
        assert_text!(widgets.label, 1);
        click(dec_button);
        assert_text!(widgets.label, 0);
        click(dec_button);
        assert_text!(widgets.label, -1);

        for _ in 0..6 {
            click(inc_button);
        }

        // Wait to receive the message on this observer.
        observer.wait();

        // Ask for the model. This will emit RecvModel.
        component.stream().emit(GetModel);

        let msg = model_observer.wait();
        if let RecvModel(model) = msg {
            assert_eq!(model.counter, 5);
        }
        else {
            panic!("Wrong message type.");
        }

        component.stream().emit(GetModel);
        relm_observer_wait!(let RecvModel(model) = model_observer);
        assert_eq!(model.counter, 5);

        let action_menu: MenuItem = widgets.menu_bar.get_children()[0].clone().downcast().expect("menu item 2");
        click(&action_menu);
        let menu: Menu = action_menu.get_submenu().expect("menu 2").downcast().expect("menu 3");
        let inc_menu: MenuItem = menu.get_children()[0].clone().downcast().expect("menu item");
        click(&inc_menu);
        assert_text!(widgets.label, 6);

        click(inc_tool_button);
        assert_text!(widgets.label, 7);

        let inc_label = inc_label.widget();
        click(inc_label);
        assert_text!(widgets.label, 8);

        assert_text!(widgets.text, "");
        click(update_button);
        assert_text!(widgets.text, "");

        wait(200);
        assert_text!(widgets.text, "Updated text");

        let inc_label = find_widget_by_name(inc_label, "label").expect("find label");
        double_click(&inc_label);
        relm_observer_wait!(let Text(text) = label_observer);
        assert_eq!(text, "Double click");
        assert_text!(widgets.label, 10);
    }

    /*
     * Starting gtk multiple in a different thread is forbidden.
    #[test]
    fn clickable_label() {
        let (component, widgets) = relm::init_test::<ClickableLabel>(()).expect("init relm test");
        let label = &widgets.label;

        assert_text!(label, "");

        component.stream().emit(Text("Test".to_string()));
        wait(200);
        assert_text!(label, "Test");
    }*/
}
