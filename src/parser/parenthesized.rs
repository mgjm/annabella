use std::{fmt, ops::Deref};

use crate::{
    tokenizer::{Span, Spanned, TokenStream},
    Token,
};

use super::{Parse, ParseStream, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParenthesizedOne<T> {
    pub paren: Paren,
    pub inner: T,
}

impl<T> Deref for ParenthesizedOne<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Spanned for ParenthesizedOne<T>
where
    T: Spanned,
{
    fn span(&self) -> Span {
        let mut span = Span::call_site();
        span.extend(self.paren.span());
        span.extend(self.inner.span());
        span
    }
}

impl<T> ParenthesizedOne<T> {
    fn parse_with(
        input: ParseStream,
        parse: impl FnOnce(ParseStream) -> Result<T>,
    ) -> Result<Self> {
        let (paren, inner) = Paren::parse_inner(input)?;
        match super::parse_with(inner, |input| input.unrecoverable(parse)) {
            Ok(inner) => Ok(Self { paren, inner }),
            Err(mut err) => {
                if err.span.is_call_site() {
                    err.span = paren.span().end();
                }
                Err(err)
            }
        }
    }
}

impl<T> Parse for ParenthesizedOne<T>
where
    T: Parse,
{
    fn parse(input: ParseStream) -> Result<Self> {
        Self::parse_with(input, T::parse)
    }
}

pub type Parenthesized<T, P = Token![,]> = ParenthesizedOne<Punctuated<T, P>>;

impl<T, P> Parse for Parenthesized<T, P>
where
    T: Parse,
    P: Parse,
{
    fn parse(input: ParseStream) -> Result<Self> {
        Self::parse_with(input, Punctuated::parse_all)
    }
}

impl<T, P> FromIterator<T> for Parenthesized<T, P>
where
    P: Default,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            paren: Paren::default(),
            inner: Punctuated::from_iter(iter),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Paren {
    span: Span,
}

impl Paren {
    fn parse_inner(input: ParseStream) -> Result<(Self, TokenStream)> {
        input.step(|cursor| {
            if let Some((group, rest)) = cursor.group() {
                Ok(((Self { span: group.span() }, group.stream.clone()), rest))
            } else {
                Err(cursor.recoverable_error("expected paren"))
            }
        })
    }
}

impl fmt::Debug for Paren {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(stringify!(Paren))
    }
}

impl Default for Paren {
    fn default() -> Self {
        Self {
            span: Span::call_site(),
        }
    }
}

impl std::cmp::Eq for Paren {}

impl PartialEq for Paren {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Spanned for Paren {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Punctuated<T, P = Token![,]> {
    inner: Vec<(T, Option<P>)>,
}

impl<T, P> fmt::Debug for Punctuated<T, P>
where
    T: fmt::Debug,
    P: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T, P> Spanned for Punctuated<T, P>
where
    T: Spanned,
    P: Spanned,
{
    fn span(&self) -> Span {
        self.inner.as_slice().span()
    }
}

impl<T, P> Punctuated<T, P>
where
    T: Parse,
    P: Parse,
{
    fn parse_all(input: ParseStream) -> Result<Self> {
        let mut inner = vec![(input.parse()?, None)];

        while !input.is_empty() {
            inner.last_mut().unwrap().1 = Some(input.parse()?);
            inner.push((input.parse()?, None));
        }
        Ok(Self { inner })
    }

    pub fn parse_while(input: super::ParseStream) -> Result<Self> {
        let mut inner = vec![(input.parse()?, None)];
        while let Some(punct) = input.try_parse()? {
            inner.last_mut().unwrap().1 = Some(punct);
            inner.push((input.parse()?, None));
        }
        Ok(Self { inner })
    }
}

impl<T, P> Punctuated<T, P> {
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &T> + Clone {
        self.inner.iter().map(|(item, _)| item)
    }

    #[expect(clippy::should_implement_trait)]
    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.inner.into_iter().map(|(item, _)| item)
    }
}

impl<T, P> FromIterator<T> for Punctuated<T, P>
where
    P: Default,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut inner: Vec<_> = iter
            .into_iter()
            .map(|item| (item, Some(Default::default())))
            .collect();
        if let Some(last) = inner.last_mut() {
            last.1 = None;
        }
        Self { inner }
    }
}
