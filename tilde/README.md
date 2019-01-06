# Purpose

This `tilde` crate utilizes the disused tilde operator `~` to generate
syntatic sugar for Rust program.

# Features

1. Postfix macro. The syntax is `first_arg.~the_macro!(rest_args)`, which will
be desugared as `the_macro!( first_arg, rest_args )`. As proposed in
[simple postfix macros #2442](https://github.com/rust-lang/rfcs/pull/2442),
`first_arg` will be evaluated excactly once.

## Example: Postfix macro

  ```rust
  macro_rules! inc { ($e:expr) => { $e+1 }}
  ```

  Suppose `i: i32`, The library user could write: `i.~inc!()`,
  `i.clone().~inc!()` etc,  which is a sugar as `inc!( i )` and
  `inc!( i.clone() )`.

  This feature is in compliance with
  [RFC 2442](https://github.com/joshtriplett/rfcs/blob/simple-postfix-macros/text/0000-simple-postfix-macros.md#guide-level-explanation):

  ```rust
  macro_rules! log_value {
      ( $self:expr, $msg:expr ) => ({
          $self.1.push_str( &format!( "{}:{}: {}: {:?}", file!(), line!(), $msg, $self.0 ));
          $self
      })
  }

  fn value<T: std::fmt::Debug>( x: T, log: &mut String ) -> (T,&mut String) {
      log.push_str( &format!( "evaluated {:?}\n", x ));
      ( x, log )
  }

  tilde! {
      #[test]
      fn rfc_pr_2442() {
          let mut log1 = String::new();
          let mut log2 = String::new();
          ( value( "hello", &mut log1 ).~log_value!( "value" ).0.len(), &mut log2 ).~log_value!( "len" );
          let log = format!( "{}\n{}", log1, log2 );
          assert_eq!( log, r#"evaluated "hello"
  tilde_derive/src/lib.rs:72: value: "hello"
  tilde_derive/src/lib.rs:72: len: 5"#
          );
      }
  }
  ```

2. Postfix function. The syntax is `first_arg.~the_fn(rest_args)`, which will
be desugared as `the_fn!( first_arg, rest_args )`.

## Example: Postfix function

  ```rust
  fn inc( i: i32 ) -> i32 { i + 1 }
  ```

  Suppose `i: i32`, The library user could write: `i.~inc()`,
  `i.clone().~inc()` etc,  which is a sugar as `inc( i )` and
  `inc( i.clone() )`.

More features will be added in the future.

# License

Licensed under MIT.
