use crate::gui::cell::Cell;
use crate::model::{self, CellRef};

use std::collections::HashMap;

use gtk::prelude::*;
use relm::{Component, Relm, Widget};
use relm_derive::{widget, Msg};

use super::cell::CellMsg;

#[derive(Msg)]
pub enum SpreadsheetMsg {
    /// A cell has been updated.
    FormulaUpdated(CellRef, String),
}

pub struct SpreadsheetModel {
    /// All the gui cells.
    cells: HashMap<CellRef, Component<Cell>>,

    /// The spreadsheet this gui is interacting with.
    spreadsheet: model::Spreadsheet,

    /// The relm of the spreadsheet.
    relm: Relm<Spreadsheet>,
}

/// The widget displaying the spread sheet.
#[widget]
impl Widget for Spreadsheet {
    fn model(relm: &Relm<Self>, _: ()) -> SpreadsheetModel {
        SpreadsheetModel {
            cells: HashMap::new(),

            spreadsheet: model::Spreadsheet::new(),

            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: SpreadsheetMsg) {
        match event {
            SpreadsheetMsg::FormulaUpdated(cellref, new_formula) => {
                // Inform the spreadsheet.
                self.model.spreadsheet.set_formula(&cellref, &new_formula);

                // Update all dirty cells.
                let dirty = self.model.spreadsheet.reset_dirty();

                for cell_ref in &dirty {
                    let cell = self.model.cells.get(cell_ref).unwrap();
                    let value = self.model.spreadsheet.get_cellcontent_at(cell_ref);

                    cell.emit(CellMsg::DisplayValue(value));
                }
            }
        }
    }

    fn init_view(&mut self) {
        // Create all gui cells.
        for row in 0..100 {
            for col in 0..26 {
                let cell_ref = CellRef::from_coordinates(col as u8, row as u8);
                let new_cell = relm::create_component::<Cell>((
                    self.model.relm.stream().clone(),
                    cell_ref.clone(),
                ));
                self.widgets.grid.attach(new_cell.widget(), col, row, 1, 1);
                self.model.cells.insert(cell_ref.clone(), new_cell);
            }
        }

        // Create the row indicator.
        for row in 0..100 {
            let cell_ref = CellRef::from_coordinates(0 as u8, row as u8);
            self.widgets.grid.attach_next_to(
                &gtk::Label::new(Some(&format!("{}", row))),
                Some(&self.model.cells.get(&cell_ref).unwrap().widget().clone()),
                gtk::PositionType::Left,
                1,
                1,
            );
        }

        // Create the column indicator.
        for col in 0..26 {
            let cell_ref = CellRef::from_coordinates(col as u8, 0 as u8);
            self.widgets.grid.attach_next_to(
                &gtk::Label::new(Some(&format!("{}", cell_ref.get_column()))),
                Some(&self.model.cells.get(&cell_ref).unwrap().widget().clone()),
                gtk::PositionType::Top,
                1,
                1,
            );
        }

        self.widgets.grid.show_all();
    }

    view! {
        gtk::ScrolledWindow {
            #[name="grid"]
            gtk::Grid {
            }
        }
    }
}
