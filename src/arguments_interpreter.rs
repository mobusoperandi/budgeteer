use anyhow::{bail, Result};

use crate::{cli, events, reports};

pub(crate) fn interpret(
    args: cli::Arguments,
) -> Result<(Option<events::Event>, Option<reports::Report>)> {
    match args.category {
        cli::Category::Account(cli::Account::Create(cli::AccountCreate { kind, name })) => Ok((
            Some(events::Event::AccountCreated(events::AccountCreated {
                name,
                kind,
            })),
            None,
        )),
        cli::Category::Transaction(cli::Transaction::Record(cli::TransactionRecord { date })) => {
            Ok((
                Some(events::Event::TransactionRecorded(
                    events::TransactionRecorded { date },
                )),
                Some(reports::Report::TransactionRecordResponse),
            ))
        }
        cli::Category::Transaction(cli::Transaction::Show(cli::TransactionShow { id })) => {
            Ok((None, Some(reports::Report::TransactionShow { id })))
        }
        cli::Category::Unit(cli::Unit::Create(cli::UnitCreate {
            name,
            decimal_places,
        })) => Ok((
            Some(events::Event::UnitCreated(events::UnitCreated {
                name,
                decimal_places,
            })),
            None,
        )),
        cli::Category::Move(cli::Move::Add(cli::MoveAdd {
            transaction,
            debit_account,
            credit_account,
            amount,
            unit,
        })) => {
            if debit_account == credit_account {
                bail!("Debit account equals credit account");
            }
            if amount.0.is_sign_negative() {
                bail!("Negative move amount");
            }
            Ok((
                Some(events::Event::MoveAdded(events::MoveAdded {
                    transaction,
                    debit_account,
                    credit_account,
                    amount,
                    unit,
                })),
                None,
            ))
        }
        cli::Category::Balances => Ok((None, Some(reports::Report::Balances))),
        cli::Category::RunningBalance(cli::RunningBalance { account, unit }) => Ok((
            None,
            Some(reports::Report::RunningBalance { account, unit }),
        )),
    }
}
