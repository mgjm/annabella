use crate::{
    tokenizer::{Ident, Literal, Span, Spanned},
    Result, Token,
};

use super::{Parenthesized, ParenthesizedOne, Parse, ParseStream};

parse!({
    enum Expr {
        Lit(ExprLit),
        Name(Name),
        Qualified(QualifiedExpr),
        Unary(ExprUnary),
        Binary(ExprBinary),
        ShortCircuit(ExprShortCircuit),
    }
});

impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        Self::parse_expression(input)
    }
}

impl Expr {
    fn parse_expression(input: ParseStream) -> Result<Self> {
        use helper::LogicalOp;

        let mut expr = Self::parse_relation(input)?;
        let mut prev = None;

        Ok(loop {
            let op = if let Some(op) = input.try_parse()? {
                BinaryOp::And(op)
            } else if let Some(op) = input.try_parse()? {
                BinaryOp::Or(op)
            } else if let Some(op) = input.try_parse()? {
                BinaryOp::Xor(op)
            } else {
                break expr;
            };

            let op = match op {
                BinaryOp::And(and) => {
                    if let Some(then) = input.try_parse()? {
                        LogicalOp::ShortCircuit(ShortCircuitOp::And(AndThen { and, then }))
                    } else {
                        LogicalOp::Binary(op)
                    }
                }
                BinaryOp::Or(or) => {
                    if let Some(else_) = input.try_parse()? {
                        LogicalOp::ShortCircuit(ShortCircuitOp::Or(OrElse { or, else_ }))
                    } else {
                        LogicalOp::Binary(op)
                    }
                }
                _ => LogicalOp::Binary(op),
            };

            if let Some(prev) = &prev {
                if *prev != op {
                    return Err(op.unrecoverable_error("expected same logical operator"));
                }
            } else {
                prev = Some(op.clone());
            }

            let rhs = Self::parse_relation(input)?;
            expr = match op {
                LogicalOp::Binary(op) => Self::Binary(ExprBinary {
                    lhs: expr.into(),
                    op,
                    rhs: rhs.into(),
                }),
                LogicalOp::ShortCircuit(op) => Self::ShortCircuit(ExprShortCircuit {
                    lhs: expr.into(),
                    op,
                    rhs: rhs.into(),
                }),
            };
        })
    }

    fn parse_relation(input: ParseStream) -> Result<Self> {
        let expr = Self::parse_simple_expression(input)?;

        {
            if let Some((not, in_)) = input.try_call(|input| {
                let not = input.try_parse()?;
                Ok((not, input.parse()?))
            })? {
                let _: Option<Token![not]> = not;
                let _: Token![in] = in_;
                return Err(in_.unrecoverable_error("`in` not yet implemented"));
            }
        }

        let op = if let Some(op) = input.try_parse()? {
            BinaryOp::Eq(op)
        } else if let Some(op) = input.try_parse()? {
            BinaryOp::Ne(op)
        } else if let Some(op) = input.try_parse()? {
            BinaryOp::Le(op)
        } else if let Some(op) = input.try_parse()? {
            BinaryOp::Ge(op)
        } else if let Some(op) = input.try_parse()? {
            BinaryOp::Lt(op)
        } else if let Some(op) = input.try_parse()? {
            BinaryOp::Gt(op)
        } else {
            return Ok(expr);
        };

        Ok(Self::Binary(ExprBinary {
            lhs: expr.into(),
            op,
            rhs: Self::parse_simple_expression(input)?.into(),
        }))
    }

    pub(super) fn parse_simple_expression(input: ParseStream) -> Result<Self> {
        #[allow(clippy::manual_map)]
        let op = if let Some(op) = input.try_parse()? {
            Some(UnaryOp::Add(op))
        } else if let Some(op) = input.try_parse()? {
            Some(UnaryOp::Sub(op))
        } else {
            //
            None
        };

        let mut expr = Self::parse_term(input)?;
        if let Some(op) = op {
            expr = Self::Unary(ExprUnary {
                op,
                expr: expr.into(),
            });
        }

        Ok(loop {
            let op = if let Some(op) = input.try_parse()? {
                BinaryOp::Add(op)
            } else if let Some(op) = input.try_parse()? {
                BinaryOp::Sub(op)
            } else if let Some(op) = input.try_parse()? {
                BinaryOp::Concat(op)
            } else {
                break expr;
            };
            expr = Self::Binary(ExprBinary {
                lhs: expr.into(),
                op,
                rhs: Self::parse_term(input)?.into(),
            });
        })
    }

