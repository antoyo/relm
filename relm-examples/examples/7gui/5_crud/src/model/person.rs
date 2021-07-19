/// A Person having a name and surname.
/// Persons are ordered by surname.
#[derive(Clone, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct Person {
    surname: String,
    name: String,
}

impl Person {
    /// Create a new `Person` using the name and surname.
    pub fn new(name: &str, surname: &str) -> Self {
        Self {
            name: name.to_string(),
            surname: surname.to_string(),
        }
    }

    /// Get the name of the person.
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Get the surname of the person.
    pub fn get_surname(&self) -> String {
        self.surname.clone()
    }

    /// Check whether the `Person` matches the filter.
    /// A `Person` matches if the surname starts with the `filter`.
    pub fn matches(&self, filter: &str) -> bool {
        self.surname.starts_with(filter)
    }
}
