// This module contains types representing the option types, e.g. spin, check,
// etc.

use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CheckType(bool);

impl Display for CheckType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "type bool default {}", self.0)
    }
}

impl From<bool> for CheckType {
    fn from(check: bool) -> Self {
        CheckType(check)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SpinType<T> {
    pub default: T,
    pub min: T,
    pub max: T,
}

impl<T> Display for SpinType<T>
where
    T: Display,
{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "type spin default {} min {} max {}",
            self.default, self.min, self.max
        )
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ComboType {
    pub default: String,
    pub var: Vec<String>,
}

impl Display for ComboType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "type combo default {}", self.default)?;
        for v in &self.var {
            write!(formatter, " var {}", v)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ButtonType;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StrType(pub String);

impl Display for StrType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "type string default {}", self.0)
    }
}