    fn parse_term(input: ParseStream) -> Result<Self> {
        let mut expr = Self::parse_factor(input)?;
        Ok(loop {
            let Ok((op, rhs)) = input.call(|input| {
                let op = if let Some(op) = input.try_parse()? {
                    BinaryOp::Mul(op)
                } else if let Some(op) = input.try_parse()? {
                    BinaryOp::Div(op)
                } else if let Some(op) = input.try_parse()? {
                    BinaryOp::Mod(op)
                } else if let Some(op) = input.try_parse()? {
                    BinaryOp::Rem(op)
                } else {
                    return Err(input.recoverable_error("expected multiplication operator"));
                };
                Ok((op, Self::parse_factor(input)?))
            }) else {
                break expr;
            };
            expr = Self::Binary(ExprBinary {
                lhs: expr.into(),
                op,
                rhs: rhs.into(),
            });
        })
    }

    fn parse_factor(input: ParseStream) -> Result<Self> {
        let op = if let Some(op) = input.try_parse()? {
            UnaryOp::Abs(op)
        } else if let Some(op) = input.try_parse()? {
            UnaryOp::Not(op)
        } else if let Some(expr) = input.try_call(Self::parse_primary)? {
            if let Some(op) = input.try_parse()? {
                return Ok(Self::Binary(ExprBinary {
                    lhs: expr.into(),
                    op: BinaryOp::Pow(op),
                    rhs: Self::parse_primary(input)?.into(),
                }));
            }
            return Ok(expr);
        } else {
            return Err(input.unrecoverable_error("expected factor"));
        };

        Ok(Self::Unary(ExprUnary {
            op,
            expr: Self::parse_primary(input)?.into(),
        }))
    }

    fn parse_primary(input: ParseStream) -> Result<Self> {
        Ok(if let Some(lit) = input.try_parse()? {
            Self::Lit(lit)
        } else if let Some(name) = input.try_parse()? {
            if let Some(tick) = input.try_parse()? {
                Self::Qualified(QualifiedExpr {
                    mark: name,
                    tick,
                    value: input.parse()?,
                })
            } else {
                Self::Name(name)
            }
        } else {
            return Err(input.unrecoverable_error("expected primary"));
        })
    }
}

parse!({
    enum ExprLit {
        Str(LitStr),
        Char(LitChar),
        Number(LitNumber),
    }
});

impl Parse for ExprLit {
    fn parse(input: ParseStream) -> Result<Self> {
        input.step(|cursor| {
            if let Some((lit, rest)) = cursor.literal() {
                let lit = lit.clone();
                Ok((
                    match lit.str.chars().next().unwrap() {
                        '"' => Self::Str(LitStr { lit }),
                        '\'' => Self::Char(LitChar { lit }),
                        _ => Self::Number(LitNumber { lit }),
                    },
                    rest,
                ))
            } else {
                Err(cursor.recoverable_error("expected literal"))
            }
        })
    }
}

parse!({
    struct LitStr {
        lit: Literal,
    }
});

impl LitStr {
    pub fn str(&self) -> String {
        self.lit
            .str
            .strip_prefix('"')
            .unwrap()
            .strip_suffix('"')
            .unwrap()
            .replace("\"\"", "\"")
    }
}

parse!({
    struct LitChar {
        lit: Literal,
    }
});

impl LitChar {
    pub fn char(&self) -> char {
        let mut chars = self
            .lit
            .str
            .strip_prefix('\'')
            .unwrap()
            .strip_suffix('\'')
            .unwrap()
            .chars();
        let c = chars.next().unwrap();
        assert!(chars.next().is_none());
        c
    }
}

