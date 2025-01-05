#include "ast-node.h"
#include "keywords.h"
#include "macros.h"
#include "str.h"
#include "token-stream.h"

typedef struct ast_if_stmt {
  ast_node_t super;
  ast_node_t *cond;
  ast_node_array_t stmts;
} ast_if_stmt_t;

static void ast_if_stmt_drop(void *_self) {
  ast_if_stmt_t *self = _self;
  free(self);
}

static void ast_if_stmt_to_string(void *_self, string_t *str) {
  ast_if_stmt_t *self = _self;
  string_append(str, "if ");
  ast_node_to_string(self->cond, str);
  string_append(str, " then\n");
  ast_node_array_to_string_lines(&self->stmts, str);
  string_append(str, "end if;\n");
}

static void ast_if_stmt_generate(void *_self, context_t *ctx) {
  ast_if_stmt_t *self = _self;
  string_append(&ctx->value, "if (annabella_value_to_bool(");
  ast_node_generate(self->cond, ctx);
  string_append(&ctx->value, ")) {\n");
  ast_node_array_generate(&self->stmts, ctx);
  string_append(&ctx->value, "}\n");
}

static const ast_node_vtable_t ast_if_stmt_vtable = {
    "if_stmt",
    ast_if_stmt_drop,
    ast_if_stmt_to_string,
    ast_if_stmt_generate,
};

ast_node_t *token_stream_if_stmt(token_stream_t *self) {
  token_stream_whitespace(self);
  ast_node_t *cond = token_stream_expr(self);

  token_stream_whitespace(self);
  token_stream_keyword(self, keyword_then);

  ast_node_array_t stmts = {};

  token_stream_whitespace(self);
  while (!token_stream_consume_if_keyword(self, keyword_end)) {
    ast_node_t *stmt = token_stream_stmt(self);
    ast_node_array_push(&stmts, stmt);
    token_stream_whitespace(self);
  }

  token_stream_whitespace(self);
  token_stream_keyword(self, keyword_if);
  token_stream_token(self, ';');

  ast_if_stmt_t *if_stmt = malloc(sizeof(*if_stmt));
  *if_stmt = (ast_if_stmt_t){
      &ast_if_stmt_vtable,
      cond,
      stmts,
  };
  return &if_stmt->super;
}
