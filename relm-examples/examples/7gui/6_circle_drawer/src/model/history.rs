/// Represents a undo/redo history.
pub struct History<T> {
    undo_hist: Vec<T>,
    current: T,
    redo_hist: Vec<T>,
}

impl<T: Clone> History<T> {
    /// Create a new `History` with the given `initial` value.
    pub fn new(initial: T) -> Self {
        History {
            undo_hist: vec![],
            current: initial,
            redo_hist: vec![],
        }
    }

    /// Undo. Will do nothing if no undo history exists.
    pub fn undo(&mut self) {
        if let Some(last) = self.undo_hist.pop() {
            self.redo_hist.push(self.current.clone());
            self.current = last;
        }
    }

    /// Redo. Will do nothing if no redo history exists.
    pub fn redo(&mut self) {
        if let Some(last) = self.redo_hist.pop() {
            self.undo_hist.push(self.current.clone());
            self.current = last;
        }
    }

    /// Add a new element to the `History`.
    /// The new element will become the current element, the redo history will be cleared.
    pub fn add(&mut self, element: T) {
        self.undo_hist.push(self.current.clone());
        self.current = element;
        self.redo_hist = vec![];
    }

    /// Get the current element.
    pub fn get_current(&self) -> T {
        self.current.clone()
    }
}
