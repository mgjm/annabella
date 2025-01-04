#include "ast-node.h"
#include "str.h"
#include "token-stream.h"

typedef struct ast_expr_stmt {
  ast_node_t super;
  ast_node_t *expr;
} ast_expr_stmt_t;

static void ast_expr_stmt_drop(void *_self) {
  ast_expr_stmt_t *self = _self;
  free(self);
}

static void ast_expr_stmt_to_string(void *_self, string_t *str) {
  ast_expr_stmt_t *self = _self;
  ast_node_to_string(self->expr, str);
  string_append(str, ";");
}

static void ast_expr_stmt_generate(void *_self, context_t *ctx) {
  ast_expr_stmt_t *self = _self;
  string_append(&ctx->value, "annabella_value_drop(");
  ast_node_generate(self->expr, ctx);
  string_append(&ctx->value, ");\n");
}

static const ast_node_vtable_t ast_expr_stmt_vtable = {
    "expr_stmt",
    ast_expr_stmt_drop,
    ast_expr_stmt_to_string,
    ast_expr_stmt_generate,
};

ast_node_t *token_stream_expr_stmt(token_stream_t *self) {
  ast_node_t *expr = token_stream_expr(self);
  token_stream_token(self, ';');

  ast_expr_stmt_t *expr_stmt = malloc(sizeof(*expr_stmt));
  *expr_stmt = (ast_expr_stmt_t){
      &ast_expr_stmt_vtable,
      expr,
  };
  return &expr_stmt->super;
}
