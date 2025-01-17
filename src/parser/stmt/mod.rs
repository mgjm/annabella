use crate::{
    tokenizer::{Ident, Span, Spanned},
    Token,
};

use super::{Expr, Name, Parenthesized, Parse, ParseStream, Result};

parse_enum! {
    enum Stmt {
        Expr(ExprStmt),
        Return(ReturnStmt),
        Function(Function),
    }
}

impl Parse for Stmt {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if let Some(stmt) = input.try_parse()? {
            Self::Function(stmt)
        } else if let Some(stmt) = input.try_parse()? {
            Self::Return(stmt)
        } else {
            Self::Expr(input.parse()?)
        })
    }
}

parse_struct! {
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

parse_struct! {
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

parse_struct! {
    struct Function {
        kind: FunctionKind,
        name: Ident,
        args: Option<Parenthesized<Param, Token![;]>>,
        return_type: Option<(Token![return], Ident)>,
        is_: Token![is],
        begin: Token![begin],
        stmts: Vec<Stmt>,
        end: Token![end],
        semi: Token![;],
    }
}

impl Function {
    pub fn args(&self) -> impl Iterator<Item = &Param> {
        self.args.iter().flat_map(|args| args.iter())
    }

    pub fn return_type(&self) -> Option<&Ident> {
        self.return_type.as_ref().map(|(_, ty)| ty)
    }
}

impl Parse for Function {
    fn parse(input: ParseStream) -> Result<Self> {
        let kind = input.parse()?;
        input.unrecoverable(|input| {
            let name = input.parse()?;
            let args = input.try_parse()?;
            let return_type = match &kind {
                FunctionKind::Procedure(_) => None,
                FunctionKind::Function(_) => {
                    let keyword = input.parse()?;
                    let ty = input.parse()?;
                    Some((keyword, ty))
                }
            };
            let is_ = input.parse()?;
            let begin = input.parse()?;
            let stmts = input.parse_until_end()?;
            let end = input.parse()?;
            if let Some(name2) = input.try_parse::<Ident>()? {
                if name != name2 {
                    return Err(name2.unrecoverable_error(format!("expected `{name}`")));
                }
            }
            let semi = input.parse()?;
            Ok(Self {
                kind,
                name,
                args,
                return_type,
                is_,
                begin,
                stmts,
                end,
                semi,
            })
        })
    }
}

parse_enum! {
    enum FunctionKind {
        Procedure(Token![procedure]),
        Function(Token![function]),
    }
}

impl Parse for FunctionKind {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if let Some(kind) = input.try_parse()? {
            Self::Procedure(kind)
        } else if let Some(kind) = input.try_parse()? {
            Self::Function(kind)
        } else {
            return Err(input.recoverable_error("expected `procedure` or `function`"));
        })
    }
}

parse_struct! {
    struct Param {
        name: Ident,
        colon: Token![:],
        ty: Ident,
    }
}

impl Parse for Param {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            colon: input.parse()?,
            ty: input.parse()?,
        })
    }
}
