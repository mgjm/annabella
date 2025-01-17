use std::fmt;

use crate::tokenizer::{Ident, Span, Spanned};

use super::{Parse, ParseStream, Result};

#[macro_export]
macro_rules! Token {
    (&) => {
        $crate::parser::token::Concat
    };
    (tick) => {
        $crate::parser::token::Tick
    };
    (*) => {
        $crate::parser::token::Mul
    };
    (+) => {
        $crate::parser::token::Add
    };
    (,) => {
        $crate::parser::token::Comma
    };
    (-) => {
        $crate::parser::token::Sub
    };
    (.) => {
        $crate::parser::token::Dot
    };
    (/) => {
        $crate::parser::token::Dib
    };
    (:) => {
        $crate::parser::token::Colon
    };
    (;) => {
        $crate::parser::token::Semicolon
    };
    (<) => {
        $crate::parser::token::Lt
    };
    (=) => {
        $crate::parser::token::Eq
    };
    (>) => {
        $crate::parser::token::Gt
    };
    (|) => {
        $crate::parser::token::Choice
    };
    (=>) => {
        $crate::parser::token::Arrow
    };
    (..) => {
        $crate::parser::token::DoubleDot
    };
    (**) => {
        $crate::parser::token::Pow
    };
    (:=) => {
        $crate::parser::token::Assign
    };
    (/=) => {
        $crate::parser::token::Ne
    };
    (>=) => {
        $crate::parser::token::Ge
    };
    (<=) => {
        $crate::parser::token::Le
    };
    (<<) => {
        $crate::parser::token::LabelStart
    };
    (>>) => {
        $crate::parser::token::LabelEnd
    };
    (<>) => {
        $crate::parser::token::Box
    };

    (abort) => {
        $crate::parser::token::Abort
    };
    (abs) => {
        $crate::parser::token::Abs
    };
    (abstract) => {
        $crate::parser::token::Abstract
    };
    (accept) => {
        $crate::parser::token::Accept
    };
    (access) => {
        $crate::parser::token::Access
    };
    (aliased) => {
        $crate::parser::token::Aliased
    };
    (all) => {
        $crate::parser::token::All
    };
    (and) => {
        $crate::parser::token::And
    };
    (array) => {
        $crate::parser::token::Array
    };
    (at) => {
        $crate::parser::token::At
    };
    (begin) => {
        $crate::parser::token::Begin
    };
    (body) => {
        $crate::parser::token::Body
    };
    (case) => {
        $crate::parser::token::Case
    };
    (constant) => {
        $crate::parser::token::Constant
    };
    (declare) => {
        $crate::parser::token::Declare
    };
    (delay) => {
        $crate::parser::token::Delay
    };
    (delta) => {
        $crate::parser::token::Delta
    };
    (digits) => {
        $crate::parser::token::Digits
    };
    (do) => {
        $crate::parser::token::Do
    };
    (else) => {
        $crate::parser::token::Else
    };
    (elsif) => {
        $crate::parser::token::Elsif
    };
    (end) => {
        $crate::parser::token::End
    };
    (entry) => {
        $crate::parser::token::Entry
    };
    (exception) => {
        $crate::parser::token::Exception
    };
    (exit) => {
        $crate::parser::token::Exit
    };
    (for) => {
        $crate::parser::token::For
    };
    (function) => {
        $crate::parser::token::Function
    };
    (generic) => {
        $crate::parser::token::Generic
    };
    (goto) => {
        $crate::parser::token::Goto
    };
    (if) => {
        $crate::parser::token::If
    };
    (in) => {
        $crate::parser::token::In
    };
    (is) => {
        $crate::parser::token::Is
    };
    (limited) => {
        $crate::parser::token::Limited
    };
    (loop) => {
        $crate::parser::token::Loop
    };
    (mod) => {
        $crate::parser::token::Mod
    };
    (new) => {
        $crate::parser::token::New
    };
    (not) => {
        $crate::parser::token::Not
    };
    (null) => {
        $crate::parser::token::Null
    };
    (of) => {
        $crate::parser::token::Of
    };
    (or) => {
        $crate::parser::token::Or
    };
    (others) => {
        $crate::parser::token::Others
    };
    (out) => {
        $crate::parser::token::Out
    };
    (package) => {
        $crate::parser::token::Package
    };
    (pragma) => {
        $crate::parser::token::Pragma
    };
    (private) => {
        $crate::parser::token::Private
    };
    (procedure) => {
        $crate::parser::token::Procedure
    };
    (protected) => {
        $crate::parser::token::Protected
    };
    (raise) => {
        $crate::parser::token::Raise
    };
    (range) => {
        $crate::parser::token::Range
    };
    (record) => {
        $crate::parser::token::Record
    };
    (rem) => {
        $crate::parser::token::Rem
    };
    (renames) => {
        $crate::parser::token::Renames
    };
    (requeue) => {
        $crate::parser::token::Requeue
    };
    (return) => {
        $crate::parser::token::Return
    };
    (reverse) => {
        $crate::parser::token::Reverse
    };
    (select) => {
        $crate::parser::token::Select
    };
    (separate) => {
        $crate::parser::token::Separate
    };
    (subtype) => {
        $crate::parser::token::Subtype
    };
    (tagged) => {
        $crate::parser::token::Tagged
    };
    (task) => {
        $crate::parser::token::Task
    };
    (terminate) => {
        $crate::parser::token::Terminate
    };
    (then) => {
        $crate::parser::token::Then
    };
    (type) => {
        $crate::parser::token::Type
    };
    (until) => {
        $crate::parser::token::Until
    };
    (use) => {
        $crate::parser::token::Use
    };
    (when) => {
        $crate::parser::token::When
    };
    (while) => {
        $crate::parser::token::While
    };
    (with) => {
        $crate::parser::token::With
    };
    (xor) => {
        $crate::parser::token::Xor
    };
}

