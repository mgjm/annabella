use std::{
    fmt::{self, Write},
    ops::Deref,
    path::PathBuf,
    rc::Rc,
};

mod span;

use crate::Result;

use self::span::SpanOffset;
pub use self::span::{Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenStream {
    inner: Rc<Vec<TokenTree>>,
}

impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for tree in self.inner.iter() {
            if !first {
                f.write_str(" ")?;
            }
            first = false;
            tree.fmt(f)?
        }
        Ok(())
    }
}

impl TokenStream {
    pub fn parse(source: &str, filename: Option<PathBuf>) -> Result<Self> {
        let span = Span::for_new_file(source.into(), filename);
        Self::parse_cursor(Cursor {
            str: source,
            offset: span.start,
        })
    }

    fn parse_cursor(mut input: Cursor) -> Result<Self> {
        let mut trees = Vec::new();
        let mut stack: Vec<(SpanOffset, Delimiter, Vec<TokenTree>)> = Vec::new();

        loop {
            input = input.skip_whitespace();

            let start = input.offset;
            let Some(first) = input.chars().next() else {
                return if let Some(&(start, ..)) = stack.last() {
                    Err(Span::new(start, start).unrecoverable_error("unclosed groups remaining"))
                } else {
                    Ok(Self {
                        inner: trees.into(),
                    })
                };
            };

            if let Some(delimiter) = match first {
                '(' => Some(Delimiter::Parenthesis),
                '[' | '{' => return Err(input.unrecoverable_error("only parenthesis allowed")),
                _ => None,
            } {
                input = input.advance(1);
                stack.push((start, delimiter, trees));
                trees = Vec::new();
            } else if let Some(close) = match first {
                ')' => Some(Delimiter::Parenthesis),
                _ => None,
            } {
                let Some((start, open, outer)) = stack.pop() else {
                    return Err(input.unrecoverable_error("missing open delimiter"));
                };
                if open != close {
                    return Err(input.unrecoverable_error(format!(
                        "delimiter does not match: expected `{}``",
                        open.close()
                    )));
                }
                input = input.advance(1);
                let group = Group {
                    delimiter: open,
                    stream: Self {
                        inner: trees.into(),
                    },
                    span: Span::new(start, input.offset - 1),
                };
                trees = outer;
                trees.push(TokenTree::Group(group));
            } else {
                let (tt, rest) = TokenTree::leaf_token(input)?;
                trees.push(tt);
                input = rest;
            }
        }
    }
}

impl Deref for TokenStream {
    type Target = [TokenTree];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> IntoIterator for &'a TokenStream {
    type Item = &'a TokenTree;
    type IntoIter = std::slice::Iter<'a, TokenTree>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenTree {
    Group(Group),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}

impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Group(group) => group.fmt(f),
            Self::Ident(ident) => ident.fmt(f),
            Self::Punct(punct) => punct.fmt(f),
            Self::Literal(literal) => literal.fmt(f),
        }
    }
}

impl Spanned for TokenTree {
    fn span(&self) -> Span {
        match self {
            Self::Group(group) => group.span(),
            Self::Ident(ident) => ident.span(),
            Self::Punct(punct) => punct.span(),
            Self::Literal(literal) => literal.span(),
        }
    }
}

impl TokenTree {
    fn leaf_token(input: Cursor) -> Result<(Self, Cursor)> {
        Ok(
            if let Some((literal, input)) = Literal::parse(input.clone())? {
                (Self::Literal(literal), input)
            } else if let Some((ident, input)) = Ident::parse(input.clone())? {
                (Self::Ident(ident), input)
            } else if let Some((punct, input)) = Punct::parse(input.clone())? {
                (Self::Punct(punct), input)
            } else {
                return Err(input.unrecoverable_error("unparsable token"));
            },
        )
    }
}

#[derive(Clone)]
#[must_use]
struct Cursor<'a> {
    str: &'a str,
    offset: SpanOffset,
}

impl Deref for Cursor<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.str
    }
}

impl Cursor<'_> {
    pub fn advance(self, bytes: usize) -> Self {
        if bytes == 0 {
            return self.clone();
        }

        let (used, str) = self.str.split_at(bytes);
        Self {
            str,
            offset: span::offset_add(self.offset, || used.chars().count()),
        }
    }

    pub fn advance_char(self) -> Option<(char, Self)> {
        let char = self.chars().next()?;
        let input = self.advance(char.len_utf8());
        Some((char, input))
    }

    fn skip_whitespace(self) -> Self {
        let mut chars = self.char_indices();
        let bytes = 'outer: loop {
            let Some((i, char)) = chars.next() else {
                break self.len();
            };

            if char == '-' && matches!(chars.next(), Some((_, '-'))) {
                loop {
                    match chars.next() {
                        None => break 'outer self.len(),
                        Some((_, '\n')) => continue 'outer,
                        Some(_) => {}
                    }
                }
            }
            if !char.is_ascii_whitespace() {
                break i;
            }
        };
        self.advance(bytes)
    }
}

