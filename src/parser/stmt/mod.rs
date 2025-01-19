use crate::{
    tokenizer::{Ident, Span, Spanned},
    Result, Token,
};

use super::{DiscreteChoice, Expr, Item, Name, Parse, ParseStream, Punctuated, Range};

parse!({
    enum Stmt {
        Label(LabelStmt),
        Expr(ExprStmt),
        Assign(AssignStmt),
        Return(ReturnStmt),
        If(IfStmt),
        Block(BlockStmt),
        Goto(GotoStmt),
        Loop(LoopStmt),
        Exit(ExitStmt),
        Case(CaseStmt),
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
            Self::Exit(stmt)
        } else if let Some(stmt) = input.try_parse()? {
            Self::Case(stmt)
        } else if let Some(stmt) = input.try_parse()? {
            Self::Loop(stmt)
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
        let ident = input.try_call(parse_ident_colon)?;
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

parse!({
    struct LoopStmt {
        ident: Option<(Ident, Token![:])>,
        scheme: LoopScheme,
        loop_: Token![loop],
        stmts: Vec<Stmt>,
        end: Token![end],
        semi: Token![;],
    }
});

impl LoopStmt {
    pub fn ident(&self) -> Option<&Ident> {
        self.ident.as_ref().map(|(ident, _)| ident)
    }
}

impl Parse for LoopStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.try_call(parse_ident_colon)?;
        let scheme = input.parse()?;
        let loop_ = input.parse()?;
        input.unrecoverable(|input| {
            let (stmts, end) = input.parse_until_end()?;
            let _: Token![loop] = input.parse()?;
            if let Some((ident, _)) = &ident {
                input.parse_ident(ident)?;
            }
            let semi = input.parse()?;
            Ok(Self {
                ident,
                scheme,
                loop_,
                stmts,
                end,
                semi,
            })
        })
    }
}

parse!({
    enum LoopScheme {
        Endless(EndlessLoopScheme),
        While(WhileLoopScheme),
        For(ForLoopScheme),
    }
});

impl Parse for LoopScheme {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if let Some(scheme) = input.try_parse()? {
            Self::While(scheme)
        } else if let Some(scheme) = input.try_parse()? {
            Self::For(scheme)
        } else {
            Self::Endless(EndlessLoopScheme)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndlessLoopScheme;

impl Spanned for EndlessLoopScheme {
    fn span(&self) -> Span {
        Span::call_site()
    }
}

parse!({
    struct WhileLoopScheme {
        while_: Token![while],
        cond: Expr,
    }
});

impl Parse for WhileLoopScheme {
    fn parse(input: ParseStream) -> Result<Self> {
        let while_ = input.parse()?;
        input.unrecoverable(|input| {
            Ok(Self {
                while_,
                cond: input.parse()?,
            })
        })
    }
}

parse!({
    struct ForLoopScheme {
        for_: Token![for],
        ident: Ident,
        in_: Token![in],
        reverse: Option<Token![reverse]>,
        range: Range,
    }
});

impl ForLoopScheme {
    pub fn reverse(&self) -> bool {
        self.reverse.is_some()
    }
}

impl Parse for ForLoopScheme {
    fn parse(input: ParseStream) -> Result<Self> {
        let for_ = input.parse()?;
        input.unrecoverable(|input| {
            Ok(Self {
                for_,
                ident: input.parse()?,
                in_: input.parse()?,
                reverse: input.try_parse()?,
                range: input.parse()?,
            })
        })
    }
}

parse!({
    struct ExitStmt {
        exit: Token![exit],
        name: Option<Ident>,
        when: Option<(Token![when], Expr)>,
        semi: Token![;],
    }
});

impl ExitStmt {
    pub fn cond(&self) -> Option<&Expr> {
        self.when.as_ref().map(|(_, cond)| cond)
    }
}

impl Parse for ExitStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let exit = input.parse()?;
        input.unrecoverable(|input| {
            Ok(Self {
                exit,
                name: input.try_parse()?,
                when: input.try_call(|input| {
                    let when = input.parse()?;
                    input.unrecoverable(|input| Ok((when, input.parse()?)))
                })?,
                semi: input.parse()?,
            })
        })
    }
}

parse!({
    struct CaseStmt {
        case: Token![case],
        expr: Expr,
        is_: Token![is],
        alternatives: Vec<CaseStmtAlternative>,
        end: Token![end],
        semi: Token![;],
    }
});

impl Parse for CaseStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let case = input.parse()?;
        input.unrecoverable(|input| {
            let expr = input.parse()?;
            let is_ = input.parse()?;
            let mut alternatives: Vec<CaseStmtAlternative> = vec![input.parse()?];
            {
                let mut vec = &mut alternatives.last_mut().unwrap().stmts;

                while !input.peek(Token![end]) {
                    if let Some(alt) = input.try_parse()? {
                        alternatives.push(alt);
                        vec = &mut alternatives.last_mut().unwrap().stmts;
                    } else {
                        vec.push(input.parse()?);
                    }
                }
            }
            let end = input.parse()?;
            let _: Token![case] = input.parse()?;
            let semi = input.parse()?;
            Ok(Self {
                case,
                expr,
                is_,
                alternatives,
                end,
                semi,
            })
        })
    }
}
parse!({
    struct CaseStmtAlternative {
        when: Token![when],
        choices: Punctuated<DiscreteChoice, Token![|]>,
        arrow: Token![=>],
        stmts: Vec<Stmt>,
    }
});

impl Parse for CaseStmtAlternative {
    fn parse(input: ParseStream) -> crate::Result<Self> {
        let when = input.parse()?;
        input.unrecoverable(|input| {
            Ok(Self {
                when,
                choices: input.call(Punctuated::parse_while)?,
                arrow: input.parse()?,
                stmts: vec![input.parse()?],
            })
        })
    }
}

fn parse_ident_colon(input: ParseStream) -> Result<(Ident, Token![:])> {
    let ident = input.parse()?;
    let colon = input.parse()?;
    Ok((ident, colon))
}
