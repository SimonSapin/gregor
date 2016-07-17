# Gregor

Simple implementation of the Gregorian calendar for Rust.

Gregorian rules are used for all time, which is not historically accurate before 1583.

* `UnixTimestamp` represents an instant as a (possibly negative) integer number of seconds
  since the Unix Epoch, January 1st 1970 at midnight UTC.
  (There is no sub-second resolution.)
* `NaiveDateTime` represents a date in the Gregorian calendar
  with components year, month, day, hour, minute, and second.
  By itself, it can not be interpreted as an instant in time.
* `DateTime` is a `NaiveDateTime` together with a time zone.


## `#![no_std]`

By default the crate uses `#![no_std]` so that it can be used in freestanding environments.
If the `system_time` Cargo feature is enabled,
it uses `std` to implement conversions to and from `std::time::SystemTime`.
