#include "ast-node.h"
#include "macros.h"
#include "str.h"
#include "tokenizer.h"

typedef struct ast_number {
  ast_node_t super;
  static_str_t value;
} ast_number_t;

static void ast_number_drop(void *_self) {
  ast_number_t *self = _self;
  free(self);
}

static void ast_number_to_string(void *_self, string_t *str) {
  ast_number_t *self = _self;
  string_append(str, "%s", self->value);
}

static void ast_number_generate(void *_self, context_t *ctx) {
  die("generate not implemented: %s\n", ast_node_class_name(_self));
}

static const ast_node_vtable_t ast_number_vtable = {
    "number",
    ast_number_drop,
    ast_number_to_string,
    ast_number_generate,
};

ast_node_t *token_stream_number(token_stream_t *self) {
  token_t token = token_stream_next(self);
  if (token.type != token_type_number) {
    die_with_token(token, "number");
  }

  ast_number_t *number = malloc(sizeof(*number));
  *number = (ast_number_t){
      &ast_number_vtable,
      token.number,
  };
  return &number->super;
}
