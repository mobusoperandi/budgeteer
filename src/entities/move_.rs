use super::{account, amount::NonNegativeAmount, transaction, unit};

pub(crate) struct Move {
    pub(crate) transaction: transaction::Id,
    pub(crate) debit_account: account::Name,
    pub(crate) credit_account: account::Name,
    pub(crate) amount: NonNegativeAmount,
    pub(crate) unit: unit::Name,
}
