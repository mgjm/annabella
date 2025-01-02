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
//
// class Value:
//   drop()
//   call(args) -> Value
//   callUnaryOperator(op) -> Value // -5
//   callBinaryOperator(op, rhs) -> Value // 2 + 3
//   callMethod(name, args) -> Value
//   getField(name) -> Value
//   setField(name, value)
//
// StringValue:
// AtomStringValue:      // maybe?
// AllocatedStringValue: // maybe?
// IntValue:
// FloatValue:
// FunctionValue: // function definition
// ClassValue: // class definition
// ObjectValue: // instance of a class

#include "atom.h"
#include <stddef.h>

typedef struct object_vtable {
  const char *class_name;
  void (*drop)(void *self);
} object_vtable_t;

typedef struct object {
  object_vtable_t *vtable;
} object_t;

static inline const char *object_class_name(object_t *self) {
  return self->vtable->class_name;
}

static inline void object_drop(object_t *self) { self->vtable->drop(self); }

typedef struct array {
  object_t **data;
  size_t len;
  size_t cap;
} array_t;

// item needs to be an instance of object_t
extern void array_push(array_t *self, void *item);

extern void array_drop(array_t *self);

static inline object_t *array_as_object(array_t *self) {
  return (object_t *)self;
};

typedef struct ast_node_vtable {
  object_vtable_t object;
  void (*eval)(void *self);
} ast_node_vtable_t;

typedef struct ast_node {
  ast_node_vtable_t *vtable;
} ast_node_t;

extern void ast_node_eval(void *self);

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
