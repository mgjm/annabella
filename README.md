# Annabella

Run Ada code without an existing Ada compiler binary.

The GNAT Ada compiler is written in Ada. You need an existing Ada compiler executable to compile the Ada compiler.

Annabella can be used to bootstrap an Ada Compiler with only a Rust and C/C++ compiler and the GNAT source code.

> [!WARNING]
> This project is not yet ready. It can't run the GNAT Ada compiler yet.
>
> Have a look at the [`ada`](ada) directory to see what already works.

> [!NOTE]
> This project is not intended to be used to run your own Ada code. See [Goals](#goals) and [Non-Goals](#non-goals).


## Workflow

1. The Ada code is transpiled into C/C++ code.
2. The C/C++ code is compiled with a C/C++ compiler.
3. The C/C++ code can be linked and executed like the original.


### Tokenizer and Parser

The tokenizer and parser closely followed the architecture established by the `proc-macro2` and `syn` crates.

This includes the well known types like `TokenStream`, `ParseStream` and the `Parse` trait.
And we also support `Span`s as a cheap way to reference source code locations for awesome error messages.

Note that it was necessary to reimplement these types, because the Ada grammer was different anough that it was not possible to reuse the existing rust tokenizer.
The string literal escaping und number literal syntax comes to mind.

The types defined in the parser module then consume the `TokenStream` via the `ParseStream` type and return the parsed syntax nodes.

The architecture was again inspired by `syn`. We use enums to represent the different possibilities during parsing.


### Codegen

In the `codegen` module we add different codegen traits to the existing AST nodes to generate the corresponding C code for items, statements and expressions.

Note that expressions can have an ambiguous type. The type must only be unambiguous at the statement level.
To support this pattern the `ExprValue` enum supports expressions with multiple types (and implementations) at the same time.
They get filtered when combined to new expressions and must be unique at the end.

The generated code is saved in a `TokenTree` from the `proc-macro2` crate. The C code has a similar enough syntax to Rust.
And is created via the `quote` macro from the `quote` crate.
Note though that this might change since we don't need most of the features. We only care about the final code as a string.
It is just a nice way to write the code via the `c_code` macro (which used the `quote` macro under the hood).
We could replace the `quote` macro with our own macro that just generates the string directly.


### C or C++

For now the generated code is plain C code. But a lot of Ada features would map nicely to C++ features:

- Ada packages -> C++ namespaces
- Ada constraint checks -> C++ assignment and conversion operator overloading
- Ada function overloading (with return type overloading) -> C++ function overloading (**without** return type overloading)
  - With a bit of "template magic" even return type overloading is possible
- Ada exceptions -> C++ exceptions
- Ada dynamic dispathed functions -> C++ virtual functions (???)
  - Ada and C++ use different defaults for when a call uses static or dynamic dispatch
  - Ada uses free stanting functions, C++ uses methods attached to the type

We can generate plain C code for all these features but this moves more logic in the codegen phase and we need to inject more helper function calls in the generated C code.
Or we can switch to C++ and generate more readable code since more of the features can be expressed in the language itself.


## Goals

- Run the GNAT Ada Compiler
- Compile the GNAT Ada Compiler by running the GNAT Ada Compiler through annabella
- On error, give enough error information to know which feature is not yet implemented


## Non-Goals

- Speed
- Support every feature of Ada
- Produce descriptive errors (e.g. invalid syntax, unknown types / variables / functions, wrong type / value)
  - Although since the move to rust, the style of error messages significantly improved

## License

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version. This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
details. You should have received a copy of the GNU General Public License
along with this program. If not, see <http://www.gnu.org/licenses/>.
