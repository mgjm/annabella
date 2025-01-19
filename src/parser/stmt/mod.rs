use crate::{
    tokenizer::{Ident, Span, Spanned},
    Token,
};

use super::{Expr, Item, Name, Parse, ParseStream, Result};

parse!({
    enum Stmt {
        Label(LabelStmt),
        Expr(ExprStmt),
        Assign(AssignStmt),
        Return(ReturnStmt),
        If(IfStmt),
        Block(BlockStmt),
        Goto(GotoStmt),
    }
});

impl Parse for Stmt {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if let Some(stmt) = input.try_parse()? {
            Self::Label(stmt)
        } else if let Some(stmt) = input.try_parse()? {
            Self::Return(stmt)
        } else if let Some(stmt) = input.try_parse()? {
            Self::If(stmt)
        } else if let Some(stmt) = input.try_parse()? {
            Self::Goto(stmt)
        } else if let Some(stmt) = input.try_parse()? {
            Self::Block(stmt)
        } else if let Some(stmt) = input.try_parse()? {
            Self::Assign(stmt)
        } else {
            Self::Expr(input.parse()?)
        })
    }
}

parse!({
    struct LabelStmt {
        open: Token![<<],
        label: Ident,
        close: Token![>>],
    }
});

impl Parse for LabelStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let open = input.parse()?;
        input.unrecoverable(|input| {
            Ok(Self {
                open,
                label: input.parse()?,
                close: input.parse()?,
            })
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

parse!({
    struct BlockStmt {
        ident: Option<(Ident, Token![:])>,
        declare: Option<(Token![declare], Vec<Item>)>,
        begin: Token![begin],
        stmts: Vec<Stmt>,
        end: Token![end],
        semi: Token![;],
    }
});

impl BlockStmt {
    pub fn ident(&self) -> Option<&Ident> {
        self.ident.as_ref().map(|(ident, _)| ident)
    }

    pub fn items(&self) -> impl Iterator<Item = &Item> + '_ {
        self.declare
            .as_ref()
            .map_or(Default::default(), |(_, items)| items.as_slice())
            .iter()
    }
}

impl Parse for BlockStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.try_call(|input| {
            let ident = input.parse()?;
            let colon = input.parse()?;
            Ok((ident, colon))
        })?;
        let declare = input.try_call(|input| {
            let declare = input.parse()?;
            input.unrecoverable(|input| Ok((declare, input.parse_until_peeked(Token![begin])?)))
        })?;
        let declare_found = declare.is_some();
        let parse = |input: ParseStream| {
            let begin = input.parse()?;
            input.unrecoverable(|input| {
                let (stmts, end) = input.parse_until_end()?;
                if let Some((ident, _)) = &ident {
                    input.parse_ident(ident)?;
                }
                let semi = input.parse()?;
                Ok(Self {
                    ident,
                    declare,
                    begin,
                    stmts,
                    end,
                    semi,
                })
            })
        };
        if declare_found {
            input.unrecoverable(parse)
        } else {
            parse(input)
        }
    }
}

parse!({
    struct GotoStmt {
        goto: Token![goto],
        label: Ident,
        semi: Token![;],
    }
});

impl Parse for GotoStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let goto = input.parse()?;
        input.unrecoverable(|input| {
            Ok(Self {
                goto,
                label: input.parse()?,
                semi: input.parse()?,
            })
        })
    }
}
