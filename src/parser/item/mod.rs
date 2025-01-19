use crate::{
    tokenizer::{Ident, Span, Spanned},
    Result, Token,
};

use super::{Expr, Parenthesized, Parse, ParseStream, Stmt};

parse!({
    enum Item {
        Function(Function),
        Type(TypeItem),
        Subtype(SubtypeItem),
        Variable(Variable),
    }
});

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if let Some(item) = input.try_parse()? {
            Self::Function(item)
        } else if let Some(item) = input.try_parse()? {
            Self::Type(item)
        } else if let Some(item) = input.try_parse()? {
            Self::Subtype(item)
        } else if let Some(item) = input.try_parse()? {
            Self::Variable(item)
        } else {
            return Err(input.unrecoverable_error("expected item"));
        })
    }
}
parse!({
    struct Function {
        kind: FunctionKind,
        name: Ident,
        args: Option<Parenthesized<Param, Token![;]>>,
        return_type: Option<(Token![return], Ident)>,
        is_: Token![is],
        items: Vec<Item>,
        begin: Token![begin],
        stmts: Vec<Stmt>,
        end: Token![end],
        semi: Token![;],
    }
});

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
            let (items, begin) = input.parse_until(Token![begin])?;
            let (stmts, end) = input.parse_until_end()?;
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
                items,
                begin,
                stmts,
                end,
                semi,
            })
        })
    }
}

parse!({
    enum FunctionKind {
        Procedure(Token![procedure]),
        Function(Token![function]),
    }
});

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

parse!({
    struct Param {
        name: Ident,
        colon: Token![:],
        ty: Ident,
    }
});

impl Parse for Param {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            colon: input.parse()?,
            ty: input.parse()?,
        })
    }
}

parse!({
    enum TypeItem {
        Full(FullTypeItem),
    }
});

impl Parse for TypeItem {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self::Full(input.parse()?))
    }
}

parse!({
    struct FullTypeItem {
        type_: Token![type],
        name: Ident,
        is_: Token![is],
        definition: TypeDefinition,
        semi: Token![;],
    }
});

impl Parse for FullTypeItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let type_ = input.parse()?;
        input.unrecoverable(|input| {
            Ok(Self {
                type_,
                name: input.parse()?,
                is_: input.parse()?,
                definition: input.parse()?,
                semi: input.parse()?,
            })
        })
    }
}

parse!({
    enum TypeDefinition {
        Enum(EnumTypeDefinition),
        Signed(SignedTypeDefinition),
    }
});

impl Parse for TypeDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if let Some(td) = input.try_parse()? {
            Self::Signed(td)
        } else if let Some(td) = input.try_parse()? {
            Self::Enum(td)
        } else {
            return Err(input.recoverable_error("expected type definition"));
        })
    }
}

parse!({
    struct EnumTypeDefinition {
        values: Parenthesized<Ident>,
    }
});

impl Parse for EnumTypeDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            values: input.parse()?,
        })
    }
}

parse!({
    struct SignedTypeDefinition {
        range_keyword: Token![range],
        range: Range,
    }
});

impl Parse for SignedTypeDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let range_keyword = input.parse()?;
        input.unrecoverable(|input| {
            Ok(Self {
                range_keyword,
                range: input.parse()?,
            })
        })
    }
}

parse!({
    struct SubtypeItem {
        type_: Token![subtype],
        name: Ident,
        is_: Token![is],
        mark: Ident,
        constraint: Option<Constraint>,
        semi: Token![;],
    }
});

impl Parse for SubtypeItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let type_ = input.parse()?;
        input.unrecoverable(|input| {
            Ok(Self {
                type_,
                name: input.parse()?,
                is_: input.parse()?,
                mark: input.parse()?,
                constraint: input.try_parse()?,
                semi: input.parse()?,
            })
        })
    }
}

parse!({
    enum Constraint {
        Range(RangeConstraint),
    }
});

impl Parse for Constraint {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if let Some(constraint) = input.try_parse()? {
            Self::Range(constraint)
        } else {
            return Err(input.recoverable_error("expected constraint"));
        })
    }
}

parse!({
    struct RangeConstraint {
        range_token: Token![range],
        range: Range,
    }
});

impl Parse for RangeConstraint {
    fn parse(input: ParseStream) -> Result<Self> {
        let range_token = input.parse()?;
        input.unrecoverable(|input| {
            Ok(Self {
                range_token,
                range: input.parse()?,
            })
        })
    }
}

parse!({
    struct Range {
        start: Expr,
        dot_dot: Token![..],
        end: Expr,
    }
});

impl Parse for Range {
    fn parse(input: ParseStream) -> Result<Self> {
        let start = input.call(Expr::parse_simple_expression)?;
        let dot_dot = input.parse()?;
        input.unrecoverable(|input| {
            Ok(Self {
                start,
                dot_dot,
                end: input.call(Expr::parse_simple_expression)?,
            })
        })
    }
}

parse!({
    struct Variable {
        name: Ident,
        colon: Token![:],
        ty: Ident,
        semi: Token![;],
    }
});

impl Parse for Variable {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            colon: input.parse()?,
            ty: input.parse()?,
            semi: input.parse()?,
        })
    }
}
