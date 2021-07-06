mod gui;
mod model;

use relm::Widget;

use crate::gui::Win;

fn main() {
    Win::run(()).expect("Could not spawn window");
}
