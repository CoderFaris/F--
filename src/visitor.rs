use crate::{
    ast::{
        self, ArrayLiteral, AssignExpression, BlockStatement, Boolean, CallExpression,
        ExpressionStatement, FunctionLiteral, Identifier, IfExpression, IndexExpression,
        InfixExpression, IntegerLiteral, LetStatement, PrefixExpression, ReturnStatement,
        WhileExpression,
    },
    token, types,
};

pub trait ExprVisitor {
    // expression related
    fn visit_expr(&mut self, expr: &ast::Expression) -> ast::Expression {
        match expr {
            ast::Expression::Int(i) => self.visit_int(i.clone()),
            ast::Expression::Bool(b) => self.visit_bool(b.clone()),
            ast::Expression::Ident(ident) => self.visit_identifier(ident.clone()),
            ast::Expression::Prefix(pre) => self.visit_prefix(*pre.clone()),
            ast::Expression::Infix(inf) => self.visit_infix(*inf.clone()),
            ast::Expression::If(ifexpr) => self.visit_if(*ifexpr.clone()),
            ast::Expression::Call(c) => self.visit_call(*c.clone()),
            ast::Expression::Function(f) => self.visit_function(f.clone()),
            ast::Expression::While(w) => self.visit_while(*w.clone()),
            ast::Expression::Array(a) => self.visit_array(a.clone()),
            ast::Expression::Index(index) => self.visit_index_expr(*index.clone()),
            ast::Expression::Assign(assign) => self.visit_assign(*assign.clone()),
            _ => panic!("this will never execute :)"),
        }
    }

    fn visit_int(&mut self, n: IntegerLiteral) -> ast::Expression {
        ast::Expression::Int(n)
    }

    fn visit_bool(&mut self, b: Boolean) -> ast::Expression {
        ast::Expression::Bool(b)
    }

    fn visit_identifier(&mut self, ident: Identifier) -> ast::Expression {
        ast::Expression::Ident(ident)
    }

    fn visit_prefix(&mut self, mut pre: PrefixExpression) -> ast::Expression {
        let visited_expr = self.visit_expr(&pre.right);
        pre.right = visited_expr;
        ast::Expression::Prefix(Box::new(pre))
    }

    fn visit_infix(&mut self, mut inf: InfixExpression) -> ast::Expression {
        let visited_left = self.visit_expr(&inf.left);
        let visited_right = self.visit_expr(&inf.right);
        inf.left = visited_left;
        inf.right = visited_right;
        ast::Expression::Infix(Box::new(inf))
    }

    fn visit_if(&mut self, mut ifexpr: IfExpression) -> ast::Expression {
        let visited_expr = self.visit_expr(&ifexpr.condition);
        let visited_consequence = self.visit_block(ifexpr.consequence.clone());

        // only if the alternative contains statements we fill vector out
        let mut visited_alternative: Option<ast::Statement> = None;
        if ifexpr.alternative.is_some() {
            let alternative = ifexpr.alternative.clone().unwrap();
            visited_alternative = Some(self.visit_block(alternative));
        }

        ifexpr.condition = visited_expr;
        if let ast::Statement::Block(b) = visited_consequence {
            ifexpr.consequence = b;
        }

        if visited_alternative.is_some() {
            if let ast::Statement::Block(b) = visited_alternative.clone().unwrap() {
                ifexpr.alternative = Some(b);
            }
        }

        ast::Expression::If(Box::new(ifexpr))
    }

    fn visit_while(&mut self, mut w: WhileExpression) -> ast::Expression {
        let visited_expr = self.visit_expr(&w.condition);
        let visited_body = self.visit_block(w.body.clone());
        w.condition = visited_expr;

        if let ast::Statement::Block(b) = visited_body {
            w.body = b;
        }

        ast::Expression::While(Box::new(w))
    }

    fn visit_call(&mut self, mut c: CallExpression) -> ast::Expression {
        let visited_args: Vec<ast::Expression> =
            c.arguments.iter().map(|arg| self.visit_expr(arg)).collect();
        c.arguments = visited_args;
        ast::Expression::Call(Box::new(c))
    }

    fn visit_function(&mut self, mut f: FunctionLiteral) -> ast::Expression {
        let visited_stmts = self.visit_block(f.body.clone());
        if let ast::Statement::Block(b) = visited_stmts {
            f.body = b;
        }
        ast::Expression::Function(f)
    }