parse!({
    struct LitNumber {
        lit: Literal,
    }
});

impl LitNumber {
    pub fn number<T>(&self) -> T
    where
        T: ParseNumber,
    {
        T::parse(&self.lit.str)
    }
}

pub trait ParseNumber: Sized {
    fn parse(s: &str) -> Self;
}

impl ParseNumber for i64 {
    fn parse(s: &str) -> Self {
        let mut n = 0;
        let (s, neg) = if let Some(s) = s.strip_prefix('-') {
            (s, true)
        } else {
            (s, false)
        };
        for c in s.bytes() {
            match c {
                b'_' => continue,
                b'0'..=b'9' => {
                    n *= 10;
                    n += i64::from(c - b'0');
                }
                _ => unreachable!("invalid char in number: {}", c as char),
            }
        }
        if neg {
            -n
        } else {
            n
        }
    }
}

parse!({
    enum Name {
        Base(BaseName),
        FunctionCall(FunctionCall),
    }
});

impl Parse for Name {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = Self::Base(input.parse()?);
        Ok(loop {
            name = if let Some(args) = input.try_parse()? {
                Self::FunctionCall(FunctionCall {
                    name: name.into(),
                    args,
                })
            } else {
                break name;
            };
        })
    }
}

parse!({
    enum BaseName {
        Ident(Ident),
    }
});

impl Parse for BaseName {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self::Ident(input.parse()?))
    }
}

parse!({
    struct QualifiedExpr {
        mark: Name,
        tick: Token![tick],
        value: QualifiedExprValue,
    }
});

parse!({
    enum QualifiedExprValue {
        Expr(QualifiedExprValueExpr),
    }
});

impl Parse for QualifiedExprValue {
    fn parse(input: ParseStream) -> crate::Result<Self> {
        Ok(Self::Expr(input.parse()?))
    }
}

parse!({
    struct QualifiedExprValueExpr {
        expr: ParenthesizedOne<Box<Expr>>,
    }
});

impl Parse for QualifiedExprValueExpr {
    fn parse(input: ParseStream) -> crate::Result<Self> {
        Ok(Self {
            expr: input.parse()?,
        })
    }
}

parse!({
    struct FunctionCall {
        name: Box<Name>,
        args: Parenthesized<Expr>,
    }
});

parse!({
    struct ExprUnary {
        op: UnaryOp,
        expr: Box<Expr>,
    }
});

parse!({
    enum UnaryOp {
        Abs(Token![abs]),
        Not(Token![not]),
        Add(Token![+]),
        Sub(Token![-]),
    }
});

parse!({
    struct ExprBinary {
        lhs: Box<Expr>,
        op: BinaryOp,
        rhs: Box<Expr>,
    }
});

parse!({
    enum BinaryOp {
        Pow(Token![**]),
        Mul(Token![*]),
        Div(Token![/]),
        Mod(Token![mod]),
        Rem(Token![rem]),
        Add(Token![+]),
        Sub(Token![-]),
        Concat(Token![&]),
        Eq(Token![=]),
        Ne(Token![/=]),
        Lt(Token![<]),
        Le(Token![<=]),
        Gt(Token![>]),
        Ge(Token![>=]),
        And(Token![and]),
        Or(Token![or]),
        Xor(Token![xor]),
    }
});

parse!({
    struct ExprShortCircuit {
        lhs: Box<Expr>,
        op: ShortCircuitOp,
        rhs: Box<Expr>,
    }
});

mod helper {
    use super::*;
    parse!({
        enum LogicalOp {
            Binary(BinaryOp),
            ShortCircuit(ShortCircuitOp),
        }
    });
}

parse!({
    enum ShortCircuitOp {
        And(AndThen),
        Or(OrElse),
    }
});

parse!({
    struct AndThen {
        and: Token![and],
        then: Token![then],
    }
});

parse!({
    struct OrElse {
        or: Token![or],
        else_: Token![else],
    }
});
