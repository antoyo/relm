use crate::model::operations;
use crate::model::CellRef;

use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Display};

/// A formula. This formula can be parsed into and parsed from a String.
pub trait Formula: PartialEq + fmt::Debug + Display + TryFrom<String> {
    /// Get all cells this formula is dependent on.
    fn get_dependent_cells(&self) -> HashSet<CellRef>;

    /// Get the value calculated with the given `HashMap`.
    fn get_value(&self, man: &HashMap<CellRef, f64>) -> Result<f64, String>;
}

/// A enum for all the different formulas.
#[derive(PartialEq, Debug, Clone)]
pub enum Formulas {
    Empty,
    Number(Number),
    Textual(Textual),
    CellRef(CellRef),
    Range(Range),
    Application(Application),
}

impl Display for Formulas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Formulas::Empty => write!(f, ""),
            Formulas::Number(num) => write!(f, "{}", num),
            Formulas::Textual(text) => write!(f, "{}", text),
            Formulas::CellRef(cell) => write!(f, "{}", cell),
            Formulas::Range(range) => write!(f, "{}", range),
            Formulas::Application(application) => write!(f, "{}", application),
        }
    }
}

impl TryFrom<String> for Formulas {
    type Error = ();
    fn try_from(string: String) -> Result<Self, Self::Error> {
        if string.trim().is_empty() {
            return Ok(Formulas::Empty);
        }

        let num_res: Result<Number, ()> = string.clone().try_into();
        if let Ok(num) = num_res {
            return Ok(Formulas::Number(num));
        }

        let cell_res: Result<CellRef, ()> = string.clone().try_into();
        if let Ok(cell) = cell_res {
            return Ok(Formulas::CellRef(cell));
        }

        let range_res: Result<Range, ()> = string.clone().try_into();
        if let Ok(range) = range_res {
            return Ok(Formulas::Range(range));
        }

        let application_res: Result<Application, ()> = string.clone().try_into();
        if let Ok(application) = application_res {
            return Ok(Formulas::Application(application));
        }

        let text_res: Result<Textual, ()> = string.clone().try_into();
        if let Ok(text) = text_res {
            return Ok(Formulas::Textual(text));
        }

        Err(())
    }
}

impl Formulas {
    /// Parses a formula from a string, but a `Application` and `CellRef` are required to have a equals sign in front.
    /// A equals sign in front of a number is optional.
    pub fn from_string_require_equals(string: String) -> Result<Self, ()> {
        if string.trim().is_empty() {
            return Ok(Formulas::Empty);
        }

        if string.starts_with('=') {
            let mut string_clone = string.clone();
            string_clone.remove(0);
            string_clone = string_clone.trim_start().to_string();

            let application_res: Result<Application, ()> = string_clone.clone().try_into();
            if let Ok(application) = application_res {
                return Ok(Formulas::Application(application));
            }

            let cell_res: Result<CellRef, ()> = string_clone.clone().try_into();
            if let Ok(cell) = cell_res {
                return Ok(Formulas::CellRef(cell));
            }

            let num_res: Result<Number, ()> = string_clone.clone().try_into();
            if let Ok(num) = num_res {
                return Ok(Formulas::Number(num));
            }

            return Err(());
        }

        let num_res: Result<Number, ()> = string.clone().try_into();
        if let Ok(num) = num_res {
            return Ok(Formulas::Number(num));
        }

        let range_res: Result<Range, ()> = string.clone().try_into();
        if let Ok(range) = range_res {
            return Ok(Formulas::Range(range));
        }

        let text_res: Result<Textual, ()> = string.clone().try_into();
        if let Ok(text) = text_res {
            return Ok(Formulas::Textual(text));
        }

        Err(())
    }
}