impl Spanned for Cursor<'_> {
    fn span(&self) -> Span {
        Span::new(self.offset, self.offset)
    }
}

#[derive(Clone)]
pub struct Group {
    pub delimiter: Delimiter,
    pub stream: TokenStream,
    span: Span,
}

impl Eq for Group {}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.delimiter == other.delimiter && self.stream == other.stream
    }
}

impl fmt::Debug for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Group")
            .field("delimiter", &self.delimiter)
            .field("stream", &self.stream)
            .finish()
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(self.delimiter.open())?;
        self.stream.fmt(f)?;
        f.write_char(self.delimiter.close())
    }
}

impl Spanned for Group {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Delimiter {
    /// `( ... )`
    Parenthesis,
}

impl Delimiter {
    fn open(self) -> char {
        match self {
            Delimiter::Parenthesis => '(',
        }
    }

    fn close(self) -> char {
        match self {
            Delimiter::Parenthesis => ')',
        }
    }
}

#[derive(Clone)]
pub struct Ident {
    pub name: Box<str>,
    pub span: Span,
}

impl Eq for Ident {}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.name.fmt(f)
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.name)
    }
}

impl Spanned for Ident {
    fn span(&self) -> Span {
        self.span
    }
}

impl Ident {
    fn parse(input: Cursor) -> Result<Option<(Self, Cursor)>> {
        let mut chars = input.char_indices();
        match chars.next() {
            Some((_, c)) if Self::is_ident_char(c) => {}
            _ => return Ok(None),
        }
        let index = loop {
            let Some((i, mut char)) = chars.next() else {
                break input.len();
            };
            if char == '_' {
                let Some((_, c)) = chars.next() else {
                    break i;
                };
                char = c;
            }
            if !Self::is_letter_or_digit(char) {
                break i;
            }
        };

        let start = input.offset;
        let name = input[..index].into();
        let input = input.advance(index);
        let span = Span::new(start, input.offset - 1);
        Ok(Some((Self { name, span }, input)))
    }

    fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphabetic()
    }

    fn is_letter_or_digit(c: char) -> bool {
        c.is_ascii_alphanumeric()
    }
}

#[derive(Clone)]
pub struct Punct {
    pub char: char,
    pub span: Span,
    pub spacing: Spacing,
}

impl Eq for Punct {}

impl PartialEq for Punct {
    fn eq(&self, other: &Self) -> bool {
        self.char == other.char
    }
}

impl fmt::Debug for Punct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.char.fmt(f)
    }
}

impl fmt::Display for Punct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(self.char)?;
        if self.char == ';' {
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl Spanned for Punct {
    fn span(&self) -> Span {
        self.span
    }
}

impl Punct {
    fn parse(input: Cursor) -> Result<Option<(Self, Cursor)>> {
        let mut chars = input.chars();
        let char = chars.next().unwrap();
        Ok(if Self::is_special_character(char) {
            let spacing = if matches!(chars.next(), Some(c) if Self::is_special_character(c)) {
                Spacing::Joint
            } else {
                Spacing::Alone
            };
            let start = input.offset;
            let input = input.advance(char.len_utf8());
            let span = Span::new(start, start);
            Some((
                Self {
                    char,
                    span,
                    spacing,
                },
                input,
            ))
        } else {
            None
        })
    }

    fn is_special_character(c: char) -> bool {
        matches!(
            c,
            '&' | '\''
                | '('
                | ')'
                | '*'
                | '+'
                | ','
                | '-'
                | '.'
                | '/'
                | ':'
                | ';'
                | '<'
                | '='
                | '>'
                | '|'
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Spacing {
    Alone,
    Joint,
}

impl Spacing {
    pub fn is_alone(self) -> bool {
        matches!(self, Self::Alone)
    }
}

#[derive(Clone)]
pub struct Literal {
    pub str: Box<str>,
    pub span: Span,
}

impl Eq for Literal {}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        self.str == other.str
    }
}

impl fmt::Debug for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.str.fmt(f)
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.str)
    }
}

impl Spanned for Literal {
    fn span(&self) -> Span {
        self.span
    }
}

