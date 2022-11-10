use std::str::FromStr;

use derive_more::{AddAssign, Display, SubAssign};
use rust_decimal::Decimal as RustDecimal;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    Eq,
    Default,
    SubAssign,
    AddAssign,
    Display,
)]
pub(crate) struct Decimal(RustDecimal);

impl Decimal {
    fn is_sign_negative(&self) -> bool {
        self.0.is_sign_negative()
    }
    fn scale(&self) -> u32 {
        self.0.scale()
    }
    #[cfg(test)]
    fn from_parts(lo: u32, mid: u32, hi: u32, negative: bool, scale: u32) -> Self {
        Self(RustDecimal::from_parts(lo, mid, hi, negative, scale))
    }
    #[cfg(test)]
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl FromStr for Amount {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Decimal::from_str(s)
            .map_err(Error::AmountFailedToParseDecimal)
            .map(Self)
    }
}

impl FromStr for Decimal {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(RustDecimal::from_str_exact(s)?))
    }
}

#[derive(
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    Eq,
    Default,
    SubAssign,
    AddAssign,
    Display,
)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub(crate) struct Amount(Decimal);

#[derive(
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    Eq,
    Default,
    SubAssign,
    AddAssign,
    Display,
)]

pub(crate) struct NonNegativeAmount(Decimal);

impl NonNegativeAmount {
    pub(crate) fn scale(&self) -> u32 {
        self.0.scale()
    }
}

impl FromStr for NonNegativeAmount {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decimal = Decimal::from_str(s).map_err(Error::NonNegativeAmountFailedToParseDecimal)?;
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

#[cfg(test)]
mod test {
    use crate::error::Error;

    use super::{Decimal, NonNegativeAmount};
    use proptest::{prelude::*, test_runner::TestRunner};
    use std::str::FromStr;

    impl Arbitrary for NonNegativeAmount {
        type Parameters = Option<u8>;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(decimal_places: Self::Parameters) -> Self::Strategy {
            Decimal::arbitrary_with((ArbitraryDecimalSignParam::NonNegative, decimal_places))
                .prop_map(Self)
                .boxed()
        }
    }

    #[derive(Debug, Default, Eq, PartialEq)]
    pub(crate) enum ArbitraryDecimalSignParam {
        #[default]
        Any,
        Negative,
        // TODO use this in another test
        NonNegative,
    }

    impl Arbitrary for Decimal {
        type Parameters = (ArbitraryDecimalSignParam, Option<u8>);
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((sign, decimal_places): Self::Parameters) -> Self::Strategy {
            (
                any::<u32>(),
                any::<u32>(),
                any::<u32>(),
                any::<bool>(),
                decimal_places.map_or_else(|| (0u8..28u8).boxed(), |v| Just(v).boxed()),
            )
                .prop_filter_map(
                    "negative zero",
                    Box::new(move |(lo, mid, hi, negative, scale)| {
                        let negative = match sign {
                            ArbitraryDecimalSignParam::Any => negative,
                            ArbitraryDecimalSignParam::Negative => true,
                            ArbitraryDecimalSignParam::NonNegative => false,
                        };
                        let decimal = Decimal::from_parts(lo, mid, hi, negative, scale as u32);
                        if decimal.is_zero() && sign == ArbitraryDecimalSignParam::Negative {
                            None
                        } else {
                            Some(decimal)
                        }
                    }),
                )
                .boxed()
        }
    }

    #[test]
    fn decimal_zero_is_non_negative() {
        let decimal = Decimal::from_str("-0").unwrap();
        assert!(!decimal.is_sign_negative());
        let decimal = Decimal::from_parts(0, 0, 0, true, 0);
        assert!(!decimal.is_sign_negative());
    }

    #[test]
    fn impl_from_str_for_non_negative_amount_negative() {
        let mut runner = TestRunner::default();
        runner
            .run(
                &Decimal::arbitrary_with((ArbitraryDecimalSignParam::NonNegative, None))
                    .prop_map(|decimal| decimal.to_string()),
                |negative_decimal| {
                    assert!(matches!(
                        NonNegativeAmount::from_str(&negative_decimal)
                            .err()
                            .unwrap(),
                        Error::NonNegativeAmountParsedNegativeDecimal
                    ));
                    Ok(())
                },
            )
            .unwrap();
    }

    #[test]
    fn impl_from_str_for_non_negative_amount_non_negative() {
        let mut runner = TestRunner::default();
        runner
            .run(
                &Decimal::arbitrary_with((ArbitraryDecimalSignParam::NonNegative, None))
                    .prop_map(|decimal| decimal.to_string()),
                |negative_decimal| {
                    assert!(NonNegativeAmount::from_str(&negative_decimal).is_ok());
                    Ok(())
                },
            )
            .unwrap();
    }
}
