use std::collections::BTreeMap;

use crate::{
    entities::{
        account::{self, Account},
        balance::Balance,
        move_::Move,
        transaction::{self, Transaction},
        unit::{self, Unit},
    },
    events::{self, Event, Events},
};

impl Events {
    pub(crate) fn all_account_names(&self) -> Vec<account::Name> {
        self.all_accounts().keys().cloned().collect()
    }
    pub(crate) fn all_accounts(&self) -> BTreeMap<account::Name, Account> {
        self.iter()
            .filter_map(|event| match event {
                Event::AccountCreated(account_created) => Some(account_created),
                _ => None,
            })
            .cloned()
            .fold(
                BTreeMap::new(),
                |mut accounts, events::AccountCreated { name, kind }| {
                    accounts.insert(
                        name.clone(),
                        Account {
                            _kind: kind,
                            _name: name,
                        },
                    );
                    accounts
                },
            )
    }
    pub(crate) fn all_unit_names(&self) -> Vec<unit::Name> {
        self.all_units().keys().cloned().collect()
    }
    pub(crate) fn all_units(&self) -> BTreeMap<unit::Name, Unit> {
        self.iter()
            .filter_map(|event| match event {
                Event::UnitCreated(unit_created) => Some(unit_created),
                _ => None,
            })
            .cloned()
            .fold(
                BTreeMap::new(),
                |mut units,
                 events::UnitCreated {
                     name,
                     decimal_places,
                 }| {
                    units.insert(
                        name.clone(),
                        Unit {
                            _name: name,
                            decimal_places,
                        },
                    );
                    units
                },
            )
    }
    pub(crate) fn all_transaction_ids(&self) -> Vec<transaction::Id> {
        let transaction_recorded_count = self
            .iter()
            .filter(|event| matches!(event, Event::TransactionRecorded(_)))
            .count();
        (1..=transaction_recorded_count)
            .into_iter()
            .map(|id| transaction::Id(id as u64))
            .collect()
    }
    pub(crate) fn get_unit(&self, unit_name: &unit::Name) -> Option<Unit> {
        self.all_units().get(unit_name).cloned()
    }
    pub(crate) fn all_moves(&'_ self) -> impl Iterator<Item = Move> + '_ {
        self.iter().filter_map(|event| match event {
            Event::MoveAdded(events::MoveAdded {
                debit_account,
                credit_account,
                amount,
                unit,
                transaction,
            }) => Some(Move {
                transaction: *transaction,
                debit_account: debit_account.clone(),
                credit_account: credit_account.clone(),
                amount: *amount,
                unit: unit.clone(),
            }),
            _ => None,
        })
    }
    pub(crate) fn get_transaction(&self, transaction_id: &transaction::Id) -> Option<Transaction> {
        self.iter()
            .filter_map(|event| {
                if let Event::TransactionRecorded(transaction_recorded) = event {
                    Some(transaction_recorded)
                } else {
                    None
                }
            })
            .enumerate()
            .find_map(|(index, transaction_created)| {
                let current_id = transaction::Id(index as u64 + 1);
                if &current_id == transaction_id {
                    Some(Transaction {
                        id: current_id,
                        date: transaction_created.date,
                    })
                } else {
                    None
                }
            })
    }
    pub(crate) fn all_balances(&self) -> BTreeMap<account::Name, Balance> {
        self.all_moves()
            .fold(BTreeMap::new(), |mut balances, move_| {
                let balance = balances.entry(move_.debit_account).or_default();
                let unit_amount = balance.0.entry(move_.unit.clone()).or_default();
                *unit_amount -= move_.amount.into();

                let balance = balances.entry(move_.credit_account).or_default();
                let unit_amount = balance.0.entry(move_.unit).or_default();
                *unit_amount += move_.amount.into();

                balances
            })
    }
    pub(crate) fn last_transaction_id(&self) -> transaction::Id {
        *self.all_transaction_ids().last().unwrap()
    }
}
