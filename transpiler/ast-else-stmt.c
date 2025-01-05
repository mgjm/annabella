#include "ast-node.h"
#include "str.h"

typedef struct ast_else_stmt {
  ast_node_t super;
} ast_else_stmt_t;

static void ast_else_stmt_drop(void *_self) {
  ast_else_stmt_t *self = _self;
  free(self);
}

static void ast_else_stmt_to_string(void *_self, string_t *str) {
  ast_else_stmt_t *self = _self;
  string_append(str, "else\n");
}

static void ast_else_stmt_generate(void *_self, context_t *ctx) {
  string_append(&ctx->value, "} else {\n");
}

static const ast_node_vtable_t ast_else_stmt_vtable = {
    "else_stmt",
    ast_else_stmt_drop,
    ast_else_stmt_to_string,
    ast_else_stmt_generate,
};

ast_node_t *token_stream_else_stmt(token_stream_t *self) {
  ast_else_stmt_t *else_stmt = malloc(sizeof(*else_stmt));
  *else_stmt = (ast_else_stmt_t){
      &ast_else_stmt_vtable,
  };
  return &else_stmt->super;
}