impl Literal {
    fn parse(input: Cursor) -> Result<Option<(Self, Cursor)>> {
        let start = input.clone();
        'block: {
            let mut chars = input.char_indices();
            let mut is_number = false;
            let input = match chars.next().unwrap().1 {
                '\'' => {
                    let Some((_, c)) = chars.next() else {
                        break 'block;
                    };
                    let Some((end, '\'')) = chars.next() else {
                        break 'block;
                    };

                    if c == '(' {
                        // special handling for the following syntax:
                        //     String'('x', ...
                        //            1 2
                        // 1: This is the open parenthesis of the `'(` token
                        // 2: This is the actual char literal
                        //
                        // We always skip 1, there might be bugs resulting from this
                        if let Some((_, c)) = chars.next() {
                            if let Some((_, '\'')) = chars.next() {
                                if !Self::is_extended_graphic_character(c) {
                                    return Err(input
                                        .advance(1)
                                        .unrecoverable_error("expected graphic_character"));
                                }
                                return Ok(None);
                            }
                        };
                    }
                    if !Self::is_extended_graphic_character(c) {
                        return Err(input
                            .advance(1)
                            .unrecoverable_error("expected graphic_character"));
                    }
                    input.advance(end + 1)
                }
                '"' => loop {
                    let Some((i, char)) = chars.next() else {
                        return Err(input.unrecoverable_error("unterminated string literal"));
                    };
                    if char == '"' {
                        if matches!(chars.next(), Some((_, '"'))) {
                            continue;
                        }
                        break input.advance(i + 1);
                    }
                },
                '0'..='9' => {
                    is_number = true;
                    let input = Self::consume_rest_of_numeral(input)?;
                    match input.clone().advance_char() {
                        Some(('#', input)) => {
                            let input = Self::consume_based_numeral(input)?;
                            let input = if let Some(('.', input)) = input.clone().advance_char() {
                                Self::consume_based_numeral(input)?
                            } else {
                                input
                            };
                            let Some(('#', input)) = input.clone().advance_char() else {
                                return Err(input.unrecoverable_error(
                                    "expected second `#` delimiter in based number literal",
                                ));
                            };
                            match input.clone().advance_char() {
                                Some(('+' | '-' | '0'..='9', _)) => {
                                    return Err(input.unrecoverable_error(
                                        "based literal with exponent not implemented",
                                    ))
                                }
                                _ => input,
                            }
                        }
                        Some(('.', input)) => {
                            let input = Self::consume_numeral(input)?;
                            if let Some(('E' | 'e', input)) = input.clone().advance_char() {
                                let input = Self::consume_sign(input);
                                Self::consume_numeral(input)?
                            } else {
                                input
                            }
                        }
                        _ => input,
                    }
                }

                _ => break 'block,
            };

            let str: Box<str> = start[..start.len() - input.len()].into();
            let span = Span::new(start.offset, input.offset - 1);
            debug_assert_eq!(span.source().as_deref(), Some(&*str));
            assert!(!is_number || Self::validate_number(&str), "invalid: {str}");
            return Ok(Some((Self { str, span }, input)));
        }
        Ok(None)
    }

    // fn is_graphic_character(c: char) -> bool {
    //     c.is_ascii_alphanumeric() || Punct::is_special_character(c) || c.is_ascii_whitespace()
    // }

    fn is_extended_graphic_character(c: char) -> bool {
        c == ' ' || c.is_ascii_graphic()
    }

    fn validate_number(s: &str) -> bool {
        let mut parts = s.split('#');
        let a = parts.next();
        let b = parts.next();
        let c = parts.next();
        let d = parts.next();
        match (a, b, c, d) {
            (Some(a), None, None, None) => {
                if let Some((a, b)) = a.split_once(['e', 'E']) {
                    !a.contains(|c: char| !c.is_ascii_digit() && c != '_' && c != '.')
                        && !b
                            .strip_prefix(['-', '+'])
                            .unwrap_or(b)
                            .contains(|c: char| !c.is_ascii_digit() && c != '_')
                } else {
                    !a.contains(|c: char| !c.is_ascii_digit() && c != '_' && c != '.')
                }
            }
            (Some(a), Some(b), Some(c), None) => {
                !a.contains(|c: char| !c.is_ascii_digit())
                    && !b.contains(|c: char| !c.is_ascii_hexdigit() && c != '_' && c != '.')
                    && !c.contains(|c: char| !c.is_ascii_digit())
            }
            _ => false,
        }
    }

    fn consume_sign(input: Cursor) -> Cursor {
        match input.clone().advance_char() {
            Some(('+' | '-', input)) => input,
            _ => input,
        }
    }

    fn consume_numeral(input: Cursor) -> Result<Cursor> {
        let span = input.span();
        let input = match input.advance_char() {
            Some((char, input)) if char.is_ascii_digit() => input,
            _ => return Err(span.unrecoverable_error("expected digit")),
        };
        Self::consume_rest_of_numeral(input)
    }

    fn consume_rest_of_numeral(mut input: Cursor) -> Result<Cursor> {
        Ok(loop {
            let Some((mut char, mut input2)) = input.clone().advance_char() else {
                break input;
            };
            if char == '_' {
                let Some(next) = input2.advance_char() else {
                    break input;
                };
                (char, input2) = next;
            }
            if !char.is_ascii_digit() {
                break input;
            }
            input = input2;
        })
    }

    fn consume_based_numeral(input: Cursor) -> Result<Cursor> {
        let span = input.span();
        let mut input = match input.advance_char() {
            Some((char, input)) if char.is_ascii_hexdigit() => input,
            _ => return Err(span.unrecoverable_error("expected hex digit")),
        };

        Ok(loop {
            let Some((mut char, mut input2)) = input.clone().advance_char() else {
                break input;
            };
            if char == '_' {
                let Some(next) = input2.advance_char() else {
                    break input;
                };
                (char, input2) = next;
            }
            if !char.is_ascii_hexdigit() {
                break input;
            }
            input = input2;
        })
    }
}
