use crate::{
    cli,
    error::{Error, Result},
    events::{self, Event},
    reports::Report,
};

#[derive(Debug, PartialEq, Eq)]
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
                return Err(Error::ArgumentsInterpreterMoveAddSameAccount(debit_account));
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

#[cfg(test)]
mod test {
    use chrono::{Datelike, NaiveDate};
    use proptest::{prelude::*, test_runner::TestRunner};

    use crate::{
        cli::{self, MoveAdd},
        entities::{account, transaction, unit},
        error::Error,
        events::{self, Event},
        reports::Report,
    };

    use super::{interpret, Actions};

    fn naive_date_strategy() -> impl Strategy<Value = NaiveDate> {
        (NaiveDate::MIN.num_days_from_ce()..NaiveDate::MAX.num_days_from_ce())
            .prop_map(NaiveDate::from_num_days_from_ce)
    }

    #[test]
    fn account_create() {
        let mut runner = TestRunner::default();
        runner
            .run(
                &(any::<account::Name>(), any::<account::Kind>()),
                |(name, kind)| {
                    assert_eq!(
                        interpret(cli::Arguments {
                            category: cli::Category::Account(cli::Account::Create(
                                cli::AccountCreate {
                                    name: name.clone(),
                                    kind,
                                }
                            ))
                        })
                        .unwrap(),
                        Actions {
                            event: Some(Event::AccountCreated(events::AccountCreated {
                                name,
                                kind
                            })),
                            report: None
                        }
                    );
                    Ok(())
                },
            )
            .unwrap();
    }

    #[test]
    fn transaction_record() {
        let mut runner = TestRunner::default();
        runner
            .run(&naive_date_strategy(), |date| {
                assert_eq!(
                    interpret(cli::Arguments {
                        category: cli::Category::Transaction(cli::Transaction::Record(
                            cli::TransactionRecord { date }
                        )),
                    })
                    .unwrap(),
                    Actions {
                        event: Some(Event::TransactionRecorded(events::TransactionRecorded {
                            date
                        })),
                        report: Some(Report::TransactionRecordResponse)
                    }
                );
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn transaction_show() {
        let mut runner = TestRunner::default();
        runner
            .run(&any::<transaction::Id>(), |id| {
                assert_eq!(
                    interpret(cli::Arguments {
                        category: cli::Category::Transaction(cli::Transaction::Show(
                            cli::TransactionShow { id }
                        )),
                    })
                    .unwrap(),
                    Actions {
                        event: None,
                        report: Some(Report::TransactionShow { id })
                    }
                );
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn unit_create() {
        let mut runner = TestRunner::default();
        runner
            .run(
                &(any::<unit::Name>(), any::<u8>()),
                |(name, decimal_places)| {
                    assert_eq!(
                        interpret(cli::Arguments {
                            category: cli::Category::Unit(cli::Unit::Create(cli::UnitCreate {
                                decimal_places,
                                name: name.clone(),
                            }))
                        })
                        .unwrap(),
                        Actions {
                            event: Some(Event::UnitCreated(events::UnitCreated {
                                name,
                                decimal_places
                            })),
                            report: None
                        }
                    );
                    Ok(())
                },
            )
            .unwrap();
    }
    #[test]
    fn move_add() {
        let mut runner = TestRunner::default();
        runner
            .run(&any::<MoveAdd>(), |move_add| {
                assert_eq!(
                    interpret(cli::Arguments {
                        category: cli::Category::Move(cli::Move::Add(move_add.clone()))
                    })
                    .unwrap(),
                    Actions {
                        event: Some(Event::MoveAdded(events::MoveAdded {
                            transaction: move_add.transaction,
                            debit_account: move_add.debit_account,
                            credit_account: move_add.credit_account,
                            amount: move_add.amount,
                            unit: move_add.unit
                        })),
                        report: None
                    }
                );
                Ok(())
            })
            .unwrap();
    }
    #[test]
    fn move_add_when_account_names_identical() {
        let mut runner = TestRunner::default();
        runner
            .run(
                &any::<MoveAdd>().prop_map(|mut move_add| {
                    move_add.credit_account.0 = move_add.debit_account.0.clone();
                    move_add
                }),
                |move_add| {
                    let error = interpret(cli::Arguments {
                        category: cli::Category::Move(cli::Move::Add(move_add)),
                    })
                    .unwrap_err();
                    assert!(matches!(
                        error,
                        Error::ArgumentsInterpreterMoveAddSameAccount(_)
                    ));
                    Ok(())
                },
            )
            .unwrap();
    }
}
