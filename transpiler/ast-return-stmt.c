#include "ast-node.h"
#include "macros.h"
#include "token-stream.h"

typedef struct ast_return_stmt {
  ast_node_t super;
  ast_node_t *expr;
} ast_return_stmt_t;

static void ast_return_stmt_drop(void *_self) {
  ast_return_stmt_t *self = _self;
  ast_node_drop(self->expr);
  free(self);
}

static void ast_return_stmt_to_string(void *_self, string_t *str) {
  ast_return_stmt_t *self = _self;
  string_append(str, "return ");
  ast_node_to_string(self->expr, str);
  string_append(str, ";");
}

static void ast_return_stmt_generate(void *_self, context_t *ctx) {
  die("generate not implemented: %s\n", ast_node_class_name(_self));
}

static const ast_node_vtable_t ast_return_stmt_vtable = {
    "return_stmt",
    ast_return_stmt_drop,
    ast_return_stmt_to_string,
    ast_return_stmt_generate,
};

ast_node_t *token_stream_return_stmt(token_stream_t *self) {
  token_stream_whitespace(self);
  ast_node_t *expr = token_stream_expr(self);
  token_stream_token(self, ';');

  ast_return_stmt_t *return_stmt = malloc(sizeof(*return_stmt));
  *return_stmt = (ast_return_stmt_t){
      &ast_return_stmt_vtable,
      expr,
  };
  return &return_stmt->super;
}
