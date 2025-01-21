use crate::{
    codegen::IdentBuilder,
    parser::{
        AssignStmt, BlockStmt, CaseStmt, DiscreteChoice, ExitStmt, ExprStmt, GotoStmt, IfStmt,
        LabelStmt, LoopScheme, LoopStmt, ReturnStmt, Stmt,
    },
    tokenizer::Spanned,
    Result,
};

use super::{
    CCode, CodeGenExpr, CodeGenStmt, Context, ExprValue, LabelValue, Permission, SingleExprValue,
    Type, Value, VariableValue,
};

impl CodeGenStmt for Stmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        match self {
            Self::Label(stmt) => stmt.generate(ctx),
            Self::Expr(stmt) => stmt.generate(ctx),
            Self::Assign(stmt) => stmt.generate(ctx),
            Self::Return(stmt) => stmt.generate(ctx),
            Self::If(stmt) => stmt.generate(ctx),
            Self::Block(stmt) => stmt.generate(ctx),
            Self::Goto(stmt) => stmt.generate(ctx),
            Self::Loop(stmt) => stmt.generate(ctx),
            Self::Exit(stmt) => stmt.generate(ctx),
            Self::Case(stmt) => stmt.generate(ctx),
        }
    }
}

impl CodeGenStmt for LabelStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let value = ctx.get_or_insert(&self.label, || {
            let ident = IdentBuilder::label(&self.label);
            Value::Label(LabelValue {
                name: c_code! { #ident },
            })
        });
        let Value::Label(LabelValue { name }) = value else {
            return Err(self.label.unrecoverable_error("expected label name"));
        };
        Ok(c_code! {
            #name:
        })
    }
}

impl CodeGenStmt for ExprStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let expr = self.expr.generate_with_type_and_check(&Type::void(), ctx)?;
        Ok(c_code! {
            #expr;
        })
    }
}

impl CodeGenStmt for AssignStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let expr = self.name.generate(ctx)?.flat_map(|name| {
            if !name.perm.can_write() {
                return Err(self
                    .name
                    .unrecoverable_error("not allowed as an assign destination"));
            }

            let expr = self.expr.generate_with_type_and_check(&name.ty, ctx)?;
            Ok(SingleExprValue {
                ty: Type::void(),
                perm: Permission::Read,
                code: c_code! {
                    #name = #expr;
                },
                value: None,
            }
            .into())
        })?;
        let ExprValue::Distinct(expr) = expr else {
            return Err(self.expr.unrecoverable_error("ambiguous assignment"));
        };

        Ok(expr.code)
    }
}

impl CodeGenStmt for ReturnStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let Some(return_type) = ctx.return_type() else {
            return Err(self
                .return_
                .unrecoverable_error("return not allowed in this context"));
        };
        let expr = self.expr.generate_with_type_and_check(&return_type, ctx)?;
        Ok(c_code! {
            return #expr;
        })
    }
}

impl CodeGenStmt for IfStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let boolean = Type::boolean(ctx)?;
        let cond = self.cond.generate_with_type_and_check(&boolean, ctx)?;
        let stmts = self
            .stmts
            .iter()
            .map(|stmt| stmt.generate(ctx))
            .collect::<Result<Vec<_>>>()?;
        let else_ifs = self
            .elsifs
            .iter()
            .map(|e| {
                let cond = e.cond.generate_with_type_and_check(&boolean, ctx)?;
                let stmts = e
                    .stmts
                    .iter()
                    .map(|stmt| stmt.generate(ctx))
                    .collect::<Result<Vec<_>>>()?;
                Ok(c_code! {
                    else if (#cond) {
                        #(#stmts)*
                    }
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let else_ = self
            .else_
            .as_ref()
            .map(|e| {
                let stmts = e
                    .stmts
                    .iter()
                    .map(|stmt| stmt.generate(ctx))
                    .collect::<Result<Vec<_>>>()?;
                Ok(c_code! {
                    else {
                        #(#stmts)*
                    }
                })
            })
            .transpose()?;
        Ok(c_code! {
            if (#cond) {
                #(#stmts)*
            }
            #(#else_ifs)*
            #else_
        })
    }
}

impl CodeGenStmt for BlockStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let mut sub_ctx = ctx.subscope(ctx.return_type());

        let items = self
            .items()
            .map(|item| item.generate(&mut sub_ctx))
            .collect::<Result<Vec<_>>>()?;

        let stmts = self
            .stmts
            .iter()
            .map(|stmt| stmt.generate(&mut sub_ctx))
            .collect::<Result<Vec<_>>>()?;

        Ok(c_code! {
            {
                #(#items)*
                #(#stmts)*
            }
        })
    }
}

