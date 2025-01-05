#include "ast-node.h"
#include "str.h"
#include "token-stream.h"

typedef struct ast_var_declaration {
  ast_node_t super;
  static_str_t name;
  ast_node_t *type;
} ast_var_declaration_t;

static void ast_var_declaration_drop(void *_self) {
  ast_var_declaration_t *self = _self;
  ast_node_drop(self->type);
  free(self);
}

static void ast_var_declaration_to_string(void *_self, string_t *str) {
  ast_var_declaration_t *self = _self;
  string_append(str, "%s : ", self->name);
  ast_node_to_string(self->type, str);
}

static void ast_var_declaration_generate(void *_self, context_t *ctx) {
  ast_var_declaration_t *self = _self;
  string_append(&ctx->value,
                "annabella_scope_insert_value(scope, \"%s\", "
                "annabella_integer_value(0));\n",
                self->name);
}

static const ast_node_vtable_t ast_var_declaration_vtable = {
    "var_declaration",
    ast_var_declaration_drop,
    ast_var_declaration_to_string,
    ast_var_declaration_generate,
};

ast_node_t *token_stream_var_declaration(token_stream_t *self) {
  token_stream_whitespace(self);
  static_str_t name = token_stream_ident(self);

  token_stream_whitespace(self);
  token_stream_token(self, ':');

  token_stream_whitespace(self);
  ast_node_t *type = token_stream_path(self);

  ast_var_declaration_t *var_declaration = malloc(sizeof(*var_declaration));
  *var_declaration = (ast_var_declaration_t){
      &ast_var_declaration_vtable,
      name,
      type,
  };
  return &var_declaration->super;
}
