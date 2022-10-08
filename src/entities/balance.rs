use super::{amount::Amount, unit};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub(crate) struct Balance(pub(crate) BTreeMap<unit::Name, Amount>);