impl Formula for Formulas {
    fn get_dependent_cells(&self) -> HashSet<CellRef> {
        match self {
            Formulas::Empty => HashSet::new(),
            Formulas::Number(num) => num.get_dependent_cells(),
            Formulas::Textual(text) => text.get_dependent_cells(),
            Formulas::CellRef(cell) => cell.get_dependent_cells(),
            Formulas::Range(range) => range.get_dependent_cells(),
            Formulas::Application(application) => application.get_dependent_cells(),
        }
    }

    fn get_value(&self, map: &HashMap<CellRef, f64>) -> Result<f64, String> {
        match self {
            Formulas::Empty => Ok(0.0),
            Formulas::Number(num) => num.get_value(map),
            Formulas::Textual(text) => text.get_value(map),
            Formulas::CellRef(cell) => cell.get_value(map),
            Formulas::Range(range) => range.get_value(map),
            Formulas::Application(application) => application.get_value(map),
        }
    }
}

/// The number formula only holds a number.
#[derive(PartialEq, Debug, Clone)]
pub struct Number {
    number: f64,
}

impl Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.number)
    }
}

impl TryFrom<String> for Number {
    type Error = ();
    fn try_from(string: String) -> Result<Self, Self::Error> {
        let num = string.parse::<f64>();
        match num {
            Ok(n) => Ok(Number { number: n }),
            Err(_) => Err(()),
        }
    }
}

impl Formula for Number {
    fn get_dependent_cells(&self) -> HashSet<CellRef> {
        HashSet::new()
    }

    fn get_value(&self, _map: &HashMap<CellRef, f64>) -> Result<f64, String> {
        Ok(self.number)
    }
}

/// The textual formula only holds a text. The conversion from a String never fails.
#[derive(PartialEq, Debug, Clone)]
pub struct Textual {
    text: String,
}

impl Display for Textual {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl TryFrom<String> for Textual {
    type Error = ();
    fn try_from(string: String) -> Result<Self, Self::Error> {
        Ok(Textual { text: string })
    }
}

impl Formula for Textual {
    fn get_dependent_cells(&self) -> HashSet<CellRef> {
        HashSet::new()
    }

    fn get_value(&self, _map: &HashMap<CellRef, f64>) -> Result<f64, String> {
        Err("A text has no value".to_string())
    }
}

/// The CellRef formula holds a reference to a cell.
impl Display for CellRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.get_column(), self.get_row())
    }
}

impl TryFrom<String> for CellRef {
    type Error = ();
    fn try_from(string: String) -> Result<Self, Self::Error> {
        let length = string.len();
        if length >= 2 && length <= 3 {
            let row = string.chars().next().unwrap();
            let col_string = string.get(1..).unwrap();

            if row.is_ascii_alphabetic() && row.is_ascii_uppercase() {
                match col_string.parse::<u8>() {
                    Ok(col) => return Ok(CellRef::new(row, col)),
                    _ => return Err(()),
                }
            } else {
                return Err(());
            }
        } else {
            return Err(());
        }
    }
}

impl Formula for CellRef {
    fn get_dependent_cells(&self) -> HashSet<CellRef> {
        let mut set = HashSet::new();
        set.insert(self.clone());
        set
    }

    fn get_value(&self, map: &HashMap<CellRef, f64>) -> Result<f64, String> {
        let entry = map.get(&self);
        if entry.is_some() {
            Ok(*entry.unwrap())
        } else {
            Err(format!("No value for the cell {}.", self))
        }
    }
}

/// A Range formula holds a square group of cells.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Range {
    upper_left: CellRef,
    lower_right: CellRef,
}

impl Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}",
            self.upper_left.to_string(),
            self.lower_right.to_string()
        )
    }
}
impl TryFrom<String> for Range {
    type Error = ();
    fn try_from(string: String) -> Result<Self, Self::Error> {
        let parts = string.split(":").collect::<Vec<_>>();
        if parts.len() == 2 {
            let upper_left_option = &parts[0].to_string().try_into();
            let lower_right_option = &parts[1].to_string().try_into();

            if upper_left_option.is_ok() && lower_right_option.is_ok() {
                return Ok(Range {
                    upper_left: upper_left_option.clone().unwrap(),
                    lower_right: lower_right_option.clone().unwrap(),
                });
            } else {
                return Err(());
            }
        } else {
            return Err(());
        }
    }
}

