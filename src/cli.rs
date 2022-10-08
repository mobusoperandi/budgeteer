use crate::entities::{account, amount::Amount, transaction, unit};
use chrono::NaiveDate;

#[derive(clap::Parser)]
pub(crate) struct Arguments {
    #[clap(subcommand)]
    pub(crate) category: Category,
}

#[derive(clap::Subcommand)]
pub(crate) enum Category {
    #[clap(subcommand)]
    Account(Account),
    #[clap(subcommand)]
    Transaction(Transaction),
    #[clap(subcommand)]
    Unit(Unit),
    #[clap(subcommand)]
    Move(Move),
    Balances,
    RunningBalance(RunningBalance),
}

#[derive(clap::Subcommand)]
pub(crate) enum Account {
    Create(AccountCreate),
}

#[derive(clap::Args)]
pub(crate) struct AccountCreate {
    #[clap(long, arg_enum)]
    pub(crate) kind: account::Kind,

    #[clap(long)]
    pub(crate) name: account::Name,
}

#[derive(clap::Subcommand)]
pub(crate) enum Transaction {
    Record(TransactionRecord),
    Show(TransactionShow),
}

#[derive(clap::Args)]
pub(crate) struct TransactionRecord {
    #[clap(long)]
    pub(crate) date: NaiveDate,
}

#[derive(clap::Args)]
pub(crate) struct TransactionShow {
    #[clap(long)]
    pub(crate) id: transaction::Id,
}

#[derive(clap::Subcommand)]
pub(crate) enum Unit {
    Create(UnitCreate),
}

#[derive(clap::Args)]
pub(crate) struct UnitCreate {
    #[clap(long)]
    pub(crate) decimal_places: u8,
    #[clap(long)]
    pub(crate) name: unit::Name,
}

#[derive(clap::Subcommand)]
pub(crate) enum Move {
    Add(MoveAdd),
}

#[derive(clap::Args)]
pub(crate) struct RunningBalance {
    #[clap(long)]
    pub(crate) account: account::Name,
    #[clap(long)]
    pub(crate) unit: unit::Name,
}

#[derive(clap::Args)]
pub(crate) struct MoveAdd {
    #[clap(long)]
    pub(crate) transaction: transaction::Id,
    #[clap(long)]
    pub(crate) debit_account: account::Name,
    #[clap(long)]
    pub(crate) credit_account: account::Name,
    #[clap(long)]
    pub(crate) amount: Amount,
    #[clap(long)]
    pub(crate) unit: unit::Name,
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Arguments::command().debug_assert();
}
