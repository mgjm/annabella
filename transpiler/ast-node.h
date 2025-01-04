#pragma once

#include "macros.h"
#include "str.h"
#include "tokenizer.h"
#include <stddef.h>

typedef struct context context_t;

typedef struct ast_node_vtable {
  static_str_t class_name;
  void (*drop)(void *self);
  void (*to_string)(void *self, string_t *str);
  void (*generate)(void *self, context_t *ctx);
} ast_node_vtable_t;

typedef struct ast_node {
  const ast_node_vtable_t *vtable;
} ast_node_t;

static inline static_str_t ast_node_class_name(ast_node_t *self) {
  return self->vtable->class_name;
}

static inline void ast_node_drop(ast_node_t *self) {
  return self->vtable->drop(self);
}

static inline void ast_node_to_string(ast_node_t *self, string_t *str) {
  return self->vtable->to_string(self, str);
}

static inline void ast_node_debug(ast_node_t *self) {
  string_t str = NULL;
  ast_node_to_string(self, &str);
  eprintf("%s\n", str);
  free(str);
}

static inline void ast_node_generate(ast_node_t *self, context_t *ctx) {
  return self->vtable->generate(self, ctx);
}

typedef struct ast_node_array {
  ast_node_t **nodes;
  size_t len;
  size_t cap;
} ast_node_array_t;

extern void ast_node_array_push(ast_node_array_t *self, ast_node_t *node);
extern void ast_node_array_drop(ast_node_array_t *self);
extern void ast_node_array_to_string_lines(ast_node_array_t *self,
                                           string_t *str);
extern void ast_node_array_to_string_comma(ast_node_array_t *self,
                                           string_t *str);

extern ast_node_t *token_stream_path(token_stream_t *self);
extern void token_stream_path_eq(token_stream_t *self, ast_node_t *path);
extern void ast_path_generate_init_fn_name(ast_node_t *self, context_t *ctx);

extern ast_node_t *token_stream_stmt(token_stream_t *self);
extern ast_node_t *token_stream_with_stmt(token_stream_t *self);
extern ast_node_t *token_stream_package_stmt(token_stream_t *self);
extern ast_node_t *token_stream_function_stmt(token_stream_t *self);
extern ast_node_t *token_stream_procedure_stmt(token_stream_t *self);
extern ast_node_t *token_stream_assignment_stmt(token_stream_t *self);
extern ast_node_t *token_stream_expr_stmt(token_stream_t *self);
extern ast_node_t *token_stream_return_stmt(token_stream_t *self);

extern ast_node_t *token_stream_var_declaration(token_stream_t *self);

extern ast_node_t *token_stream_expr(token_stream_t *self);
extern ast_node_t *token_stream_suffix_expr(token_stream_t *self);
extern ast_node_t *token_stream_call_expr(token_stream_t *self,
                                          ast_node_t *function);
extern ast_node_t *token_stream_value_expr(token_stream_t *self);

extern ast_node_t *token_stream_number(token_stream_t *self);
extern ast_node_t *token_stream_string(token_stream_t *self);
