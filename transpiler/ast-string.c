#include "ast-node.h"
#include "macros.h"
#include "str.h"
#include "tokenizer.h"

typedef struct ast_string {
  ast_node_t super;
  static_str_t value;
} ast_string_t;

static void ast_string_drop(void *_self) {
  ast_string_t *self = _self;
  free(self);
}

static void ast_string_to_string(void *_self, string_t *str) {
  ast_string_t *self = _self;
  string_append(str, "\"%s\"", self->value);
}

static void ast_string_generate(void *_self, context_t *ctx) {
  die("generate not implemented: %s\n", ast_node_class_name(_self));
}

static const ast_node_vtable_t ast_string_vtable = {
    "string",
    ast_string_drop,
    ast_string_to_string,
    ast_string_generate,
};

ast_node_t *token_stream_string(token_stream_t *self) {
  token_t token = token_stream_next(self);
  if (token.type != token_type_string) {
    die_with_token(token, "string");
  }

  ast_string_t *string = malloc(sizeof(*string));
  *string = (ast_string_t){
      &ast_string_vtable,
      token.string,
  };
  return &string->super;
}
