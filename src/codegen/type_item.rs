use crate::{
    parser::{
        Constraint, EnumTypeDefinition, FullTypeItem, RangeConstraint, SubtypeItem, TypeDefinition,
        TypeItem,
    },
    tokenizer::Ident,
    Result,
};

use super::{
    standard, CCode, CodeGenExpr, CodeGenStmt, Context, EnumType, FunctionType, FunctionValue,
    SubtypeType, Type, TypeValue, Value,
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

trait CodeGenType {
    fn generate(&self, name: &Ident, ctx: &mut Context) -> Result<CCode>;
}

impl CodeGenType for TypeDefinition {
    fn generate(&self, name: &Ident, ctx: &mut Context) -> Result<CCode> {
        match self {
            TypeDefinition::Enum(definition) => definition.generate(name, ctx),
        }
    }
}

impl CodeGenType for EnumTypeDefinition {
    fn generate(&self, name: &Ident, ctx: &mut Context) -> Result<CCode> {
        ctx.push_type(c_code! {
            typedef int #name;
        });

        let ty = Type::enum_(EnumType {
            name: name.clone(),
            values: self.values.iter().cloned().collect(),
        });

        for (i, value) in self.values.iter().enumerate() {
            let ident = quote::format_ident!("annabella__{name}__{value}");
            ctx.push_function(c_code! {
                #name #ident() {
                    return #i;
                }
            });
            ctx.insert(
                value,
                Value::Function(FunctionValue::new(
                    c_code! { #ident },
                    Type::function(FunctionType {
                        args: vec![],
                        return_type: ty.clone(),
                    }),
                )),
            )?;
        }

        ctx.insert(
            name,
            Value::Type(TypeValue {
                name: c_code! { #name },
                ty: ty.clone(),
            }),
        )?;

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

impl CodeGenStmt for SubtypeItem {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let name = &self.name;
        let parent = Type::parse_ident(&self.mark, ctx)?;

        let constraint = self
            .constraint
            .as_ref()
            .map(|constraint| constraint.generate(&parent, ctx))
            .transpose()?;

        let constraint_check = if let Some(constraint) = constraint {
            let ident = quote::format_ident!("annabella_constraint__{name}");

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
        let c_name = quote::format_ident!("{}", ty.to_str());
        ctx.insert(
            name,
            Value::Type(TypeValue {
                name: c_code! { #c_name},
                ty,
            }),
        )?;

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
