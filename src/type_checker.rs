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
        typecheck_stmt(stmt, &mut env, None)?;
    }
    Ok(())
}

pub fn typecheck_stmt(
    stmt: &mut ast::Statement,
    env: &mut TypeEnv,
    ret_ty: Option<&Type>,
) -> Result<Type, String> {
    match stmt {
        ast::Statement::Let(let_stmt) => {
            if let ast::Expression::Function(func) = &let_stmt.value {
                env.insert(let_stmt.name.value.clone(), func.ty.clone());
            }
            typecheck_expr(&mut let_stmt.value, env, ret_ty)?;
            let var_type = let_stmt.value.ty().clone();
            env.insert(let_stmt.name.value.clone(), var_type.clone());
            Ok(var_type)
        }
        ast::Statement::Return(ret_stmt) => {
            typecheck_expr(&mut ret_stmt.return_value, env, ret_ty)?;
            let actual = ret_stmt.return_value.ty().clone();
            if let Some(expected) = ret_ty {
                expected.unify(&actual).map_err(|e| {
                    format!(
                        "(Line: {}) Return type mismatch: expected {:#?} got {:#?}, ({})",
                        ret_stmt.token.line, expected, actual, e
                    )
                })?;
            }
            Ok(actual)
        }
        ast::Statement::Expression(expr_stmt) => {
            typecheck_expr(&mut expr_stmt.expression, env, ret_ty)?;
            Ok(expr_stmt.expression.ty().clone())
        }
        ast::Statement::Block(block_stmt) => {
            let mut block_env = env.clone();
            let mut last_type = Type::Void;
            for stmt in block_stmt.statements.iter_mut() {
                last_type = typecheck_stmt(stmt, &mut block_env, ret_ty)?;
            }
            Ok(last_type)
        }
    }
}

