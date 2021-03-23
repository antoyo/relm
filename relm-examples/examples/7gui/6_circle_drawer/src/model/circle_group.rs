use crate::model::Circle;

/// A group of circles.
/// Only one circle can be selected at any time.
#[derive(Clone)]
pub struct CircleGroup {
    circles: Vec<Circle>,
}

impl CircleGroup {
    /// Create a new, empty circle group.
    pub fn new() -> Self {
        Self { circles: vec![] }
    }

    /// Add a circle to the group.
    /// The new circle is selected by default.
    pub fn add(&mut self, circle: Circle) {
        // Deselect all circles.
        for circle in self.circles.iter_mut() {
            circle.select(false);
        }

        let mut circle_clone = circle.clone();
        circle_clone.select(true);
        self.circles.push(circle_clone);
    }

    /// Selects the circle containing the given position, if such a circle exists.
    pub fn select_at(&mut self, pos_x: u64, pos_y: u64) {
        self.delesect_all();

        // Select the first circle containing the point.
        for circle in self.circles.iter_mut() {
            if circle.contains(pos_x, pos_y) {
                circle.select(true);
                return;
            }
        }
    }

    /// Deselect all circles.
    pub fn delesect_all(&mut self) {
        for circle in self.circles.iter_mut() {
            circle.select(false);
        }
    }

    /// Return all the circles this group holds.
    pub fn get_all(&self) -> Vec<Circle> {
        self.circles.clone()
    }

    /// Returns the selected circle, if such a circle exists.
    pub fn get_selected(&self) -> Option<Circle> {
        for circle in &self.circles {
            if circle.is_selected() {
                return Some(circle.clone());
            }
        }

        None
    }

    /// Resize the selected circle to the given radius.
    /// Does nothing if no circle is selected.
    pub fn resize_selected(&mut self, radius: u64) {
        for circle in self.circles.iter_mut() {
            if circle.is_selected() {
                circle.set_radius(radius);
            }
        }
    }
}
