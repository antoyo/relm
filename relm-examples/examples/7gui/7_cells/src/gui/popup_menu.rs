use gtk::prelude::*;
use gtk::{EventBox, Rectangle};
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

use super::cell::CellMsg;

#[derive(Msg)]
pub enum PopupMenuMsg {
    /// Show the popup menu at the given position with the given string as the default value.
    ShowAt(u64, u64, String),

    /// The `Ok` button was clicked or the popup was closed.
    Finish,
}

pub struct PopupMenuModel {
    msg_stream: StreamHandle<CellMsg>,
    relative_to: EventBox,
}

/// The popup menu to change the formula of a cell.
#[widget]
impl Widget for PopupMenu {
    fn model(
        _: &Relm<Self>,
        (msg_stream, relative_to): (StreamHandle<CellMsg>, EventBox),
    ) -> PopupMenuModel {
        PopupMenuModel {
            msg_stream,
            relative_to,
        }
    }

    fn update(&mut self, event: PopupMenuMsg) {
        match event {
            PopupMenuMsg::ShowAt(pos_x, pos_y, default) => {
                self.widgets.popover.set_pointing_to(&Rectangle {
                    x: pos_x as i32,
                    y: pos_y as i32,
                    width: 1,
                    height: 1,
                });

                self.widgets.formula_entry.set_text(&default);
            }
            PopupMenuMsg::Finish => {
                self.widgets.popover.popdown();
                self.model.msg_stream.emit(CellMsg::FormulaChanged(
                    self.widgets.formula_entry.text().to_string(),
                ));
            }
        }
    }

    fn init_view(&mut self) {
        self.widgets
            .popover
            .set_relative_to(Some(&self.model.relative_to));
    }

    view! {
        #[name="popover"]
        gtk::Popover {
            gtk::Box {
                #[name="formula_entry"]
                gtk::Entry {

                },
                gtk::Button {
                    label: "Ok",
                    clicked => PopupMenuMsg::Finish
                }
            },
            // Closing the popup menu will still notify the cell, so it can be clicked again.
            closed => PopupMenuMsg::Finish
        }
    }
}
