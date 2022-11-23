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
    #[error("event invalid for appending: `AccountCreated`: `account::Name` collision: {0}")]
    EventValidateForAppendingToAccountCreatedNameCollision(account::Name),
    #[error("event invalid for appending: `UnitCreated`: `unit::Name` collision: {0}")]
    EventValidateForAppendingToUnitCreatedNameCollision(unit::Name),
    #[error("event invalid for appending: `MoveAdded`: `transaction:Id` not found: {0}")]
    EventValidateForAppendingToMoveAddedTransactionNotFound(transaction::Id),
    #[error("event invalid for appending: `MoveAdded`: debit account not found: {0}")]
    EventValidateForAppendingToMoveAddedDebitAccountNotFound(account::Name),
    #[error("event invalid for appending: `MoveAdded`: credit account not found: {0}")]
    EventValidateForAppendingToMoveAddedCreditAccountNotFound(account::Name),
    #[error("event invalid for appending: `MoveAdded`: unit not found: {0}")]
    EventValidateForAppendingToMoveAddedUnitNotFound(unit::Name),
    #[error("event invalid for appending: `MoveAdded`: decimal places mismatch: unit scale: {unit_scale}, amount scale: {amount_scale}")]
    EventValidateForAppendingToMoveAddedDecimalPlacesMismatch { unit_scale: u8, amount_scale: u32 },
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

pub(crate) type Result<T> = std::result::Result<T, Error>;
