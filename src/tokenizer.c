#include "tokenizer.h"
#include "ast.h"
#include "atom.h"
#include "macros.h"
#include "scope.h"
#include <fcntl.h>
#include <stdbool.h>
#include <stdio.h>
#include <unistd.h>

void file_buffer_open(file_buffer_t *self, const char *path) {
  eprintf("open file: %s\n", path);

  self->fd = open(path, O_RDONLY | O_CLOEXEC);
  if (self->fd < 0) {
    die_errno("failed to open file: %s\n");
  }
}

uint8_t file_buffer_peek_byte(file_buffer_t *self) {
  if (self->start == self->end) {
    int n = read(self->fd, self->buffer, array_len(self->buffer));
    if (n < 0) {
      die_errno("filed to read from file: %s\n");
    }
    if (n == 0) {
      return 0;
    }
    eprintf("read %d bytes\n", n);

    self->start = 0;
    self->end = n;
  }

  if (self->start >= self->end) {
    die("file buffer out of bounds: %ld >= %ld\n", self->start, self->end);
  }

  uint8_t b = self->buffer[self->start];
  if (b == 0) {
    die("source code contained null byte\n");
  }
  return b;
}

uint8_t file_buffer_read_byte(file_buffer_t *self) {
  uint8_t b = file_buffer_peek_byte(self);
  if (b == 0) {
    return 0;
  }
  self->start++;
  return b;
}

typedef enum byte_type {
  byte_type_null,         // \0
  byte_type_whitespace,   // \t \n ' '
  byte_type_double_quote, // "
  byte_type_token,        // everything else
  byte_type_ident,        // [a-z] [A-Z] _
  byte_type_number,       // [0-9]
} byte_type_t;

byte_type_t byte_type(uint8_t b) {
  if (b == 0) {
    return byte_type_null;
  } else if (b == ' ' || b == '\t' || b == '\n') {
    return byte_type_whitespace;
  } else if (b == '"') {
    return byte_type_double_quote;
  } else if ((b >= 'a' && b <= 'z') || (b >= 'A' && b <= 'Z') || b == '_') {
    return byte_type_ident;
  } else if (b >= '0' && b <= '9') {
    return byte_type_number;
  } else {
    return byte_type_token;
  }
}

void token_stream_unshift(token_stream_t *self, token_t token) {

  if (self->peeked.type != token_type_end) {
    die("token stream unshifted twice in a row\n");
  }
  self->peeked = token;
}

token_t token_stream_next(token_stream_t *self) {
  if (self->peeked.type != token_type_end) {
    token_t token = self->peeked;
    self->peeked = (token_t){};
    return token;
  }

  uint8_t b = file_buffer_read_byte(&self->file_buffer);
  char buffer[1024] = {};
  size_t index = 0;
  atom_t atom;

  switch (byte_type(b)) {
  case byte_type_null: // \0
    return (token_t){token_type_end};

  case byte_type_whitespace: // \t \n ' '
    index++;
    while (byte_type(file_buffer_peek_byte(&self->file_buffer)) ==
           byte_type_whitespace) {
      file_buffer_read_byte(&self->file_buffer);
      index++;
    }
    return (token_t){token_type_whitespace};

  case byte_type_double_quote: // "
    while (true) {
      b = file_buffer_read_byte(&self->file_buffer);
      switch (b) {
      case 0:
        die("unterminated string\n");
      case '"':
        b = 0;
        break;
      case '\\':
        b = file_buffer_read_byte(&self->file_buffer);
        switch (b) {
        case '\\':
        case '"':
          break;
        case 'n':
          b = '\n';
          break;
        case 't':
          b = '\t';
          break;
        default:
          die("unknown string escape: \\%c\n", b);
        }
        break;
      }

      if (b == 0) {
        break;
      }

      if (index >= array_len(buffer)) {
        die("token too long\n");
      }
      buffer[index++] = b;
    }
    atom = atom_new(buffer);
    return (token_t){token_type_string, {.string = atom}};

  case byte_type_token: // everything else
    return (token_t){token_type_token, {.token = b}};

  case byte_type_ident: // [a-z]
    while (true) {
      if (index >= array_len(buffer)) {
        die("token too long\n");
      }
      buffer[index++] = b;

      b = file_buffer_peek_byte(&self->file_buffer);
      switch (byte_type(b)) {
      case byte_type_ident:
      case byte_type_number:
        file_buffer_read_byte(&self->file_buffer);
        continue;
      default:
        break;
      }
      break;
    }

    atom = atom_new(buffer);
    return (token_t){token_type_ident, {.ident = atom}};

  case byte_type_number: // [0-9]
    die("todo: number parser");
    return (token_t){token_type_number, {.number = atom}};
  }
}

