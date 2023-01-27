use crate::entities::{account, transaction, unit};

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("parsing `NonNegativeAmount`: {0}")]
    NonNegativeAmountFailedToParseDecimal(rust_decimal::Error),
    #[error("parsing `NonNegativeAmount`: negative decimal")]
    NonNegativeAmountParsedNegativeDecimal,
    #[error("parsing `Amount`: {0}")]
    AmountFailedToParseDecimal(rust_decimal::Error),
    #[error("parsing `transaction::Id`: {0}")]
    TransactionIdFailedToParse(std::num::ParseIntError),
    #[error("event invalid for appending: {0}")]
    EventValidateForAppendingTo(#[from] EventValidateForAppendingToError),
    #[error("reading serialized events into string: {0}")]
    EventsFailedToReadIntoString(std::io::Error),
    #[error("deserializing events: {0}")]
    EventsFailedToDeserialize(ron::error::SpannedError),
    #[error("serializing events: {0}")]
    EventsFailedToSerialize(ron::Error),
    #[error("generating report `TransactionShow`: transaction not found: {0}")]
    ReportTransactionShowTransactionNotFound(transaction::Id),
    #[error("invalid arguments: `MoveAdd`: same account: {0}")]
    ArgumentsInterpreterMoveAddSameAccount(crate::entities::account::Name),
    #[error("failed to open persistence file: {0}")]
    PersistenceFileOpenFailed(std::io::Error),
    #[error("failed to initialize persistence file: {0}")]
    PersistenceFailedToInitialze(std::io::Error),
    #[error("failed to rewind initialized persistence file: {0}")]
    PersistenceFailedToRewindInitializedFile(std::io::Error),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq, PartialOrd, Ord, Default)]
#[error("{self:?}")]
pub(crate) struct EventValidateForAppendingToErrorMoveAdded {
    pub(crate) transaction_not_found: Option<transaction::Id>,
    pub(crate) debit_account_not_found: Option<account::Name>,
    pub(crate) credit_account_not_found: Option<account::Name>,
    pub(crate) unit: Option<EventValidateForAppendingToErrorMoveAddedUnit>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum EventValidateForAppendingToErrorMoveAddedUnit {
    UnitNotFound(unit::Name),
    DecimalPlacesMismatch { unit_scale: u8, amount_scale: u32 },
}

#[derive(Debug, thiserror::Error, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum EventValidateForAppendingToError {
    #[error("`AccountCreated`: `account::Name` collision: {0}")]
    AccountCreatedNameCollision(account::Name),
    #[error("`UnitCreated`: `unit::Name` collision: {0}")]
    UnitCreatedNameCollision(unit::Name),
    #[error("{0}")]
    MoveAdded(EventValidateForAppendingToErrorMoveAdded),
}

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;
