use crate::gui::spreadsheet::Spreadsheet;

use gtk::prelude::*;
use gtk::Inhibit;
use relm::Widget;
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum WinMsg {
    Quit,
}

/// The displayed window only showing the spreadsheet.
#[widget]
impl Widget for Win {
    fn model() -> () {}

    fn update(&mut self, event: WinMsg) {
        match event {
            WinMsg::Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            Spreadsheet {},
            delete_event(_, _) => (WinMsg::Quit, Inhibit(false)),
        }
    }
}