    fn visit_array(&mut self, mut a: ArrayLiteral) -> ast::Expression {
        let visited_elements = a.elements.iter().map(|e| self.visit_expr(e)).collect();
        a.elements = visited_elements;
        ast::Expression::Array(a)
    }

    fn visit_index_expr(&mut self, mut index: IndexExpression) -> ast::Expression {
        let visited_left = self.visit_expr(&index.left);
        let visited_index = self.visit_expr(&index.index);
        index.left = visited_left;
        index.index = visited_index;
        ast::Expression::Index(Box::new(index))
    }

    fn visit_assign(&mut self, mut assign: AssignExpression) -> ast::Expression {
        let visited_val = self.visit_expr(&assign.value);
        assign.value = visited_val;
        ast::Expression::Assign(Box::new(assign))
    }

    // statement related
    fn visit_stmt(&mut self, stmt: &ast::Statement) -> ast::Statement {
        match stmt {
            ast::Statement::Return(r) => self.visit_return(r.clone()),
            ast::Statement::Let(l) => self.visit_let(l.clone()),
            ast::Statement::Expression(e) => self.visit_expr_stmt(e.clone()),
            ast::Statement::Block(b) => self.visit_block(b.clone()),
        }
    }

    fn visit_return(&mut self, mut r: ReturnStatement) -> ast::Statement {
        let visited_expr = self.visit_expr(&r.return_value);
        r.return_value = visited_expr;
        ast::Statement::Return(r)
    }

    fn visit_let(&mut self, mut l: LetStatement) -> ast::Statement {
        let visited_expr = self.visit_expr(&l.value);
        l.value = visited_expr;
        ast::Statement::Let(l)
    }

    fn visit_expr_stmt(&mut self, mut e: ExpressionStatement) -> ast::Statement {
        let visited_expr = self.visit_expr(&e.expression);
        e.expression = visited_expr;
        ast::Statement::Expression(e)
    }

    fn visit_block(&mut self, mut b: BlockStatement) -> ast::Statement {
        let visited_stmts: Vec<ast::Statement> =
            b.statements.iter().map(|s| self.visit_stmt(s)).collect();
        b.statements = visited_stmts;
        ast::Statement::Block(b)
    }
}

pub struct ConstantFolder;

impl ConstantFolder {
    pub fn new() -> Self {
        ConstantFolder
    }

    pub fn fold(mut program: ast::Program) -> ast::Program {
        let mut folder = ConstantFolder::new();
        let visited_stmts = program
            .statements
            .iter()
            .map(|s| folder.visit_stmt(s))
            .collect();
        program.statements = visited_stmts;
        program
    }
}

impl Default for ConstantFolder {
    fn default() -> Self {
        Self::new()
    }
}

impl ExprVisitor for ConstantFolder {
    fn visit_infix(&mut self, mut inf: InfixExpression) -> ast::Expression {
        let l = self.visit_expr(&inf.left);
        let r = self.visit_expr(&inf.right);

        inf.left = l.clone();
        inf.right = r.clone();

        let mut folded_const = IntegerLiteral {
            token: token::Token {
                token_type: token::TokenType::Placeholder,
                literal: String::from("PLACEHOLDER"),
                line: 0,
            },
            value: 0,
            ty: types::Type::Int,
        };

        let mut folded_const_bool = Boolean {
            token: token::Token {
                token_type: token::TokenType::Placeholder,
                literal: String::from("PLACEHOLDER"),
                line: 0,
            },
            value: false,
            ty: types::Type::Bool,
        };

        if let (ast::Expression::Int(lv), ast::Expression::Int(rv)) = (&l, &r) {
            folded_const.token = lv.token.clone();
            folded_const.ty = lv.ty.clone();
            let result: Option<_>;
            // probably better to have BinaryOp enum with all valid operations
            if inf.operator == "+" {
                result = Some(lv.value + rv.value);
            } else if inf.operator == "-" {
                result = Some(lv.value - rv.value);
            } else if inf.operator == "*" {
                result = Some(lv.value * rv.value);
            } else if inf.operator == "/" {
                if rv.value != 0 {
                    result = Some(lv.value / rv.value);
                } else {
                    result = None; // division by zero will not be folded
                }
            } else {
                result = None;
            }

            if let Some(v) = result {
                folded_const.value = v;
                folded_const.token.literal = v.to_string();
                return ast::Expression::Int(folded_const);
            }
        }

        if let (ast::Expression::Int(lv), ast::Expression::Int(rv)) = (&l, &r) {
            folded_const_bool.token = lv.token.clone();

            let result: Option<_>;
            if inf.operator == ">" {
                result = Some(lv.value > rv.value);
            } else if inf.operator == "<" {
                result = Some(lv.value < rv.value);
            } else if inf.operator == "==" {
                result = Some(lv.value == rv.value);
            } else if inf.operator == "!=" {
                result = Some(lv.value != rv.value);
            } else {
                result = None;
            }

            if let Some(v) = result {
                folded_const_bool.value = v;
                folded_const_bool.token.literal = v.to_string();
                return ast::Expression::Bool(folded_const_bool);
            }
        }

        ast::Expression::Infix(Box::new(inf))
    }

