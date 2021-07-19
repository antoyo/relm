use crate::gui::circle_drawing::{CircleDrawing, CircleDrawingMsg};
use crate::model::{CircleGroup, History};

use gtk::prelude::*;
use gtk::Orientation;
use relm::{Relm, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum WinMsg {
    /// Undo message sent from the undo button.
    Undo,
    /// Redo message sent from the redo button.
    Redo,

    /// Adding a circle group send from the drawing area.
    AddCircleGroup(CircleGroup),

    /// The window was closed.
    Quit,
}

pub struct WinModel {
    relm: Relm<Win>,
    history: History<CircleGroup>,
}

#[widget]
impl Widget for Win {
    fn model(relm: &Relm<Self>, _: ()) -> WinModel {
        WinModel {
            relm: relm.clone(),
            history: History::new(CircleGroup::new()),
        }
    }

    fn update(&mut self, event: WinMsg) {
        match event {
            WinMsg::Undo => {
                self.model.history.undo();
                self.update_drawing();
            }
            WinMsg::Redo => {
                self.model.history.redo();
                self.update_drawing();
            }

            WinMsg::AddCircleGroup(circles) => {
                self.model.history.add(circles);
            }

            // Quit the application
            WinMsg::Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Orientation::Vertical,
                gtk::Box {
                    gtk::Button {
                        label: "Undo",
                        clicked => WinMsg::Undo,
                    },
                    gtk::Button {
                        label: "Redo",
                        clicked => WinMsg::Redo,
                    },
                },
                #[name="circle_drawing"]
                CircleDrawing(self.model.relm.stream().clone()) {
                    vexpand: true
                }
            },
            delete_event(_, _) => (WinMsg::Quit, Inhibit(false)),
        }
    }
}

impl Win {
    /// Update the drawing area after undoing/redoing.
    fn update_drawing(&self) {
        self.components
            .circle_drawing
            .emit(CircleDrawingMsg::SetCircles(
                self.model.history.get_current(),
            ))
    }
}
