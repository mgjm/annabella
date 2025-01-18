use crate::{
    tokenizer::{Span, Spanned},
    Token,
};

use super::{Expr, Name, Parse, ParseStream, Result};

parse! {
    enum Stmt {
        Expr(ExprStmt),
        Return(ReturnStmt),
        Assign(AssignStmt),
    }
}

impl Parse for Stmt {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if let Some(stmt) = input.try_parse()? {
            Self::Return(stmt)
        } else if let Some(stmt) = input.try_parse()? {
            Self::Assign(stmt)
        } else {
            Self::Expr(input.parse()?)
        })
    }
}

parse! {
    struct ExprStmt {
        expr: Expr,
        semi: Token![;],
    }
}

impl Parse for ExprStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            expr: input.parse()?,
            semi: input.parse()?,
        })
    }
}

parse! {
    struct ReturnStmt {
    return_: Token![return],
        expr: Expr,
        semi: Token![;],
    }
}

impl Parse for ReturnStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let return_ = input.parse()?;
        input.unrecoverable(|input| {
            Ok(Self {
                return_,
                expr: input.parse()?,
                semi: input.parse()?,
            })
        })
    }
}

parse! {
    struct AssignStmt {
        name: Name,
        assign: Token![:=],
        expr: Expr,
        semi: Token![;],
    }
}

impl Parse for AssignStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        let assign = input.parse()?;
        input.unrecoverable(|input| {
            Ok(Self {
                name,
                assign,
                expr: input.parse()?,
                semi: input.parse()?,
            })
        })
    }
}
