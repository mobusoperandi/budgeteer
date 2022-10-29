use std::{collections::BTreeMap, ops};

use crate::{
    entities::{account, amount::Amount, transaction, unit},
    events::Events,
};
use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use cli_table::{Cell, Row, Table};
use itertools::Itertools;

pub(crate) enum Report {
    TransactionRecordResponse,
    Balances,
    RunningBalance {
        account: account::Name,
        unit: unit::Name,
    },
    TransactionShow {
        id: transaction::Id,
    },
}

impl Report {
    pub(crate) fn compile(&self, events: &Events) -> Result<String> {
        let output = match self {
            Report::TransactionRecordResponse => {
                let last_transaction_id = events.last_transaction_id();
                format!("Recorded transaction {last_transaction_id}\n")
            }
            Report::Balances => format_table(
                events.all_balances().into_iter().map(|(name, balance)| {
                    let sums = balance.0.into_iter().flat_map(|(name, amount)| {
                        [
                            amount
                                .0
                                .to_string()
                                .cell()
                                .justify(cli_table::format::Justify::Right),
                            name.0.cell(),
                        ]
                    });
                    [name.0.cell()].into_iter().chain(sums)
                }),
                ["account", "balance", ""],
            ),
            Report::RunningBalance { account, unit } => format_table(
                events
                    .all_moves()
                    .filter_map(|move_| {
                        if [&move_.debit_account, &move_.credit_account].contains(&account)
                            && move_.unit == *unit
                        {
                            Some((events.get_transaction(&move_.transaction)?, move_))
                        } else {
                            None
                        }
                    })
                    .sorted_by_key(|(transaction, _move)| transaction.id)
                    .fold(
                        (
                            BTreeMap::<transaction::Id, (NaiveDate, Amount, Amount)>::new(),
                            Amount::default(),
                        ),
                        |(mut rows, running_balance), (transaction, move_)| {
                            let (_transaction_date, row_affect, row_balance) = rows
                                .entry(transaction.id)
                                .or_insert((transaction.date, Default::default(), running_balance));
                            let operation = if account == &move_.debit_account {
                                ops::SubAssign::sub_assign
                            } else if account == &move_.credit_account {
                                ops::AddAssign::add_assign
                            } else {
                                unreachable!()
                            };
                            operation(row_affect, move_.amount.into());
                            operation(row_balance, move_.amount.into());
                            let running_balance = *row_balance;
                            (rows, running_balance)
                        },
                    )
                    .0
                    .into_iter()
                    .map(|(transaction_id, (transaction_date, affect, balance))| {
                        [
                            format!("{} {}", transaction_id, transaction_date).cell(),
                            format!("{affect:+}").cell(),
                            balance.cell(),
                        ]
                    }),
                ["transaction", "affect", "balance"],
            ),
            Report::TransactionShow { id } => {
                let table = format_table(
                    events.all_moves().filter_map(|move_| {
                        if &move_.transaction == id {
                            Some([
                                move_.debit_account.cell(),
                                move_.credit_account.cell(),
                                format!("{} {}", move_.amount, move_.unit).cell(),
                            ])
                        } else {
                            None
                        }
                    }),
                    ["from", "to", "amount"],
                );

                let transaction_date = events
                    .get_transaction(id)
                    .ok_or_else(|| anyhow!("No such trasnaction"))?
                    .date
                    .format("%F");

                format!("{transaction_date}\n{table}")
            }
        };
        // TODO perhaps if we use the table crate to print, it would detect TTY
        Ok(ansitok::parse_ansi(&output)
            .into_iter()
            .filter_map(|text_or_code| match text_or_code {
                ansitok::Output::Text(text) => Some(String::from(text)),
                ansitok::Output::Escape(_) => None,
            })
            .collect())
    }
}

fn format_table(rows: impl IntoIterator<Item = impl Row>, titles: impl Row) -> String {
    let table_border = cli_table::format::Border::builder().build();
    let table_separator = cli_table::format::Separator::builder().build();
    rows.table()
        .title(titles)
        .border(table_border)
        .separator(table_separator)
        .display()
        .unwrap()
        .to_string()
}
