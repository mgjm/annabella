#include "ast-node.h"
#include "tokenizer.h"

ast_node_t *token_stream_value_expr(token_stream_t *self) {
  token_stream_t clone = *self;
  token_t token = token_stream_next(&clone);
  switch (token.type) {
  case token_type_end:
    die_with_token(token, "start of value expr");
  case token_type_whitespace:
    die_with_token(token, "start of value expr");
  case token_type_token:
    die_with_token(token, "start of value expr");
  case token_type_keyword:
    die_with_token(token, "start of value expr");
  case token_type_ident:
    return token_stream_path(self);
  case token_type_number:
    return token_stream_number(self);
  case token_type_string:
    die_with_token(token, "start of value expr");
  }
}
