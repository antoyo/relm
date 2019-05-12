/// To check if the widget's label matches the given string.
///
/// Example:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::{Button, ButtonExt, LabelExt};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let but = Button::new();
/// but.set_label("text");
/// assert_label!(but, "text");
/// # }
/// ```
#[macro_export]
macro_rules! assert_label {
    ($widget:expr, $string:expr) => {
        assert_eq!($widget.get_label().expect("get label"), $string.to_string());
    };
}

/// To check if the widget's text matches the given string.
///
/// Example:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::{Label, LabelExt};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let label = Label::new("I'm a label!");
/// assert_text!(label, "I'm a label!");
/// # }
/// ```
#[macro_export]
macro_rules! assert_text {
    ($widget:expr, $string:expr) => {
        assert_eq!($widget.get_text().expect("get text"), $string.to_string());
    };
}

/// To check if the widget's title matches the given string.
///
/// Example:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::{GtkWindowExt, Window, WindowType};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let window = Window::new(WindowType::Toplevel);
/// window.set_title("Fromage ?");
/// assert_title!(window, "Fromage ?");
/// # }
/// ```
#[macro_export]
macro_rules! assert_title {
    ($widget:expr, $string:expr) => {
        assert_eq!($widget.get_title().expect("get text"), $string.to_string());
    };
}

/// To check if the widget's name matches the given string.
///
/// Example:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::{WidgetExt, Button};
///
/// # fn main() {
/// gtk::init().expect("GTK init failed");
/// let button = Button::new();
/// button.set_name("Omelette");
/// assert_name!(button, "Omelette");
/// # }
/// ```
#[macro_export]
macro_rules! assert_name {
    ($widget:expr, $string:expr) => {
        assert_eq!($widget.get_name().expect("get text"), $string.to_string());
    };
}

/// Create a new observer for signals.
///
/// Example:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::GtkWindowExt;
///
/// # fn main() {
/// gtk::init().expect("initialization failed");
/// let window = gtk::Window::new(gtk::WindowType::Toplevel);
///
/// let observer = observer_new!(window, connect_activate_focus, |_|);
/// window.emit_activate_focus();
/// observer.wait();
/// # }
/// ```
///
/// You can also give a block to the macro that will be called when the signal is triggered:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::GtkWindowExt;
///
/// # fn main() {
/// gtk::init().expect("initialization failed");
/// let window = gtk::Window::new(gtk::WindowType::Toplevel);
///
/// let observer = observer_new!(window, connect_activate_focus, |w| {
///     w.set_title("Caribou !");
/// });
/// window.emit_activate_focus();
/// observer.wait();
/// assert_title!(window, "Caribou !");
/// # }
/// ```
#[macro_export]
macro_rules! observer_new {
    ($widget:expr, $signal_name:ident, |$e1:pat $(,$e:pat)*|) => {{
        let observer = $crate::Observer::new();
        let res = (*observer.get_inner()).clone();
        $widget.$signal_name(move |$e1 $(,$e:expr)*| {
            *res.borrow_mut() = true;
        });
        observer
    }};
    ($widget:expr, $signal_name:ident, |$e1:pat $(,$e:pat)*| $block:block) => {{
        let observer = $crate::Observer::new();
        let res = (*observer.get_inner()).clone();
        $widget.$signal_name(move |$e1 $(,$e:expr)*| {
            let _ = $block;
            *res.borrow_mut() = true;
        });
        observer
    }}
}

/// Create a new observer for relm messages.
#[macro_export]
macro_rules! relm_observer_new {
    ($component:expr, $pat:pat) => {
        $crate::RelmObserver::new($component.stream(), |msg|
            if let $pat = msg {
                true
            }
            else {
                false
            }
        );
    };
}

/// Wait for a specific message.
#[macro_export]
macro_rules! relm_observer_wait {
    (let $($variant:ident)::*($name1:ident, $name2:ident $(,$rest:ident)*) = $observer:expr) => {
        let ($name1, $name2 $(, $rest)*) = {
            let msg = $observer.wait();
            if let $($variant)::*($name1, $name2 $(, $rest)*) = msg {
                ($name1, $name2 $(, $rest)*)
            }
            else {
                panic!("Wrong message type.");
            }
        };
    };
    (let $($variant:ident)::*($name:ident) = $observer:expr) => {
        let $name = {
            let msg = $observer.wait();
            if let $($variant)::*($name) = msg {
                $name
            }
            else {
                panic!("Wrong message type.");
            }
        };
    };
    (let $($variant:ident)::* = $observer:expr) => {
        let () = {
            let msg = $observer.wait();
            if let $($variant)::* = msg {
                ()
            }
            else {
                panic!("Wrong message type.");
            }
        };
    };
}
