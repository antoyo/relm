use crate::gui::circle_drawing::CircleDrawingMsg;

use gtk::prelude::*;
use gtk::{DrawingArea, Rectangle};
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum PopupMenuMsg {
    ShowAt(u64, u64),
    ClickedResize,
}

pub struct PopupMenuModel {
    msg_stream: StreamHandle<CircleDrawingMsg>,
    relative_to: DrawingArea,
}

/// The popup menu shown when right-clicking a circle.
#[widget]
impl Widget for PopupMenu {
    /// This widgets needs two arguments for the model, the `StreamHandle` to report back and
    /// drawing area the popup will be shown on.
    fn model(
        _relm: &Relm<Self>,
        (msg_stream, relative_to): (StreamHandle<CircleDrawingMsg>, DrawingArea),
    ) -> PopupMenuModel {
        PopupMenuModel {
            msg_stream,
            relative_to,
        }
    }

    fn update(&mut self, event: PopupMenuMsg) {
        match event {
            // Open the popup menu at the given position in the drawing area.
            PopupMenuMsg::ShowAt(pos_x, pos_y) => {
                self.widgets.popover.set_pointing_to(&Rectangle::new(
                    pos_x as i32,
                    pos_y as i32,
                    1,
                    1,
                ));
                self.widgets.popover.popup();
            }
            // The resize button was clicked.
            PopupMenuMsg::ClickedResize => {
                self.model.msg_stream.emit(CircleDrawingMsg::StartResize);
                self.widgets.popover.popdown();
            }
        }
    }

    fn init_view(&mut self) {
        // Close the popup by default.
        self.widgets
            .popover
            .set_relative_to(Some(&self.model.relative_to));
        self.widgets.popover.popdown();
    }

    view! {
        #[name="popover"]
        gtk::Popover {
            gtk::Box {
                gtk::Button {
                    label: "Adjust diameter...",
                    clicked => PopupMenuMsg::ClickedResize
                }
            }
        }
    }
}
