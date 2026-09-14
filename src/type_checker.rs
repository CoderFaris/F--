use std::collections::HashMap;

use crate::{
    ast::{self, BlockStatement, Expression},
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
        ast::Statement::Return(ret_stmt) => {
            typecheck_expr(&mut ret_stmt.return_value, env)?;
            Ok(ret_stmt.return_value.ty().clone())
        }
        ast::Statement::Expression(expr_stmt) => {
            typecheck_expr(&mut expr_stmt.expression, env)?;
            Ok(expr_stmt.expression.ty().clone())
        }
        ast::Statement::Block(block_stmt) => {
            let mut block_env = env.clone();
            let mut last_type = Type::Unknown; // Type::Unit was here
            for stmt in block_stmt.statements.iter_mut() {
                last_type = typecheck_stmt(stmt, &mut block_env)?;
            }
            Ok(last_type)
        }
    }
}

pub fn typecheck_expr(expr: &mut ast::Expression, env: &mut TypeEnv) -> Result<(), String> {
    match expr {
        ast::Expression::Int(int) => {
            int.ty = Type::Int;
        }
        ast::Expression::Bool(bool) => {
            bool.ty = Type::Bool;
        }
        ast::Expression::Ident(ident) => {
            if let Some(ty) = env.get(&ident.value) {
                ident.ty = ty.clone();
            } else {
                return Err(format!("Undefined var: {}", ident.value));
            }
        }
        ast::Expression::Prefix(pre) => {
            typecheck_expr(&mut pre.right, env)?;
            if pre.operator == "-" {
                if *pre.right.ty() != Type::Int {
                    return Err(format!("Cannot negate non-integer: {:#?}", pre.right.ty()));
                }
                pre.ty = Type::Int;
            } else if pre.operator == "!" {
                if *pre.right.ty() != Type::Bool {
                    return Err(format!("Cannot negate non-boolean: {:#?}", pre.right.ty()));
                }
                pre.ty = Type::Bool;
            } else {
                return Err(format!("Invalid prefix operator: {}", pre.operator));
            }
        }
        ast::Expression::Infix(inf) => {
            typecheck_expr(&mut inf.left, env)?;
            typecheck_expr(&mut inf.right, env)?;

            if inf.operator == "+"
                || inf.operator == "-"
                || inf.operator == "*"
                || inf.operator == "/"
            {
                if *inf.left.ty() != Type::Int || *inf.right.ty() != Type::Int {
                    return Err(format!(
                        "Arithmetic operation requires both types to be int, got {:#?} and {:#?}",
                        inf.left.ty(),
                        inf.right.ty()
                    ));
                }

                inf.ty = Type::Int;
            } else if inf.operator == "<" || inf.operator == ">" {
                if *inf.left.ty() != Type::Int || *inf.right.ty() != Type::Int {
                    return Err(format!(
                        "Comparison requires both types to be int, got {:#?} and {:#?}",
                        inf.left.ty(),
                        inf.right.ty()
                    ));
                }
                inf.ty = Type::Bool;
            } else if inf.operator == "==" || inf.operator == "!=" {
                let _ = inf.left.ty().unify(inf.right.ty());
                inf.ty = Type::Bool;
            } else {
                return Err(format!("Invalid infix operator: {}", inf.operator));
            }
        }

        ast::Expression::Call(c) => {
            typecheck_expr(&mut c.function, env)?;
            let func_type: Type;
            let func_name: String;
            match &c.function {
                ast::Expression::Ident(ident) => {
                    func_type = env
                        .get(&ident.value)
                        .ok_or_else(|| format!("Undefined function: {}", ident.value.clone()))?
                        .clone();
                    func_name = ident.value.clone();
                }
                _ => return Err(format!("Function call can only be from identifier")), // not sure whether I can do this
            }
            if let Type::Function {
                params,
                return_type,
            } = func_type
            {
                if c.arguments.len() != params.len() {
                    return Err(format!(
                        "Function {} expects {} args, got {} args",
                        func_name,
                        params.len(),
                        c.arguments.len()
                    ));
                }

                for (arg, param_type) in c.arguments.iter_mut().zip(params.iter()) {
                    typecheck_expr(arg, env)?;
                    let _ = param_type.unify(arg.ty());
                }
                c.ty = *return_type;
            } else {
                return Err(format!("{} is not a function", func_name));
            }
        }

        ast::Expression::If(if_stmt) => {
            typecheck_expr(&mut if_stmt.condition, env)?;
            if *if_stmt.condition.ty() != Type::Bool {
                return Err(format!(
                    "If condition must be boolean, got {:#?}",
                    if_stmt.condition.ty()
                ));
            }

            let mut consequence_env = env.clone();
            let mut consequence_ty = Type::Unknown; // Unit?

            for stmt in if_stmt.consequence.statements.iter_mut() {
                consequence_ty = typecheck_stmt(stmt, &mut consequence_env)?;
            }

            let mut alternative_env = env.clone();
            let mut alternative_ty = Type::Unknown;

            let mut alternative_stmts: BlockStatement;
            let alternative = if_stmt.alternative.clone();
            if alternative.is_some() {
                alternative_stmts = alternative.unwrap();

                for stmt in alternative_stmts.statements.iter_mut() {
                    alternative_ty = typecheck_stmt(stmt, &mut alternative_env)?
                }
            }

            // unify both branches
            if_stmt.ty = consequence_ty.unify(&alternative_ty)?;
        }

        ast::Expression::Function(f) => {
            let mut local_env = env.clone();
            for ident in f.parameters.iter_mut() {
                local_env.insert(ident.value.clone(), ident.ty.clone());
            }
            let mut body_type = Type::Unknown; // unit?
            for stmt in f.body.statements.iter_mut() {
                body_type = typecheck_stmt(stmt, &mut local_env)?;
            }

            if f.return_type != Type::Unknown && body_type != Type::Unknown {
                let _ = f.return_type.unify(&body_type)?;
            }
        }
        _ => panic!("unknown expression in typecheck_expr"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    fn typecheck_ast(code: &str) -> Result<ast::Program, String> {
        let mut l = Lexer::new(code);
        let mut p = Parser::new(l);
        let mut program = p.parse();
        if p.errors().len() > 0 {
            return Err(String::from("Parser errors inside code"));
        }
        typecheck(&mut program)?;
        Ok(program)
    }

    #[test]
    fn test_literal_typecheck() {
        let program = typecheck_ast("3;").unwrap();
        if let ast::Statement::Expression(e) = &program.statements[0] {
            assert_eq!(*e.expression.ty(), Type::Int);
        }
    }

    #[test]
    fn test_arithmetic_typecheck() {
        let program = typecheck_ast("3 * 3 - 3").unwrap();
        if let ast::Statement::Expression(e) = &program.statements[0] {
            assert_eq!(*e.expression.ty(), Type::Int);
        }
    }

    #[test]
    fn test_comparison_typecheck() {
        let program = typecheck_ast("3 > 2").unwrap();
        if let ast::Statement::Expression(e) = &program.statements[0] {
            assert_eq!(*e.expression.ty(), Type::Bool);
        }
    }

    #[test]
    fn test_function_typecheck() {
        let program = typecheck_ast(
            "   let add = fn(a: int, b: int) -> int {
                            return a + b;    
                      }
                      add(9, 10);       
                ",
        )
        .unwrap();
        if let ast::Statement::Expression(e) = &program.statements[1] {
            assert_eq!(*e.expression.ty(), Type::Int);
        }
    }

    #[test]
    fn test_error_typecheck() {
        let program = typecheck_ast("1 + true;");
        assert!(program.is_err());
    }

    #[test]
    fn test_fibonacci_typecheck() {
        let program = typecheck_ast(
            "let fib = fn(n: int) -> int {
                        if (n < 2) {
                            return n;
                        } else {
                            return fib(n-1) + fib(n-2);
                        } 

                   }
                   fib(10)",
        )
        .unwrap();
        if let ast::Statement::Expression(e) = &program.statements[1] {
            assert_eq!(*e.expression.ty(), Type::Int);
        }
    }
}
