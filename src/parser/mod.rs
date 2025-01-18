use crate::{
    tokenizer::{Group, Ident, Literal, Punct, Span, Spanned, TokenStream, TokenTree},
    Result, Token,
};

#[macro_use]
mod macros;

mod expr;
mod item;
mod parenthesized;
mod stmt;
pub mod token;

pub use expr::*;
pub use item::*;
pub use parenthesized::*;
pub use stmt::*;
use token::{Token, TokenFn};

pub trait Parse: Sized {
    fn parse(input: ParseStream) -> crate::Result<Self>;
}

fn is_keyword(ident: &Ident) -> bool {
    matches!(
        &*ident.name,
        "abort"
            | "abs"
            | "abstract"
            | "accept"
            | "access"
            | "aliased"
            | "all"
            | "and"
            | "array"
            | "at"
            | "begin"
            | "body"
            | "case"
            | "constant"
            | "declare"
            | "delay"
            | "delta"
            | "digits"
            | "do"
            | "else"
            | "elsif"
            | "end"
            | "entry"
            | "exception"
            | "exit"
            | "for"
            | "function"
            | "generic"
            | "goto"
            | "if"
            | "in"
            | "is"
            | "limited"
            | "loop"
            | "mod"
            | "new"
            | "not"
            | "null"
            | "of"
            | "or"
            | "others"
            | "out"
            | "package"
            | "pragma"
            | "private"
            | "procedure"
            | "protected"
            | "raise"
            | "range"
            | "record"
            | "rem"
            | "renames"
            | "requeue"
            | "return"
            | "reverse"
            | "select"
            | "separate"
            | "subtype"
            | "tagged"
            | "task"
            | "terminate"
            | "then"
            | "type"
            | "until"
            | "use"
            | "when"
            | "while"
            | "with"
            | "xor"
    )
}

impl Parse for Ident {
    fn parse(input: ParseStream) -> Result<Self> {
        input.step(|cursor| {
            if let Some((ident, rest)) = cursor.ident() {
                if is_keyword(ident) {
                    Err(ident.recoverable_error("expected identifier, found keyword `{ident}`"))
                } else {
                    Ok((ident.clone(), rest))
                }
            } else {
                Err(cursor.recoverable_error("expected identifier"))
            }
        })
    }
}

impl<T> Parse for Box<T>
where
    T: Parse,
{
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Box::new(T::parse(input)?))
    }
}

impl<T> Parse for Vec<T>
where
    T: Parse,
{
    fn parse(input: ParseStream) -> Result<Self> {
        let mut vec = Vec::new();
        while !input.is_empty() {
            vec.push(input.parse()?);
        }
        Ok(vec)
    }
}

pub type ParseStream<'a, 'b> = &'a mut ParseBuffer<'b>;

pub struct ParseBuffer<'a> {
    inner: &'a [TokenTree],
}

impl Spanned for ParseBuffer<'_> {
    fn span(&self) -> Span {
        self.inner.first_span()
    }
}

impl ParseBuffer<'_> {
    fn step<R>(&mut self, f: impl FnOnce(Cursor) -> Result<(R, Cursor)>) -> Result<R> {
        let (value, cursor) = f(Cursor { inner: self.inner })?;
        self.inner = cursor.inner;
        Ok(value)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn peek<T>(&self, _token: impl TokenFn<Token = T>) -> bool
    where
        T: Token,
    {
        let mut clone = Self { inner: self.inner };
        T::parse(&mut clone).is_ok()
    }

    pub fn parse<T>(&mut self) -> Result<T>
    where
        T: Parse,
    {
        self.call(T::parse)
    }

    fn call<T>(&mut self, parse: impl FnOnce(ParseStream) -> Result<T>) -> Result<T> {
        let mut clone = Self { inner: self.inner };
        let value = parse(&mut clone)?;
        *self = clone;
        Ok(value)
    }

    pub fn try_parse<T>(&mut self) -> Result<Option<T>>
    where
        T: Parse,
    {
        self.try_call(T::parse)
    }

    fn try_call<T>(&mut self, parse: impl FnOnce(ParseStream) -> Result<T>) -> Result<Option<T>> {
        let mut clone = Self { inner: self.inner };
        match parse(&mut clone) {
            Ok(value) => {
                *self = clone;
                Ok(Some(value))
            }
            Err(err) if err.recoverable => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub fn unrecoverable<T>(&mut self, parse: impl FnOnce(ParseStream) -> Result<T>) -> Result<T> {
        let mut result = self.call(parse);
        if let Err(err) = &mut result {
            err.recoverable = false;
        }
        result
    }

    fn parse_until<T, F>(&mut self, token: F) -> Result<(Vec<T>, F::Token)>
    where
        T: Parse,
        F: TokenFn,
    {
        let value = self.call(|input| {
            let mut vec = Vec::new();
            while !input.peek(token) {
                vec.push(input.parse()?);
            }
            Ok(vec)
        })?;
        Ok((value, self.parse()?))
    }

    fn parse_until_end<T>(&mut self) -> Result<(Vec<T>, Token![end])>
    where
        T: Parse,
    {
        self.parse_until(Token![end])
    }
}

#[derive(Clone, Copy)]
pub struct Cursor<'a> {
    inner: &'a [TokenTree],
}

impl Spanned for Cursor<'_> {
    fn span(&self) -> Span {
        self.inner.first_span()
    }
}

impl<'a> Cursor<'a> {
    fn next(self) -> Option<(&'a TokenTree, Self)> {
        let (tt, rest) = self.inner.split_first()?;
        Some((tt, Self { inner: rest }))
    }

    pub fn group(self) -> Option<(&'a Group, Self)> {
        match self.next()? {
            (TokenTree::Group(group), cursor) => Some((group, cursor)),
            _ => None,
        }
    }

    pub fn ident(self) -> Option<(&'a Ident, Self)> {
        match self.next()? {
            (TokenTree::Ident(ident), cursor) => Some((ident, cursor)),
            _ => None,
        }
    }

    pub fn punct(self) -> Option<(&'a Punct, Self)> {
        match self.next()? {
            (TokenTree::Punct(punct), cursor) => Some((punct, cursor)),
            _ => None,
        }
    }

    fn literal(self) -> Option<(&'a Literal, Self)> {
        match self.next()? {
            (TokenTree::Literal(lit), cursor) => Some((lit, cursor)),
            _ => None,
        }
    }
}

pub fn parse<T>(input: TokenStream) -> Result<T>
where
    T: Parse,
{
    parse_with(input, T::parse)
}

fn parse_with<T>(input: TokenStream, parse: impl FnOnce(ParseStream) -> Result<T>) -> Result<T> {
    let mut input = ParseBuffer { inner: &input };
    let value = parse(&mut input)?;
    if input.inner.is_empty() {
        Ok(value)
    } else {
        Err(input.unrecoverable_error("unexpected trailing tokens"))
    }
}
