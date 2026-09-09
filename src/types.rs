#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Int,
    Bool,
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Unknown,
}

impl Type {
    pub fn unify(&self, other: &Type) -> Result<Type, String> {
        match (self, other) {
            (Type::Unknown, t) | (t, Type::Unknown) => Ok(t.clone()),
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Bool, Type::Bool) => Ok(Type::Bool),
            (
                Type::Function {
                    params: p1,
                    return_type: r1,
                },
                Type::Function {
                    params: p2,
                    return_type: r2,
                },
            ) => {
                if p1.len() != p2.len() {
                    return Err(format!("Type mismatch (len of params)"));
                }
                let mut unified_params = Vec::with_capacity(p1.len());
                for (a, b) in p1.iter().zip(p2.iter()) {
                    unified_params.push(a.unify(b)?);
                }
                let unified_return = r1.unify(r2)?;
                Ok(Type::Function {
                    params: unified_params,
                    return_type: Box::new(unified_return),
                })
            }
            _ => Err(format!(
                "Type mismatch, expected {:#?} got {:#?}",
                self, other
            )),
        }
    }
}
