use itertools::Itertools;
use std::fmt;

use crate::{LocalRw, RcLocal, has_side_effects};

use super::RValue;

#[derive(Debug, Clone, Default)]
pub struct Return {
    pub values: Vec<RValue>,
}

has_side_effects!(Return);

impl Return {
    pub fn new(values: Vec<RValue>) -> Self {
        Self { values }
    }
}

impl LocalRw for Return {
    fn values_read(&self) -> Vec<&RcLocal> {
        self.values.iter().flat_map(|r| r.values_read()).collect()
    }

    fn values_read_mut(&mut self) -> Vec<&mut RcLocal> {
        self.values
            .iter_mut()
            .flat_map(|r| r.values_read_mut())
            .collect()
    }
}

impl fmt::Display for Return {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "return {}", self.values.iter().join(", "))
    }
}
