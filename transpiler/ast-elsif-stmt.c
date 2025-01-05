#include "ast-node.h"
#include "str.h"
#include "token-stream.h"

typedef struct ast_elsif_stmt {
  ast_node_t super;
  ast_node_t *cond;
} ast_elsif_stmt_t;

static void ast_elsif_stmt_drop(void *_self) {
  ast_elsif_stmt_t *self = _self;
  ast_node_drop(self->cond);
  free(self);
}

static void ast_elsif_stmt_to_string(void *_self, string_t *str) {
  ast_elsif_stmt_t *self = _self;
  string_append(str, "elsif ");
  ast_node_to_string(self->cond, str);
  string_append(str, "then\n");
}

static void ast_elsif_stmt_generate(void *_self, context_t *ctx) {
  ast_elsif_stmt_t *self = _self;
  string_append(&ctx->value, "} else if (annabella_value_to_bool(");
  ast_node_generate(self->cond, ctx);
  string_append(&ctx->value, ")) {\n");
}

static const ast_node_vtable_t ast_elsif_stmt_vtable = {
    "elsif_stmt",
    ast_elsif_stmt_drop,
    ast_elsif_stmt_to_string,
    ast_elsif_stmt_generate,
};

ast_node_t *token_stream_elsif_stmt(token_stream_t *self) {
  token_stream_whitespace(self);
  ast_node_t *cond = token_stream_expr(self);

  token_stream_whitespace(self);
  token_stream_keyword(self, keyword_then);

  ast_elsif_stmt_t *elsif_stmt = malloc(sizeof(*elsif_stmt));
  *elsif_stmt = (ast_elsif_stmt_t){
      &ast_elsif_stmt_vtable,
      cond,
  };
  return &elsif_stmt->super;
}
