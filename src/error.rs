use std::borrow::Cow;

use crate::tokenizer::Span;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
    pub span: Span,
    pub msg: Cow<'static, str>,
    pub recoverable: bool,
}