pub trait TokenFn: Copy {
    type Token: Token;
}

impl<F, T> TokenFn for F
where
    F: Fn(Span) -> T + Copy,
    T: Token,
{
    type Token = T;
}

pub trait Token: Parse {
    const DISPLAY: &'static str;

    fn peek() -> bool;
}

macro_rules! keyword_token {
    ($token:literal, $ident:ident) => {
        #[derive(Clone, Copy)]
        pub struct $ident {
            span: Span,
        }

        #[expect(non_snake_case)]
        pub fn $ident(span: Span) -> $ident {
            $ident { span }
        }

        impl fmt::Debug for $ident {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(stringify!($ident))
            }
        }

        impl Default for $ident {
            fn default() -> Self {
                Self {
                    span: Span::call_site(),
                }
            }
        }

        impl std::cmp::Eq for $ident {}

        impl PartialEq for $ident {
            fn eq(&self, _other: &Self) -> bool {
                true
            }
        }

        impl $ident {
            pub fn operator_symbol(&self) -> Ident {
                Ident {
                    name: concat!('"', $token, '"').into(),
                    span: self.span,
                }
            }
        }

        impl Spanned for $ident {
            fn span(&self) -> Span {
                self.span
            }
        }

        impl Parse for $ident {
            fn parse(input: ParseStream) -> Result<Self> {
                Ok(Self {
                    span: parse_keyword(input, $token)?,
                })
            }
        }

        impl Token for $ident {
            const DISPLAY: &'static str = concat!("`", $token, "`");

            fn peek() -> bool {
                todo!()
            }
        }
    };
}

macro_rules! punct_token {
    ($token:literal, $ident:ident) => {
        #[derive(Clone, Copy)]
        pub struct $ident {
            span: Span,
        }

        #[expect(non_snake_case)]
        pub fn $ident(span: Span) -> $ident {
            $ident { span }
        }

        impl fmt::Debug for $ident {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(stringify!($ident))
            }
        }

        impl Default for $ident {
            fn default() -> Self {
                Self {
                    span: Span::call_site(),
                }
            }
        }

        impl std::cmp::Eq for $ident {}

        impl PartialEq for $ident {
            fn eq(&self, _other: &Self) -> bool {
                true
            }
        }

        impl $ident {
            pub fn operator_symbol(&self) -> Ident {
                Ident {
                    name: concat!('"', $token, '"').into(),
                    span: self.span,
                }
            }
        }

        impl Spanned for $ident {
            fn span(&self) -> Span {
                self.span
            }
        }

        impl Parse for $ident {
            fn parse(input: ParseStream) -> Result<Self> {
                Ok(Self {
                    span: parse_punct(input, $token)?,
                })
            }
        }

        impl Token for $ident {
            const DISPLAY: &'static str = concat!("`", $token, "`");

            fn peek() -> bool {
                todo!()
            }
        }
    };
}

