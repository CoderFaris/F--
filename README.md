# <F-->

A small statically typed programming language with a compiler written in Rust. Source code goes through a hand-written lexer, a Pratt parser, a type checker and an optimization pass, and is then compiled to native code with [LLVM](https://llvm.org/) (via [inkwell](https://github.com/TheDan64/inkwell)) and run through LLVM's JIT.

I built this to learn how compilers work end to end, from tokens to machine code.

```
let fib = fn(n: int) -> int {
    if (n < 2) {
        return n;
    } else {
        return fib(n - 1) + fib(n - 2);
    }
}
fib(10);
```

## Features

- **Types:** `int` (64-bit signed), `bool`, arrays (e.g. `[1, 2, 3]`), functions
- **Variables:** `let` bindings, shadowing, and reassignment (`x = x + 1`)
- **Functions:** first-class syntax with mandatory parameter and return types, recursion, mutual recursion and forward references
- **Control flow:** `if` / `else` (usable as an expression), `while` loops, early `return`
- **Operators:** `+ - * /`, `< > == !=`, prefix `-` and `!`, array indexing `a[i]`
- **Comments:** `// line comments` (revolutionary feature I know I know)
- **Static type checking** with local type inference: annotate function signatures, everything else is inferred
- **Constant folding:** expressions like `2 + 2 * 3` are evaluated at compile time

## Language tour

```
// Variables: types are inferred from the value
let x = 5;
let flag = x > 3;

// Functions: parameter and return types are required
let add = fn(a: int, b: int) -> int {
    return a + b;
};

// if/else can produce a value
let max = if (x > 10) { x; } else { 10; };

// Loops and assignment
let i = 0;
let sum = 0;
while (i < 5) {
    sum = sum + i;
    i = i + 1;
}

// Arrays
let nums = [10, 20, 30];
nums[1] + add(sum, max);
```

The value of the last top-level expression is printed when the program finishes.

## How it works

```
source → Lexer → Parser → Type checker → Optimizer → LLVM codegen → JIT
```

| Stage | File | What it does |
|---|---|---|
| Lexer | `src/lexer.rs` | Turns source text into tokens |
| Parser | `src/parser.rs` | Pratt parser that builds the AST |
| Type checker | `src/type_checker.rs` | Two passes: collects function signatures, then checks every expression and fills in its type (unification-based). Local type inference is used |
| Optimizer | `src/visitor.rs` | Visitor-based AST passes; currently constant folding |
| Codegen | `src/codegen.rs` | Lowers the typed AST to LLVM IR in three passes: declare functions, compile function bodies, wrap top-level statements in an entry function |

A few design notes:

- Variables live in stack slots (`alloca`), which LLVM's `mem2reg` pass later promotes to registers.
- `if` expressions are compiled to `then` / `else` / `merge` blocks joined with a `phi` node.
- Arrays are represented as pointers to their first element; indexing uses `getelementptr` + `load`.

## Building

### Requirements

- Rust 1.85+
- LLVM 21.1.x **with development files** (`llvm-config` and static libraries)
- On Windows: Visual Studio 2022 with the "Desktop development with C++" workload

### LLVM setup on Windows

The official LLVM installer does **not** include `llvm-config.exe` or the static libraries, so `llvm-sys` can't use it.

1. Download `llvm-21.1.1-windows-amd64-msvc17-msvcrt.7z` from [vovkos/llvm-package-windows](https://github.com/vovkos/llvm-package-windows/releases).
2. Extract it to e.g. `C:\LLVM`.
3. Point `llvm-sys` at it, then restart your terminal:
   ```
   setx LLVM_SYS_211_PREFIX C:\LLVM
   ```

The prebuilt package only includes the x86 backend, so inkwell is configured with only that target enabled:

```toml
inkwell = { version = "0.9", default-features = false, features = ["llvm21-1", "target-x86"] }
```

### Linux / macOS

Install LLVM 21 from your package manager (or llvm.org) and set `LLVM_SYS_211_PREFIX` to its install prefix if it isn't found automatically.

### Build and test

```
cargo build --release
cargo test
```

## Usage

```
cargo run -- examples/main.my
```

Debug flags:

| Flag | Output |
|---|---|
| `--tokens` | Print the token stream and exit |
| `--ast` | Print the parsed AST and exit |
| `--ir` | Print the generated LLVM IR |

## Limitations

This is a learning project, and some things are intentionally (or not yet) supported:

- Functions can't read variables declared outside them (no closures or globals yet)
- Arrays have no length, no bounds checking, and live on the stack, so returning one from a function is unsafe
- Empty array literals (`[]`) can't be compiled
- No `print` yet; the program's result is the value of the last expression
- The type checker doesn't verify that every path in a function returns (codegen catches it instead)
- Compiles via JIT only; no standalone executable output yet

## Acknowledgements

- LLVM's [Kaleidoscope tutorial](https://llvm.org/docs/tutorial/)
- [inkwell](https://github.com/TheDan64/inkwell)