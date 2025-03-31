# Curated list of suppressions for the Rust Standard Library

## Background
Valgrind may detect leaks in the Rust standard library as it has done in the past (e.g. [here][rust1.83] and [here][beta]).
Those "leaks" of Valgrind are not considered as leaks by the Rust team when they are small permanent allocation, which is not growing (as described [here][comment-1]) and there is no guarantee, that the Rust standard library is free of memory leaks (as described [here][comment-2]).
Therefore some reports of Valgrind are entirely non-actionable by a user of this crate.

In order to solve this, this directory contains a list of suppressions for the Rust `std`.
Those are applied automatically, so that those "leaks" are never reported.

## When to add new suppressions?
This repository runs a [periodic test against the beta compiler][beta-job], that there are no new leaks in the standard library.
If such a new leak is detected, an issue is this repository is created, so that actions can be performed.
Then there should be two steps:
1. report the issue to the [Rust-project][new-rust-issue] similar to [this one][example-issue]
2. if this leak is accepted over there, then a suppression should be added here

## How to add a new suppression?
Create a minimal reproducer of the leak (often a blank cargo project is enough, since the leak is part of the runtime).
Then execute the following commands:
```shell
$ cargo new --lib reproducer
$ cd reproducer
$ cargo test --lib  # note down the path executed, as this is required here ↓↓↓↓↓
$ valgrind --leak-check=full --gen-suppressions=yes target/debug/reproducer-$HASH
```
Valgrind will generate output in a form suitable to be used as suppressions.
That output should be trimmed down as necessary and then added to this directory.


[rust1.83]: https://github.com/rust-lang/rust/issues/133574
[beta]: https://github.com/rust-lang/rust/issues/138430
[comment-1]: https://github.com/rust-lang/rust/issues/133574#issuecomment-2506547194
[comment-2]: https://github.com/rust-lang/rust/issues/135608#issuecomment-2597205627
[beta-job]: https://github.com/jfrimmel/cargo-valgrind/actions/workflows/beta.yaml
[new-rust-issue]: https://github.com/rust-lang/rust/issues/new?template=regression.md
[example-issue]: https://github.com/rust-lang/rust/issues/138430
