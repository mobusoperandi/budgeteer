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
$ budgeteer account create external "initial balance"
```

Create some budget accounts.

```console
$ budgeteer account create budget wallet
$ budgeteer account create budget bank
```

Record a transaction that will explain your current balances.

```console
$ budgeteer transaction record --date 2022-08-27
Recorded transaction #1
```

Create a currency unit.

```console
$ budgeteer unit create USD 2
```

Add moves to that transaction.

```console
$ budgeteer move add 1 "initial balance" wallet 147.13 USD
$ budgeteer move add 1 "initial balance" bank 5650.3 USD
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
$ budgeteer account create external rent
$ budgeteer transaction record --date 2022-08-28
Recorded transaction #2
$ budgeteer move add 2 bank rent 1200 USD
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
