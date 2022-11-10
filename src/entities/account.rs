use std::{fmt::Display, str::FromStr};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::error::Error;

pub(crate) struct Account {
    pub(crate) _kind: Kind,
    pub(crate) _name: Name,
}

#[derive(Clone, Copy, ValueEnum, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub(crate) enum Kind {
    External,
    Budget,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub(crate) struct Name(pub(crate) String);

impl FromStr for Name {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(String::from(s)))
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::Name;
    use proptest::{prelude::*, test_runner::TestRunner};

    #[test]
    fn impl_from_str_for_name() {
        let mut runner = TestRunner::default();
        runner
            .run(&any::<String>(), |string| {
                assert_eq!(
                    <Name as std::str::FromStr>::from_str(&string).unwrap(),
                    Name(string)
                );
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn impl_display_for_name() {
        let mut runner = TestRunner::default();
        runner
            .run(&any::<Name>(), |name| {
                assert_eq!(name.to_string(), name.0);
                Ok(())
            })
            .unwrap();
    }
}
