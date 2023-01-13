use crate::entities::{account, amount::NonNegativeAmount, transaction, unit};
use crate::error::{Error, EventValidateForAppendingToError, Result};
use chrono::NaiveDate;
use itertools::Itertools;
use readext::ReadExt;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    AccountCreated(AccountCreated),
    TransactionRecorded(TransactionRecorded),
    UnitCreated(UnitCreated),
    MoveAdded(MoveAdded),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct AccountCreated {
    pub(crate) name: account::Name,
    pub(crate) kind: account::Kind,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(crate) struct TransactionRecorded {
    pub(crate) date: NaiveDate,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct UnitCreated {
    pub(crate) name: unit::Name,
    pub(crate) decimal_places: u8,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(crate) struct MoveAdded {
    pub(crate) transaction: transaction::Id,
    pub(crate) debit_account: account::Name,
    pub(crate) credit_account: account::Name,
    pub(crate) amount: NonNegativeAmount,
    pub(crate) unit: unit::Name,
}

#[derive(Debug)]
pub(crate) struct Events(pub(super) Vec<Event>);

impl Events {
    pub(crate) fn iter(&self) -> std::slice::Iter<Event> {
        self.0.iter()
    }
    pub(crate) fn try_from_reader(reader: &mut impl io::Read) -> Result<Events> {
        let contents = reader
            .read_into_string()
            .map_err(Error::EventsFailedToReadIntoString)?;

        let events: Vec<Event> =
            ron::from_str(&contents).map_err(Error::EventsFailedToDeserialize)?;
        let validated_events =
            events
                .into_iter()
                .try_fold(Events(vec![]), |mut validated_events, event| {
                    validated_events.try_push(event)?;
                    Ok::<Events, Error>(validated_events)
                })?;
        Ok(validated_events)
    }
    pub(crate) fn try_push(&mut self, event: Event) -> Result<()> {
        event.validate_for_appending_to(self)?;
        self.0.push(event);
        Ok(())
    }
}
#[test]
fn ron() {
    let value = vec![Event::AccountCreated(AccountCreated {
        name: account::Name("hello".into()),
        kind: account::Kind::Budget,
    })];
    let string = ron::to_string(&value).unwrap();
    let parsed: Vec<Event> = ron::from_str(&string).unwrap();
    assert_eq!(value, parsed);
}
impl Event {
    fn validate_for_appending_to(
        &self,
        events: &Events,
    ) -> Result<(), EventValidateForAppendingToError> {
        match self {
            Event::AccountCreated(AccountCreated { name, .. }) => {
                let name_collision = events.all_account_names().into_iter().contains(name);
                match name_collision {
                    true => Err(
                        EventValidateForAppendingToError::AccountCreatedNameCollision(name.clone()),
                    ),
                    false => Ok(()),
                }
            }
            Event::TransactionRecorded(_) => Ok(()),
            Event::UnitCreated(UnitCreated { name, .. }) => {
                let name_collision = events.all_unit_names().into_iter().contains(name);
                match name_collision {
                    true => Err(EventValidateForAppendingToError::UnitCreatedNameCollision(
                        name.clone(),
                    )),
                    false => Ok(()),
                }
            }
            Event::MoveAdded(MoveAdded {
                transaction,
                debit_account,
                credit_account,
                amount,
                unit,
            }) => {
                let transaction_found = events
                    .all_transaction_ids()
                    .into_iter()
                    .contains(transaction);
                if !transaction_found {
                    return Err(
                        EventValidateForAppendingToError::MoveAddedTransactionNotFound(
                            *transaction,
                        ),
                    );
                }
                let debit_account_found = events
                    .all_account_names()
                    .into_iter()
                    .contains(debit_account);
                if !debit_account_found {
                    return Err(
                        EventValidateForAppendingToError::MoveAddedDebitAccountNotFound(
                            debit_account.clone(),
                        ),
                    );
                }
                let credit_account_found = events
                    .all_account_names()
                    .into_iter()
                    .contains(credit_account);
                if !credit_account_found {
                    return Err(
                        EventValidateForAppendingToError::MoveAddedCreditAccountNotFound(
                            credit_account.clone(),
                        ),
                    );
                }
                let unit = events.get_unit(unit).ok_or_else(|| {
                    EventValidateForAppendingToError::MoveAddedUnitNotFound(unit.clone())
                })?;
                if amount.scale() != unit.decimal_places as u32 {
                    return Err(
                        EventValidateForAppendingToError::MoveAddedDecimalPlacesMismatch {
                            unit_scale: unit.decimal_places,
                            amount_scale: amount.scale(),
                        },
                    );
                }
                Ok(())
            }
        }
    }
}
