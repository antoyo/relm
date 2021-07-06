use crate::model::{Formula, Formulas};

use std::collections::{HashMap, HashSet};
use std::fmt;

/// The coordinates of a cell.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CellRef {
    column: char,
    row: u8,
}

impl CellRef {
    /// Create a new cell reference from the given coordinates.
    pub fn new(column: char, row: u8) -> Self {
        CellRef { row, column }
    }

    pub fn from_coordinates(column: u8, row: u8) -> Self {
        let column_char = ('A' as u8 + column).into();

        CellRef {
            column: column_char,
            row,
        }
    }

    /// Get the row of the cell reference.
    pub fn get_row(&self) -> u8 {
        self.row
    }

    /// Get the column of the cell reference.
    pub fn get_column(&self) -> char {
        self.column
    }
}

/// A cell of the spreadsheat.
#[derive(Clone, Debug)]
pub struct Cell {
    /// The coordinates of this cell.
    coordinates: CellRef,

    /// The formula in the cell as a string.
    formula_string: String,

    /// The formula in the cell.
    formula: Option<Formulas>,

    /// The current value of this cell calculated from the formula.
    /// This value is a error if something in the calculation went wrong or the formula is invalid.
    value: Result<f64, String>,

    /// All the cells that depend on this cell
    dependent_cells: HashSet<CellRef>,

    /// All the cells this cell depends on.
    depends_on: HashSet<CellRef>,
}

impl Cell {
    /// Create a new `Cell` at the given `CellRef`.
    /// This cell will not have a formula.
    pub fn new(coordinates: CellRef) -> Self {
        Self {
            coordinates,
            formula: Some(Formulas::Empty),
            formula_string: "".to_string(),
            value: Ok(0.0),
            dependent_cells: HashSet::new(),
            depends_on: HashSet::new(),
        }
    }

    /// Tries to set the formula from a string.
    /// This will not update the value of the cell if the formula is valid, you have to call `update_value`.
    /// If the formula was not valid, the value will be updated to display a error message.
    pub fn set_formula_from_string(&mut self, formula_string: &str) {
        self.formula_string = formula_string.to_string();
        let formula_res: Result<Formulas, ()> =
            Formulas::from_string_require_equals(formula_string.to_string());
        if let Ok(formula) = formula_res {
            self.set_formula(formula);
        } else {
            self.formula = None;
            self.value = Err("Formula invalid".to_string());
        }
    }

    /// Set a formula for this `Cell`.
    /// This will not update the value of the cell, you have to call `update_value`.
    pub fn set_formula(&mut self, formula: Formulas) {
        self.depends_on = formula.get_dependent_cells();
        self.formula = Some(formula);
    }

    /// Updates the value in this `Cell`.
    pub fn update_value(&mut self, map: &HashMap<CellRef, Cell>) {
        if let Some(formula) = &self.formula {
            let map_f64: HashMap<CellRef, Result<f64, String>> = map
                .iter()
                .filter(|(cell_ref, _cell)| self.depends_on.contains(cell_ref))
                .map(|(cell_ref, cell)| (cell_ref.clone(), cell.value.clone()))
                .collect();

            if map_f64.iter().any(|(_cell_ref, value)| value.is_err()) {
                self.value = Err("Dependent cell has no value".to_string());
            } else {
                let map_values = map_f64
                    .into_iter()
                    .map(|(cell_ref, value)| (cell_ref, value.unwrap()))
                    .collect();
                self.value = formula.get_value(&map_values);
            }
        }
    }

    /// Returns all the cells that depend on  this cell.
    pub fn get_dependent_cells(&self) -> HashSet<CellRef> {
        self.dependent_cells.clone()
    }

    /// Get all cells this cell depends on.
    pub fn get_dependencies(&self) -> HashSet<CellRef> {
        self.depends_on.clone()
    }

    /// Add a cell to the list of cells, that depend on this cell.
    pub fn add_dependent_cell(&mut self, cell: CellRef) {
        self.dependent_cells.insert(cell);
    }

    /// Remove a cell to the list of cells, that depend on this cell.
    pub fn remove_dependent_cell(&mut self, cell: &CellRef) {
        self.dependent_cells.remove(cell);
    }
}