void token_stream_whitespace(token_stream_t *self) {
  while (true) {
    token_t token = token_stream_next(self);

    if (token.type != token_type_whitespace) {
      token_stream_unshift(self, token);
      break;
    }
  }
}

char *token_to_string(token_t token) {
  char *str;
  switch (token.type) {
  case token_type_end:
    return "end token";
  case token_type_whitespace:
    return "whitespace";
  case token_type_token:
    asprintf(&str, "token '%c'", token.token);
    break;
  case token_type_ident:
    asprintf(&str, "ident `%s`", atom_get(token.ident));
    break;
  case token_type_number:
    asprintf(&str, "number %s", atom_get(token.ident));
    break;
  case token_type_string:
    asprintf(&str, "string \"%s\"", atom_get(token.ident));
    break;
  }
  return str;
}

#define die_with_token(token, msg, ...)                                        \
  die("unexpected %s (expected " msg ")\n",                                    \
      token_to_string(token) __VA_OPT__(, __VA_ARGS__))

atom_t token_stream_ident(token_stream_t *self) {
  token_stream_whitespace(self);
  token_t token = token_stream_next(self);
  if (token.type != token_type_ident) {
    die_with_token(token, "ident");
  }
  return token.ident;
}

void token_stream_consume_ident(token_stream_t *self, atom_t expected) {
  atom_t ident = token_stream_ident(self);
  if (!atom_eq(ident, expected)) {
    die("unexpected ident: %s != %s\n", atom_get(ident), atom_get(expected));
  }
}

path_t token_stream_path(token_stream_t *self) {
  token_stream_whitespace(self);

  path_t path = {};
  while (true) {
    path_push(&path, token_stream_ident(self));

    token_t token = token_stream_next(self);
    if (token.type != token_type_token || token.token != '.') {
      token_stream_unshift(self, token);
      break;
    }
  }
  return path;
}

void token_stream_semi(token_stream_t *self) {
  token_t token = token_stream_next(self);
  if (token.type != token_type_token || token.token != ';') {
    die_with_token(token, "semicolon");
  }
}

stmt_t *token_stream_with_statement(token_stream_t *self) {
  path_t path = token_stream_path(self);
  token_stream_semi(self);
  return (stmt_t *)with_stmt_new(path);
}

stmt_t *token_stream_statement(token_stream_t *self);

stmt_t *token_stream_procedure_statement(token_stream_t *self) {
  atom_t ident = token_stream_ident(self);
  token_stream_consume_ident(self, atom_is);
  token_stream_consume_ident(self, atom_begin);
  array_t body = {};
  while (true) {
    token_stream_whitespace(self);
    token_t token = token_stream_next(self);
    if (token.type == token_type_end) {
      die("unterminated precedure: %s\n", atom_get(ident));
    }
    if (token.type == token_type_ident && atom_eq(token.ident, atom_end)) {
      break;
    }
    token_stream_unshift(self, token);
    stmt_t *stmt = token_stream_statement(self);
    if (stmt == NULL) {
      die("unterminated precedure: %s\n", atom_get(ident));
    }
    array_push(&body, stmt);
  }
  token_stream_consume_ident(self, ident);
  token_stream_semi(self);
  return (stmt_t *)procedure_stmt_new(ident, body);
}

