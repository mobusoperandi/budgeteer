use std::{fmt::Display, str::FromStr};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub(crate) struct Id(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Transaction {
    pub(crate) id: Id,
    pub(crate) date: NaiveDate,
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse()
            .map_err(Error::TransactionIdFailedToParse)
            .map(Self)
    }
}
