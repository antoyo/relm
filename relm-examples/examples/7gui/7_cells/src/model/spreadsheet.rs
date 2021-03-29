use crate::model::{Cell, CellRef};

use std::collections::{HashMap, HashSet};

/// A spreadsheet of cells.
pub struct Spreadsheet {
    /// All the cells that have been edited of are required by another cell.
    cells: HashMap<CellRef, Cell>,

    /// The dirty cells that have been updated but not read.
    dirty: HashSet<CellRef>,
}

impl Spreadsheet {
    /// Create a new spreadsheet without any cells.
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            dirty: HashSet::new(),
        }
    }

    /// Set the formula at the given cell reference to the given string.
    pub fn set_formula(&mut self, cell: &CellRef, formula: &str) {
        // Remove all dependencies to the cell.
        self.remove_dependencies_of(cell);

        // Change the cell.
        if let Some(cell_at) = self.cells.get_mut(cell) {
            cell_at.set_formula_from_string(formula);
        } else {
            let mut new_cell = Cell::new(cell.clone());
            new_cell.set_formula_from_string(formula);
            self.cells.insert(cell.clone(), new_cell);
        }

        // Update all dependencies of the cell.
        self.set_dependencies_of(cell);

        // Cascade changes.
        self.update_cascade(cell);
    }

    /// Get the displayed content at the given cell reference.
    pub fn get_cellcontent_at(&self, cell: &CellRef) -> String {
        if let Some(cell_at) = self.cells.get(cell) {
            format!("{}", cell_at)
        } else {
            "".to_string()
        }
    }

    /// Get all dirty cells and remove these cells from the dirty set.
    pub fn reset_dirty(&mut self) -> HashSet<CellRef> {
        let dirty = self.dirty.clone();
        self.dirty = HashSet::new();
        dirty
    }

    /// Removes the dependencies from all cells, that depend on the given cell.
    fn remove_dependencies_of(&mut self, cell: &CellRef) {
        if let Some(cell_at) = self.cells.get(cell) {
            for dependent_cell in cell_at.get_dependencies() {
                self.cells
                    .get_mut(&dependent_cell)
                    .unwrap()
                    .remove_dependent_cell(cell);
            }
        }
    }

    /// Adds a reference to the given cell to all the cells, that the given cell depends on.
    fn set_dependencies_of(&mut self, cell: &CellRef) {
        if let Some(cell_at) = self.cells.get(cell) {
            for dependent_cell_ref in cell_at.get_dependencies() {
                if let Some(dependent_cell) = self.cells.get_mut(&dependent_cell_ref) {
                    dependent_cell.add_dependent_cell(cell.clone())
                } else {
                    let mut new_cell = Cell::new(dependent_cell_ref.clone());
                    new_cell.add_dependent_cell(cell.clone());
                    self.cells.insert(dependent_cell_ref.clone(), new_cell);
                }
            }
        }
    }

    /// Updates all the cells that depend on the `starting_cell` in any way.
    fn update_cascade(&mut self, starting_cell: &CellRef) {
        let mut dependencies_todo: Vec<CellRef> = vec![starting_cell.clone()];

        let mut done: HashSet<CellRef> = HashSet::new();

        while let Some(next_cell_ref) = dependencies_todo.pop() {
            let cells_clone = self.cells.clone();
            let next_cell = self.cells.get_mut(&next_cell_ref).unwrap();

            next_cell.update_value(&cells_clone);

            let new_dependencies = next_cell.get_dependent_cells();
            dependencies_todo.append(&mut new_dependencies.into_iter().collect());
            done.insert(next_cell_ref.clone());
        }

        self.dirty = self.dirty.union(&done).cloned().collect();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_unknown_cell() {
        let mut sheet = Spreadsheet::new();

        sheet.set_formula(&CellRef::new('A', 1), "");
        assert_eq!(sheet.cells.len(), 1);

        sheet.set_formula(&CellRef::new('B', 2), "2");
        assert_eq!(sheet.cells.len(), 2);
    }

    #[test]
    fn test_add_referenced_cell() {
        let mut sheet = Spreadsheet::new();

        sheet.set_formula(&CellRef::new('A', 1), "= B2");
        assert_eq!(sheet.cells.len(), 2);

        let cell_a1 = sheet.cells.get(&CellRef::new('A', 1)).unwrap();
        assert_eq!(cell_a1.get_dependencies().len(), 1);
        assert_eq!(cell_a1.get_dependent_cells().len(), 0);

        let cell_b2 = sheet.cells.get(&CellRef::new('B', 2)).unwrap();
        assert_eq!(cell_b2.get_dependencies().len(), 0);
        assert_eq!(cell_b2.get_dependent_cells().len(), 1);
    }

    #[test]
    fn test_change_formula_adds_cells() {
        let mut sheet = Spreadsheet::new();

        sheet.set_formula(&CellRef::new('A', 1), "= 42");
        assert_eq!(sheet.cells.len(), 1);

        sheet.set_formula(&CellRef::new('A', 1), "=B2");
        assert_eq!(sheet.cells.len(), 2);

        let cell_a1 = sheet.cells.get(&CellRef::new('A', 1)).unwrap();
        assert_eq!(cell_a1.get_dependencies().len(), 1);
        assert_eq!(cell_a1.get_dependent_cells().len(), 0);

        let cell_b2 = sheet.cells.get(&CellRef::new('B', 2)).unwrap();
        assert_eq!(cell_b2.get_dependencies().len(), 0);
        assert_eq!(cell_b2.get_dependent_cells().len(), 1);
    }

    #[test]
    fn test_updates_value() {
        let mut sheet = Spreadsheet::new();

        sheet.set_formula(&CellRef::new('A', 1), "=mul(2, B2)");
        assert_eq!(sheet.cells.len(), 2);

        sheet.set_formula(&CellRef::new('B', 2), "=mul(3, C7)");
        assert_eq!(sheet.cells.len(), 3);

        sheet.set_formula(&CellRef::new('C', 7), "2");
        assert_eq!(sheet.cells.len(), 3);

        assert_eq!(
            format!("{}", sheet.cells.get(&CellRef::new('A', 1)).unwrap()),
            "12"
        );

        sheet.set_formula(&CellRef::new('C', 7), "3");

        assert_eq!(
            format!("{}", sheet.cells.get(&CellRef::new('A', 1)).unwrap()),
            "18"
        );
    }
}
