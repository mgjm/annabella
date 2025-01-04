#include "ast-node.h"
#include "macros.h"
#include "str.h"
#include "token-stream.h"

typedef struct ast_procedure_stmt {
  ast_node_t super;
  static_str_t name;
  ast_node_array_t vars;
  ast_node_array_t body;
} ast_procedure_stmt_t;

static void ast_procedure_stmt_drop(void *_self) {
  ast_procedure_stmt_t *self = _self;
  ast_node_array_drop(&self->vars);
  ast_node_array_drop(&self->body);
  free(self);
}

static void ast_procedure_stmt_to_string(void *_self, string_t *str) {
  ast_procedure_stmt_t *self = _self;
  string_append(str, "procedure %s ", self->name);
  string_append(str, "is\n");
  ast_node_array_to_string_lines(&self->vars, str);
  string_append(str, "begin\n");
  ast_node_array_to_string_lines(&self->body, str);
  string_append(str, "end %s;\n", self->name);
}

static void ast_procedure_stmt_generate(void *_self, context_t *ctx) {
  die("generate not implemented: %s\n", ast_node_class_name(_self));
}

static const ast_node_vtable_t ast_procedure_stmt_vtable = {
    "procedure_stmt",
    ast_procedure_stmt_drop,
    ast_procedure_stmt_to_string,
    ast_procedure_stmt_generate,
};

ast_node_t *token_stream_procedure_stmt(token_stream_t *self) {
  token_stream_whitespace(self);
  static_str_t name = token_stream_ident(self);

  token_stream_whitespace(self);
  token_stream_keyword(self, keyword_is);

  ast_node_array_t vars = {};

  token_stream_whitespace(self);
  while (!token_stream_consume_if_keyword(self, keyword_begin)) {
    ast_node_t *var = token_stream_var_declaration(self);
    ast_node_array_push(&vars, var);

    token_stream_token(self, ';');
    token_stream_whitespace(self);
  }

  eprintf("start of procedure %s\n", name);

  ast_node_array_t body = {};

  while (!token_stream_consume_if_keyword(self, keyword_end)) {
    ast_node_t *stmt = token_stream_stmt(self);
    ast_node_array_push(&body, stmt);
    token_stream_whitespace(self);
  }

  token_stream_whitespace(self);
  token_stream_ident_eq(self, name);
  token_stream_token(self, ';');
  eprintf("end of procedure %s\n", name);

  ast_procedure_stmt_t *procedure_stmt = malloc(sizeof(*procedure_stmt));
  *procedure_stmt = (ast_procedure_stmt_t){
      &ast_procedure_stmt_vtable,
      name,
      vars,
      body,
  };
  return &procedure_stmt->super;
}
