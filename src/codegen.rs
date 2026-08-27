use std::collections::HashMap;

use inkwell::{builder::Builder, context::Context, module::Module, values::BasicValueEnum};

use crate::ast;

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub named_values: HashMap<String, BasicValueEnum<'ctx>>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Compiler<'ctx> {
        let builder = context.create_builder();
        let module = context.create_module("my compiler");
        Compiler {
            context: context,
            builder: builder,
            module: module,
            named_values: HashMap::new(),
        }
    }
    pub fn compile_program(&mut self, program: ast::Program) -> Result<(), &'static str> {
        for statement in program.statements {
            self.compile_stmt(statement);
        }
        Ok(())
    }
    pub fn compile_stmt(&mut self, statement: ast::Statement) -> Result<(), &'static str> {
        match statement {
            ast::Statement::Let(l) => {
                let value = self.compile_expr(l.value)?;
                self.named_values.insert(l.name.value, value);
                Ok(())
            }
            ast::Statement::Return(r) => {
                // let value = self.compile_expr(r.return_value)?;
                todo!("later");
            }
            ast::Statement::Expression(e) => {
                self.compile_expr(e.expression);
                Ok(())
            }
            ast::Statement::Block(b) => {
                for stmt in b.statements {
                    self.compile_stmt(stmt);
                }
                Ok(())
            }
        }
    }
    pub fn compile_expr(
        &mut self,
        expr: ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, &'static str> {
        match expr {
            ast::Expression::Int(i) => Ok(BasicValueEnum::IntValue(
                self.context.i64_type().const_int(i.value as u64, false),
            )),
            ast::Expression::Infix(inf) => {
                let l = self.compile_expr(inf.left)?.into_int_value();
                let r = self.compile_expr(inf.right)?.into_int_value();

                if inf.operator == "+".to_string() {
                    self.builder
                        .build_int_add(l, r, "addtmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build add")
                } else if inf.operator == "-".to_string() {
                    self.builder
                        .build_int_sub(l, r, "subtmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build sub")
                } else if inf.operator == "*".to_string() {
                    self.builder
                        .build_int_mul(l, r, "multmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build mul")
                } else if inf.operator == "/".to_string() {
                    self.builder
                        .build_int_signed_div(l, r, "divtmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build div")
                } else if inf.operator == ">".to_string() {
                    self.builder
                        .build_int_compare(inkwell::IntPredicate::SGT, l, r, "gtcmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build gtcmp")
                } else if inf.operator == "<".to_string() {
                    self.builder
                        .build_int_compare(inkwell::IntPredicate::SLT, l, r, "ltcmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build ltcmp")
                } else if inf.operator == "==".to_string() {
                    self.builder
                        .build_int_compare(inkwell::IntPredicate::EQ, l, r, "eqcmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build eqcmp")
                } else if inf.operator == "!=".to_string() {
                    self.builder
                        .build_int_compare(inkwell::IntPredicate::NE, l, r, "necmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build necmp")
                } else {
                    Err("Invalid operator")
                }
            }

            ast::Expression::Ident(ident) => self
                .named_values
                .get(&ident.value)
                .cloned()
                .ok_or("Unknown variable"),
            _ => todo!("handle rest of expression types"),
        }
    }
}