expr_t *token_stream_expr(token_stream_t *self) {
  token_t token = token_stream_next(self);
  switch (token.type) {
  case token_type_string:
    return (expr_t *)string_expr_new(token.string);
  case token_type_ident: {
    token_stream_unshift(self, token);
    path_t path = token_stream_path(self);
    return (expr_t *)path_expr_new(path);
  }
  default:
    die_with_token(token, "start of expression");
  }
}

array_t token_stream_brackets(token_stream_t *self) {
  token_t start = token_stream_next(self);
  if (start.type != token_type_token) {
    die_with_token(start, "open bracket");
  }
  char end_token;
  switch (start.token) {
  case '(':
    end_token = ')';
    break;
  case '[':
    end_token = ']';
    break;
  default:
    die_with_token(start, "open bracket");
  }

  array_t values = {};
  while (true) {
    token_t end = token_stream_next(self);
    if (end.type == token_type_token && end.token == end_token) {
      break;
    }

    token_stream_unshift(self, end);
    expr_t *expr = token_stream_expr(self);
    array_push(&values, expr);
  }
  return values;
}

stmt_t *token_stream_non_keyword_statement(token_stream_t *self) {
  path_t path = token_stream_path(self);
  array_t args = token_stream_brackets(self);
  token_stream_semi(self);
  return (stmt_t *)function_call_expr_new(path, args);
}

stmt_t *token_stream_statement(token_stream_t *self) {
  token_stream_whitespace(self);

  token_t token = token_stream_next(self);
  if (token.type == token_type_end) {
    return NULL;
  }
  if (token.type != token_type_ident) {
    die_with_token(token, "start of statement");
  }

  switch (token.ident.id) {
  case static_atom_with:
    return token_stream_with_statement(self);
  case static_atom_procedure:
    return token_stream_procedure_statement(self);
  default:
    if (atom_is_keyword(token.ident)) {
      die("unknown keyword statement: %s\n", atom_get(token.ident));
    }
    token_stream_unshift(self, token);
    return token_stream_non_keyword_statement(self);
  }
}

void tokenize_file(const char *path) {
  token_stream_t token_stream = {};
  file_buffer_open(&token_stream.file_buffer, path);

  array_t stmts = {};
  while (true) {
    stmt_t *stmt = token_stream_statement(&token_stream);
    if (stmt == NULL) {
      break;
    }
    array_push(&stmts, stmt);
  }
  eprintf("end of file\n");
  close(token_stream.file_buffer.fd);

  scope_t global_scope = {};

  eprintf("\n\n");
  for (size_t i = 0; i < stmts.len; i++) {
    ast_node_eval(stmts.data[i], &global_scope);
  }
  eprintf("\n\n");
  // while (true) {
  //   token_t token = token_stream_next(&token_stream);
  //   switch (token.type) {
  //   case token_type_end:
  //     eprintf("\n\n");
  //     eprintf("end of file\n");
  //     close(token_stream.file_buffer.fd);
  //     return;
  //   case token_type_whitespace:
  //     eprintf("whitespace\n");
  //     break;
  //   case token_type_token:
  //     eprintf("token: %ld %s\n", token.value.id, atom_get(token.value));
  //     break;
  //   case token_type_ident:
  //     eprintf("ident: %ld %s\n", token.value.id, atom_get(token.value));
  //     break;
  //   case token_type_number:
  //     eprintf("number: %ld %s\n", token.value.id, atom_get(token.value));
  //     break;
  //   case token_type_string:
  //     eprintf("string: %ld %s\n", token.value.id, atom_get(token.value));
  //     break;
  //   }
  // }
}
