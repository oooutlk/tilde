// See the COPYRIGHT file at the top-level directory of this distribution.
// Licensed under MIT license<LICENSE-MIT or http://opensource.org/licenses/MIT>

//! This `tilde` crate utilizes the disused tilde operator `~` to generate
//! syntatic sugar for Rust program.
//! # Features
//! 
//! 1. Postfix macro. The syntax is `first_arg.~the_macro!(rest_args)`, which will
//! be desugared as `the_macro!( first_arg, rest_args )`.
//! 
//! 2. Postfix function. The syntax is `first_arg.~the_fn(rest_args)`, which will
//! be desugared as `the_fn( first_arg, rest_args )`.
//! 
//! More features will be added in the future.
//! 
//! # License
//! 
//! Licensed under MIT.

extern crate proc_macro;
use self::proc_macro::{Delimiter,Group,Ident,Punct,
    Spacing,Span,TokenStream,TokenTree};

use std::iter::once;

enum Expect { Obj, Tilde, Method, MacroOrFunc, Macro }

struct State {
    expect : Expect,
    obj    : Option<TokenStream>,
    method : Option<TokenStream>,
    acc    : TokenStream,
}

fn opt_ts_extend( opt_ts: &mut Option<TokenStream>, tt: TokenTree ) {
    if let Some( ref mut ts ) = opt_ts.as_mut() {
        ts.extend( once( tt ));
    } else {
        *opt_ts = Some( TokenStream::from( tt ));
    }
}

impl State {
    fn new() -> Self {
        State {
            expect : Expect::Obj,
            obj    : None,
            method : None,
            acc    : TokenStream::new(),
        }
    }

    fn reset_obj( &mut self ) {
        self.expect = Expect::Obj;
        if let Some( obj ) = self.obj.take() {
            self.acc.extend( obj );
        }
    }

    fn acc_extend( &mut self, tt: TokenTree ) {
        self.acc.extend( once( tt ));
    }

    fn obj_extend( &mut self, tt: TokenTree ) {
        opt_ts_extend( &mut self.obj, tt );
    }

    fn method_extend( &mut self, tt: TokenTree ) {
        opt_ts_extend( &mut self.method, tt );
    }

    fn next( &mut self ) {
        self.expect = match self.expect {
            Expect::Obj    => Expect::Tilde,
            Expect::Tilde  => Expect::Method,
            Expect::Method => Expect::MacroOrFunc,
            Expect::Macro  => Expect::Obj,
            Expect::MacroOrFunc =>
                panic!( "unexpected state in define_tilde" ),
        };
    }
}

trait TT {
    fn tt( self ) -> TokenTree;
}

impl TT for char {
    fn tt( self ) -> TokenTree {
        TokenTree::Punct( Punct::new( self, Spacing::Alone ))
    }
}

impl TT for &'static str {
    fn tt( self ) -> TokenTree {
        TokenTree::Ident( Ident::new( self, Span::call_site() ))
    }
}

fn punct( s: &'static str  ) -> impl Iterator<Item=TokenTree> {
    s.chars().map( |ch|
        TokenTree::Punct( Punct::new( ch, Spacing::Joint ))
    )
}

