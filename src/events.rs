use crate::entities::{account, amount, transaction, unit};
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
    pub(crate) amount: amount::Amount,
    pub(crate) unit: unit::Name,
}

#[derive(Debug)]
pub(crate) struct Events(Vec<Event>);

impl Events {
    pub(crate) fn iter(&self) -> std::slice::Iter<Event> {
        self.0.iter()
    }
    pub(crate) fn try_from_reader(
        reader: &mut impl io::Read,
    ) -> Result<Events, Box<dyn std::error::Error>> {
        let contents = reader.read_into_string()?;

        let events: Vec<Event> = ron::from_str(&contents)?;
        let validated_events = events.into_iter().try_fold(
            Events(vec![]),
            |mut validated_events, event| -> Result<Events, String> {
                validated_events.try_push(event)?;
                Ok(validated_events)
            },
        )?;
        Ok(validated_events)
    }
    pub(crate) fn try_push(&mut self, event: Event) -> Result<(), String> {
        event.validate_for_appending_to(self)?;
        self.0.push(event);
        Ok(())
    }
    pub(crate) fn try_write(
        &self,
        writer: &mut impl io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = ron::to_string(&self.0)?;
        writer.write_all(serialized.as_bytes())?;
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
    fn validate_for_appending_to(&self, events: &Events) -> Result<(), String> {
        match self {
            Event::AccountCreated(AccountCreated { name, .. }) => {
                let name_collision = events.all_account_names().into_iter().contains(name);
                match name_collision {
                    true => Err("Name collision detected!".into()),
                    false => Ok(()),
                }
            }
            Event::TransactionRecorded(_) => Ok(()),
            Event::UnitCreated(UnitCreated { name, .. }) => {
                let name_collision = events.all_unit_names().into_iter().contains(name);
                match name_collision {
                    true => Err("Name collision detected!".into()),
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
                    return Err(format!("Transaction not found for id: {transaction}"));
                }
                let debit_account_found = events
                    .all_account_names()
                    .into_iter()
                    .contains(debit_account);
                if !debit_account_found {
                    return Err(format!("Debit account not found {debit_account}"));
                }
                let credit_account_found = events
                    .all_account_names()
                    .into_iter()
                    .contains(credit_account);
                if !credit_account_found {
                    return Err(format!("Credit account not found {credit_account}"));
                }
                let unit = events
                    .get_unit(unit)
                    .ok_or(format!("Unit not found {unit}"))?;
                if amount.0.scale() != unit.decimal_places as u32 {
                    return Err(format!(
                        "Amount decimal places {} is different than the unit decimal places {}",
                        amount.0.scale(),
                        unit.decimal_places
                    ));
                }
                Ok(())
            }
        }
    }
}
