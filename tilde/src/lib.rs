// See the COPYRIGHT file at the top-level directory of this distribution.
// Licensed under MIT license<LICENSE-MIT or http://opensource.org/licenses/MIT>

extern crate proc_macro;
use self::proc_macro::{Punct,Spacing,TokenStream,TokenTree};

#[proc_macro]
pub fn tilde( input: TokenStream ) -> TokenStream {
    enum Expect { Obj, Tilde, Ident, Bang, Group }

    struct State {
        expect : Expect,
        method : Option<TokenStream>,
        acc    : TokenStream,
        obj    : Option<TokenStream>,
    }

    impl State {
        fn reset_obj( &mut self ) {
            self.expect = Expect::Obj;
            if let Some( obj ) = self.obj.take() {
                self.acc.extend( obj );
            }
        }

        fn append_obj( &mut self, ts: TokenStream ) {
            if let Some( ref mut obj ) = self.obj {
                obj.extend( ts );
            } else {
                self.obj = Some( ts );
            }
        }

        fn next( &mut self ) {
            self.expect = match self.expect {
                Expect::Obj   => Expect::Tilde,
                Expect::Tilde => Expect::Ident,
                Expect::Ident => Expect::Bang,
                Expect::Bang  => Expect::Group,
                Expect::Group => Expect::Obj,
            };
        }
    }

    trait ToTS {
        fn to_ts( self ) -> TokenStream;
    }

    impl ToTS for char {
        fn to_ts( self ) -> TokenStream {
            TokenStream::from( TokenTree::Punct(
                Punct::new( self, Spacing::Alone )))
        }
    }

    fn define_tilde( input: TokenStream ) -> TokenStream {
        use proc_macro::{TokenTree,Group};

        let mut state = State {
            expect : Expect::Obj,
            method : None,
            acc    : TokenStream::new(),
            obj    : None,
        };

        for tt in input {
            match state.expect {
                Expect::Obj => match tt {
                    TokenTree::Punct( ref punct ) => {
                        if punct.as_char() == '.' {
                            state.next();
                        } else {
                            state.reset_obj();
                            state.acc.extend( TokenStream::from( tt ));
                        }
                    },
                    TokenTree::Group( group ) => {
                        state.append_obj( TokenStream::from(
                            TokenTree::Group( Group::new(
                                group.delimiter(),
                                define_tilde( group.stream() )))));
                    },
                    _ => state.append_obj( TokenStream::from( tt )),
                },
                Expect::Tilde => match tt {
                    TokenTree::Punct( ref punct ) if punct.as_char() == '~' => {
                        state.next();
                    },
                    _ => {
                        state.reset_obj();
                        state.acc.extend( '.'.to_ts() );
                        state.acc.extend( TokenStream::from( tt ));
                    }
                },
                Expect::Ident => match tt {
                    TokenTree::Ident(_) => {
                        state.method = Some( TokenStream::from( tt ));
                        state.next();
                    },
                    _ => {
                        state.reset_obj();
                        state.acc.extend( '.'.to_ts() );
                        state.acc.extend( '~'.to_ts() );
                        state.acc.extend( TokenStream::from( tt ));
                    }
                },
                Expect::Bang => match tt {
                    TokenTree::Punct( ref punct ) if punct.as_char() == '!' => {
                        state.next();
                    },
                    _ => {
                        state.reset_obj();
                        let method = state.method.take()
                            .expect("The last tt should be some identity");
                        state.acc.extend( '.'.to_ts() );
                        state.acc.extend( '~'.to_ts() );
                        state.acc.extend( method );
                        state.acc.extend( TokenStream::from( tt ));
                    }
                },
                Expect::Group => match tt {
                    TokenTree::Group( group ) => {
                        let obj = state.obj.take()
                            .expect("The state should have got some obj");
                        let method = state.method.take()
                            .expect("The state should have got some method");
                        let mut new_obj = TokenStream::new();
                        new_obj.extend( method );
                        new_obj.extend( '!'.to_ts() );

                        let mut inner = obj;
                        let delimiter = group.delimiter();
                        if !group.stream().is_empty() {
                            let args = TokenStream::from(
                                TokenTree::Group( Group::new(
                                    delimiter,
                                    define_tilde( group.stream() )
                            )));
                            inner.extend( ','.to_ts() );
                            inner.extend( args );
                        }
                        let group = Group::new( delimiter, inner );
                        new_obj.extend( TokenStream::from(
                            TokenTree::Group( group )));

                        state.obj = Some( new_obj );
                        state.next();
                    },
                    _ => {
                        state.reset_obj();
                        let method = state.method.take()
                            .expect("The state should have got some method!");
                        state.acc.extend( '.'.to_ts() );
                        state.acc.extend( '~'.to_ts() );
                        state.acc.extend( method );
                        state.acc.extend( TokenStream::from( tt ));
                    }
                }
            }
        }
        state.reset_obj();
        state.acc
    }

    define_tilde( input )
}