pub fn typecheck_expr(
    expr: &mut ast::Expression,
    env: &mut TypeEnv,
    ret_ty: Option<&Type>,
) -> Result<(), String> {
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
                return Err(format!(
                    "(Line: {}) Undefined var: {}",
                    ident.token.line, ident.value
                ));
            }
        }
        ast::Expression::Prefix(pre) => {
            typecheck_expr(&mut pre.right, env, ret_ty)?;
            if pre.operator == "-" {
                if *pre.right.ty() != Type::Int {
                    return Err(format!(
                        "(Line: {}) Cannot negate non-integer: {:#?}",
                        pre.token.line,
                        pre.right.ty()
                    ));
                }
                pre.ty = Type::Int;
            } else if pre.operator == "!" {
                if *pre.right.ty() != Type::Bool {
                    return Err(format!(
                        "(Line: {}) Cannot negate non-boolean: {:#?}",
                        pre.token.line,
                        pre.right.ty()
                    ));
                }
                pre.ty = Type::Bool;
            } else {
                return Err(format!(
                    "(Line: {}) Invalid prefix operator: {}",
                    pre.token.line, pre.operator
                ));
            }
        }
        ast::Expression::Infix(inf) => {
            typecheck_expr(&mut inf.left, env, ret_ty)?;
            typecheck_expr(&mut inf.right, env, ret_ty)?;

            if inf.operator == "+"
                || inf.operator == "-"
                || inf.operator == "*"
                || inf.operator == "/"
            {
                if *inf.left.ty() != Type::Int || *inf.right.ty() != Type::Int {
                    return Err(format!(
                        "(Line: {}) Arithmetic operation requires both types to be int, got {:#?} and {:#?}",
                        inf.token.line,
                        inf.left.ty(),
                        inf.right.ty()
                    ));
                }

                inf.ty = Type::Int;
            } else if inf.operator == "<" || inf.operator == ">" {
                if *inf.left.ty() != Type::Int || *inf.right.ty() != Type::Int {
                    return Err(format!(
                        "(Line: {}) Comparison requires both types to be int, got {:#?} and {:#?}",
                        inf.token.line,
                        inf.left.ty(),
                        inf.right.ty()
                    ));
                }
                inf.ty = Type::Bool;
            } else if inf.operator == "==" || inf.operator == "!=" {
                inf.left.ty().unify(inf.right.ty())?;
                inf.ty = Type::Bool;
            } else {
                return Err(format!(
                    "(Line: {}) Invalid infix operator: {}",
                    inf.token.line, inf.operator
                ));
            }
        }

        ast::Expression::Call(c) => {
            typecheck_expr(&mut c.function, env, ret_ty)?;
            let func_type: Type;
            let func_name: String;
            match &c.function {
                ast::Expression::Ident(ident) => {
                    func_type = env
                        .get(&ident.value)
                        .ok_or_else(|| {
                            format!(
                                "(Line: {}) Undefined function: {}",
                                c.token.line,
                                ident.value.clone()
                            )
                        })?
                        .clone();
                    func_name = ident.value.clone();
                }
                _ => {
                    return Err(format!(
                        "(Line: {}) Function call can only be from identifier",
                        c.token.line
                    ));
                }
            }
            if let Type::Function {
                params,
                return_type,
            } = func_type
            {
                if c.arguments.len() != params.len() {
                    return Err(format!(
                        "(Line: {}) Function {} expects {} args, got {} args",
                        c.token.line,
                        func_name,
                        params.len(),
                        c.arguments.len()
                    ));
                }

                for (arg, param_type) in c.arguments.iter_mut().zip(params.iter()) {
                    typecheck_expr(arg, env, ret_ty)?;
                    param_type.unify(arg.ty())?;
                }
                c.ty = *return_type;
            } else {
                return Err(format!(
                    "(Line: {}) {} is not a function",
                    c.token.line, func_name
                ));
            }
        }

        ast::Expression::If(if_stmt) => {
            typecheck_expr(&mut if_stmt.condition, env, ret_ty)?;
            if *if_stmt.condition.ty() != Type::Bool {
                return Err(format!(
                    "(Line: {}) If condition must be boolean, got {:#?}",
                    if_stmt.token.line,
                    if_stmt.condition.ty()
                ));
            }

            let mut consequence_env = env.clone();
            let mut consequence_ty = Type::Void;

            for stmt in if_stmt.consequence.statements.iter_mut() {
                consequence_ty = typecheck_stmt(stmt, &mut consequence_env, ret_ty)?;
            }

            match if_stmt.alternative.as_mut() {
                Some(alt) => {
                    let mut alternative_env = env.clone();
                    let mut alternative_ty = Type::Void;

                    for stmt in alt.statements.iter_mut() {
                        alternative_ty = typecheck_stmt(stmt, &mut alternative_env, ret_ty)?;
                    }

                    if_stmt.ty = consequence_ty.unify(&alternative_ty)?;
                }
                None => if_stmt.ty = Type::Void,
            }
        }

        ast::Expression::Function(f) => {
            let mut local_env = env.clone();
            for ident in f.parameters.iter_mut() {
                local_env.insert(ident.value.clone(), ident.ty.clone());
            }
            let mut body_type = Type::Void;
            for stmt in f.body.statements.iter_mut() {
                body_type = typecheck_stmt(stmt, &mut local_env, Some(&f.return_type))?;
            }

            if f.return_type != Type::Unknown && body_type != Type::Void {
                f.return_type.unify(&body_type)?;
            }
        }

        ast::Expression::While(w) => {
            typecheck_expr(&mut w.condition, env, ret_ty)?;

            if *w.condition.ty() != Type::Bool {
                return Err(format!(
                    "(Line: {}) While condition must be boolean, got {:#?}",
                    w.token.line,
                    w.condition.ty()
                ));
            }
            let mut body_env = env.clone();
            for stmt in w.body.statements.iter_mut() {
                typecheck_stmt(stmt, &mut body_env, ret_ty)?;
            }
            w.ty = Type::Void;
        }

        ast::Expression::Array(a) => {
            let mut elem_ty = Type::Unknown;
            for el in a.elements.iter_mut() {
                typecheck_expr(el, env, ret_ty)?;
                elem_ty = elem_ty.unify(el.ty()).map_err(|_| {
                    format!(
                        "(Line: {}) Array elements must all have the same type, expected {:?}, got {:?}",
                        a.token.line,
                        elem_ty,
                        el.ty()
                    )
                })?;
            }
            a.ty = Type::Array(Box::new(elem_ty))
        }
        ast::Expression::Index(index) => {
            typecheck_expr(&mut index.left, env, ret_ty)?;

            let elem_ty = match index.left.ty() {
                Type::Array(elem) => (**elem).clone(),
                other => {
                    return Err(format!(
                        "(Line: {}) IndexExpression.left must be of type array, got={:#?}",
                        index.token.line, other
                    ));
                }
            };

            typecheck_expr(&mut index.index, env, ret_ty)?;
            if *index.index.ty() != Type::Int {
                return Err(format!(
                    "(Line: {}) IndexExpression.index must be of type int, got={:#?}",
                    index.token.line,
                    index.index.ty()
                ));
            }
            index.ty = elem_ty;
        }
        ast::Expression::Assign(assign) => {
            typecheck_expr(&mut assign.value, env, ret_ty)?;
            let ident_ty = env
                .get(&assign.target.value)
                .ok_or_else(|| format!("undefined variable"))?;
            let ty = ident_ty.unify(&assign.value.ty()).map_err(|e| {
                format!(
                    "(Line: {}) assignement mismatch, expected {:#?}, got {:#?}, ({})",
                    assign.token.line, assign.target, assign.value, e
                )
            })?;
            assign.ty = ty;
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
            let mut errlist: Vec<String> = vec![];
            for err in &p.errors() {
                errlist.push(err.clone());
            }
            return Err(String::from(errlist.join(", ")));
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
        } else {
            panic!("expected statement to be ExpressionStatement");
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
        } else {
            panic!("expected statement to be ExpressionStatement");
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
        } else {
            panic!("expected statement to be ExpressionStatement");
        }
    }

    #[test]
    fn test_early_return_type_mismatch_should_fail() {
        let program = typecheck_ast(
            "let f = fn(a: int) -> int {
                if (a > 0) {
                    return true;
                }
                return 1;
              }
              f(5);
        ",
        );
        assert!(
            program.is_err(),
            "expected a type error because the early `return true;` doesn't match the declared return type `int`, but got: {:?}",
            program
        );
    }

    #[test]
    fn test_undefined_variable_should_fail() {
        let program = typecheck_ast("x;");
        assert!(
            program.is_err(),
            "referencing an undefined variable should fail"
        );
    }

    #[test]
    fn test_undefined_function_call_should_fail() {
        let program = typecheck_ast("foo(1, 2);");
        assert!(
            program.is_err(),
            "calling an undefined function should fail"
        );
    }

    #[test]
    fn test_call_non_function_should_fail() {
        let program = typecheck_ast(
            "let x = 5;
             x(1, 2);",
        );
        assert!(program.is_err(), "calling a non-function value should fail");
    }

    #[test]
    fn test_wrong_argument_count_should_fail() {
        let program = typecheck_ast(
            "let add = fn(a: int, b: int) -> int { return a + b; };
             add(1);",
        );
        assert!(
            program.is_err(),
            "calling with too few arguments should fail"
        );
    }

    #[test]
    fn test_wrong_argument_type_should_fail() {
        let program = typecheck_ast(
            "let add = fn(a: int, b: int) -> int { return a + b; };
             add(1, true);",
        );
        assert!(
            program.is_err(),
            "calling with a mismatched argument type should fail"
        );
    }

    #[test]
    fn test_prefix_negate_non_integer_should_fail() {
        let program = typecheck_ast("-true;");
        assert!(program.is_err(), "negating a boolean with `-` should fail");
    }

    #[test]
    fn test_prefix_bang_non_boolean_should_fail() {
        let program = typecheck_ast("!5;");
        assert!(program.is_err(), "applying `!` to an integer should fail");
    }

    #[test]
    fn test_equality_type_mismatch_should_fail() {
        let program = typecheck_ast("5 == true;");
        assert!(
            program.is_err(),
            "comparing an int and a bool with `==` should fail"
        );
    }

    #[test]
    fn test_shadowing_inside_if_does_not_leak_outward() {
        let program = typecheck_ast(
            "let x = 5;
             if (x > 0) {
                 let x = true;
             }
             x + 1;",
        )
        .unwrap();
        if let ast::Statement::Expression(e) = &program.statements[2] {
            assert_eq!(*e.expression.ty(), Type::Int);
        } else {
            panic!("expected last statement to be an expression statement");
        }
    }

    #[test]
    fn test_nested_local_function_recursion() {
        let program = typecheck_ast(
            "let outer = fn(n: int) -> int {
                 let helper = fn(m: int) -> int {
                     if (m < 0) {
                         return 0;
                     }
                     return helper(m - 1);
                 };
                 return helper(n);
             };
             outer(3);",
        );
        assert!(
            program.is_ok(),
            "expected nested local recursive function to type-check, got: {:?}",
            program
        );
    }

    #[test]
    fn test_correct_return_type_with_early_return_passes() {
        let program = typecheck_ast(
            "let f = fn(a: int) -> int {
                 if (a > 0) {
                     return 1;
                 }
                 return 0;
             };
             f(5);",
        );
        assert!(
            program.is_ok(),
            "expected correctly typed early return to pass: {:?}",
            program
        );
    }

    #[test]
    fn test_function_passed_wrong_arity_nested_call() {
        let program = typecheck_ast(
            "let add = fn(a: int, b: int) -> int { return a + b; };
             let use_add = fn() -> int { return add(1, 2, 3); };
             use_add();",
        );
        assert!(program.is_err(), "nested call with wrong arity should fail");
    }

    #[test]
    fn test_array_typecheck() {
        let program = typecheck_ast("let arr = [1, 2*2, 3+3]");
        assert!(program.is_ok(), "expected regular array definition to pass");
    }

    #[test]
    fn test_array_typecheck_ty() {
        let program = typecheck_ast("let arr = [1, 2*2, 3+3]").unwrap();
        if let ast::Statement::Let(l) = &program.statements[0] {
            if let ast::Expression::Array(a) = &l.value {
                assert_eq!(a.ty, Type::Array(Box::new(Type::Int)));
            }
        } else {
            panic!("statement not let, got={:#?}", &program.statements[0]);
        }
    }

    #[test]
    fn test_array_typecheck_one_element() {
        let program = typecheck_ast("let arr = [1]");
        assert!(
            program.is_ok(),
            "expected regular array definition with 1 element to pass"
        );
    }

    #[test]
    fn test_array_typecheck_zero_elements() {
        let program = typecheck_ast("let arr = []");
        assert!(
            program.is_ok(),
            "expected regular array definition with 0 elements to pass"
        );
    }

    #[test]
    fn test_array_should_fail() {
        let program = typecheck_ast("let arr = [1, 2*2, false]");
        assert!(program.is_err(), "array with different types should fail");
    }

    #[test]
    fn test_while_condition_must_be_bool() {
        let program = typecheck_ast("while (1) { 2; }");
        assert!(program.is_err(), "non-boolean while condition should fail");
    }

    #[test]
    fn test_while_valid_passes() {
        let program = typecheck_ast("while (1 < 2) { 3; }");
        assert!(
            program.is_ok(),
            "valid while loop should pass: {:?}",
            program
        );
    }

    #[test]
    fn test_while_body_return_checked_against_function_return_type() {
        let program = typecheck_ast(
            "let f = fn() -> int {
                 while (true) {
                     return true;
                 }
                 return 1;
             };",
        );
        assert!(
            program.is_err(),
            "return inside while body must match function return type"
        );
    }

    #[test]
    fn test_while_body_scope_does_not_leak() {
        let program = typecheck_ast(
            "while (true) { let inner = 1; }
             inner;",
        );
        assert!(
            program.is_err(),
            "variable declared in while body should not be visible outside"
        );
    }

    #[test]
    fn test_index_expression_undefined() {
        let program = typecheck_ast("let a = arr[3];");
        assert!(program.is_err(), "arr is not defined");
    }

    #[test]
    fn test_index_expression_int() {
        let program = typecheck_ast(
            "let arr = [1, 2, 3, 4];
            let a = arr[3];
            ",
        );
        assert!(program.is_ok(), "arr is defined");
    }

    #[test]
    fn test_index_expression_bool() {
        let program = typecheck_ast(
            "let arr = [true, false, true, false];
            let a = arr[3];
            ",
        );
        assert!(program.is_ok(), "arr is defined");
    }

    #[test]
    fn test_wrong_argument_type_should_fail_arrays() {
        let program = typecheck_ast(
            "let add = fn(a: [int], b: int) -> int { return a + b; };
             add(1, true);",
        );
        assert!(
            program.is_err(),
            "calling with a mismatched argument type should fail"
        );
    }

    #[test]
    fn test_array_in_function_signature() {
        let program = typecheck_ast(
            "let f = fn(a: [int]) -> [int] {
                return a;
            }",
        );
        assert!(program.is_ok(), "receiving and returing an array");
    }
}
