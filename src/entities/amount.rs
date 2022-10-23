use std::{fmt::Display, ops, str::FromStr};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
pub(crate) struct Amount(pub(crate) Decimal);

impl ops::SubAssign<Self> for Amount {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl ops::AddAssign<Self> for Amount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Decimal as Display>::fmt(&self.0, f)
    }
}

impl FromStr for Amount {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Decimal::from_str_exact(s)?))
    }
}
