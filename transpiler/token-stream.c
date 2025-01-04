#include "token-stream.h"
#include "macros.h"
#include "tokenizer.h"
#include <string.h>

bool token_stream_is_end(token_stream_t *self) {
  token_stream_t clone = *self;
  switch (token_stream_next(&clone).type) {
  case token_type_end:
    return true;
  case token_type_whitespace:
    return token_stream_next(&clone).type == token_type_end;
  default:
    return false;
  }
}

void token_stream_whitespace(token_stream_t *self) {
  token_stream_t clone = *self;
  if (token_stream_next(&clone).type == token_type_whitespace) {
    *self = clone;
  }
}

void token_stream_token(token_stream_t *self, char token) {
  token_t t = token_stream_next(self);
  if (t.type != token_type_token || t.token != token) {
    die_with_token(t, "'%c' token", token)
  }
}

bool token_stream_consume_if_token(token_stream_t *self, char token) {
  token_stream_t clone = *self;
  token_t t = token_stream_next(&clone);
  if (t.type != token_type_token || t.token != token) {
    return false;
  }
  *self = clone;
  return true;
}

static_str_t token_stream_ident(token_stream_t *self) {
  token_t token = token_stream_next(self);
  if (token.type != token_type_ident) {
    die_with_token(token, "ident");
  }
  return token.ident;
}

void token_stream_ident_eq(token_stream_t *self, static_str_t ident) {
  token_t token = token_stream_next(self);
  if (token.type != token_type_ident || strcmp(token.ident, ident) != 0) {
    die_with_token(token, "ident `%s`", ident);
  }
}

void token_stream_keyword(token_stream_t *self, keyword_t keyword) {
  token_t token = token_stream_next(self);
  if (token.type != token_type_keyword || token.keyword != keyword) {
    die_with_token(token, "keyword %s", keyword_get(keyword));
  }
}

bool token_stream_consume_if_keyword(token_stream_t *self, keyword_t keyword) {
  token_stream_t clone = *self;
  token_t token = token_stream_next(&clone);
  if (token.type != token_type_keyword || token.keyword != keyword) {
    return false;
  }
  *self = clone;
  return true;
}
