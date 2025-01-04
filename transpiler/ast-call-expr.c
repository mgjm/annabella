#include "ast-node.h"
#include "macros.h"
#include "str.h"
#include "token-stream.h"

typedef struct ast_call_expr {
  ast_node_t super;
  ast_node_t *function;
  ast_node_array_t args;
} ast_call_expr_t;

static void ast_call_expr_drop(void *_self) {
  ast_call_expr_t *self = _self;
  ast_node_array_drop(&self->args);
  free(self);
}

static void ast_call_expr_to_string(void *_self, string_t *str) {
  ast_call_expr_t *self = _self;

  ast_node_to_string(self->function, str);

  for (size_t i = 0; i < self->args.len; i++) {
    string_append(str, "%c", i == 0 ? '(' : ',');
    ast_node_to_string(self->args.nodes[i], str);
  }

  string_append(str, "%s", self->args.len == 0 ? "()" : ")");
}

static void ast_call_expr_generate(void *_self, context_t *ctx) {
  die("generate not implemented: %s\n", ast_node_class_name(_self));
}

static const ast_node_vtable_t ast_call_expr_vtable = {
    "call_expr",
    ast_call_expr_drop,
    ast_call_expr_to_string,
    ast_call_expr_generate,
};

ast_node_t *token_stream_call_expr(token_stream_t *self, ast_node_t *function) {
  ast_node_array_t args = {};
  token_stream_token(self, '(');
  while (!token_stream_consume_if_token(self, ')')) {
    ast_node_t *arg = token_stream_expr(self);
    ast_node_array_push(&args, arg);
  }

  ast_call_expr_t *call_expr = malloc(sizeof(*call_expr));
  *call_expr = (ast_call_expr_t){
      &ast_call_expr_vtable,
      function,
      args,
  };
  return &call_expr->super;
}