impl fmt::Display for Cell {
    /// `fmt` will first try to display the text if the formula is a text.
    /// Then it will try to show the value of the cell.
    /// If the value is a error, the error message will be shown.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(Formulas::Textual(text)) = &self.formula {
            write!(f, "{}", text)
        } else if let Ok(value) = &self.value {
            write!(f, "{}", value)
        } else if let Err(e) = &self.value {
            if !self.formula_string.trim().is_empty() {
                write!(f, "{}", e)
            } else {
                write!(f, "")
            }
        } else {
            write!(f, "")
        }
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryFrom;

    use crate::model::{Application, Number, Textual};

    use super::*;

    #[test]
    fn test_diplay_text_cell() {
        let mut text_cell = Cell::new(CellRef::new('A', 1));
        text_cell.set_formula(Formulas::Textual(
            Textual::try_from("Hello World".to_string()).unwrap(),
        ));
        text_cell.update_value(&HashMap::new());

        assert_eq!(format!("{}", text_cell), "Hello World");
    }

    #[test]
    fn test_display_value_cell() {
        let mut value_cell = Cell::new(CellRef::new('A', 1));

        value_cell.set_formula(Formulas::Number(Number::try_from("1".to_string()).unwrap()));
        value_cell.update_value(&HashMap::new());
        assert_eq!(format!("{}", value_cell), "1");
    }

    #[test]
    fn test_display_error_cell() {
        let mut error_cell1 = Cell::new(CellRef::new('A', 1));

        error_cell1.set_formula_from_string("=div(2, 0)");
        error_cell1.update_value(&HashMap::new());

        assert_eq!(format!("{}", error_cell1), "cannot divide by zero");

        let mut error_cell2 = Cell::new(CellRef::new('A', 1));

        error_cell2.set_formula_from_string("=add(2, 0");

        error_cell2.update_value(&HashMap::new());

        assert_eq!(format!("{}", error_cell2), "Formula invalid");
    }

    #[test]
    fn test_application_requires_equals() {
        let mut cell = Cell::new(CellRef::new('A', 1));

        // This will just be text
        cell.set_formula_from_string("add(2, 3)");
        assert_eq!(
            cell.formula.clone().unwrap(),
            Formulas::Textual(Textual::try_from("add(2, 3)".to_string()).unwrap())
        );

        // This will be a application
        cell.set_formula_from_string("=div(2, 3)");
        assert_eq!(
            cell.formula.clone().unwrap(),
            Formulas::Application(Application::try_from("div(2, 3)".to_string()).unwrap())
        );

        // This will also be a application, a space after the equals sign does not matter.
        cell.set_formula_from_string("= mul(2, 3)");
        assert_eq!(
            cell.formula.clone().unwrap(),
            Formulas::Application(Application::try_from("mul(2, 3)".to_string()).unwrap())
        );
    }

    #[test]
    fn test_cellref_requires_equals() {
        let mut cell = Cell::new(CellRef::new('A', 1));

        // This will just be text
        cell.set_formula_from_string("B5");
        assert_eq!(
            cell.formula.clone().unwrap(),
            Formulas::Textual(Textual::try_from("B5".to_string()).unwrap())
        );

        // This will be a cell reference
        cell.set_formula_from_string("=B5");
        assert_eq!(
            cell.formula.clone().unwrap(),
            Formulas::CellRef(CellRef::try_from("B5".to_string()).unwrap())
        );

        // This will also be a application, a space after the equals sign does not matter.
        cell.set_formula_from_string("= B5");
        assert_eq!(
            cell.formula.clone().unwrap(),
            Formulas::CellRef(CellRef::try_from("B5".to_string()).unwrap())
        );
    }

    #[test]
    fn test_number_optional_equals() {
        let mut cell = Cell::new(CellRef::new('A', 1));

        cell.set_formula_from_string("42");
        assert_eq!(
            cell.formula.clone().unwrap(),
            Formulas::Number(Number::try_from("42".to_string()).unwrap())
        );

        cell.set_formula_from_string("=42");
        assert_eq!(
            cell.formula.clone().unwrap(),
            Formulas::Number(Number::try_from("42".to_string()).unwrap())
        );

        cell.set_formula_from_string("= 42");
        assert_eq!(
            cell.formula.clone().unwrap(),
            Formulas::Number(Number::try_from("42".to_string()).unwrap())
        );
    }
}
