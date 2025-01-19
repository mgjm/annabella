use crate::{
    parser::{
        Constraint, EnumTypeDefinition, Expr, FullTypeItem, ModularTypeDefinition, Range,
        RangeConstraint, SignedTypeDefinition, SubtypeItem, TypeDefinition, TypeItem,
    },
    tokenizer::Ident,
    Result,
};

use super::{
    standard, CCode, CodeGenExpr, CodeGenStmt, CodeGenType, Context, EnumType, FunctionType,
    FunctionValue, IdentBuilder, SignedType, SubtypeType, Type, TypeValue, Value,
};

impl CodeGenStmt for TypeItem {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        match self {
            TypeItem::Full(item) => item.generate(ctx),
        }
    }
}

impl CodeGenStmt for FullTypeItem {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        self.definition.generate(&self.name, ctx)
    }
}

impl CodeGenType for TypeDefinition {
    fn generate(&self, name: &Ident, ctx: &mut Context) -> Result<CCode> {
        match self {
            Self::Enum(definition) => definition.generate(name, ctx),
            Self::Signed(definition) => definition.generate(name, ctx),
            Self::Modular(definition) => definition.generate(name, ctx),
        }
    }
}

impl CodeGenType for EnumTypeDefinition {
    fn generate(&self, name: &Ident, ctx: &mut Context) -> Result<CCode> {
        let ident = IdentBuilder::type_(name);

        ctx.push_type(c_code! {
            typedef int #ident;
        });

        let ty = Type::enum_(EnumType {
            name: name.clone(),
            ident: ident.clone(),
            values: self.values.iter().cloned().collect(),
        });

        for (i, value) in self.values.iter().enumerate() {
            let value_ident = IdentBuilder::enum_value(name, value);
            ctx.push_function(c_code! {
                #ident #value_ident() {
                    return #i;
                }
            });
            ctx.insert(
                value,
                Value::Function(FunctionValue::new(
                    c_code! { #value_ident},
                    Type::function(FunctionType {
                        args: vec![],
                        return_type: ty.clone(),
                    }),
                )),
            )?;
        }

        ctx.insert(name, Value::Type(TypeValue { ty: ty.clone() }))?;

        standard::generate_comparison_ops(&ty, ctx)?;

        let values_str = self.values.iter().map(|v| &*v.name);
        standard::generate_custom_print(
            ty,
            c_code! {
                static const char *const values[] = {
                    #(#values_str,)*
                };
                printf("%s\n", values[self]);
            },
            ctx,
        )?;

        Ok(c_code!())
    }
}

impl CodeGenType for SignedTypeDefinition {
    fn generate(&self, name: &Ident, ctx: &mut Context) -> Result<CCode> {
        let ident = IdentBuilder::type_(name);

        ctx.push_type(c_code! {
            typedef ssize_t #ident;
        });

        let constraint_check = {
            let constraint = RangeConstraint {
                range_token: self.range_keyword,
                range: self.range.clone(),
            }
            .generate(&Type::integer(), ctx)?;
            let constraint_ident = IdentBuilder::constraint_check(name);
            ctx.push_function(c_code! {
                #ident #constraint_ident (#ident self) {
                    #constraint
                    return self;
                }
            });
            Some(c_code! { #constraint_ident})
        };

        let ty = Type::signed(SignedType {
            name: name.clone(),
            ident: ident.clone(),
            constraint_check,
        });

        ctx.insert(name, Value::Type(TypeValue { ty: ty.clone() }))?;

        standard::generate_signed_ops(&ty, ctx)?;
        standard::generate_print(ty, "%ld", ctx)?;

        Ok(c_code!())
    }
}

impl CodeGenType for ModularTypeDefinition {
    fn generate(&self, name: &Ident, ctx: &mut Context) -> Result<CCode> {
        let ident = IdentBuilder::type_(name);

        ctx.push_type(c_code! {
            typedef ssize_t #ident;
        });

        let constraint_check = {
            let constraint = RangeConstraint {
                range_token: Default::default(),
                range: Range {
                    start: Expr::number(0),
                    dot_dot: Default::default(),
                    end: self.modulus.clone(),
                },
            }
            .generate(&Type::integer(), ctx)?;
            let constraint_ident = IdentBuilder::constraint_check(name);
            ctx.push_function(c_code! {
                #ident #constraint_ident (#ident self) {
                    #constraint
                    return self;
                }
            });
            Some(c_code! { #constraint_ident})
        };

        let ty = Type::signed(SignedType {
            name: name.clone(),
            ident: ident.clone(),
            constraint_check,
        });

        ctx.insert(name, Value::Type(TypeValue { ty: ty.clone() }))?;

        let modulus = self
            .modulus
            .generate_with_type_and_check(&Type::integer(), ctx)?;

        standard::generate_modular_ops(&ty, &modulus, ctx)?;
        standard::generate_print(ty, "%ld", ctx)?;

        Ok(c_code!())
    }
}

impl CodeGenStmt for SubtypeItem {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let name = &self.name;
        let parent = Type::from_ident(&self.mark, ctx)?;

        let constraint = self
            .constraint
            .as_ref()
            .map(|constraint| constraint.generate(&parent, ctx))
            .transpose()?;

        let constraint_check = if let Some(constraint) = constraint {
            let ident = IdentBuilder::constraint_check(name);
            ctx.push_function(c_code! {
                #parent #ident(#parent self) {
                    #constraint
                    return self;
                }
            });
            Some(c_code! { #ident })
        } else {
            None
        };

        let ty = Type::subtype(SubtypeType {
            parent,
            constraint_check,
        });
        ctx.insert(name, Value::Type(TypeValue { ty }))?;

        Ok(c_code!())
    }
}

impl Constraint {
    fn generate(&self, ty: &Type, ctx: &mut Context) -> Result<CCode> {
        match self {
            Self::Range(constraint) => constraint.generate(ty, ctx),
        }
    }
}

impl RangeConstraint {
    fn generate(&self, ty: &Type, ctx: &mut Context) -> Result<CCode> {
        let start = self.range.start.generate_with_type_and_check(ty, ctx)?;
        let end = self.range.end.generate_with_type_and_check(ty, ctx)?;
        Ok(c_code! {
            if (self < #start || #end < self) {
                throw_Constraint_Error();
            }
        })
    }
}
