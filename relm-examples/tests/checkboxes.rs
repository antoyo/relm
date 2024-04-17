/*
 * Copyright (c) 2017-2020 Boucher, Antoni <bouanto@zoho.com>
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
    Inhibit,
    Window,
    WindowType,
    prelude::ButtonExt,
    prelude::ContainerExt,
    prelude::ToggleButtonExt,
    prelude::WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::{
    connect,
    Component,
    ContainerWidget,
    Relm,
    Update,
    Widget,
    WidgetTest,
};
use relm_derive::Msg;

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
        let button = gtk::CheckButton::with_label(model.label);

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

struct Components {
    minus_button: Component<CheckButton>,
    plus_button: Component<CheckButton>,
}

#[derive(Clone)]
struct Widgets {
    minus_button: gtk::CheckButton,
    plus_button: gtk::CheckButton,
    window: Window,
}

struct Win {
    components: Components,
    widgets: Widgets,
}

impl Update for Win {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) {
    }

    fn update(&mut self, event: Msg) {
        match event {
            Quit => gtk::main_quit(),
            MinusToggle => {
                if self.widgets.minus_button.is_active() {
                    self.components.plus_button.emit(Uncheck);
                }
                else {
                    self.components.plus_button.emit(Check);
                }
            },
            PlusToggle => {
                if self.widgets.plus_button.is_active() {
                    self.components.minus_button.emit(Uncheck);
                }
                else {
                    self.components.minus_button.emit(Check);
                }
            },
        }
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
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
            widgets: Widgets {
                minus_button: minus_button.widget().clone(),
                plus_button: plus_button.widget().clone(),
                window,
            },
            components: Components {
                minus_button,
                plus_button,
            },
        }
    }
}

impl WidgetTest for Win {
    type Streams = ();

    fn get_streams(&self) -> Self::Streams {
    }

    type Widgets = Widgets;

    fn get_widgets(&self) -> Self::Widgets {
        self.widgets.clone()
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}

#[cfg(test)]
mod tests {
    use gtk::prelude::ToggleButtonExt;

    use relm_test::click;

    use crate::Win;

    #[test]
    fn check_uncheck() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let plus_button = &widgets.plus_button;
        let minus_button = &widgets.minus_button;

        assert!(!plus_button.is_active());
        assert!(!minus_button.is_active());

        click(plus_button);
        assert!(plus_button.is_active());
        assert!(!minus_button.is_active());

        click(plus_button);
        assert!(!plus_button.is_active());
        assert!(minus_button.is_active());

        click(minus_button);
        assert!(plus_button.is_active());
        assert!(!minus_button.is_active());

        click(minus_button);
        assert!(!plus_button.is_active());
        assert!(minus_button.is_active());
    }
}
