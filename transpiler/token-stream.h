#pragma once

#include "keywords.h"
#include "str.h"
#include "tokenizer.h"
#include <stdbool.h>

extern void token_stream_whitespace(token_stream_t *self);
extern void token_stream_token(token_stream_t *self, char token);
extern bool token_stream_consume_if_token(token_stream_t *self, char token);
extern static_str_t token_stream_ident(token_stream_t *self);
extern void token_stream_ident_eq(token_stream_t *self, static_str_t ident);
extern void token_stream_keyword(token_stream_t *self, keyword_t keyword);
extern bool token_stream_consume_if_keyword(token_stream_t *self,
                                            keyword_t keyword);
