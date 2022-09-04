<!-- TOC -->

# Table of contents

1. [Versioning and guarantees](#versioning-and-guarantees)
<!-- TOC -->

# Versioning and guarantees

This project versions releases under [Semantic Versioning 2.0.0][semver].

The CLI is not inteded for scripting.
Any release may introduce a change that will break attempted scripting.

It is intended that any release will be able to read persistance data from any earlier release.

# Tutorial

Create an _initial balance_ account where all initial balances originate.

```console
$ budgeteer account create --kind external --name "initial balance"
```

Create some budget accounts.

```console
$ budgeteer account create --kind budget --name wallet
$ budgeteer account create --kind budget --name bank
```

Record a transaction that will explain your current balances.

```console
$ budgeteer transaction record --date 2022-08-27
Recorded transaction #1
```

Create a currency unit.

```console
$ budgeteer unit create --decimal-places 2 --name USD
```

Add moves to that transaction.

```console
$ budgeteer move add --transaction 1
> --debit-account "initial balance" --credit-account wallet
> --amount 147.13 --unit USD
$ budgeteer move add --transaction 1
> --debit-account "initial balance" --credit-account bank
> --amount 5650.3 --unit USD
```

Show current balances.

```console
$ budgeteer balances
account         balance
bank             5,650.30 USD
initial balance -5,797.43 USD
wallet             147.13 USD
```

Pay rent.

```console
$ budgeteer account create --kind external --name rent
$ budgeteer transaction record --date 2022-08-28
Recorded transaction #2
$ budgeteer move add --transaction 2
> --debit-account bank --credit-account rent
> --amount 1200 --unit USD
```

Show the running balance of the bank account.

```console
$ budgeteer running-balance bank
transaction   affect        balance
#1 2022-08-27 +5,650.30 USD 5,650.30 USD
#2 2022-08-28 -1,200.00 USD 4,450.30 USD
```

Show a particular transaction.

```console
$ budgeteer transaction show 2
2022-08-28

move from to   USD
#1   bank rent 1,200.00
```

[semver]: https://semver.org/spec/v2.0.0.html
