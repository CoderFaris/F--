use std::{collections::HashMap, hash::Hash};

use inkwell::{
    AddressSpace,
    basic_block::BasicBlock,
    builder::{Builder, BuilderError},
    context::Context,
    module::Module,
    types::{ArrayType, BasicMetadataTypeEnum, BasicType, BasicTypeEnum, IntType},
    values::{
        BasicMetadataValueEnum, BasicValueEnum, FunctionValue, InstructionValue, PointerValue,
    },
};

use crate::{
    ast::{self, Identifier},
    types::Type,
};

pub const MAIN_FN_NAME: &str = "__main";

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub functions: HashMap<String, FunctionValue<'ctx>>,
    pub variables: Vec<HashMap<String, PointerValue<'ctx>>>,
    pub current_fn: Option<FunctionValue<'ctx>>,
    pub named_values: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Compiler<'ctx> {
        let builder = context.create_builder();
        let module = context.create_module("my compiler");
        Compiler {
            context: context,
            builder: builder,
            module: module,
            functions: HashMap::new(),
            variables: Vec::new(),
            current_fn: None,
            named_values: HashMap::new(),
        }
    }
    // three passes
    pub fn compile_program(&mut self, program: ast::Program) -> Result<(), String> {
        let statements = program.statements;
        for stmt in &statements {
            if let ast::Statement::Let(l) = stmt {
                if let ast::Expression::Function(f) = &l.value {
                    self.declare_function(&l.name.value, &f.parameters, &f.return_type)?;
                }
            }
        }

        for stmt in &statements {
            if let ast::Statement::Let(l) = stmt {
                if let ast::Expression::Function(f) = &l.value {
                    let function =
                        self.declare_function(&l.name.value, &f.parameters, &f.return_type)?;
                    self.compile_function_body(&l.name.value, function, f)?;
                }
            }
        }

        let i64_ty = self.context.i64_type();
        let main_fn = self
            .module
            .add_function(MAIN_FN_NAME, i64_ty.fn_type(&[], false), None);
        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);
        self.current_fn = Some(main_fn);
        self.push_scope();

        let mut last: Option<BasicValueEnum<'ctx>> = None;

        for stmt in statements {
            let is_function_let = matches!(
                &stmt,
                ast::Statement::Let(l) if matches!(l.value, ast::Expression::Function(_))
            );
            if is_function_let {
                continue;
            }
            last = self.compile_stmt(stmt)?;
        }

        let ret = match last {
            Some(BasicValueEnum::IntValue(v)) if v.get_type().get_bit_width() == 64 => v,
            _ => i64_ty.const_int(0, false),
        };

        self.builder
            .build_return(Some(&ret))
            .map_err(|e| e.to_string())?;

        self.pop_scope();
        self.current_fn = None;

        if !main_fn.verify(true) {
            return Err("LLVM rejected the IR for `main`".to_string());
        }

        Ok(())
    }

    fn declare_function(
        &mut self,
        name: &String,
        parameters: &Vec<Identifier>,
        ret_ty: &Type,
    ) -> Result<FunctionValue<'ctx>, String> {
        if let Some(existing) = self.functions.get(name) {
            return Ok(*existing);
        }

        let param_types: Vec<BasicMetadataTypeEnum> = parameters
            .iter()
            .map(|i| self.to_llvm_type(&i.ty).map(Into::into))
            .collect::<Result<_, _>>()?;

        let fn_type = match ret_ty {
            Type::Void => self.context.void_type().fn_type(&param_types, false),
            other => self.to_llvm_type(other)?.fn_type(&param_types, false),
        };

        let function = self.module.add_function(name, fn_type, None);

        for (idx, identifier) in parameters.iter().enumerate() {
            function
                .get_nth_param(idx as u32)
                .ok_or("parameter index out of range")?
                .set_name(&identifier.value[..])
        }
        self.functions.insert(name.to_string(), function);
        Ok(function)
    }

    fn compile_function_body(
        &mut self,
        name: &str,
        function: FunctionValue<'ctx>,
        f: &ast::FunctionLiteral,
    ) -> Result<(), String> {
        let saved_block = self.builder.get_insert_block();
        let saved_fn = self.current_fn;

        let saved_scopes = std::mem::take(&mut self.variables);

        self.current_fn = Some(function);
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);
        self.push_scope();

        for (arg, param) in function.get_param_iter().zip(f.parameters.iter()) {
            let llvm_ty = self.to_llvm_type(&param.ty)?;
            let slot = self.entry_alloca(function, &param.value, llvm_ty)?;
            self.builder
                .build_store(slot, arg)
                .map_err(|e| e.to_string())?;
            self.declare_var(param.value.clone(), slot);
        }

        self.compile_stmt(ast::Statement::Block(f.body.clone()))?;

        let end = self.builder.get_insert_block().unwrap();
        if end.get_terminator().is_none() {
            if f.return_type == Type::Void {
                self.builder.build_return(None).map_err(|e| e.to_string())?;
            } else {
                return Err(format!(
                    "function `{}` can reach its end without returning a value",
                    name
                ));
            }
        }

        self.pop_scope();

        self.variables = saved_scopes;
        self.current_fn = saved_fn;
        if let Some(bb) = saved_block {
            self.builder.position_at_end(bb);
        }

        if !function.verify(true) {
            return Err(format!("LLVM rejected the IR for function `{}`", name));
        }

        Ok(())
    }

    fn to_llvm_type(&self, ty: &Type) -> Result<BasicTypeEnum<'ctx>, String> {
        match ty {
            Type::Int => Ok(BasicTypeEnum::IntType(self.context.i64_type())),
            Type::Bool => Ok(BasicTypeEnum::IntType(self.context.bool_type())),
            Type::Array(_) => Ok(BasicTypeEnum::PointerType(
                self.context.ptr_type(AddressSpace::default()).into(),
            )),
            Type::Void => Err("Type::Void has no BasicTypeEnum representation".to_string()),
            Type::Unknown => Err(format!("internal error: Type::Unknown reached codegen")),
            Type::Function { .. } => Err("No LLVM type for function ".to_string()),
        }
    }

    fn to_llvm_array_type(&self, elem_ty: &Type, len: u32) -> Result<ArrayType<'ctx>, String> {
        let llvm_ty = self.to_llvm_type(elem_ty)?;
        Ok(llvm_ty.array_type(len))
    }

    fn push_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.variables.pop();
    }

    fn declare_var(&mut self, name: String, ptr: PointerValue<'ctx>) {
        self.variables
            .last_mut()
            .expect("declare_var called with no active scope")
            .insert(name, ptr);
    }

    fn lookup_var(&self, name: &str) -> Option<PointerValue<'ctx>> {
        self.variables
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn create_entry_block_alloca(
        &self,
        function: &FunctionValue<'ctx>,
        name: &String,
        ty: &Type,
    ) -> Result<PointerValue<'ctx>, String> {
        let builder = self.context.create_builder();
        let entry = function.get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(inst) => builder.position_before(&inst),
            None => builder.position_at_end(entry),
        }

        let llvm_type = self.to_llvm_type(ty)?;
        Ok(builder.build_alloca(llvm_type, name).unwrap())
    }

    fn entry_alloca(
        &self,
        function: FunctionValue<'ctx>,
        name: &str,
        ty: BasicTypeEnum<'ctx>,
    ) -> Result<PointerValue<'ctx>, String> {
        let tmp = self.context.create_builder();

        let entry = function
            .get_first_basic_block()
            .ok_or("function has no entry block")?;

        match entry.get_first_instruction() {
            Some(inst) => tmp.position_before(&inst),
            None => tmp.position_at_end(entry),
        }

        tmp.build_alloca(ty, name).map_err(|e| e.to_string())
    }

    pub fn compile_stmt(
        &mut self,
        statement: ast::Statement,
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        match statement {
            ast::Statement::Let(l) => {
                let value = self.compile_expr(&l.value.clone())?;
                let name = l.name.value;
                let function = self.current_fn.ok_or("`let` used outside a function")?;
                let alloca = self.create_entry_block_alloca(&function, &name, l.value.ty())?;
                self.builder.build_store(alloca, value).unwrap();
                self.declare_var(name, alloca);
                Ok(Some(value))
            }
            ast::Statement::Return(r) => {
                let value = self.compile_expr(&r.return_value)?;
                self.builder.build_return(Some(&value)).unwrap();
                Ok(Some(value)) // Ok(Some(value)) in original
            }
            ast::Statement::Expression(e) => {
                let value = self.compile_expr(&e.expression)?;
                Ok(Some(value))
            }
            ast::Statement::Block(b) => {
                self.push_scope();
                let mut value: Option<BasicValueEnum> = None;
                for stmt in b.statements {
                    value = self.compile_stmt(stmt)?;
                }
                self.pop_scope();
                Ok(value)
            }
        }
    }
    pub fn compile_expr(&mut self, expr: &ast::Expression) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            ast::Expression::Int(i) => Ok(BasicValueEnum::IntValue(
                self.context.i64_type().const_int(i.value as u64, false),
            )),
            ast::Expression::Bool(b) => Ok(BasicValueEnum::IntValue(
                self.context.bool_type().const_int(b.value as u64, false),
            )),
            ast::Expression::Infix(inf) => {
                let l = self.compile_expr(&inf.left)?.into_int_value();
                let r = self.compile_expr(&inf.right)?.into_int_value();

                if inf.operator == "+".to_string() {
                    self.builder
                        .build_int_add(l, r, "addtmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build add".to_string())
                } else if inf.operator == "-".to_string() {
                    self.builder
                        .build_int_sub(l, r, "subtmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build sub".to_string())
                } else if inf.operator == "*".to_string() {
                    self.builder
                        .build_int_mul(l, r, "multmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build mul".to_string())
                } else if inf.operator == "/".to_string() {
                    self.builder
                        .build_int_signed_div(l, r, "divtmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build div".to_string())
                } else if inf.operator == ">".to_string() {
                    self.builder
                        .build_int_compare(inkwell::IntPredicate::SGT, l, r, "gtcmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build gtcmp".to_string())
                } else if inf.operator == "<".to_string() {
                    self.builder
                        .build_int_compare(inkwell::IntPredicate::SLT, l, r, "ltcmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build ltcmp".to_string())
                } else if inf.operator == "==".to_string() {
                    self.builder
                        .build_int_compare(inkwell::IntPredicate::EQ, l, r, "eqcmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build eqcmp".to_string())
                } else if inf.operator == "!=".to_string() {
                    self.builder
                        .build_int_compare(inkwell::IntPredicate::NE, l, r, "necmp")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build necmp".to_string())
                } else {
                    Err("Invalid operator".to_string())
                }
            }
            ast::Expression::Prefix(pre) => {
                let r = self.compile_expr(&pre.right)?;
                if pre.operator == "-".to_string() {
                    self.builder
                        .build_int_neg(r.into_int_value(), "neg")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build neg".to_string())
                } else if pre.operator == "!".to_string() {
                    self.builder
                        .build_not(r.into_int_value(), "not")
                        .map(BasicValueEnum::IntValue)
                        .map_err(|_| "Failed to build not".to_string())
                } else {
                    Err("Unknown operator".to_string())
                }
            }

            ast::Expression::Ident(ident) => {
                let ptr = self
                    .lookup_var(&ident.value)
                    .ok_or_else(|| format!("Unknown variable {}", ident.value))?;
                let llvm_ty = self.to_llvm_type(&ident.ty)?;
                self.builder
                    .build_load(llvm_ty, ptr, &ident.value)
                    .map_err(|e| e.to_string())
            }

            ast::Expression::Call(c) => {
                let function_name: String;
                match &c.function {
                    ast::Expression::Ident(id) => {
                        function_name = id.value.clone();
                    }
                    _ => return Err("Function call can only be from identifier".to_string()),
                }

                let function = self
                    .functions
                    .get(&function_name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined function {}", &function_name))?;

                let arg_values: Vec<BasicMetadataValueEnum> = c
                    .arguments
                    .iter()
                    .map(|e| self.compile_expr(e).map(|v| v.into()))
                    .collect::<Result<_, _>>()?;

                let call = self
                    .builder
                    .build_call(function, &arg_values, &function_name)
                    .unwrap();
                Ok(call.try_as_basic_value().unwrap_basic())
            }

            ast::Expression::If(ifstmt) => {
                let dummy = BasicValueEnum::IntValue(self.context.i64_type().const_int(0, false));

                let cond = self.compile_expr(&ifstmt.condition)?.into_int_value();

                let function = self.current_fn.ok_or("`if` used outside a function")?;
                let then_bb = self.context.append_basic_block(function, "then");
                let else_bb = self.context.append_basic_block(function, "else");
                let merge_bb = self.context.append_basic_block(function, "ifcont");

                self.builder
                    .build_conditional_branch(cond, then_bb, else_bb)
                    .map_err(|e| e.to_string())?;

                self.builder.position_at_end(then_bb);

                // for stmt in ifstmt.consequence.statements.iter() {
                //     then_val = self.compile_stmt(stmt.clone())?;
                // }
                let then_val =
                    self.compile_stmt(ast::Statement::Block(ifstmt.consequence.clone()))?;

                let then_end = self.builder.get_insert_block().unwrap();
                let then_terminated = then_end.get_terminator().is_some();
                if !then_terminated {
                    self.builder.build_unconditional_branch(merge_bb).unwrap();
                }

                self.builder.position_at_end(else_bb);
                let mut else_val: Option<BasicValueEnum<'ctx>> = None;
                if let Some(alt) = &ifstmt.alternative {
                    // for stmt in alt.statements.iter() {
                    //     else_val = self.compile_stmt(stmt.clone())?;
                    // }
                    else_val = self.compile_stmt(ast::Statement::Block(alt.clone()))?;
                }

                let else_end = self.builder.get_insert_block().unwrap();
                let else_terminated = else_end.get_terminator().is_some();
                if !else_terminated {
                    self.builder.build_unconditional_branch(merge_bb).unwrap();
                }

                if then_terminated && else_terminated {
                    unsafe {
                        merge_bb.delete().unwrap();
                    }
                    return Ok(dummy);
                }

                self.builder.position_at_end(merge_bb);

                if ifstmt.ty == Type::Void {
                    return Ok(dummy);
                }

                let llvm_ty = self.to_llvm_type(&ifstmt.ty)?;
                let phi = self.builder.build_phi(llvm_ty, "iftmp").unwrap();

                if !then_terminated {
                    let v = then_val.ok_or("`then` branch produced no value")?;
                    phi.add_incoming(&[(&v, then_end)]);
                }
                if !else_terminated {
                    let v = else_val.ok_or("`else` branch produced no value")?;
                    phi.add_incoming(&[(&v, else_end)]);
                }

                Ok(phi.as_basic_value())
            }
            ast::Expression::While(w) => {
                let function = self.current_fn.ok_or("`while` used outside a function")?;
                let cond_bb = self.context.append_basic_block(function, "while_cond");
                let body_bb = self.context.append_basic_block(function, "while_body");
                let end_bb = self.context.append_basic_block(function, "while_end");

                self.builder.build_unconditional_branch(cond_bb).unwrap();

                self.builder.position_at_end(cond_bb);

                let cond_val = self.compile_expr(&w.condition)?.into_int_value();

                let cond_bool = self
                    .builder
                    .build_int_truncate(cond_val, self.context.bool_type(), "cond")
                    .map_err(|_| "Failed to truncate cond in `while`")?;
                self.builder
                    .build_conditional_branch(cond_bool, body_bb, end_bb)
                    .map_err(|_| "Failed to build cond branch in `while`")?;
                self.builder.position_at_end(body_bb);

                self.compile_stmt(ast::Statement::Block(w.body.clone()))?;

                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.builder
                        .build_unconditional_branch(cond_bb)
                        .map_err(|_| "Failed to build unconditional branch in `while`")?;
                }

                self.builder.position_at_end(end_bb);
                Ok(BasicValueEnum::IntValue(
                    self.context.i64_type().const_int(0, false),
                ))
            }

            ast::Expression::Array(a) => {
                let Type::Array(elem_ty) = &a.ty else {
                    return Err(format!("array literal has non-array type {:?}", a.ty));
                };
                let function = self
                    .current_fn
                    .ok_or("array literal used outside a function")?;

                let arr_llvm_ty = self.to_llvm_array_type(elem_ty, a.elements.len() as u32)?;
                let arr_ptr =
                    self.entry_alloca(function, &"arr".to_string(), arr_llvm_ty.into())?;

                let zero = self.context.i64_type().const_int(0, false);
                for (i, el) in a.elements.iter().enumerate() {
                    let value = self.compile_expr(el)?;
                    let idx = self.context.i64_type().const_int(i as u64, false);

                    let elem_ptr = unsafe {
                        self.builder
                            .build_in_bounds_gep(arr_llvm_ty, arr_ptr, &[zero, idx], "elem_ptr")
                            .map_err(|e| e.to_string())?
                    };
                    self.builder
                        .build_store(elem_ptr, value)
                        .map_err(|e| e.to_string())?;
                }

                Ok(BasicValueEnum::PointerValue(arr_ptr))
            }

            ast::Expression::Index(i) => {
                let Type::Array(elem_ty) = i.left.ty() else {
                    return Err(format!("indexing a non-array type {:?}", i.left.ty()));
                };
                let elem_llvm_ty = self.to_llvm_type(elem_ty)?;
                let base = self.compile_expr(&i.left)?.into_pointer_value();
                let idx = self.compile_expr(&i.index)?.into_int_value();

                let elem_ptr = unsafe {
                    self.builder
                        .build_in_bounds_gep(elem_llvm_ty, base, &[idx], "elem_ptr")
                        .map_err(|e| e.to_string())?
                };

                let value = self
                    .builder
                    .build_load(elem_llvm_ty, elem_ptr, "elem")
                    .map_err(|e| e.to_string())?;

                Ok(value)
            }

            ast::Expression::Assign(assign) => {
                let ptr = self
                    .lookup_var(&assign.target.value)
                    .ok_or_else(|| format!("Unknown variable {}", assign.target.value))?;
                let value = self.compile_expr(&assign.value)?;
                self.builder
                    .build_store(ptr, value)
                    .map_err(|e| format!("codegen error: {}", e))?;
                Ok(value)
            }
            ast::Expression::Function(_) => {
                Err("function literals must be bound with `let name = fn...`;".to_string())
            }
            _ => todo!("handle rest of expression types"),
        }
    }
}

