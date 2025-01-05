#include "ast-node.h"
#include "str.h"
#include "token-stream.h"

typedef struct ast_assignment_stmt {
  ast_node_t super;
  ast_node_t *name;
  ast_node_t *expr;
} ast_assignment_stmt_t;

static void ast_assignment_stmt_drop(void *_self) {
  ast_assignment_stmt_t *self = _self;
  free(self);
}

static void ast_assignment_stmt_to_string(void *_self, string_t *str) {
  ast_assignment_stmt_t *self = _self;
  ast_node_to_string(self->name, str);
  string_append(str, " := ");
  ast_node_to_string(self->expr, str);
  string_append(str, ";");
}

static void ast_assignment_stmt_generate(void *_self, context_t *ctx) {
  ast_assignment_stmt_t *self = _self;

  string_append(&ctx->value, "annabella_value_assign(\n");
  ast_node_generate(self->name, ctx);
  string_append(&ctx->value, ",\n");
  ast_node_generate(self->expr, ctx);
  string_append(&ctx->value, ");\n\n");
}

static const ast_node_vtable_t ast_assignment_stmt_vtable = {
    "assignment_stmt",
    ast_assignment_stmt_drop,
    ast_assignment_stmt_to_string,
    ast_assignment_stmt_generate,
};

ast_node_t *token_stream_assignment_stmt(token_stream_t *self) {
  token_stream_t clone = *self;
  ast_node_t *name = token_stream_path(&clone);
  token_stream_whitespace(&clone);

  token_t token = token_stream_next(&clone);
  if (token.type == token_type_token && token.token == ':') {
    token = token_stream_next(&clone);
    if (token.type == token_type_token && token.token == '=') {
      *self = clone;
      token_stream_whitespace(self);
      ast_node_t *expr = token_stream_expr(self);
      token_stream_token(self, ';');

      ast_assignment_stmt_t *assignment_stmt = malloc(sizeof(*assignment_stmt));
      *assignment_stmt = (ast_assignment_stmt_t){
          &ast_assignment_stmt_vtable,
          name,
          expr,
      };
      return &assignment_stmt->super;
    }
  }

  return token_stream_expr_stmt(self);
}
