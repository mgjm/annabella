#pragma once

#include "keywords.h"
#include "str.h"

typedef enum token_type {
  token_type_end,
  token_type_whitespace,
  token_type_token,
  token_type_keyword,
  token_type_ident,
  token_type_number,
  token_type_string,
} token_type_t;

typedef struct token {
  token_type_t type;
  union {
    char token;
    keyword_t keyword;
    static_str_t ident;
    static_str_t number;
    static_str_t string;
  };
} token_t;

typedef struct token_stream {
  static_str_t content;
  string_t buffer;
} token_stream_t;

extern token_stream_t token_stream_open(str_t path);
extern token_t token_stream_next(token_stream_t *self);

extern char *token_to_string(token_t self);

#define die_with_token(token, msg, ...)                                        \
  die("unexpected %s (expected " msg ")\n",                                    \
      token_to_string(token) __VA_OPT__(, __VA_ARGS__))
