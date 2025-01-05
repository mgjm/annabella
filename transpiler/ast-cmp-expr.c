#include "ast-node.h"
#include "macros.h"
#include "str.h"
#include "token-stream.h"
#include "tokenizer.h"

typedef enum cmp_op {
  _not_an_cmp_op,
  cmp_op_equal,
  cmp_op_not_equal,
  cmp_op_less,
  cmp_op_less_or_equal,
  cmp_op_greater,
  cmp_op_greater_or_equal,
} cmp_op_t;

typedef struct ast_cmp_expr {
  ast_node_t super;
  ast_node_t *lhs;
  cmp_op_t op;
  ast_node_t *rhs;
} ast_cmp_expr_t;

static void ast_cmp_expr_drop(void *_self) {
  ast_cmp_expr_t *self = _self;
  ast_node_drop(self->lhs);
  ast_node_drop(self->rhs);
  free(self);
}

static static_str_t cmp_op_ada[] = {
    [_not_an_cmp_op] = "<not a comparision operator>",
    [cmp_op_equal] = "=",
    [cmp_op_not_equal] = "/=",
    [cmp_op_less] = "<",
    [cmp_op_less_or_equal] = "<=",
    [cmp_op_greater] = ">",
    [cmp_op_greater_or_equal] = ">=",
};

static void ast_cmp_expr_to_string(void *_self, string_t *str) {
  ast_cmp_expr_t *self = _self;
  ast_node_to_string(self->lhs, str);
  string_append(str, " %s ", cmp_op_ada[self->op]);
  ast_node_to_string(self->rhs, str);
}

static static_str_t cmp_op_c[] = {
    [_not_an_cmp_op] = "<not a comparision operator>",
    [cmp_op_equal] = "annabella_cmp_op_equal",
    [cmp_op_not_equal] = "annabella_cmp_op_not_equal",
    [cmp_op_less] = "annabella_cmp_op_less",
    [cmp_op_less_or_equal] = "annabella_cmp_op_less_or_equal",
    [cmp_op_greater] = "annabella_cmp_op_greater",
    [cmp_op_greater_or_equal] = "annabella_cmp_op_greater_or_equal",
};

static void ast_cmp_expr_generate(void *_self, context_t *ctx) {
  ast_cmp_expr_t *self = _self;
  string_append(&ctx->value, "annabella_value_cmp(");
  ast_node_generate(self->lhs, ctx);
  string_append(&ctx->value, ", %s, ", cmp_op_c[self->op]);
  ast_node_generate(self->rhs, ctx);
  string_append(&ctx->value, ")");
}

static const ast_node_vtable_t ast_cmp_expr_vtable = {
    "cmp_expr",
    ast_cmp_expr_drop,
    ast_cmp_expr_to_string,
    ast_cmp_expr_generate,
};

static cmp_op_t token_stream_cmp_op(token_stream_t *self) {
  token_stream_t clone = *self;
  token_t token = token_stream_next(&clone);
  if (token.type != token_type_token) {
    return _not_an_cmp_op;
  }

  switch (token.token) {
  case '/':
    if (token_stream_consume_if_token(&clone, '=')) {
      *self = clone;
      return cmp_op_not_equal;
    }
    break;
  case '=':
    *self = clone;
    return cmp_op_equal;
  case '<':
    *self = clone;
    if (token_stream_consume_if_token(self, '=')) {
      return cmp_op_less_or_equal;
    }
    return cmp_op_less;
  case '>':
    *self = clone;
    if (token_stream_consume_if_token(self, '=')) {
      return cmp_op_greater_or_equal;
    }
    return cmp_op_greater;
  }
  return _not_an_cmp_op;
}

ast_node_t *token_stream_cmp_expr(token_stream_t *self) {
  ast_node_t *lhs = token_stream_suffix_expr(self);

  token_stream_whitespace(self);
  cmp_op_t op = token_stream_cmp_op(self);
  if (op == _not_an_cmp_op) {
    return lhs;
  }

  token_stream_whitespace(self);
  ast_node_t *rhs = token_stream_suffix_expr(self);

  ast_cmp_expr_t *cmp_expr = malloc(sizeof(*cmp_expr));
  *cmp_expr = (ast_cmp_expr_t){
      &ast_cmp_expr_vtable,
      lhs,
      op,
      rhs,
  };
  return &cmp_expr->super;
}
