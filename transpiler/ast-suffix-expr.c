#include "ast-node.h"
#include "str.h"

typedef struct ast_suffix_expr {
  ast_node_t super;
  ast_node_t *expr;
} ast_suffix_expr_t;

static void ast_suffix_expr_drop(void *_self) {
  ast_suffix_expr_t *self = _self;
  ast_node_drop(self->expr);
  free(self);
}

static void ast_suffix_expr_to_string(void *_self, string_t *str) {
  ast_suffix_expr_t *self = _self;
  ast_node_to_string(self->expr, str);
}

static void ast_suffix_expr_generate(void *_self, context_t *ctx) {

  ast_suffix_expr_t *self = _self;
  string_append(&ctx->value, "annabella_value_to_value(");
  ast_node_generate(self->expr, ctx);
  string_append(&ctx->value, ", scope)");
}

static const ast_node_vtable_t ast_suffix_expr_vtable = {
    "suffix_expr",
    ast_suffix_expr_drop,
    ast_suffix_expr_to_string,
    ast_suffix_expr_generate,
};

static ast_node_t *token_stream_suffix_expr_inner(token_stream_t *self) {

  ast_node_t *value = token_stream_value_expr(self);

  token_stream_t clone = *self;
  token_t token = token_stream_next(&clone);
  if (token.type != token_type_token) {
    return value;
  }

  switch (token.token) {
  case '(':
    return token_stream_call_expr(self, value);
  default:
    return value;
  }
}

ast_node_t *token_stream_suffix_expr(token_stream_t *self) {
  ast_node_t *expr = token_stream_suffix_expr_inner(self);
  ast_suffix_expr_t *suffix_expr = malloc(sizeof(*suffix_expr));
  *suffix_expr = (ast_suffix_expr_t){
      &ast_suffix_expr_vtable,
      expr,
  };
  return &suffix_expr->super;
}
