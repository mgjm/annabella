#pragma once

#include "atom.h"
#include <stddef.h>
#include <stdint.h>

typedef struct file_buffer {
  int fd;
  uint8_t buffer[4096];
  size_t start;
  size_t end;
} file_buffer_t;

typedef enum token_type {
  token_type_end,
  token_type_whitespace,
  token_type_token,
  token_type_ident,
  token_type_number,
  token_type_string,
} token_type_t;

typedef struct token {
  token_type_t type;
  atom_t value;
} token_t;

typedef struct token_stream {
  file_buffer_t file_buffer;
  token_t peeked;
} token_stream_t;

extern token_t token_stream_next(token_stream_t *self);
extern void tokenize_file(const char *path);
