#include "ast-node.h"
#include "macros.h"
#include "token-stream.h"

typedef struct ast_package_stmt {
  ast_node_t super;
  ast_node_t *name;
  ast_node_array_t stmts;
} ast_package_stmt_t;

static void ast_package_stmt_drop(void *_self) {
  ast_package_stmt_t *self = _self;
  ast_node_drop(self->name);
  ast_node_array_drop(&self->stmts);
  free(self);
}

static void ast_package_stmt_to_string(void *_self, string_t *str) {
  ast_package_stmt_t *self = _self;
  string_append(str, "package body ");
  ast_node_to_string(self->name, str);
  string_append(str, "is\nbegin\n\n");
  ast_node_array_to_string_lines(&self->stmts, str);
  string_append(str, "end ");
  ast_node_to_string(self->name, str);
  string_append(str, ";\n");
}

static void ast_package_stmt_generate(void *_self, context_t *ctx) {
  die("generate not implemented: %s\n", ast_node_class_name(_self));
}

static const ast_node_vtable_t ast_package_stmt_vtable = {
    "package_stmt",
    ast_package_stmt_drop,
    ast_package_stmt_to_string,
    ast_package_stmt_generate,
};

ast_node_t *token_stream_package_stmt(token_stream_t *self) {
  token_stream_whitespace(self);
  token_stream_keyword(self, keyword_body);

  token_stream_whitespace(self);
  ast_node_t *name = token_stream_path(self);

  token_stream_whitespace(self);
  token_stream_keyword(self, keyword_is);

  ast_node_array_t stmts = {};

  while (!token_stream_consume_if_keyword(self, keyword_end)) {
    ast_node_t *stmt = token_stream_stmt(self);
    ast_node_array_push(&stmts, stmt);

    token_stream_whitespace(self);
  }

  token_stream_whitespace(self);
  token_stream_path_eq(self, name);
  token_stream_token(self, ';');

  ast_package_stmt_t *package_stmt = malloc(sizeof(*package_stmt));
  *package_stmt = (ast_package_stmt_t){
      &ast_package_stmt_vtable,
      name,
      stmts,
  };
  return &package_stmt->super;
}
