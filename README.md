# cmakefmt
[![Crates.io Version](https://img.shields.io/crates/v/cmakefmt)](https://crates.io/crates/cmakefmt)
- - -
Good enough CMake auto formatter. No settings, prints to STDOUT.

## Install
```
cargo install cmakefmt
```

## Usage
```
cmakefmt <file>
```

## State

* Basic syntax is handled and prints
* Based on smoketests on around 400 cmake files, there are ~8% of cases which
  aren't parsed correctly
* Error handling prints nice error messages, but it's very verbose and sometimes
  the true parsing error is deeper into the file

## Rules

* Statements and commands are printed in one line or break into multiple lines
  if there isn't space
* Command arguments are broken into one line each, unless they are an upper-case
  word. In case an argument is an upper-case word, it creates a group with all
  the following non-uppercase arguments. This helps format key-value style
  bringings.
* Group-like statements (foreach, if, block, macro) indent their children

## License
MIT
