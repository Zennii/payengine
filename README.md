# PayEngine

This project processes sets of transactions represented by CSV files.

I spent time digging around and covering as many cases as possible even if unnecessary,
such as CSV still being "valid" in various ways. This requires the occasional check in
the code to validate as opposed to being fed a valid value after deserialization.

Async was considered although transactions would race and otherwise minimal apparent
gain was present.

Memory usage is not exceptional due to the dispute functionality storing a log in memory,
but some optimization is there. This kind of thing seems like a valid usecase for a
database or key-value store (embedded for a toy project like this) but disk is slow and
current popular embedded solutions for rust don't seem appropriate. For some examples,
PickleDB seems to take up a lot more memory, and SQLite requires a dependency which the
handling of that is unclear for this example.

Speed measured at around 13.7 microseconds per transaction in debug mode and around 1
microsecond in release in my benchmarking using an average over a 167MB file on my
Ryzen 9 5950X CPU.

A series of 24 tests exist in CSV form to make sure possible usages pass. These test all
standard usage as well as irregular usages and various usecases unique to the assumptions
made. These are available in `src/test/` with a CSV file included for each test, so tests
run through the whole system from deserialization to results.

6 tests exist for the Account functions. These are already somewhat narrated by the CSV
tests, but more narrow in scope here.

# Transaction types
5 transaction types exist currently. For amounts, going more than 4 places past the
decimal is not handled and may result in unexpected behaviour.
## deposit
Requires client ID (u16), tx ID (u32), amount (f32)
```
deposit, 1, 1, 1.0
```

## withdrawal
Requires client ID (u16), tx ID (u32), amount (f32)
```
withdrawal, 1, 1, 1.0
```

## dispute
Requires client ID (u16), tx ID (u32) to existing deposit
```
deposit, 1, 1, 1.0
```

## resolve
Requires client ID (u16), tx ID (u32) to existing deposit
```
deposit, 1, 1, 1.0
```

# chargeback
Requires client ID (u16), tx ID (u32) to existing deposit
```
deposit, 1, 1, 1.0
```

# Assumptions
- Headers are required on CSV files.
- A locked account can only deposit funds, similar to frozen accounts in real scenarios.
- More than 4 decimals are never going to be fed in, and don't need to be capped off
at entry.
- Display formatting rounding decimals that may go past 4 is not an issue.
- The client of the transaction and the referenced transaction will be the same, eg:

```
This would fail:
deposit, 1, 1, 1.0
dispute, 2, 1

This would succeed:
deposit, 1, 1, 1.0
dispute, 1, 1
```

- Only memory from reading in the csv file is a problem, and logging transactions
is not. I've done my best to reduce the memory used by the log, but there is only
so much to be done without resorting to storing the log elsewhere like on a database.
- UTF16 files are not going to be fed in. Only UTF8.
- A chargeback does not resolve a dispute.
- Entries without amounts may or may not end with a trailing comma.
- Only deposits can be disputed. Transaction specification strongly implies this.