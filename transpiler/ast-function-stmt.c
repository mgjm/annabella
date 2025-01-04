#include "ast-node.h"
#include "macros.h"
#include "str.h"
#include "token-stream.h"

typedef struct ast_function_stmt {
  ast_node_t super;
  static_str_t name;
  ast_node_array_t args;
  ast_node_t *return_type;
  ast_node_array_t vars;
  ast_node_array_t body;
} ast_function_stmt_t;

static void ast_function_stmt_drop(void *_self) {
  ast_function_stmt_t *self = _self;
  ast_node_array_drop(&self->args);
  ast_node_drop(self->return_type);
  ast_node_array_drop(&self->vars);
  ast_node_array_drop(&self->body);
  free(self);
}

static void ast_function_stmt_to_string(void *_self, string_t *str) {
  ast_function_stmt_t *self = _self;
  string_append(str, "function %s ", self->name);
  if (self->args.len != 0) {
    string_append(str, "(");
    ast_node_array_to_string_comma(&self->args, str);
    string_append(str, ") ");
  }
  string_append(str, "return ");
  ast_node_to_string(self->return_type, str);
  string_append(str, " is\n");
  ast_node_array_to_string_lines(&self->vars, str);
  string_append(str, "begin\n");
  ast_node_array_to_string_lines(&self->body, str);
  string_append(str, "end %s;\n", self->name);
}

static void ast_function_stmt_generate(void *_self, context_t *ctx) {
  die("generate not implemented: %s\n", ast_node_class_name(_self));
}

static const ast_node_vtable_t ast_function_stmt_vtable = {
    "function_stmt",
    ast_function_stmt_drop,
    ast_function_stmt_to_string,
    ast_function_stmt_generate,
};

ast_node_t *token_stream_function_stmt(token_stream_t *self) {
  token_stream_whitespace(self);
  static_str_t name = token_stream_ident(self);

  ast_node_array_t args = {};

  token_stream_whitespace(self);
  if (!token_stream_consume_if_keyword(self, keyword_return)) {
    token_stream_token(self, '(');

    bool expect_dot = false;
    while (!token_stream_consume_if_token(self, ')')) {
      if (expect_dot) {
        token_stream_token(self, ',');
      }
      expect_dot = true;
      ast_node_t *arg = token_stream_var_declaration(self);
      ast_node_array_push(&args, arg);
    }

    token_stream_whitespace(self);
    token_stream_keyword(self, keyword_return);
  }

  token_stream_whitespace(self);
  ast_node_t *return_type = token_stream_path(self);

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

  // eprintf("start of function %s\n", name);

  ast_node_array_t body = {};

  while (!token_stream_consume_if_keyword(self, keyword_end)) {
    ast_node_t *stmt = token_stream_stmt(self);
    ast_node_array_push(&body, stmt);
    token_stream_whitespace(self);
  }

  token_stream_whitespace(self);
  token_stream_ident_eq(self, name);
  token_stream_token(self, ';');
  // eprintf("end of function %s\n", name);

  ast_function_stmt_t *function_stmt = malloc(sizeof(*function_stmt));
  *function_stmt = (ast_function_stmt_t){
      &ast_function_stmt_vtable, name, args, return_type, vars, body,
  };
  return &function_stmt->super;
}
