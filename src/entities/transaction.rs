use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::error::Error;

use super::date::NaiveDate;

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug, PartialOrd, Ord)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
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

#[cfg(test)]
mod test {
    use proptest::prelude::*;
    use proptest::test_runner::TestRunner;

    use super::Id;

    #[test]
    fn impl_display_for_id() {
        let mut runner = TestRunner::default();
        runner
            .run(&any::<Id>(), |id| {
                assert_eq!(id.to_string(), format!("#{}", id.0));
                Ok(())
            })
            .unwrap();
    }
}
