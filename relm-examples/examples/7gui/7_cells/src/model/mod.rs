mod cell;
mod formula;
mod operations;
mod spreadsheet;

pub use cell::{Cell, CellRef};
pub use formula::{Application, Formula, Formulas, Number, Range, Textual};
pub use spreadsheet::Spreadsheet;