impl Formula for Range {
    fn get_dependent_cells(&self) -> HashSet<CellRef> {
        let row_from = self.upper_left.get_row();
        let row_to = self.lower_right.get_row();
        let col_from = self.upper_left.get_column();
        let col_to = self.lower_right.get_column();

        let mut set = HashSet::new();

        for row in row_from..=row_to {
            for col in col_from..=col_to {
                set.insert(CellRef::new(col, row));
            }
        }

        set
    }

    fn get_value(&self, _man: &HashMap<CellRef, f64>) -> Result<f64, String> {
        Err("A range does not hold a value".to_string())
    }
}

/// The application formula is like a function.
/// The function has a name and takes formulas as arguments.
#[derive(PartialEq, Debug, Clone)]
pub struct Application {
    function: String,
    arguments: Vec<Formulas>,
}

impl Display for Application {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let arguments_len = self.arguments.len();

        let mut arguments_str = "".to_string();
        for i in 0..arguments_len {
            let argument = &self.arguments[i];
            arguments_str += &argument.to_string();

            if i != arguments_len - 1 {
                arguments_str += ", ";
            }
        }

        write!(f, "{}({})", self.function, arguments_str)
    }
}

impl TryFrom<String> for Application {
    type Error = ();
    fn try_from(string: String) -> Result<Self, Self::Error> {
        let mut working_string = string.clone();
        if let Some(')') = working_string.pop() {
            let mut split = working_string.split('(');

            if let Some(function) = split.next() {
                let arguments_list: String = split.collect();
                let arguments_str: Vec<&str> = arguments_list
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();

                let mut arguments = vec![];

                for argument_str in arguments_str {
                    let boxed_argument = argument_str.to_string().try_into();
                    if let Ok(argument) = boxed_argument {
                        arguments.push(argument);
                    } else {
                        return Err(());
                    }
                }

                return Ok(Application {
                    function: function.to_string(),
                    arguments,
                });
            } else {
                return Err(());
            }
        } else {
            return Err(());
        }
    }
}

impl Formula for Application {
    fn get_dependent_cells(&self) -> HashSet<CellRef> {
        let mut set = HashSet::new();

        for argument in &self.arguments {
            for dependency in &argument.get_dependent_cells() {
                set.insert(dependency.clone());
            }
        }

        set
    }

