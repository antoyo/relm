mod gui;
mod model;

use crate::gui::Win;

use relm::Widget;

fn main() {
    Win::run(()).expect("Could not run window");
}
