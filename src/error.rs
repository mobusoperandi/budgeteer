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

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum EventValidateForAppendingToError {
    #[error("`AccountCreated`: `account::Name` collision: {0}")]
    AccountCreatedNameCollision(account::Name),
    #[error("`UnitCreated`: `unit::Name` collision: {0}")]
    UnitCreatedNameCollision(unit::Name),
    #[error("`MoveAdded`: `transaction:Id` not found: {0}")]
    MoveAddedTransactionNotFound(transaction::Id),
    #[error("`MoveAdded`: debit account not found: {0}")]
    MoveAddedDebitAccountNotFound(account::Name),
    #[error("`MoveAdded`: credit account not found: {0}")]
    MoveAddedCreditAccountNotFound(account::Name),
    #[error("`MoveAdded`: unit not found: {0}")]
    MoveAddedUnitNotFound(unit::Name),
    #[error("`MoveAdded`: decimal places mismatch: unit scale: {unit_scale}, amount scale: {amount_scale}")]
    MoveAddedDecimalPlacesMismatch { unit_scale: u8, amount_scale: u32 },
}

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;