    fn visit_prefix(&mut self, mut pre: PrefixExpression) -> ast::Expression {
        let visited_expr = self.visit_expr(&pre.right);
        pre.right = visited_expr.clone();

        match visited_expr {
            ast::Expression::Int(mut i) => {
                if pre.operator == "-" {
                    i.value = -i.value;
                    return ast::Expression::Int(i);
                }
                ast::Expression::Prefix(Box::new(pre))
            }
            ast::Expression::Bool(mut b) => {
                if pre.operator == "!" {
                    b.value = !b.value;
                    return ast::Expression::Bool(b);
                }
                ast::Expression::Prefix(Box::new(pre))
            }
            _ => ast::Expression::Prefix(Box::new(pre)),
        }
    }
}

pub fn optimize_program(mut program: ast::Program) -> ast::Program {
    // multipass optimization

    // first we fold
    let program = ConstantFolder::fold(program);
    program
}

#[cfg(test)]

mod tests {
    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    fn optimize_ast(code: &str) -> Result<ast::Program, String> {
        let mut l = Lexer::new(code);
        let mut p = Parser::new(l);
        let mut program = p.parse();
        if p.errors().len() > 0 {
            let mut err_list: Vec<String> = vec![];
            for e in &p.errors() {
                err_list.push(e.clone());
            }
            return Err(String::from(err_list.join(",")));
        }
        program = optimize_program(program);
        Ok(program)
    }

    #[test]
    fn test_constant_folding_arithmetic() {
        let program = optimize_ast("let a = fn() -> int { return 2 + 2 * 3 }").unwrap();
        if let ast::Statement::Let(l) = &program.statements[0] {
            if let ast::Expression::Function(f) = &l.value {
                if let ast::Statement::Return(r) = &f.body.statements[0] {
                    if let ast::Expression::Int(i) = &r.return_value {
                        assert_eq!(i.value, 8);
                        return;
                    }
                }
            }
        }
        panic!(
            "expected a folded integer literal, got={:#?}",
            program.statements[0]
        );
    }

    #[test]
    fn test_constant_folding_comparison() {
        let program = optimize_ast("let compare = fn() -> bool { return 1 < 2 }").unwrap();
        if let ast::Statement::Let(l) = &program.statements[0] {
            if let ast::Expression::Function(f) = &l.value {
                if let ast::Statement::Return(r) = &f.body.statements[0] {
                    if let ast::Expression::Bool(b) = &r.return_value {
                        assert_eq!(b.value, true);
                        return;
                    }
                }
            }
        }
        panic!("expected a folded boolean literal");
    }

    #[test]
    fn test_constant_folding_arithmetic_has_correct_type() {
        let program = optimize_ast("let a = fn() -> int { return 2 + 2 * 3 }").unwrap();
        if let ast::Statement::Let(l) = &program.statements[0] {
            if let ast::Expression::Function(f) = &l.value {
                if let ast::Statement::Return(r) = &f.body.statements[0] {
                    if let ast::Expression::Int(i) = &r.return_value {
                        assert_eq!(
                            i.ty,
                            types::Type::Int,
                            "folded comparison should have type Int"
                        );
                        return;
                    }
                }
            }
        }
        panic!("expected a folded integer literal");
    }

