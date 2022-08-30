use chrono::{Date, Utc};

use crate::{account::AccountKind, sum::Sum};

#[expect(dead_code)]
struct AccountCreated {
    name: String,
    kind: AccountKind,
}

#[expect(dead_code)]
struct TransactionRecorded {
    date: Date<Utc>,
}

#[expect(dead_code)]
struct UnitCreated {
    name: String,
    decimal_places: u8,
}

#[expect(dead_code)]
struct MoveAdded {
    transaction_id: usize,
    debit_account_id: usize,
    credit_account_id: usize,
    sum: Sum,
}
