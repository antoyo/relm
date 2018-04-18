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
extern crate relm_test;

use gtk::{
    ButtonExt,
    ContainerExt,
    Inhibit,
    ToggleButtonExt,
    WidgetExt,
    Window,
    WindowType,
};
use gtk::Orientation::Vertical;
use relm::{
    Component,
    ContainerWidget,
    Relm,
    Update,
    Widget,
    WidgetTest,
};

use self::CheckMsg::*;
use self::Msg::*;

struct CheckModel {
    check: bool,
    label: &'static str,
}

#[derive(Msg)]
enum CheckMsg {
    Check,
    Toggle,
    Uncheck,
}

struct CheckButton {
    button: gtk::CheckButton,
    model: CheckModel,
    relm: Relm<CheckButton>,
}

impl Update for CheckButton {
    type Model = CheckModel;
    type ModelParam = &'static str;
    type Msg = CheckMsg;

    fn model(_: &Relm<Self>, label: &'static str) -> CheckModel {
        CheckModel {
            check: false,
            label,
        }
    }

    fn update(&mut self, event: CheckMsg) {
        match event {
            Check => {
                self.model.check = true;
                // Lock the stream so that the call to set_active does not emit a Toggle message
                // because that would cause an infinite recursion
                // The Toggle message is emitted because the button connect signal is handled.
                let _lock = self.relm.stream().lock();
                self.button.set_active(true);
            },
            Toggle => {
                self.model.check = !self.model.check;
                self.button.set_active(self.model.check);
            },
            Uncheck => {
                self.model.check = false;
                let _lock = self.relm.stream().lock();
                self.button.set_active(false);
            },
        }
    }
}

impl Widget for CheckButton {
    type Root = gtk::CheckButton;

    fn root(&self) -> Self::Root {
        self.button.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let button = gtk::CheckButton::new_with_label(model.label);

        connect!(relm, button, connect_clicked(_), Toggle);

        CheckButton {
            button,
            model,
            relm: relm.clone(),
        }
    }
}

#[derive(Msg)]
enum Msg {
    MinusToggle,
    PlusToggle,
    Quit,
}

#[derive(Clone)]
struct Win {
    minus_button: Component<CheckButton>,
    plus_button: Component<CheckButton>,
    window: Window,
}

impl Update for Win {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> () {
    }

    fn update(&mut self, event: Msg) {
        match event {
            Quit => gtk::main_quit(),
            MinusToggle => {
                if self.minus_button.widget().get_active() {
                    self.plus_button.emit(Uncheck);
                }
                else {
                    self.plus_button.emit(Check);
                }
            },
            PlusToggle => {
                if self.plus_button.widget().get_active() {
                    self.minus_button.emit(Uncheck);
                }
                else {
                    self.minus_button.emit(Check);
                }
            },
        }
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, _model: Self::Model) -> Self {
        let vbox = gtk::Box::new(Vertical, 0);

        let plus_button = vbox.add_widget::<CheckButton>("+");
        let minus_button = vbox.add_widget::<CheckButton>("-");

        let window = Window::new(WindowType::Toplevel);
        window.add(&vbox);
        window.show_all();

        connect!(plus_button@Toggle, relm, PlusToggle);
        connect!(minus_button@Toggle, relm, MinusToggle);
        connect!(relm, window, connect_delete_event(_, _), return (Some(Quit), Inhibit(false)));

        Win {
            minus_button,
            plus_button,
            window: window,
        }
    }
}

impl WidgetTest for Win {
    type Widgets = Win;

    fn get_widgets(&self) -> Self::Widgets {
        self.clone()
    }
}

fn main() {
    Win::run(()).unwrap();
}

#[cfg(test)]
mod tests {
    use gtk::ToggleButtonExt;

    use relm;
    use relm_test::click;

    use Win;

    #[test]
    fn check_uncheck() {
        let (_component, widgets) = relm::init_test::<Win>(()).unwrap();
        let plus_button = widgets.plus_button.widget();
        let minus_button = widgets.minus_button.widget();

        assert!(!plus_button.get_active());
        assert!(!minus_button.get_active());

        click(plus_button);
        assert!(plus_button.get_active());
        assert!(!minus_button.get_active());

        click(plus_button);
        assert!(!plus_button.get_active());
        assert!(minus_button.get_active());

        click(minus_button);
        assert!(plus_button.get_active());
        assert!(!minus_button.get_active());

        click(minus_button);
        assert!(!plus_button.get_active());
        assert!(minus_button.get_active());
    }
}