    #[test]
    fn test_constant_folding_comparison_has_correct_type() {
        let program = optimize_ast("let compare = fn() -> bool { return 1 < 2 }").unwrap();
        if let ast::Statement::Let(l) = &program.statements[0] {
            if let ast::Expression::Function(f) = &l.value {
                if let ast::Statement::Return(r) = &f.body.statements[0] {
                    if let ast::Expression::Bool(b) = &r.return_value {
                        assert_eq!(
                            b.ty,
                            types::Type::Bool,
                            "folded comparison should have type Bool"
                        );
                        return;
                    }
                }
            }
        }
        panic!("expected a folded boolean literal");
    }

    #[test]
    fn test_constant_folding_division_by_zero_does_not_panic() {
        let program = optimize_ast("let f = fn() -> int { return 1 / 0 }");
        assert!(
            program.is_ok(),
            "division-by-zero folding should not crash compilation"
        );
    }

    #[test]
    fn test_constant_folding_nested_expression() {
        let program = optimize_ast("let f = fn() -> int { return (2 + 3) * (4 - 1) }").unwrap();
        if let ast::Statement::Let(l) = &program.statements[0] {
            if let ast::Expression::Function(f) = &l.value {
                if let ast::Statement::Return(r) = &f.body.statements[0] {
                    if let ast::Expression::Int(i) = &r.return_value {
                        assert_eq!(i.value, 15);
                        return;
                    }
                }
            }
        }
        panic!("expected a fully folded integer literal");
    }

    #[test]
    fn test_constant_folding_prefix_negation() {
        let program = optimize_ast("let f = fn() -> int { return -(3 + 2) }").unwrap();
        if let ast::Statement::Let(l) = &program.statements[0] {
            if let ast::Expression::Function(f) = &l.value {
                if let ast::Statement::Return(r) = &f.body.statements[0] {
                    if let ast::Expression::Int(i) = &r.return_value {
                        assert_eq!(i.value, -5);
                        return;
                    }
                }
            }
        }
        panic!("expected a folded negative integer literal");
    }

    #[test]
    fn test_constant_folding_prefix_bang() {
        let program = optimize_ast("let f = fn() -> bool { return !(1 < 2) }").unwrap();
        if let ast::Statement::Let(l) = &program.statements[0] {
            if let ast::Expression::Function(f) = &l.value {
                if let ast::Statement::Return(r) = &f.body.statements[0] {
                    if let ast::Expression::Bool(b) = &r.return_value {
                        assert_eq!(b.value, false);
                        return;
                    }
                }
            }
        }
        panic!("expected a folded boolean literal");
    }

    #[test]
    fn test_non_foldable_expression_left_untouched() {
        let program = optimize_ast("let f = fn(a: int) -> int { return a + 2 * 3 }").unwrap();
        if let ast::Statement::Let(l) = &program.statements[0] {
            if let ast::Expression::Function(f) = &l.value {
                if let ast::Statement::Return(r) = &f.body.statements[0] {
                    if let ast::Expression::Infix(inf) = &r.return_value {
                        assert!(matches!(&inf.left, ast::Expression::Ident(_)));
                        if let ast::Expression::Int(i) = &inf.right {
                            assert_eq!(i.value, 6);
                            return;
                        }
                    }
                }
            }
        }
        panic!("expected `a + 6` with the right-hand side folded and left-hand side untouched");
    }

    #[test]
    fn test_constant_array_folding() {
        let program = optimize_ast("let a = [1, 2+2, 3*3]").unwrap();
        if let ast::Statement::Let(l) = &program.statements[0] {
            if let ast::Expression::Array(a) = &l.value {
                if let ast::Expression::Int(i) = &a.elements[2] {
                    assert_eq!(i.value, 9, "constant folding should work here");
                    return;
                }
            }
        }
        panic!(
            "program.statements[0] not let, got={:#?}",
            &program.statements[0]
        );
    }

    #[test]
    fn test_constant_index_folding() {
        let program = optimize_ast(
            "
            let a = [1, 2+2, 3*3]
            let b = a[1+1]
            ",
        )
        .unwrap();
        if let ast::Statement::Let(l) = &program.statements[1] {
            if let ast::Expression::Index(index) = &l.value {
                if let ast::Expression::Int(i) = &index.index {
                    assert_eq!(i.value, 2, "constant folding should work here");
                    return;
                }
            }
        }
        panic!(
            "program.statements[1] not let, got={:#?}",
            &program.statements[1]
        );
    }

    // one test also for multiple optimization passes
}
