// See the COPYRIGHT file at the top-level directory of this distribution.
// Licensed under MIT license<LICENSE-MIT or http://opensource.org/licenses/MIT>

#[cfg( test )]
mod test {
    use tilde::tilde;

    macro_rules! inc {
        ($e:expr) => { $e +1 }
    }

    tilde! {
        fn test_inc( input: i32 ) -> i32 {
            (   input
              + input.~inc!()
              + input.~inc!().~inc!()
              + input.~inc!().~inc!().~inc!()
            ).~inc!()
        }
    }

    macro_rules! add {
        ( $lhs:expr, $rhs:expr ) => { $lhs + $rhs }
    }

    tilde! {
        fn test_add( input: i32 ) -> i32 {
            (   input
              + input.~add!( input )
              + input.~add!( input.~add!( input ))
              + input.~add!( input.~add!( input.~add!( input )))
            ).~add!( input )
        }
    }

    #[test]
    fn test() {
        assert_eq!( test_inc(3), 19 );
        assert_eq!( test_add(1), 11 );
    }
}
