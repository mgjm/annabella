use crate::{
    tokenizer::{Span, Spanned},
    Token,
};

use super::{Expr, Name, Parse, ParseStream, Result};

parse!({
    enum Stmt {
        Expr(ExprStmt),
        Assign(AssignStmt),
        Return(ReturnStmt),
        If(IfStmt),
    }
});

impl Parse for Stmt {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if let Some(stmt) = input.try_parse()? {
            Self::Return(stmt)
        } else if let Some(stmt) = input.try_parse()? {
            Self::If(stmt)
        } else if let Some(stmt) = input.try_parse()? {
            Self::Assign(stmt)
        } else {
            Self::Expr(input.parse()?)
        })
    }
}

parse!({
    struct ExprStmt {
        expr: Expr,
        semi: Token![;],
    }
});

impl Parse for ExprStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            expr: input.parse()?,
            semi: input.parse()?,
        })
    }
}

parse!({
    struct AssignStmt {
        name: Name,
        assign: Token![:=],
        expr: Expr,
        semi: Token![;],
    }
});

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

parse!({
    struct ReturnStmt {
        return_: Token![return],
        expr: Expr,
        semi: Token![;],
    }
});

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

parse!({
    struct IfStmt {
        if_: Token![if],
        cond: Expr,
        then: Token![then],
        stmts: Vec<Stmt>,
        elsifs: Vec<ElsIf>,
        else_: Option<Else>,
        end: Token![end],
        semi: Token![;],
    }
});

impl Parse for IfStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let if_ = input.parse()?;
        input.unrecoverable(|input| {
            let cond = input.parse()?;
            let then = input.parse()?;

            let mut stmts = vec![input.parse()?];
            let mut elsifs = Vec::new();
            let mut else_ = None;

            {
                let mut else_allowed = true;
                let mut vec = &mut stmts;

                while !input.peek(Token![end]) {
                    if else_allowed {
                        if let Some(elsif) = input.try_parse()? {
                            let cond = input.parse()?;
                            let then = input.parse()?;
                            elsifs.push(ElsIf {
                                elsif,
                                cond,
                                then,
                                stmts: Vec::new(),
                            });
                            vec = &mut elsifs.last_mut().unwrap().stmts;
                        } else if let Some(token) = input.try_parse()? {
                            else_allowed = false;
                            else_ = Some(Else {
                                else_: token,
                                stmts: Vec::new(),
                            });
                            vec = &mut else_.as_mut().unwrap().stmts
                        }
                    }
                    vec.push(input.parse()?);
                }
            }

            let end = input.parse()?;
            let _: Token![if] = input.parse()?;
            let semi = input.parse()?;
            Ok(Self {
                if_,
                cond,
                then,
                stmts,
                elsifs,
                else_,
                end,
                semi,
            })
        })
    }
}

parse!({
    struct Else {
        else_: Token![else],
        stmts: Vec<Stmt>,
    }
});

parse!({
    struct ElsIf {
        elsif: Token![elsif],
        cond: Expr,
        then: Token![then],
        stmts: Vec<Stmt>,
    }
});
