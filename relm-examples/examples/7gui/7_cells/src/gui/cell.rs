use crate::gui::popup_menu::{PopupMenu, PopupMenuMsg};
use crate::gui::spreadsheet::SpreadsheetMsg;
use crate::model::CellRef;

use gdk::EventButton;
use gtk::prelude::*;
use relm::{Component, Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum CellMsg {
    /// The cell has been clicked.
    Clicked(EventButton),

    /// The formula of the cell has been changed. This message will be emitted by the popup.
    FormulaChanged(String),

    /// Change the displayed value. This value will be emitted by the spreadsheet.
    DisplayValue(String),
}

pub struct CellModel {
    /// A reference to the cell beeing displayed.
    reference: CellRef,

    /// The displayed text.
    value_text: String,

    /// The formula of this cell.
    formula_text: String,

    /// The stream to send messages to the spreadsheet.
    msg_stream: StreamHandle<SpreadsheetMsg>,

    /// The relm of this cell.
    relm: Relm<Cell>,

    /// The popup menu to change the cells formula.
    /// This menu will only be set when needed to improve performance.
    popup_menu: Option<Component<PopupMenu>>,
}

/// Represents a single cell of the spreadsheet displaying its value in a `gtk::Label`.
#[widget]
impl Widget for Cell {
    fn model(
        relm: &Relm<Self>,
        (msg_stream, reference): (StreamHandle<SpreadsheetMsg>, CellRef),
    ) -> CellModel {
        CellModel {
            reference: reference.clone(),
            value_text: "".to_string(),
            formula_text: "".to_string(),

            msg_stream,

            relm: relm.clone(),
            popup_menu: None,
        }
    }

    fn update(&mut self, event: CellMsg) {
        match event {
            CellMsg::Clicked(event_button) => {
                // Check that the right mouse button was clicked and the popup menu is not visible.
                if event_button.get_button() == 3 && self.model.popup_menu.is_none() {
                    let (pos_x, pos_y) = event_button.get_position();

                    // Create the popup and show it at the clicked position.
                    self.model.popup_menu = Some(relm::create_component::<PopupMenu>((
                        self.model.relm.stream().clone(),
                        self.widgets.event_box.clone(),
                    )));
                    self.model
                        .popup_menu
                        .as_ref()
                        .unwrap()
                        .emit(PopupMenuMsg::ShowAt(
                            pos_x as u64,
                            pos_y as u64,
                            self.model.formula_text.clone(),
                        ));
                }
            }
            CellMsg::FormulaChanged(new_formula) => {
                // Delete the popup.
                self.model.popup_menu = None;

                // Set the local formula.
                self.model.formula_text = new_formula.clone();

                // Send a message to the spreadsheet.
                self.model.msg_stream.emit(SpreadsheetMsg::FormulaUpdated(
                    self.model.reference.clone(),
                    new_formula,
                ));
            }
            CellMsg::DisplayValue(new_value) => {
                // Change the displayed value.
                self.model.value_text = new_value;
            }
        }
    }

    view! {
        // The event box is used to recieve click events.
        #[name="event_box"]
        gtk::EventBox {
            button_press_event(_, event) => (CellMsg::Clicked(event.clone()), Inhibit(false)),
            // The border around the label
            gtk::Frame {
                gtk::Label {
                    label: &self.model.value_text,
                    // The minimal size
                    property_height_request: 25,
                    property_width_request: 150,
                }
            }
        }
    }
}