fn define_tilde( input: TokenStream ) -> TokenStream {
    let mut state = State::new();
    for tt in input {
        match state.expect {
            Expect::Obj => match tt {
                TokenTree::Punct( ref punct ) => match punct.as_char() {
                    '.' => state.next(),
                    ':' => state.obj_extend( tt ),
                    _ => {
                        state.reset_obj();
                        state.acc_extend( tt );
                    },
                },
                TokenTree::Group( group ) => {
                    state.obj_extend( TokenTree::Group( Group::new(
                        group.delimiter(),
                        define_tilde( group.stream() ))));
                },
                _ => state.obj_extend( tt ),
            },
            Expect::Tilde => match tt {
                TokenTree::Punct( ref punct ) if punct.as_char() == '~' => {
                    state.next();
                },
                TokenTree::Ident(_) => {
                    state.obj_extend( '.'.tt() );
                    state.obj_extend( tt );
                    state.expect = Expect::Obj;
                },
                _ => {
                    state.reset_obj();
                    state.acc_extend( '.'.tt() );
                    state.acc_extend( tt );
                }
            },
            Expect::Method => match tt {
                TokenTree::Ident(_) => {
                    state.method = Some( TokenStream::from( tt ));
                    state.next();
                },
                TokenTree::Group( group ) => { // unreachable arm
                    state.method_extend( TokenTree::Group( Group::new(
                        group.delimiter(),
                        define_tilde( group.stream() ))));
                },
                _ => {
                    state.reset_obj();
                    state.acc_extend( '.'.tt() );
                    state.acc_extend( '~'.tt() );
                    state.acc_extend( tt );
                }
            },
            Expect::MacroOrFunc => match tt {
                TokenTree::Punct( ref punct ) if punct.as_char() == '!' => {
                    state.expect = Expect::Macro;
                },
                TokenTree::Group( ref group )
                    if group.delimiter() == Delimiter::Parenthesis =>
                {
                    define_func( &mut state, group );
                    state.expect = Expect::Obj;
                },
                _ => {
                    state.reset_obj();
                    let method = state.method.take()
                        .expect("The last tt should be some identity");
                    state.acc_extend( '.'.tt() );
                    state.acc_extend( '~'.tt() );
                    state.acc.extend( method );
                    state.acc_extend( tt );
                },
            },
            Expect::Macro => {
                define_macro( &mut state, tt );
                state.next();
            }
        }
    }
    state.reset_obj();
    state.acc
}

fn define_macro( state: &mut State, tt: TokenTree ) {
    match tt {
        TokenTree::Group( group ) => {
           let mut prefixed = state.method.take()
               .expect("The state should have got some method");
           prefixed.extend( once( '!'.tt() ));

           let self_ = "__tilde_postfix_macro_self__".tt();

           let mut inner = TokenStream::from( self_.clone() );

           let delimiter = group.delimiter();
           if !group.stream().is_empty() {
               let args = TokenTree::Group( Group::new(
                   delimiter,
                   define_tilde( group.stream() )
               ));
               inner.extend( once( ','.tt() ));
               inner.extend( once( args ));
           }
           let group = Group::new( delimiter, inner );
           prefixed.extend( once( TokenTree::Group( group )));

           let mut match_body = TokenStream::from( self_ );
           match_body.extend( punct( "=>" ));
           match_body.extend( prefixed );

           let mut match_ = TokenStream::from( "match".tt() );
           match_.extend( state.obj.take()
               .expect("The state should have got some obj"));
           match_.extend( once( TokenTree::Group(
               Group::new( Delimiter::Brace, match_body ))));

           state.obj = Some( match_ );
       },
       _ => {
           // Who else will use `.~ident!` syntax without a group?
           // However, leave it as is.
           state.reset_obj();
           let method = state.method.take()
               .expect("The state should have got some method!");
           state.acc_extend( '.'.tt() );
           state.acc_extend( '~'.tt() );
           state.acc.extend( method );
           state.acc_extend( tt );
       }
    }
}

fn define_func( state: &mut State, group: &Group ) {
    let mut inner = state.obj.take()
        .expect("The state should have got some obj");

    let delimiter = group.delimiter();
    if !group.stream().is_empty() {
        let args = TokenTree::Group( Group::new(
            delimiter,
            define_tilde( group.stream() )
        ));
        inner.extend( once( ','.tt() ));
        inner.extend( once( args ));
    }
    let group = Group::new( delimiter, inner );

    let mut call = state.method.take()
        .expect("The state should have got some method");
    call.extend( once( TokenTree::Group( group )));

    state.obj = Some( call );
}

#[proc_macro]
pub fn tilde( input: TokenStream ) -> TokenStream {
    define_tilde( input )
}
