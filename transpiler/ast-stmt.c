#include "ast-node.h"
#include "keywords.h"
#include "token-stream.h"

ast_node_t *token_stream_stmt(token_stream_t *self) {
  token_stream_whitespace(self);

  token_stream_t clone = *self;
  token_t token = token_stream_next(&clone);

  switch (token.type) {
  case token_type_ident:
    return token_stream_assignment_stmt(self);
  case token_type_keyword:
    *self = clone;
    break;
  default:
    die_with_token(token, "start of statement");
  }

  switch (token.keyword) {
  case keyword_with:
    return token_stream_with_stmt(self);
  case keyword_procedure:
    return token_stream_procedure_stmt(self);
  case keyword_function:
    return token_stream_function_stmt(self);
  case keyword_package:
    return token_stream_package_stmt(self);
  case keyword_return:
    return token_stream_return_stmt(self);
  case keyword_if:
    return token_stream_if_stmt(self);
  case keyword_elsif:
    return token_stream_elsif_stmt(self);
  case keyword_else:
    return token_stream_else_stmt(self);
  default:
    die("unknown keyword statement: %s\n", keyword_get(token.keyword));
  }

  return NULL;
}
