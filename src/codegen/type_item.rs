use crate::{
    parser::{EnumTypeDefinition, FullTypeItem, Result, TypeDefinition, TypeItem},
    tokenizer::Ident,
};

use super::{
    standard, CCode, CodeGenStmt, Context, EnumType, FunctionType, FunctionValue, Type, TypeValue,
    Value,
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
                    Type::Function(FunctionType {
                        args: vec![],
                        return_type: ty.clone(),
                    })
                    .into(),
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
