#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Circle {
    pos_x: u64,
    pos_y: u64,
    radius: u64,

    is_selected: bool,
}

impl Circle {
    /// Create a new circle.
    /// The circle will not be selected by default.
    pub fn new(pos_x: u64, pos_y: u64, radius: u64) -> Self {
        Circle {
            pos_x,
            pos_y,
            radius,

            is_selected: false,
        }
    }

    /// Get the x-position of the circle.
    pub fn get_x(&self) -> u64 {
        self.pos_x
    }

    /// Get the y-position of the circle.
    pub fn get_y(&self) -> u64 {
        self.pos_y
    }

    /// Get the radius of the circle.
    pub fn get_radius(&self) -> u64 {
        self.radius
    }

    /// Check whether the circle is selected.
    pub fn is_selected(&self) -> bool {
        self.is_selected
    }

    /// Select or unselect the circle.
    pub fn select(&mut self, select: bool) {
        self.is_selected = select;
    }

    /// Set the radius of the circle.
    pub fn set_radius(&mut self, radius: u64) {
        self.radius = radius;
    }

    /// Check whether the point `(pos_x, pos_y)` is contained in the circle.
    pub fn contains(&self, pos_x: u64, pos_y: u64) -> bool {
        let dist_x = ((pos_x as i64) - (self.pos_x as i64)) as f64;
        let dist_y = ((pos_y as i64) - (self.pos_y as i64)) as f64;

        let dist_sqr = dist_x.powi(2) + dist_y.powi(2);

        self.radius.pow(2) as f64 >= dist_sqr
    }
}
