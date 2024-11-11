use std::fmt::{Display, Formatter};

use crate::NONE_STR;

#[derive(Clone, Debug, PartialEq)]
pub struct DisplayOption<T>(pub Option<T>);

#[derive(Clone, Debug, PartialEq)]
pub struct DisplayOptionCustom<T>(pub Option<T>, pub &'static str);

impl<T: Display> Display for DisplayOption<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.0 {
            Some(ref v) => write!(f, "{}", v),
            None => write!(f, "{}", *NONE_STR),
        }
    }
}

impl<T: Display> Display for DisplayOptionCustom<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.0 {
            Some(ref v) => write!(f, "{}", v),
            None => write!(f, "{}", self.1),
        }
    }
}
