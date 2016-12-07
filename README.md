# POET Rust Bindings

The `poet-sys` crate provides declarations and linkage for the `poet` C
library.
Following the *-sys package conventions, the `poet-sys` crate does not define
higher-level abstractions over the native `poet` library functions.

The latest `poet` C library can be found at
[https://github.com/libpoet/poet](https://github.com/libpoet/poet).

## Dependencies

In order to use the `poet-sys` crate, you must have the `poet` library
installed to the system.

## Usage
Add `poet-sys` as a dependency in `Cargo.toml`:

```toml
[dependencies.poet-sys]
git = "https://github.com/libpoet/poet-sys.git"
```