impl CodeGenStmt for GotoStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let value = ctx.get_or_insert(&self.label, || {
            let ident = IdentBuilder::label(&self.label);
            Value::Label(LabelValue {
                name: c_code! { #ident },
            })
        });
        let Value::Label(LabelValue { name }) = value else {
            return Err(self.label.unrecoverable_error("expected label name"));
        };
        Ok(c_code! {
            goto #name;
        })
    }
}

impl CodeGenStmt for LoopStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let mut sub_ctx = ctx.subscope(ctx.return_type());

        let stmts = |ctx: &mut Context| {
            let stmts = self
                .stmts
                .iter()
                .map(|stmt| stmt.generate(ctx))
                .collect::<Result<Vec<_>>>()?;

            let stmts = c_code! {
                {
                    #(#stmts)*
                }
            };

            Ok(stmts)
        };

        let code = match &self.scheme {
            LoopScheme::Endless(_) => {
                let stmts = stmts(&mut sub_ctx)?;
                c_code! { while (1) #stmts }
            }
            LoopScheme::While(scheme) => {
                let cond = scheme.cond.generate_to_boolean(&mut sub_ctx)?;
                let stmts = stmts(&mut sub_ctx)?;
                c_code! { while (#cond) #stmts }
            }
            LoopScheme::For(scheme) => {
                let code = scheme
                    .range
                    .start
                    .generate(&mut sub_ctx)?
                    .flat_map(|start| {
                        let ty = &start.ty;

                        let end = scheme
                            .range
                            .end
                            .generate_with_type_and_check(ty, &mut sub_ctx)?;
                        let ident = IdentBuilder::variable(&scheme.ident);

                        sub_ctx.insert(
                            &scheme.ident,
                            Value::Variable(VariableValue {
                                name: c_code! { #ident },
                                ty: ty.clone(),
                                perm: Permission::Read,
                            }),
                        )?;

                        let stmts = stmts(&mut sub_ctx)?;

                        if scheme.reverse() {
                            Ok(SingleExprValue {
                                ty: Type::void(),
                                perm: Permission::Read,
                                code: c_code! {
                                    {
                                        #ty #ident = #end;
                                        if (#start <= #end) {
                                            while (1) {
                                                #stmts
                                                if (#ident <= #start) {
                                                    break;
                                                }
                                                #ident -= 1;
                                            }
                                        }
                                    }
                                },
                                value: None,
                            }
                            .into())
                        } else {
                            Ok(SingleExprValue {
                                ty: Type::void(),
                                perm: Permission::Read,
                                code: c_code! {
                                    {
                                        #ty #ident = #start;
                                        if (#start <= #end) {
                                            while(1) {
                                                #stmts
                                                if (#ident >= #end) {
                                                    break;
                                                }
                                                #ident += 1;
                                            }
                                        }
                                    }
                                },
                                value: None,
                            }
                            .into())
                        }
                    })?;
                let ExprValue::Distinct(code) = code else {
                    return Err(scheme
                        .range
                        .unrecoverable_error("ambiguous range expression"));
                };
                code.code
            }
        };

        Ok(code)
    }
}

impl CodeGenStmt for ExitStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        if let Some(name) = &self.name {
            return Err(name.unrecoverable_error("exit to named loop not yet implemented"));
        }

        Ok(if let Some(cond) = self.cond() {
            let cond = cond.generate_to_boolean(ctx)?;
            c_code! {
                if (#cond) {
                    break;
                }
            }
        } else {
            c_code! {
                break;
            }
        })
    }
}

impl CodeGenStmt for CaseStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let ExprValue::Distinct(expr) = self.expr.generate(ctx)? else {
            return Err(self.expr.unrecoverable_error("ambiguous case expression"));
        };
        let ty = expr.ty;
        let expr = expr.code;
        let alternatives = self
            .alternatives
            .iter()
            .map(|alt| {
                let choices = alt
                    .choices
                    .iter()
                    .map(|choice| choice.generate(&ty, ctx))
                    .collect::<Result<Vec<_>>>()?;

                let stmts = alt
                    .stmts
                    .iter()
                    .map(|stmt| stmt.generate(ctx))
                    .collect::<Result<Vec<_>>>()?;

                Ok(c_code! {
                    if (#(#choices)||*) {
                        #(#stmts)*
                    } else
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(c_code! {
            {
                #ty case_expr = #expr;
                #(#alternatives)* {}
            }
        })
    }
}

impl DiscreteChoice {
    fn generate(&self, ty: &Type, ctx: &mut Context) -> Result<CCode> {
        Ok(match self {
            Self::Others(_) => c_code! { 1 },
            Self::Expr(expr) => {
                let expr = expr.generate_with_type_and_check(ty, ctx)?;
                c_code! { case_expr == #expr }
            }
            Self::Range(range) => {
                let start = range.start.generate_with_type_and_check(ty, ctx)?;
                let end = range.end.generate_with_type_and_check(ty, ctx)?;
                c_code! { (case_expr >= #start && case_expr <= #end) }
            }
        })
    }
}
