use std::collections::HashMap;

use crate::{
    ast::{self, Expression},
    types::Type,
};

type TypeEnv = HashMap<String, Type>;

pub fn typecheck(program: &mut ast::Program) -> Result<(), String> {
    let mut env = TypeEnv::new();

    for stmt in &program.statements {
        if let ast::Statement::Let(let_stmt) = stmt {
            if let ast::Expression::Function(func) = let_stmt.value.clone() {
                env.insert(let_stmt.name.value.clone(), func.ty);
            }
        }
    }

    for stmt in program.statements.iter_mut() {
        typecheck_stmt(stmt, &mut env)?;
    }
    Ok(())
}

pub fn typecheck_stmt(stmt: &mut ast::Statement, env: &mut TypeEnv) -> Result<Type, String> {
    match stmt {
        ast::Statement::Let(let_stmt) => {
            typecheck_expr(&mut let_stmt.value, env)?;
            let var_type = let_stmt.value.ty().clone();
            env.insert(let_stmt.name.value.clone(), var_type.clone());
            Ok(var_type)
        }
        _ => todo!("cover rest of arms"),
    }
}

pub fn typecheck_expr(expr: &mut ast::Expression, env: &TypeEnv) -> Result<(), String> {
    todo!("work in progress")
}
