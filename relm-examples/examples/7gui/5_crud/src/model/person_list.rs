use crate::model::Person;

use std::ops::Index;

/// A `PersonList` holds a group of persons.
/// The group is automatically sorted.
/// Every person in the list must be unique.
#[derive(Clone, Debug)]
pub struct PersonList {
    persons: Vec<Person>,
}

impl Index<usize> for PersonList {
    type Output = Person;

    fn index(&self, index: usize) -> &Self::Output {
        &self.persons[index]
    }
}

impl PersonList {
    /// Create a new, empty person list.
    pub fn new() -> Self {
        Self { persons: vec![] }
    }

    /// Add a `Person` to the list.
    pub fn add(&mut self, person: Person) {
        if !self.persons.contains(&person) {
            self.persons.push(person);
            self.persons.sort_unstable();
        }
    }

    /// Remove given person.
    pub fn remove(&mut self, person: Person) {
        self.persons = self
            .persons
            .clone()
            .into_iter()
            .filter(|p| p != &person)
            .collect();
    }

    /// Filter out the persons in the list by the given filter.
    /// Does not modify the original filter but creates a new one.
    pub fn filter(&self, filter: &str) -> Self {
        let persons = self.persons.clone();

        let filtered_persons = persons.into_iter().filter(|p| p.matches(filter)).collect();

        Self {
            persons: filtered_persons,
        }
    }

    /// Return all persons in the list as a vector.
    pub fn get_all(&self) -> Vec<Person> {
        self.persons.clone()
    }
}
