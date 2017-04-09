extern crate glib_sys;
extern crate gtk;

use gtk::ButtonExt;

#[macro_export]
macro_rules! assert_text {
    ($widget:expr, $string:expr) => {
        assert_eq!($widget.get_text().unwrap(), $string.to_string());
    };
}

/// Simulate a click on a button.
pub fn click<B: ButtonExt>(button: &B) {
    button.clicked();
    run_loop();
}

pub fn run_loop() {
    unsafe { glib_sys::g_usleep(1000) };
    while gtk::events_pending() {
        gtk::main_iteration();
    }
}
