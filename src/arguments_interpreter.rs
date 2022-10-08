use anyhow::{bail, Result};

use crate::{
    cli,
    events::{self, Event},
    reports::Report,
};

pub(crate) struct Actions {
    pub(crate) event: Option<Event>,
    pub(crate) report: Option<Report>,
}

pub(crate) fn interpret(args: cli::Arguments) -> Result<Actions> {
    match args.category {
        cli::Category::Account(cli::Account::Create(cli::AccountCreate { kind, name })) => {
            Ok(Actions {
                event: Some(Event::AccountCreated(events::AccountCreated { name, kind })),
                report: None,
            })
        }
        cli::Category::Transaction(cli::Transaction::Record(cli::TransactionRecord { date })) => {
            Ok(Actions {
                event: Some(Event::TransactionRecorded(events::TransactionRecorded {
                    date,
                })),
                report: Some(Report::TransactionRecordResponse),
            })
        }
        cli::Category::Transaction(cli::Transaction::Show(cli::TransactionShow { id })) => {
            Ok(Actions {
                event: None,
                report: Some(Report::TransactionShow { id }),
            })
        }
        cli::Category::Unit(cli::Unit::Create(cli::UnitCreate {
            name,
            decimal_places,
        })) => Ok(Actions {
            event: Some(Event::UnitCreated(events::UnitCreated {
                name,
                decimal_places,
            })),
            report: None,
        }),
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
            Ok(Actions {
                event: Some(Event::MoveAdded(events::MoveAdded {
                    transaction,
                    debit_account,
                    credit_account,
                    amount,
                    unit,
                })),
                report: None,
            })
        }
        cli::Category::Balances => Ok(Actions {
            event: None,
            report: Some(Report::Balances),
        }),
        cli::Category::RunningBalance(cli::RunningBalance { account, unit }) => Ok(Actions {
            event: None,
            report: Some(Report::RunningBalance { account, unit }),
        }),
    }
}
