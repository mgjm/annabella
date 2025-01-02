#pragma once
// class AstNode:
//   drop()
//   eval(scop) -> Value;
//
// class WithStmt extends AstNode:
// class ProcedureStmt extends AstNode:
// class ExprStmt extends AstNode:
//
// class Expr extends AstNode:
// class FnCallExpr extends Expr:
// class StringExpr extends Expr:

#include "atom.h"
#include "object.h"
#include "scope.h"
#include "value.h"

typedef struct ast_node_vtable {
  object_vtable_t object;
  value_t *(*eval)(void *self, scope_t *scope);
} ast_node_vtable_t;

typedef struct ast_node {
  ast_node_vtable_t *vtable;
} ast_node_t;

extern value_t *ast_node_eval(void *self, scope_t *scope);

typedef ast_node_t stmt_t;

typedef struct path {
  atom_t *components;
  size_t len;
  size_t cap;
} path_t;

extern void path_push(path_t *self, atom_t component);

extern void path_drop(path_t *self);

typedef struct with_stmt {
  ast_node_vtable_t *vtable;
  path_t path;
} with_stmt_t;

extern with_stmt_t *with_stmt_new(path_t path);

typedef struct procedure_stmt {
  ast_node_vtable_t *vtable;
  atom_t ident;
  array_t body; // array of stmt_t*
} procedure_stmt_t;

extern procedure_stmt_t *procedure_stmt_new(atom_t ident, array_t body);

typedef ast_node_t expr_t;

typedef expr_t expr_stmt_t;

typedef struct function_call_expr {
  ast_node_vtable_t *vtable;
  path_t path;
  array_t args; // array of expr_t*
} function_call_expr_t;

extern function_call_expr_t *function_call_expr_new(path_t path, array_t args);

typedef struct string_expr {
  ast_node_vtable_t *vtable;
  atom_t value;
} string_expr_t;

extern string_expr_t *string_expr_new(atom_t value);

typedef struct path_expr {
  ast_node_vtable_t *vtable;
  path_t path;
} path_expr_t;

extern path_expr_t *path_expr_new(path_t path);