punct_token!("&", Concat);
punct_token!("'", Tick);
punct_token!("*", Mul);
punct_token!("+", Add);
punct_token!(",", Comma);
punct_token!("-", Sub);
punct_token!(".", Dot);
punct_token!("/", Dib);
punct_token!(":", Colon);
punct_token!(";", Semicolon);
punct_token!("<", Lt);
punct_token!("=", Eq);
punct_token!(">", Gt);
punct_token!("|", Choice);
punct_token!("=>", Arrow);
punct_token!("..", DoubleDot);
punct_token!("**", Pow);
punct_token!(":=", Assign);
punct_token!("/=", Ne);
punct_token!(">=", Ge);
punct_token!("<=", Le);
punct_token!("<<", LabelStart);
punct_token!(">>", LabelEnd);
punct_token!("<>", Box);

keyword_token!("abort", Abort);
keyword_token!("abs", Abs);
keyword_token!("abstract", Abstract);
keyword_token!("accept", Accept);
keyword_token!("access", Access);
keyword_token!("aliased", Aliased);
keyword_token!("all", All);
keyword_token!("and", And);
keyword_token!("array", Array);
keyword_token!("at", At);
keyword_token!("begin", Begin);
keyword_token!("body", Body);
keyword_token!("case", Case);
keyword_token!("constant", Constant);
keyword_token!("declare", Declare);
keyword_token!("delay", Delay);
keyword_token!("delta", Delta);
keyword_token!("digits", Digits);
keyword_token!("do", Do);
keyword_token!("else", Else);
keyword_token!("elsif", Elsif);
keyword_token!("end", End);
keyword_token!("entry", Entry);
keyword_token!("exception", Exception);
keyword_token!("exit", Exit);
keyword_token!("for", For);
keyword_token!("function", Function);
keyword_token!("generic", Generic);
keyword_token!("goto", Goto);
keyword_token!("if", If);
keyword_token!("in", In);
keyword_token!("is", Is);
keyword_token!("limited", Limited);
keyword_token!("loop", Loop);
keyword_token!("mod", Mod);
keyword_token!("new", New);
keyword_token!("not", Not);
keyword_token!("null", Null);
keyword_token!("of", Of);
keyword_token!("or", Or);
keyword_token!("others", Others);
keyword_token!("out", Out);
keyword_token!("package", Package);
keyword_token!("pragma", Pragma);
keyword_token!("private", Private);
keyword_token!("procedure", Procedure);
keyword_token!("protected", Protected);
keyword_token!("raise", Raise);
keyword_token!("range", Range);
keyword_token!("record", Record);
keyword_token!("rem", Rem);
keyword_token!("renames", Renames);
keyword_token!("requeue", Requeue);
keyword_token!("return", Return);
keyword_token!("reverse", Reverse);
keyword_token!("select", Select);
keyword_token!("separate", Separate);
keyword_token!("subtype", Subtype);
keyword_token!("tagged", Tagged);
keyword_token!("task", Task);
keyword_token!("terminate", Terminate);
keyword_token!("then", Then);
keyword_token!("type", Type);
keyword_token!("until", Until);
keyword_token!("use", Use);
keyword_token!("when", When);
keyword_token!("while", While);
keyword_token!("with", With);
keyword_token!("xor", Xor);

fn parse_keyword(input: ParseStream, token: &str) -> Result<Span> {
    input.step(|cursor| {
        if let Some((ident, rest)) = cursor.ident() {
            if ident.name.eq_ignore_ascii_case(token) {
                return Ok((ident.span(), rest));
            }
        }
        Err(cursor.recoverable_error(format!("expected `{token}`")))
    })
}

fn parse_punct(input: ParseStream, token: &str) -> Result<Span> {
    input.step(|mut cursor| {
        let mut span = cursor.span();
        for (i, c) in token.chars().enumerate() {
            match cursor.punct() {
                Some((punct, rest)) => {
                    span.extend(punct.span());
                    if punct.char != c {
                        break;
                    } else if i == token.len() - 1 {
                        return Ok((span, rest));
                    } else if punct.spacing.is_alone() {
                        break;
                    }
                    cursor = rest;
                }
                None => break,
            }
        }

        Err(span.recoverable_error(format!("expected `{token}`")))
    })
}

// impl<T> Parse for Option<T>
// where
//     T: Token,
// {
//     fn parse(input: ParseStream) -> Result<Self> {
//         Ok(input.parse().ok())
//     }
// }
