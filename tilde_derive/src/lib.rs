// See the COPYRIGHT file at the top-level directory of this distribution.
// Licensed under MIT license<LICENSE-MIT or http://opensource.org/licenses/MIT>

#[cfg( test )]
mod test {
    use tilde::tilde;

    macro_rules! inc {
        ($e:expr) => { $e + 1 }
    }

    tilde! { mod unary {
        pub fn basic( i: i32 ) -> i32 { i.~inc!() }
        pub fn method( i: i32 ) -> i32 { i.clone().~inc!() }
        pub fn func( i: i32 ) -> i32 { Clone::clone( &i ).~inc!() }
        pub fn misc( i: i32 ) -> i32 {
            (   i
              + i.clone().~inc!()
              + i.~inc!().clone().~inc!()
              + i.~inc!().clone().~inc!().clone().~inc!()
            ).~inc!()
        }
    }}

    #[test]
    fn test_unary() {
        use self::unary::*;
        assert_eq!( basic(  0 ), 1  );
        assert_eq!( method( 1 ), 2  );
        assert_eq!( func(   2 ), 3  );
        assert_eq!( misc(   3 ), 19 );
    }

    macro_rules! add {
        ( $lhs:expr, $rhs:expr ) => { $lhs + $rhs }
    }

    tilde! { mod binary {
        pub fn basic( i: i32 ) -> i32 { i.~add!(i) }
        pub fn method( i: i32 ) -> i32 { i.clone().~add!(i) }
        pub fn func( i: i32 ) -> i32 { Clone::clone( &i ).~add!(i) }
        pub fn misc( i: i32 ) -> i32 {
            (   i
              + i.clone().~add!(i)
              + i.~add!(i).clone().~add!(i)
              + i.~add!(i).clone().~add!(i).clone().~add!(i)
            ).~add!(i)
        }
    }}

    #[test]
    fn test_binary() {
        use self::binary::*;
        assert_eq!( basic(  0 ),  0 );
        assert_eq!( method( 1 ),  2 );
        assert_eq!( func(   2 ),  4 );
        assert_eq!( misc(   3 ), 33 );
    }

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
}