pub fn jit_run(program: ast::Program) -> Result<i64, String> {
    let context = Context::create();
    let mut codegen = Compiler::new(&context);

    codegen.compile_program(program)?;

    let engine = codegen
        .module
        .create_jit_execution_engine(inkwell::OptimizationLevel::Default)
        .map_err(|e| format!("Failed to create JIT: {}", e.to_string()));

    unsafe {
        let func: inkwell::execution_engine::JitFunction<unsafe extern "C" fn() -> i64> = engine?
            .get_function(MAIN_FN_NAME)
            .map_err(|e| e.to_string())?;
        Ok(func.call())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::type_checker::{self, typecheck};
    use crate::visitor::optimize_program;

    fn typecheck_ast(code: &str) -> Result<ast::Program, String> {
        let l = Lexer::new(code);
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
        let program = optimize_program(program);
        Ok(program)
    }

    fn run(source: &str) -> i64 {
        let program = typecheck_ast(source)
            .unwrap_or_else(|e| panic!("front-end rejected program: {}\n{}", e, source));
        jit_run(program).unwrap_or_else(|e| panic!("codegen/JIT failed: {}\n{}", e, source))
    }

    fn compile_only(source: &str) -> Result<String, String> {
        let program = typecheck_ast(source)?;
        let context = Context::create();
        let mut codegen = Compiler::new(&context);
        codegen.compile_program(program)?;
        Ok(codegen.module.print_to_string().to_string())
    }

    #[test]
    fn test_compile_simple_function() {
        let source = "
        let random_function = fn() -> int {
            return 10;
        }
        random_function();
        ";
        let mut program = typecheck_ast(source).unwrap();

        let context = Context::create();
        let mut codegen = Compiler::new(&context);
        codegen.compile_program(program).unwrap();
    }

    #[test]
    fn test_compile_add() {
        let source = "
        let add = fn(a: int, b: int) -> int {
            return a + b;
        }
        add(3, 4);
        ";
        let mut program = typecheck_ast(source).unwrap();
        let context = Context::create();
        let mut codegen = Compiler::new(&context);
        codegen.compile_program(program).unwrap();
    }

    #[test]
    fn test_run_precedence() {
        assert_eq!(run("2 + 3 * 4;"), 14);
    }

    #[test]
    fn test_run_parentheses() {
        assert_eq!(run("(2 + 3) * 4;"), 20);
    }

    #[test]
    fn test_run_prefix_negation() {
        assert_eq!(run("-5 + 2;"), -3);
    }

    #[test]
    fn test_run_signed_division_truncates() {
        assert_eq!(run("7 / 2;"), 3);
    }

    #[test]
    fn test_run_variables() {
        assert_eq!(run("let x = 5; let y = x * 2; y + 1;"), 11);
    }

    #[test]
    fn test_run_shadowing_same_scope() {
        assert_eq!(run("let x = 5; let x = x + 1; x;"), 6);
    }

    #[test]
    fn test_run_shadowing_inside_if_does_not_leak() {
        assert_eq!(
            run("let x = 5;
                 if (x > 0) { let x = 100; }
                 x;"),
            5
        );
    }

    #[test]
    fn test_run_call() {
        assert_eq!(
            run("let add = fn(a: int, b: int) -> int { return a + b; }
                 add(3, 4);"),
            7
        );
    }

    #[test]
    fn test_run_nested_calls() {
        assert_eq!(
            run("let add = fn(a: int, b: int) -> int { return a + b; }
                 add(add(1, 2), add(3, 4));"),
            10
        );
    }

    #[test]
    fn test_run_fibonacci() {
        assert_eq!(
            run("let fib = fn(n: int) -> int {
                     if (n < 2) { return n; } else { return fib(n - 1) + fib(n - 2); }
                 }
                 fib(10);"),
            55
        );
    }

    #[test]
    fn test_run_factorial_one_armed_if_with_return() {
        assert_eq!(
            run("let fact = fn(n: int) -> int {
                     if (n < 2) { return 1; }
                     return n * fact(n - 1);
                 }
                 fact(5);"),
            120
        );
    }

    #[test]
    fn test_run_forward_reference() {
        assert_eq!(
            run("let a = fn() -> int { return b() + 1; }
                 let b = fn() -> int { return 41; }
                 a();"),
            42
        );
    }

    #[test]
    fn test_run_mutual_recursion_with_bool_returns() {
        assert_eq!(
            run("let is_even = fn(n: int) -> bool {
                     if (n == 0) { return true; }
                     return is_odd(n - 1);
                 }
                 let is_odd = fn(n: int) -> bool {
                     if (n == 0) { return false; }
                     return is_even(n - 1);
                 }
                 let r = if (is_even(10)) { 1; } else { 0; };
                 r;"),
            1
        );
    }

    #[test]
    fn test_run_if_else_both_branches() {
        let src = "let max = fn(a: int, b: int) -> int {
                       if (a > b) { return a; } else { return b; }
                   }";
        assert_eq!(run(&format!("{}\n max(3, 9);", src)), 9);
        assert_eq!(run(&format!("{}\n max(10, 2);", src)), 10);
    }

    #[test]
    fn test_run_if_as_value() {
        // exercises the phi node
        assert_eq!(
            run("let x = if (3 > 2) { 10; } else { 20; };
                 x;"),
            10
        );
    }

    #[test]
    fn test_run_bool_variable() {
        // catches an Ident load that hardcodes i64
        assert_eq!(
            run("let b = 1 < 2;
                 let r = if (!b) { 1; } else { 2; };
                 r;"),
            2
        );
    }

    #[test]
    fn test_run_bool_parameter() {
        assert_eq!(
            run("let pick = fn(flag: bool, a: int, b: int) -> int {
                     if (flag) { return a; } else { return b; }
                 }
                 pick(true, 1, 2);"),
            1
        );
    }

    #[test]
    fn test_run_array_index_constant() {
        assert_eq!(run("let a = [10, 20, 30]; a[1];"), 20);
    }

    #[test]
    fn test_run_array_index_runtime_value() {
        assert_eq!(run("let a = [10, 20, 30]; let i = 2; a[i] + a[0];"), 40);
    }

    #[test]
    fn test_run_array_parameter() {
        assert_eq!(
            run("let first = fn(xs: [int]) -> int { return xs[0]; }
                 first([7, 8, 9]);"),
            7
        );
    }

    #[test]
    fn test_run_while_sum() {
        assert_eq!(
            run("let i = 0; let sum = 0;
                 while (i < 5) { sum = sum + i; i = i + 1; }
                 sum;"),
            10
        );
    }

    #[test]
    fn test_ir_function_signature() {
        let ir = compile_only(
            "let add = fn(a: int, b: int) -> int { return a + b; }
             add(1, 2);",
        )
        .unwrap();
        assert!(ir.contains("define i64 @add(i64"), "IR was:\n{}", ir);
    }

    #[test]
    fn test_ir_if_else_value_uses_phi() {
        let ir = compile_only("let x = if (3 > 2) { 10; } else { 20; };\n x;").unwrap();
        assert!(ir.contains("phi i64"), "IR was:\n{}", ir);
    }

    #[test]
    fn test_missing_return_is_a_codegen_error() {
        let err = compile_only(
            "let f = fn(a: int) -> int { if (a > 0) { return 1; } }
             f(1);",
        )
        .unwrap_err();
        assert!(err.contains("without returning"), "got: {}", err);
    }
}
