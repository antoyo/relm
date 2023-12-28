use crate::gui::circle_drawing::CircleDrawingMsg;

use glib::Propagation;
use gtk::prelude::*;
use gtk::Adjustment;
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum WindowResizeMsg {
    ValueChanged(f64),
    Quit,
}

pub struct WindowResizeModel {
    msg_stream: StreamHandle<CircleDrawingMsg>,
    default: u64,
}

/// The resizing window showing the slider to change the radius of the selected circle.
#[widget]
impl Widget for WindowResize {
    fn model(
        _: &Relm<Self>,
        (msg_stream, default): (StreamHandle<CircleDrawingMsg>, u64),
    ) -> WindowResizeModel {
        WindowResizeModel {
            msg_stream,
            default,
        }
    }

    fn update(&mut self, event: WindowResizeMsg) {
        match event {
            WindowResizeMsg::ValueChanged(value) => self
                .model
                .msg_stream
                .emit(CircleDrawingMsg::Resize(value as u64)),
            WindowResizeMsg::Quit => {
                self.model.msg_stream.emit(CircleDrawingMsg::StopResize);
            }
        }
    }

    view! {
        gtk::Window {
            gtk::Scale {
                adjustment: &Adjustment::new(self.model.default as f64, 10.0, 500.0, 1.0, 10.0, 0.0),
                value_changed(scale) => {
                    let value = scale.value();
                    WindowResizeMsg::ValueChanged(value)
                }
            },
            delete_event(_, _) => (WindowResizeMsg::Quit, Propagation::Proceed),
        }
    }
}
