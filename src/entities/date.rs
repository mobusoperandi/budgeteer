use derive_more::{Display, FromStr};
#[cfg(test)]
use proptest::strategy::Strategy;
use serde::{Deserialize, Serialize};

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, FromStr, Display)]
pub(crate) struct NaiveDate(
    #[cfg_attr(test, proptest(strategy = "naive_date_strategy()"))] chrono::NaiveDate,
);

impl NaiveDate {
    pub(crate) fn format<'a>(&self, fmt: &'a str) -> impl std::fmt::Display + 'a {
        self.0.format(fmt)
    }
}

#[cfg(test)]
fn naive_date_strategy() -> impl Strategy<Value = chrono::NaiveDate> {
    use chrono::Datelike;

    (chrono::NaiveDate::MIN.num_days_from_ce()..chrono::NaiveDate::MAX.num_days_from_ce())
        .prop_map(chrono::NaiveDate::from_num_days_from_ce)
}
