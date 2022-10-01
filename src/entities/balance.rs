use super::{amount, unit};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub(crate) struct Balance(pub(crate) BTreeMap<unit::Name, amount::Amount>);
