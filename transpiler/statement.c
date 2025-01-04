#include "statement.h"
#include "ast-node.h"
#include "keywords.h"
#include "macros.h"
#include "str.h"
#include "token-stream.h"
#include "tokenizer.h"

void token_stream_with_statement(token_stream_t *self) {
  token_stream_whitespace(self);

  ast_node_t *path = token_stream_path(self);

  printf("annabella_scope_insert_package(&scope, _annabella_package_");
  ast_path_generate_init_fn_name(path, NULL);
  printf("_init());\n");
  token_stream_token(self, ';');
}

void token_stream_variable_definition(token_stream_t *self) {
  token_stream_whitespace(self);
  static_str_t ident = token_stream_ident(self);

  token_stream_whitespace(self);
  token_stream_token(self, ':');
  token_stream_whitespace(self);

  ast_node_t *path = token_stream_path(self);

  token_stream_token(self, ';');
}

void token_stream_procedure_statement(token_stream_t *self) {
  token_stream_whitespace(self);
  static_str_t ident = token_stream_ident(self);

  token_stream_whitespace(self);
  token_stream_keyword(self, keyword_is);

  while (!token_stream_consume_if_keyword(self, keyword_begin)) {
    token_stream_variable_definition(self);
    token_stream_whitespace(self);
  }

  eprintf("start of %s\n", ident);

  while (!token_stream_consume_if_keyword(self, keyword_end)) {
    token_stream_statement(self);
    token_stream_whitespace(self);
  }

  token_stream_whitespace(self);
  token_stream_ident_eq(self, ident);
  token_stream_token(self, ';');
  eprintf("end of %s\n", ident);
}

void token_stream_ident_statement(token_stream_t *self) {
  token_stream_t clone = *self;
  ast_node_t *path = token_stream_path(&clone);
  token_stream_whitespace(&clone);

  token_t token = token_stream_next(&clone);
  if (token.type == token_type_token && token.token == ':') {
    token = token_stream_next(&clone);
    if (token.type == token_type_token && token.token == '=') {
      *self = clone;
      token_stream_whitespace(self);
      ast_node_t *expr = token_stream_expr(self);
      token_stream_token(self, ';');

      string_t str = NULL;
      ast_node_to_string(path, &str);
      string_append(&str, " := ");
      ast_node_to_string(expr, &str);
      eprintf("%s;\n", str);
      free(str);

      return;
    }
  }

  ast_node_t *expr = token_stream_expr(self);
  token_stream_token(self, ';');

  string_t str = NULL;
  ast_node_to_string(expr, &str);
  eprintf("%s;\n", str);
  free(str);
}

bool token_stream_statement(token_stream_t *self) {
  token_stream_whitespace(self);

  token_stream_t clone = *self;
  token_t token = token_stream_next(&clone);
  if (token.type == token_type_end) {
    return false;
  }

  switch (token.type) {

  case token_type_end:
    return false;
  case token_type_ident:
    token_stream_ident_statement(self);
    return true;
  case token_type_keyword:
    *self = clone;
    break;
  default:
    die_with_token(token, "start of statement");
  }

  switch (token.keyword) {
  case keyword_with:
    token_stream_with_statement(self);
    break;
  case keyword_procedure:
    token_stream_procedure_statement(self);
    break;
  default:
    die("unknown keyword statement: %s\n", keyword_get(token.keyword));
  }

  return true;
}
