# Purpose

This `tilde` crate utilizes the disused tilde operator `~` to generate
syntatic sugar for Rust program.

# Features

1. Postfix macro. The syntax is `first_arg.~the_macro!(rest_args)`, which will
be desugared as `the_macro!( first_arg, rest_args )`.

More features will be added in the future.

# License

Licensed under MIT.
