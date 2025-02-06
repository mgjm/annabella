# Annabella

> [!NOTE]
> This is the first experimental version of annabella in C.
> I abandoned this path and instead continued to develop an approach written in Rust (main branch).

Run Ada code without an existing Ada compiler binary.

The GNAT Ada compiler is written in Ada. You need an existing Ada compiler executable to compile the Ada compiler.

Annabella can be used to bootstrap an Ada Compiler with only a C compiler and the GNAT source code.

> [!WARNING]
> This project is not yet ready. It can't run the GNAT Ada compiler yet.
>
> Have a look at the [`examples`](examples) to see what already works.

> [!NOTE]
> This project is not intended to be used to run your own Ada code. See [Goals](#goals) and [Non-Goals](#non-goals).

## Workflow

1. The Ada code is transpiled into C code.
2. The C code is compiled with a C compiler.
3. The C code uses a runtime library to execute.


### Transpiler

The [`transpiler`](transpiler) converts the Ada code to C code:

The process consists of three steps:

1. The tokenizer converts the input file into a stream of tokens
2. The stream gets parsed into AST nodes
3. Each AST node generates C code


All AST nodes implement a common interface (vtable) and can be nested to create AST nodes out of other AST nodes.
They can even be nested recursively (e.g. an function call expression contains multiple expression nodes as arguments and an argument can be another function call expression).


### Runtime

The [`runtime`](runtime) is used by the generated C code to execute the Ada code.

It is implemented as a dynamically typed fully object oriented system.

Everything is a `value_t`. Even packages, procedures, functions and types.

Each value only implements the methods that it actually supports. You can't invoke a number or multiply a function.


Variables are stored in a `scope_t`. Scopes are a map-like structure storing the available identifiers in the current context.

Scopes can be nested. The inner child scope can access identifiers defined in a parent scope. But a child identifier hides a parent identifier with the same name.


## Goals

- Run the GNAT Ada Compiler
- Compile the GNAT Ada Compiler by running the GNAT Ada Compiler with annabella
- On error, give enough error information to know which feature is not yet implemented


## Non-Goals

- Speed
- Support every feature of Ada
- Produce descriptive errors (e.g. invalid syntax, unknown types / variables / functions, wrong type / value)

## License

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version. This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
details. You should have received a copy of the GNU General Public License
along with this program. If not, see <http://www.gnu.org/licenses/>.