    fn get_value(&self, man: &HashMap<CellRef, f64>) -> Result<f64, String> {
        // Expand the ranges into single cells.
        let mut arguments_expanded: Vec<Formulas> = vec![];

        for arg in &self.arguments {
            match arg {
                Formulas::Range(range) => arguments_expanded.append(
                    &mut range
                        .get_dependent_cells()
                        .iter()
                        .map(|c| Formulas::CellRef(c.clone()))
                        .collect(),
                ),
                _ => arguments_expanded.push(arg.clone()),
            }
        }

        let values_res: Vec<Result<f64, String>> = arguments_expanded
            .iter()
            .map(|f| f.get_value(man))
            .collect();

        if let Some(err) = values_res.iter().find(|v| v.is_err()) {
            err.clone()
        } else {
            let values: Vec<f64> = values_res.iter().map(|v| v.clone().unwrap()).collect();
            operations::operate(&self.function, values)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_number_to_string() {
        let number1 = Number { number: 1.0 };
        assert_eq!(number1.to_string(), "1".to_string());

        let number2 = Number { number: 2.2 };
        assert_eq!(number2.to_string(), "2.2".to_string());

        let number3 = Number { number: 102.3 };
        assert_eq!(number3.to_string(), "102.3".to_string());

        let number3 = Number { number: -0.3 };
        assert_eq!(number3.to_string(), "-0.3".to_string());
    }

    #[test]
    fn test_number_from_string() {
        let num1: Result<Number, ()> = "1.0".to_string().try_into();
        assert_eq!(num1.unwrap().number, 1.0);

        let num2: Result<Number, ()> = "2.2".to_string().try_into();
        assert_eq!(num2.unwrap().number, 2.2);

        let num3: Result<Number, ()> = "102.3".to_string().try_into();
        assert_eq!(num3.unwrap().number, 102.3);

        let num4: Result<Number, ()> = "-0.3".to_string().try_into();
        assert_eq!(num4.unwrap().number, -0.3);

        let not_a_number: Result<Number, ()> = "Hi".to_string().try_into();
        assert!(not_a_number.is_err());
    }

    #[test]
    fn test_textual_to_string() {
        let text1 = Textual {
            text: "Hello".to_string(),
        };
        assert_eq!(text1.to_string(), "Hello".to_string());

        let text2 = Textual {
            text: "World".to_string(),
        };
        assert_eq!(text2.to_string(), "World".to_string());

        let text3 = Textual {
            text: "".to_string(),
        };
        assert_eq!(text3.to_string(), "".to_string());

        let text3 = Textual {
            text: "_Hello_ World".to_string(),
        };
        assert_eq!(text3.to_string(), "_Hello_ World".to_string());
    }

    #[test]
    fn test_textual_from_string() {
        let text1: Result<Textual, ()> = "Hello".to_string().try_into();
        assert_eq!(text1.unwrap().text, "Hello".to_string());

        let text2: Result<Textual, ()> = "World".to_string().try_into();
        assert_eq!(text2.unwrap().text, "World".to_string());

        let text3: Result<Textual, ()> = "".to_string().try_into();
        assert_eq!(text3.unwrap().text, "".to_string());

        let text4: Result<Textual, ()> = "_Hello_ World".to_string().try_into();
        assert_eq!(text4.unwrap().text, "_Hello_ World".to_string());
    }

    #[test]
    fn test_cellref_to_string() {
        let cellref1 = CellRef::new('A', 1);
        assert_eq!(cellref1.to_string(), "A1");

        let cellref2 = CellRef::new('N', 87);
        assert_eq!(cellref2.to_string(), "N87");
    }

    #[test]
    fn test_cellref_from_string() {
        let cellref1: Result<CellRef, ()> = "A1".to_string().try_into();
        assert_eq!(cellref1.unwrap(), CellRef::new('A', 1));

        let cellref2: Result<CellRef, ()> = "N87".to_string().try_into();
        assert_eq!(cellref2.unwrap(), CellRef::new('N', 87));

        let cellref3: Result<CellRef, ()> = "C10".to_string().try_into();
        assert_eq!(cellref3.unwrap(), CellRef::new('C', 10));

        let no_cellref1: Result<CellRef, ()> = "A".to_string().try_into();
        assert!(no_cellref1.is_err());

        let no_cellref2: Result<CellRef, ()> = "N878".to_string().try_into();
        assert!(no_cellref2.is_err());

        let no_cellref3: Result<CellRef, ()> = "a8".to_string().try_into();
        assert!(no_cellref3.is_err());

        let no_cellref4: Result<CellRef, ()> = "Az".to_string().try_into();
        assert!(no_cellref4.is_err());
    }

    #[test]
    fn test_range_to_string() {
        let range1 = Range {
            upper_left: "A1".to_string().try_into().unwrap(),
            lower_right: "N87".to_string().try_into().unwrap(),
        };

        assert_eq!(range1.to_string(), "A1:N87");

        let range2 = Range {
            upper_left: "B9".to_string().try_into().unwrap(),
            lower_right: "Z10".to_string().try_into().unwrap(),
        };

        assert_eq!(range2.to_string(), "B9:Z10");
    }

    #[test]
    fn test_range_from_string() {
        let range1: Result<Range, ()> = "A1:N87".to_string().try_into();
        assert!(range1.clone().is_ok());
        assert_eq!(range1.clone().unwrap().upper_left, CellRef::new('A', 1));
        assert_eq!(range1.clone().unwrap().lower_right, CellRef::new('N', 87));

        let range2: Result<Range, ()> = "B9:Z10".to_string().try_into();
        assert!(range2.clone().is_ok());
        assert_eq!(range2.clone().unwrap().upper_left, CellRef::new('B', 9));
        assert_eq!(range2.clone().unwrap().lower_right, CellRef::new('Z', 10));

        let no_range1: Result<Range, ()> = "A1N87".to_string().try_into();
        assert!(no_range1.is_err());

        let no_range2: Result<Range, ()> = "A1:N87:B10".to_string().try_into();
        assert!(no_range2.is_err());

        let no_range3: Result<Range, ()> = "A1:n87".to_string().try_into();
        assert!(no_range3.is_err());
    }

    #[test]
    fn test_application_to_string() {
        let application1 = Application {
            function: "add".to_string(),
            arguments: vec![],
        };

        assert_eq!(application1.to_string(), "add()");

        let application2 = Application {
            function: "mult".to_string(),
            arguments: vec![
                Formulas::Range(Range {
                    upper_left: CellRef::new('A', 1),
                    lower_right: CellRef::new('B', 10),
                }),
                Formulas::Number(Number { number: 2.0 }),
            ],
        };

        assert_eq!(application2.to_string(), "mult(A1:B10, 2)");
    }

    #[test]
    fn test_application_from_string() {
        let application1: Result<Application, ()> = "add()".to_string().try_into();
        assert!(application1.is_ok());
        let app1 = application1.unwrap();
        assert_eq!(app1.function, "add");
        assert_eq!(app1.arguments.len(), 0);

        let application2: Result<Application, ()> = "mult(A1:B10, 2)".to_string().try_into();
        assert!(application2.is_ok());
        let app2 = application2.unwrap();
        assert_eq!(app2.function, "mult");
        assert_eq!(app2.arguments.len(), 2);
        assert_eq!(
            app2.arguments[0],
            Formulas::Range(Range {
                upper_left: CellRef::new('A', 1),
                lower_right: CellRef::new('B', 10)
            })
        );
    }

    #[test]
    fn test_dependent_cells_number() {
        let num1 = Number { number: 1.0 };
        assert_eq!(num1.get_dependent_cells().len(), 0);

        let num1 = Number { number: 2.2 };
        assert_eq!(num1.get_dependent_cells().len(), 0);
    }

    #[test]
    fn test_dependent_cells_text() {
        let text1 = Textual {
            text: "Hello".to_string(),
        };
        assert_eq!(text1.get_dependent_cells().len(), 0);

        let text2 = Textual {
            text: "World".to_string(),
        };
        assert_eq!(text2.get_dependent_cells().len(), 0);
    }

    #[test]
    fn test_dependent_cells_cellref() {
        let cell1 = CellRef::new('A', 1);
        let cell1_dependencies = cell1.get_dependent_cells();
        assert_eq!(cell1_dependencies.len(), 1);
        assert!(cell1_dependencies.contains(&cell1));

        let cell2 = CellRef::new('N', 87);
        let cell2_dependencies = cell2.get_dependent_cells();
        assert_eq!(cell2_dependencies.len(), 1);
        assert!(cell2_dependencies.contains(&cell2));
    }

    #[test]
    fn test_dependent_cells_range() {
        let cell1_1 = CellRef::new('A', 1);
        let cell1_2 = CellRef::new('A', 2);
        let range1_dependencies = Range {
            upper_left: cell1_1.clone(),
            lower_right: cell1_2.clone(),
        }
        .get_dependent_cells();

        assert_eq!(range1_dependencies.len(), 2);
        assert!(range1_dependencies.contains(&cell1_1));
        assert!(range1_dependencies.contains(&cell1_2));

        let cell2_1 = CellRef::new('C', 3);
        let cell2_2 = CellRef::new('E', 11);
        let range2_dependencies = Range {
            upper_left: cell2_1.clone(),
            lower_right: cell2_2.clone(),
        }
        .get_dependent_cells();

        assert_eq!(range2_dependencies.len(), 27);
        assert!(range2_dependencies.contains(&cell2_1));
        assert!(range2_dependencies.contains(&cell2_2));
        assert!(range2_dependencies.contains(&CellRef::new('D', 8)));

        let cell3_1 = CellRef::new('C', 3);
        let cell3_2 = CellRef::new('A', 11);
        let range3_dependencies = Range {
            upper_left: cell3_1.clone(),
            lower_right: cell3_2.clone(),
        }
        .get_dependent_cells();

        assert_eq!(range3_dependencies.len(), 0);
    }

    #[test]
    fn test_depencent_cells_application() {
        let application1: Application = "add()".to_string().try_into().unwrap();
        assert_eq!(application1.get_dependent_cells().len(), 0);

        let application2: Application = "mult(A1:B20)".to_string().try_into().unwrap();
        let application2_dependencies = application2.get_dependent_cells();
        assert_eq!(application2_dependencies.len(), 40);
        assert!(application2_dependencies.contains(&CellRef::new('B', 8)));
        assert!(application2_dependencies.contains(&CellRef::new('A', 20)));

        let application3: Application = "mult(A1:B3, C10)".to_string().try_into().unwrap();
        let application3_dependencies = application3.get_dependent_cells();

        assert_eq!(application3_dependencies.len(), 7);
        assert!(application3_dependencies.contains(&CellRef::new('B', 1)));
        assert!(application3_dependencies.contains(&CellRef::new('A', 2)));
        assert!(application3_dependencies.contains(&CellRef::new('C', 10)));
    }

    #[test]
    fn test_value_number() {
        assert_eq!(
            Number { number: 1.0 }.get_value(&HashMap::new()).unwrap(),
            1.0
        );

        assert_eq!(
            Number { number: 2.2 }.get_value(&HashMap::new()).unwrap(),
            2.2
        );

        assert_eq!(
            Number { number: 103.2 }.get_value(&HashMap::new()).unwrap(),
            103.2
        );
    }

    #[test]
    fn test_value_textual() {
        assert!(Textual {
            text: "Hello".to_string()
        }
        .get_value(&HashMap::new())
        .is_err());
        assert!(Textual {
            text: "World".to_string()
        }
        .get_value(&HashMap::new())
        .is_err());
    }

    #[test]
    fn test_value_cellref() {
        let mut map = HashMap::new();

        let cell1 = CellRef::new('A', 1);
        let cell2 = CellRef::new('N', 87);

        map.insert(cell1.clone(), 32.0);
        map.insert(cell2.clone(), 10.2);

        let value1 = cell1.get_value(&map);
        let value2 = cell2.get_value(&map);

        assert!(value1.is_ok());
        assert_eq!(value1.unwrap(), 32.0);

        assert!(value2.is_ok());
        assert_eq!(value2.unwrap(), 10.2);

        assert!(CellRef::new('B', 10).get_value(&map).is_err());
    }

    #[test]
    fn test_value_application() {
        let mut map = HashMap::new();

        let cell1 = CellRef::new('A', 1);
        let cell2 = CellRef::new('A', 2);
        let cell3 = CellRef::new('N', 87);

        map.insert(cell1.clone(), 32.0);
        map.insert(cell2.clone(), 10.2);
        map.insert(cell3.clone(), 5.4);

        let application1: Application = "add(A1, N87)".to_string().try_into().unwrap();
        assert_eq!(application1.get_value(&map), Ok(37.4));

        let application2: Application = "sum(A1:A2)".to_string().try_into().unwrap();
        assert_eq!(application2.get_value(&map), Ok(42.2));
    }
}
