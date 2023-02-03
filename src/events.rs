use crate::entities::date::NaiveDate;
use crate::entities::{account, amount::NonNegativeAmount, transaction, unit};
use crate::error::{
    Error, EventValidateForAppendingToError, EventValidateForAppendingToErrorMoveAdded,
    EventValidateForAppendingToErrorMoveAddedUnit, Result,
};
use itertools::Itertools;
use readext::ReadExt;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
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

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub(crate) struct TransactionRecorded {
    pub(crate) date: NaiveDate,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct UnitCreated {
    pub(crate) name: unit::Name,
    pub(crate) decimal_places: u8,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub(crate) struct MoveAdded {
    pub(crate) transaction: transaction::Id,
    pub(crate) debit_account: account::Name,
    pub(crate) credit_account: account::Name,
    pub(crate) amount: NonNegativeAmount,
    pub(crate) unit: unit::Name,
}

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub(crate) struct Events(pub(super) Vec<Event>);

impl IntoIterator for Events {
    type Item = Event;
    type IntoIter = std::vec::IntoIter<Event>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

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
                let mut error: Option<EventValidateForAppendingToErrorMoveAdded> = None;

                let transaction_found = events
                    .all_transaction_ids()
                    .into_iter()
                    .contains(transaction);
                if !transaction_found {
                    error
                        .get_or_insert(Default::default())
                        .transaction_not_found = Some(*transaction);
                }
                let debit_account_found = events
                    .all_account_names()
                    .into_iter()
                    .contains(debit_account);
                if !debit_account_found {
                    error
                        .get_or_insert(Default::default())
                        .debit_account_not_found = Some(debit_account.clone());
                }
                let credit_account_found = events
                    .all_account_names()
                    .into_iter()
                    .contains(credit_account);
                if !credit_account_found {
                    error
                        .get_or_insert(Default::default())
                        .credit_account_not_found = Some(credit_account.clone());
                }
                let Some(unit) = events.get_unit(unit) else {
                    let mut error = error.unwrap_or_default();
                    error.unit = Some(EventValidateForAppendingToErrorMoveAddedUnit::UnitNotFound(unit.clone()));
                    return Err(EventValidateForAppendingToError::MoveAdded(error));
                };
                if amount.scale() != unit.decimal_places as u32 {
                    error.get_or_insert(Default::default()).unit = Some(
                        EventValidateForAppendingToErrorMoveAddedUnit::DecimalPlacesMismatch {
                            unit_scale: unit.decimal_places,
                            amount_scale: amount.scale(),
                        },
                    );
                }
                if let Some(error) = error {
                    Err(EventValidateForAppendingToError::MoveAdded(error))
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::{collections::BTreeSet, io, default};

    use crate::{
        entities::{account, amount::Amount, transaction, unit::Unit},
        error::Error,
        events::AccountCreated,
    };

    use super::*;
    use bytes::{buf::Reader, Buf, Bytes};
    use proptest::{
        prelude::*,
        sample::{select, Index},
        strategy::Union,
        test_runner::TestRunner,
    };

    #[derive(Debug, Default, Clone)]
    pub(crate) struct Observations {
        account_names: BTreeSet<account::Name>,
        transaction_recorded_events: u64,
        unit_created_events: Vec<UnitCreated>,
    }

    impl<'a> FromIterator<&'a Event> for Observations {
        fn from_iter<T: IntoIterator<Item = &'a Event>>(iter: T) -> Self {
            iter.into_iter()
                .fold(Observations::default(), |mut observations, event| {
                    match event {
                        Event::AccountCreated(account_created) => {
                            observations
                                .account_names
                                .insert(account_created.name.clone());
                        }
                        Event::TransactionRecorded(_) => {
                            observations.transaction_recorded_events += 1;
                        }
                        Event::UnitCreated(unit_created) => {
                            observations.unit_created_events.push(unit_created.clone());
                        }
                        _ => {}
                    };
                    observations
                })
        }
    }

    #[derive(Default)]
    pub(crate) enum ArbitraryEventParam {
        #[default]
        Any,
        ValidAfter(Events),
        InvalidAfter(Events),
    }

    impl Arbitrary for Events {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (0usize..=20)
                .prop_flat_map(|length| {
                    fn recurse(
                        length: usize,
                        events_strategy: BoxedStrategy<Events>,
                    ) -> BoxedStrategy<Events> {
                        events_strategy
                            .prop_flat_map(move |events| {
                                if events.0.len() == length {
                                    return Just(events).boxed();
                                }
                                let events_strategy = (
                                    Just(events.clone()),
                                    Event::arbitrary_with(ArbitraryEventParam::ValidAfter(events)),
                                )
                                    .prop_flat_map(|(mut events, event)| {
                                        events.0.push(event);
                                        Just(events)
                                    });
                                recurse(length, events_strategy.boxed())
                            })
                            .boxed()
                    }
                    recurse(length, Just(Events::default()).boxed())
                })
                .boxed()
        }
    }

    #[derive(Debug, Default)]
    pub(crate) enum ArbitraryMoveAddedParam {
        #[default]
        Any,
        With(Observations, MoveAddedInvalidities),
    }

    impl MoveAdded {
        fn is_possible(observations: &Observations) -> bool {
            observations.transaction_recorded_events > 0
                && (observations.account_names.len() >= 2)
                && !observations.unit_created_events.is_empty()
        }
    }
    
    #[derive(Default)]
    enum ArbitraryMoveAddedInvaliditiesParam {
        #[default]
        Valid,
        Invalid
    }

    impl Arbitrary for MoveAddedInvalidities {
        type Parameters = ArbitraryMoveAddedInvaliditiesParam;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(arg: Self::Parameters) -> Self::Strategy {
            match arg {
                ArbitraryMoveAddedInvaliditiesParam::Valid => todo!(),
                ArbitraryMoveAddedInvaliditiesParam::Invalid => todo!(),
            }
        }
    }

    impl Arbitrary for MoveAdded {
        type Parameters = ArbitraryMoveAddedParam;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(arg: Self::Parameters) -> Self::Strategy {
            match arg {
                ArbitraryMoveAddedParam::Any => (
                    any::<transaction::Id>(),
                    any::<account::Name>(),
                    any::<account::Name>(),
                    any::<Unit>(),
                )
                    .prop_filter(
                        "debit and credit account names identical",
                        |(_transaction_id, debit_account_name, credit_account_name, _unit)| {
                            debit_account_name != credit_account_name
                        },
                    )
                    .prop_flat_map(|(transaction, debit_account, credit_account, unit)| {
                        NonNegativeAmount::arbitrary_with(Some(unit.decimal_places)).prop_map(
                            move |amount| Self {
                                transaction,
                                debit_account: debit_account.clone(),
                                credit_account: credit_account.clone(),
                                amount,
                                unit: unit._name.clone(),
                            },
                        )
                    })
                    .boxed(),
                ArbitraryMoveAddedParam::With(observations, invalidities) => {
                    let account_names_length = observations.account_names.len();
                    let transaction_ids_length = observations.transaction_recorded_events;
                    let units_length = observations.unit_created_events.len();
                    assert!(transaction_ids_length > 0);
                    assert!(account_names_length >= 2);
                    assert!(units_length >= 1);

                    let transaction_id_strategy = if invalidities.transaction_not_found {
                        (transaction_ids_length..).boxed()
                    } else {
                        (0..transaction_ids_length).boxed()
                    }
                    .prop_map(transaction::Id);
                    let debit_account_strategy = if invalidities.debit_account_not_found {
                        any::<account::Name>()
                            .prop_filter("debit account name happens to be valid", |name| {
                                !observations.account_names.contains(name)
                            })
                            .boxed()
                    } else {
                        select(observations.account_names.into_iter().collect_vec()).boxed()
                    };
                    let credit_account_strategy =
                        (&debit_account_strategy, &debit_account_strategy.clone()).prop_filter_map(
                            "debit and credit account names identical",
                            |(debit, credit)| (debit != credit).then_some(credit),
                        );
                    let unit_and_amount_strategy = match invalidities.unit_related {
                        Some(UnitRelatedInvalidMoveAddedReason::UnitNotFound) => {
                            (any::<unit::Name>(), any::<NonNegativeAmount>())
                                .prop_filter(
                                    "unit name happens to be valid",
                                    |(unit_name, _amount)| {
                                        !observations.unit_created_events.iter().any(
                                            |unit_created_event| {
                                                unit_name == &unit_created_event.name
                                            },
                                        )
                                    },
                                )
                                .boxed()
                        }
                        Some(UnitRelatedInvalidMoveAddedReason::DecimalPlacesMismatch) => (
                            select(observations.unit_created_events),
                            any::<NonNegativeAmount>(),
                        )
                            .prop_filter_map(
                                "amount scale happens to match unit",
                                |(unit_event, amount)| {
                                    (amount.scale() == unit_event.decimal_places as u32)
                                        .then_some((unit_event.name, amount))
                                },
                            )
                            .boxed(),
                        None => select(observations.unit_created_events)
                            .prop_flat_map(|unit_created_event| {
                                (
                                    Just(unit_created_event.name),
                                    NonNegativeAmount::arbitrary_with(Some(
                                        unit_created_event.decimal_places,
                                    )),
                                )
                            })
                            .boxed(),
                    };
                    (
                        transaction_id_strategy,
                        debit_account_strategy,
                        credit_account_strategy,
                        unit_and_amount_strategy,
                    )
                        .prop_map(
                            |(transaction, debit_account, credit_account, (unit, amount))| {
                                MoveAdded {
                                    transaction,
                                    debit_account,
                                    credit_account,
                                    amount,
                                    unit,
                                }
                            },
                        )
                        .boxed()
                }
            }
        }
    }

    #[derive(Debug, Clone, Default)]
    pub(crate) struct MoveAddedInvalidities {
        transaction_not_found: bool,
        debit_account_not_found: bool,
        credit_account_not_found: bool,
        unit_related: Option<UnitRelatedInvalidMoveAddedReason>,
    }

    #[derive(Debug, Clone)]
    pub(crate) enum UnitRelatedInvalidMoveAddedReason {
        UnitNotFound,
        DecimalPlacesMismatch,
    }

    #[derive(Debug, Clone)]
    pub(crate) enum ArbitraryUnitCreatedParam {
        With(BTreeSet<unit::Name>),
        InvalidWith(BTreeSet<unit::Name>),
    }

    impl Default for ArbitraryUnitCreatedParam {
        fn default() -> Self {
            Self::With(Default::default())
        }
    }

    impl Arbitrary for UnitCreated {
        type Parameters = ArbitraryUnitCreatedParam;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(arg: Self::Parameters) -> Self::Strategy {
            let name = match arg {
                ArbitraryUnitCreatedParam::With(disallowed_names) => any::<unit::Name>()
                    .prop_filter("name is not allowed", move |name| {
                        !disallowed_names.contains(name)
                    })
                    .boxed(),
                ArbitraryUnitCreatedParam::InvalidWith(names) => any::<Index>()
                    .prop_map(move |index| {
                        names.iter().nth(index.index(names.len())).cloned().unwrap()
                    })
                    .boxed(),
            };
            (name, any::<u8>())
                .prop_map(|(name, decimal_places)| Self {
                    name,
                    decimal_places,
                })
                .boxed()
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) enum ArbitraryAccountCreatedParam {
        With(BTreeSet<account::Name>),
        InvalidWith(BTreeSet<account::Name>),
    }

    impl Default for ArbitraryAccountCreatedParam {
        fn default() -> Self {
            Self::With(Default::default())
        }
    }

    impl Arbitrary for AccountCreated {
        type Parameters = ArbitraryAccountCreatedParam;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(arg: Self::Parameters) -> Self::Strategy {
            let name = match arg {
                ArbitraryAccountCreatedParam::With(disallowed_names) => any::<account::Name>()
                    .prop_filter("name is not allowed", move |name| {
                        !disallowed_names.contains(name)
                    })
                    .boxed(),
                ArbitraryAccountCreatedParam::InvalidWith(names) => any::<Index>()
                    .prop_map(move |index| {
                        names.iter().nth(index.index(names.len())).cloned().unwrap()
                    })
                    .boxed(),
            };

            (name, any::<account::Kind>())
                .prop_map(|(name, kind)| Self { name, kind })
                .boxed()
        }
    }

    impl Arbitrary for Event {
        type Parameters = ArbitraryEventParam;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            match args {
                ArbitraryEventParam::Any => prop_oneof![
                    any::<AccountCreated>().prop_map(Event::AccountCreated),
                    any::<TransactionRecorded>().prop_map(Event::TransactionRecorded),
                    any::<UnitCreated>().prop_map(Event::UnitCreated),
                    any::<MoveAdded>().prop_map(Event::MoveAdded),
                ]
                .boxed(),
                ArbitraryEventParam::ValidAfter(events) => {
                    let observations: Observations = events.iter().collect();
                    let strategy = Union::new([
                        AccountCreated::arbitrary_with(ArbitraryAccountCreatedParam::With(
                            observations.account_names.clone(),
                        ))
                        .prop_map(Event::AccountCreated)
                        .boxed(),
                        any::<TransactionRecorded>()
                            .prop_map(Event::TransactionRecorded)
                            .boxed(),
                        UnitCreated::arbitrary_with(ArbitraryUnitCreatedParam::With(
                            observations
                                .unit_created_events
                                .iter()
                                .map(|unit_created| unit_created.name.clone())
                                .collect(),
                        ))
                        .prop_map(Event::UnitCreated)
                        .boxed(),
                    ]);
                    let strategy = if MoveAdded::is_possible(&observations) {
                        strategy.or(MoveAdded::arbitrary_with(ArbitraryMoveAddedParam::With(
                            observations,
                            Default::default(),
                        ))
                        .prop_map(Event::MoveAdded)
                        .boxed())
                    } else {
                        strategy
                    };
                    strategy.boxed()
                }
                ArbitraryEventParam::InvalidAfter(events) => {
                    let observations: Observations = events.iter().collect();
                    // TODO: refactor "let mut strategy"
                    let strategy = Union::new([MoveAddedInvalidities::arbitrary_with(
                        ArbitraryMoveAddedInvaliditiesParam::Invalid,
                    )
                    .prop_map(|invalidities| {
                        MoveAdded::arbitrary_with(ArbitraryMoveAddedParam::With(
                            observations.clone(),
                            invalidities,
                        ))
                    })
                    .prop_map(Event::MoveAdded)
                    .boxed()]);

                    let strategy = if !observations.account_names.is_empty() {
                        strategy.or(AccountCreated::arbitrary_with(
                            ArbitraryAccountCreatedParam::InvalidWith(
                                observations.account_names.clone(),
                            ),
                        )
                        .prop_map(Event::AccountCreated)
                        .boxed())
                    } else {
                        strategy
                    };

                    let strategy = if !observations.unit_created_events.is_empty() {
                        strategy.or(UnitCreated::arbitrary_with(
                            ArbitraryUnitCreatedParam::InvalidWith(
                                observations
                                    .unit_created_events
                                    .into_iter()
                                    .map(|unit_created| unit_created.name)
                                    .collect(),
                            ),
                        )
                        .prop_map(Event::UnitCreated)
                        .boxed())
                    } else {
                        strategy
                    };
                    strategy.boxed()
                }
            }
        }
    }

    #[test]
    fn serialization_deserialization() {
        let mut runner = TestRunner::default();
        runner
            .run(&any::<Events>(), |events| {
                let string = ron::to_string(&events.0).unwrap();
                let parsed: Vec<Event> = ron::from_str(&string).unwrap();
                assert_eq!(events.0, parsed);
                Ok(())
            })
            .unwrap();
    }

    #[derive(Debug)]
    struct EventsReader(Reader<Bytes>);

    impl io::Read for EventsReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.0.read(buf)
        }
    }

    impl Arbitrary for EventsReader {
        type Parameters = ArbitraryReaderParam;
        type Strategy = BoxedStrategy<EventsReader>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            let bytes_strategy = match args {
                ArbitraryReaderParam::Any => any::<Vec<u8>>().boxed(),
                ArbitraryReaderParam::InvalidUtf8 => any::<Vec<u8>>()
                    .prop_filter("valid UTF-8", |bytes| {
                        String::from_utf8(bytes.clone()).is_err()
                    })
                    .boxed(),
                ArbitraryReaderParam::ValidUtf8FailsToParse => any::<String>()
                    .prop_filter_map("successfully parsed into `Vec<Event>`", |string| {
                        ron::from_str::<Vec<Event>>(&string)
                            .is_err()
                            .then(|| string.into_bytes())
                    })
                    .boxed(),
            };
            bytes_strategy
                .prop_map(|bytes| Self(Bytes::from(bytes).reader()))
                .boxed()
        }
    }

    #[derive(Default)]
    enum ArbitraryReaderParam {
        #[default]
        Any,
        InvalidUtf8,
        ValidUtf8FailsToParse,
    }

    #[test]
    fn events_try_from_reader_not_utf8() {
        let mut runner = TestRunner::default();
        runner
            .run(
                &EventsReader::arbitrary_with(ArbitraryReaderParam::InvalidUtf8),
                |mut reader| {
                    assert!(matches!(
                        Events::try_from_reader(&mut reader),
                        Err(Error::EventsFailedToReadIntoString(_))
                    ));
                    Ok(())
                },
            )
            .unwrap();
    }

    #[test]
    fn events_try_from_reader_fails_to_parse() {
        let mut runner = TestRunner::default();
        runner
            .run(
                &EventsReader::arbitrary_with(ArbitraryReaderParam::ValidUtf8FailsToParse),
                |mut reader| {
                    assert!(matches!(
                        &Events::try_from_reader(&mut reader),
                        Err(Error::EventsFailedToDeserialize(_))
                    ));
                    Ok(())
                },
            )
            .unwrap();
    }

    #[test]
    fn events_try_from_reader_event_fails_validation() {
        let mut runner = TestRunner::default();
        runner
            .run(
                &any::<Events>().prop_flat_map(|events| {
                    let event =
                        Event::arbitrary_with(ArbitraryEventParam::InvalidAfter(events.clone()));
                    (Just(events), event)
                }),
                |(events, event)| {
                    let mut invalid_events = events.0;
                    invalid_events.push(event);
                    let events_str: String = ron::to_string(&invalid_events).unwrap();
                    let mut deserialized_events: Reader<Bytes> = Bytes::from(events_str).reader();
                    assert!(matches!(
                        Events::try_from_reader(&mut deserialized_events),
                        Err(Error::EventValidateForAppendingTo(_)),
                    ));
                    Ok(())
                },
            )
            .unwrap()
    }

    #[test]
    fn events_try_from_reader_success() {
        let mut runner = TestRunner::default();
        runner
            .run(&any::<Events>(), |expected| {
                let actual = ron::to_string(&expected.0).unwrap();
                let mut reader = Bytes::from(actual).reader();
                let actual = Events::try_from_reader(&mut reader).unwrap();
                assert_eq!(actual, expected);
                Ok(())
            })
            .unwrap()
    }

    #[test]
    fn events_try_push_failed_validation() {
        let mut runner = TestRunner::default();
        let strategy = any::<Events>().prop_flat_map(|events| {
            let event = Event::arbitrary_with(ArbitraryEventParam::InvalidAfter(events.clone()));
            (Just(events), event)
        });
        runner
            .run(&strategy, |(mut events, event)| {
                let error = events.try_push(event).unwrap_err();
                assert!(matches!(error, Error::EventValidateForAppendingTo(_),));
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn events_try_push_success() {
        let mut runner = TestRunner::default();
        let strategy = any::<Events>().prop_flat_map(|events| {
            let event = Event::arbitrary_with(ArbitraryEventParam::ValidAfter(events.clone()));
            (Just(events), event)
        });
        runner
            .run(&strategy, |(mut events, event)| {
                let original_len = events.0.len();
                events.try_push(event).unwrap();
                assert_eq!(events.0.len(), original_len + 1);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn event_validate_for_appending_to_account_created_name_collision() {
        let mut runner = TestRunner::default();
        let strategy = any::<Events>()
            .prop_filter("does not include `AccountCreated` variant", |events| {
                events
                    .iter()
                    .any(|event| matches!(event, Event::AccountCreated(_)))
            })
            .prop_flat_map(|events| {
                let observations: Observations = events.iter().collect();
                let account_created = AccountCreated::arbitrary_with(
                    ArbitraryAccountCreatedParam::InvalidWith(observations.account_names),
                );
                (Just(events), account_created)
            });
        runner
            .run(&strategy, |(events, account_created)| {
                let error = Event::AccountCreated(account_created.clone())
                    .validate_for_appending_to(&events)
                    .unwrap_err();

                assert_eq!(
                    error,
                    EventValidateForAppendingToErrorSet::single(
                        EventValidateForAppendingToError::AccountCreatedNameCollision(
                            account_created.name
                        )
                    )
                );

                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn event_validate_for_appending_to_unit_created_name_collision() {
        let mut runner = TestRunner::default();
        let strategy = any::<Events>()
            .prop_filter("does not include `UnitCreated` variant", |events| {
                events
                    .iter()
                    .any(|event| matches!(event, Event::UnitCreated(_)))
            })
            .prop_flat_map(|events| {
                let observations: Observations = events.iter().collect();
                let unit_names = observations
                    .unit_created_events
                    .into_iter()
                    .map(|event| event.name)
                    .collect();
                let unit_created =
                    UnitCreated::arbitrary_with(ArbitraryUnitCreatedParam::InvalidWith(unit_names));
                (Just(events), unit_created)
            });
        runner
            .run(&strategy, |(events, unit_created)| {
                let error = Event::UnitCreated(unit_created.clone())
                    .validate_for_appending_to(&events)
                    .unwrap_err();

                assert_eq!(
                    error,
                    EventValidateForAppendingToErrorSet::single(
                        EventValidateForAppendingToError::UnitCreatedNameCollision(
                            unit_created.name
                        )
                    )
                );

                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn event_validate_for_appending_to_move_added_transaction_not_found() {
        let mut runner = TestRunner::default();
        let strategy = any::<Events>().prop_flat_map(|events| {
            let observations: Observations = events.iter().collect();
            let move_added = MoveAdded::arbitrary_with(ArbitraryMoveAddedParam::With(
                observations,
                MoveAddedInvalidities {
                    transaction_not_found: true,
                    ..Default::default()
                },
            ));
            (Just(events), move_added)
        });
        runner
            .run(&strategy, |(events, move_added)| {
                let error = Event::MoveAdded(move_added.clone())
                    .validate_for_appending_to(&events)
                    .unwrap_err();

                assert_eq!(
                    error,
                    EventValidateForAppendingToErrorSet::single(
                        EventValidateForAppendingToError::MoveAddedTransactionNotFound(
                            move_added.transaction
                        )
                    )
                );

                Ok(())
            })
            .unwrap();
    }

    // TODO the rest of the failures
    // MoveAddedTransactionNotFound(transaction::Id),
    // MoveAddedDebitAccountNotFound(account::Name),
    // MoveAddedCreditAccountNotFound(account::Name),
    // MoveAddedUnitNotFound(unit::Name),
    // MoveAddedDecimalPlacesMismatch { unit_scale: u8, amount_scale: u32 },

    #[test]
    fn event_validate_for_appending_to_success() {
        let mut runner = TestRunner::default();
        let strategy = any::<Events>().prop_flat_map(|events| {
            let event = Event::arbitrary_with(ArbitraryEventParam::ValidAfter(events.clone()));
            (Just(events), event)
        });
        runner
            .run(&strategy, |(events, event)| {
                event.validate_for_appending_to(&events).unwrap();
                Ok(())
            })
            .unwrap();
    }
}
