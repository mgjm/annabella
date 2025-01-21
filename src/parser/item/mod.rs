use crate::{
    tokenizer::{Ident, Span, Spanned},
    Result, Token,
};

use super::{DiscreteChoice, Expr, Parenthesized, Parse, ParseStream, Punctuated, Stmt};

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
            input.try_parse_ident(&name)?;
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
        mode: ParamMode,
        ty: Ident,
    }
});

impl Parse for Param {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            colon: input.parse()?,
            mode: input.parse()?,
            ty: input.parse()?,
        })
    }
}

parse!({
    enum ParamMode {
        In(Option<Token![in]>),
        Out(Token![out]),
        InOut(InOut),
    }
});

impl Parse for ParamMode {
    fn parse(input: ParseStream) -> Result<Self> {
        let in_ = input.try_parse()?;
        let out = input.try_parse()?;
        Ok(match (in_, out) {
            (in_, None) => Self::In(in_),
            (None, Some(out)) => Self::Out(out),
            (Some(in_), Some(out)) => Self::InOut(InOut { in_, out }),
        })
    }
}

parse!({
    struct InOut {
        in_: Token![in],
        out: Token![out],
    }
});

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
        Modular(ModularTypeDefinition),
        Record(RecordTypeDefinition),
    }
});

impl Parse for TypeDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if let Some(td) = input.try_parse()? {
            Self::Signed(td)
        } else if let Some(td) = input.try_parse()? {
            Self::Modular(td)
        } else if let Some(td) = input.try_parse()? {
            Self::Record(td)
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
    struct ModularTypeDefinition {
        mod_: Token![mod],
        modulus: Expr,
    }
});

impl Parse for ModularTypeDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let mod_ = input.parse()?;
        input.unrecoverable(|input| {
            Ok(Self {
                mod_,
                modulus: input.parse()?,
            })
        })
    }
}

parse!({
    struct RecordTypeDefinition {
        record: Token![record],
        components: RecordComponentList,
        end: Token![end],
    }
});

impl Parse for RecordTypeDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let record = input.parse()?;
        input.unrecoverable(|input| {
            let components = input.parse()?;
            let end = input.parse()?;
            let _: Token![record] = input.parse()?;
            Ok(Self {
                record,
                components,
                end,
            })
        })
    }
}

parse!({
    struct RecordComponentList {
        components: Vec<Variable>,
        variant: Option<RecordVariant>,
        null: Option<Token![null]>,
    }
});

impl Parse for RecordComponentList {
    fn parse(input: ParseStream) -> Result<Self> {
        let null = input.try_parse()?;
        let mut components = Vec::new();
        let variant = if null.is_some() {
            None
        } else {
            while let Some(component) = input.try_parse()? {
                components.push(component);
            }

            if components.is_empty() {
                Some(input.parse()?)
            } else {
                input.try_parse()?
            }
        };
        Ok(Self {
            components,
            variant,
            null,
        })
    }
}

parse!({
    struct RecordVariant {
        case: Token![case],
        expr: Expr,
        is_: Token![is],
        alternatives: Vec<RecordVariantAlternative>,
        end: Token![end],
        semi: Token![;],
    }
});

impl Parse for RecordVariant {
    fn parse(input: ParseStream) -> Result<Self> {
        let case = input.parse()?;
        input.unrecoverable(|input| {
            let expr = input.parse()?;
            let is_ = input.parse()?;
            let (alternatives, end) = input.parse_until_end()?;
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
    struct RecordVariantAlternative {
        when: Token![when],
        choices: Punctuated<DiscreteChoice, Token![|]>,
        arrow: Token![=>],
        components: RecordComponentList,
    }
});

impl Parse for RecordVariantAlternative {
    fn parse(input: ParseStream) -> Result<Self> {
        let when = input.parse()?;
        input.unrecoverable(|input| {
            Ok(Self {
                when,
                choices: input.call(Punctuated::parse_while)?,
                arrow: input.parse()?,
                components: input.parse()?,
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
