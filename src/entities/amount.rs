use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops, str::FromStr};

use crate::error::Error;

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
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Decimal::from_str_exact(s)
            .map_err(Error::AmountFailedToParseDecimal)
            .map(Self)
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
pub(crate) struct NonNegativeAmount(Decimal);

impl NonNegativeAmount {
    pub(crate) fn scale(&self) -> u32 {
        self.0.scale()
    }
}

impl FromStr for NonNegativeAmount {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decimal =
            Decimal::from_str_exact(s).map_err(Error::NonNegativeAmountFailedToParseDecimal)?;
        if decimal.is_sign_negative() {
            Err(Error::NonNegativeAmountParsedNegativeDecimal)
        } else {
            Ok(Self(decimal))
        }
    }
}

impl From<NonNegativeAmount> for Amount {
    fn from(non_negative_amount: NonNegativeAmount) -> Self {
        Amount(non_negative_amount.0)
    }
}

impl Display for NonNegativeAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Decimal as Display>::fmt(&self.0, f)
    }
}
