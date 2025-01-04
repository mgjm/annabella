#include "ast-node.h"
#include "macros.h"
#include "token-stream.h"

typedef struct ast_with_stmt {
  ast_node_t super;
  ast_node_t *path;
} ast_with_stmt_t;

static void ast_with_stmt_drop(void *_self) {
  ast_with_stmt_t *self = _self;
  ast_node_drop(self->path);
  free(self);
}

static void ast_with_stmt_to_string(void *_self, string_t *str) {
  ast_with_stmt_t *self = _self;
  string_append(str, "with ");
  ast_node_to_string(self->path, str);
  string_append(str, ";");
}

static void ast_with_stmt_generate(void *_self, context_t *ctx) {
  die("generate not implemented: %s\n", ast_node_class_name(_self));
}

static const ast_node_vtable_t ast_with_stmt_vtable = {
    "with_stmt",
    ast_with_stmt_drop,
    ast_with_stmt_to_string,
    ast_with_stmt_generate,
};

ast_node_t *token_stream_with_stmt(token_stream_t *self) {
  token_stream_whitespace(self);

  ast_node_t *path = token_stream_path(self);
  token_stream_token(self, ';');

  ast_with_stmt_t *with_stmt = malloc(sizeof(*with_stmt));
  *with_stmt = (ast_with_stmt_t){
      &ast_with_stmt_vtable,
      path,
  };
  return &with_stmt->super;
}