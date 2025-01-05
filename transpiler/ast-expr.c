#include "ast-node.h"
#include "tokenizer.h"

ast_node_t *token_stream_expr(token_stream_t *self) {
  return token_stream_cmp_expr(self);
}
