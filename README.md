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
> --amount 5650.30 --unit USD
```

Show current balances.

```console
$ budgeteer balances
 account          balance       
 bank              5650.30  USD 
 initial balance  -5797.43  USD 
 wallet             147.13  USD 

```

Pay rent.

```console
$ budgeteer account create --kind external --name rent
$ budgeteer transaction record --date 2022-08-28
Recorded transaction #2

$ budgeteer move add --transaction 2
> --debit-account bank --credit-account rent
> --amount 1200.00 --unit USD
```

Show the running balance of the bank account.

```console
$ budgeteer running-balance --account bank --unit USD
 transaction    affect    balance 
 #1 2022-08-27  +5650.30  5650.30 
 #2 2022-08-28  -1200.00  4450.30 

```

Show a particular transaction.

```console
$ budgeteer transaction show --id 2
2022-08-28
 from  to    amount      
 bank  rent  1200.00 USD 

```

[semver]: https://semver.org/spec/v2.0.0.html
