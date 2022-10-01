use std::{fmt::Display, str::FromStr};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub(crate) struct Account {
    pub(crate) _kind: Kind,
    pub(crate) _name: Name,
}

#[derive(Clone, Copy, ValueEnum, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    External,
    Budget,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub(crate) struct Name(pub(crate) String);

impl FromStr for Name {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(String::from(s)))
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
