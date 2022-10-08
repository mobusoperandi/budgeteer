use super::{account, amount::Amount, transaction, unit};

pub(crate) struct Move {
    pub(crate) transaction: transaction::Id,
    pub(crate) debit_account: account::Name,
    pub(crate) credit_account: account::Name,
    pub(crate) amount: Amount,
    pub(crate) unit: unit::Name,
}
