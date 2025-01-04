#include "ast-node.h"
#include "tokenizer.h"

ast_node_t *token_stream_suffix_expr(token_stream_t *self) {
  ast_node_t *value = token_stream_value_expr(self);

  token_stream_t clone = *self;
  token_t token = token_stream_next(&clone);
  if (token.type != token_type_token) {
    return value;
  }

  switch (token.token) {
  case '(':
    return token_stream_call_expr(self, value);
  default:
    return value;
  }
}
