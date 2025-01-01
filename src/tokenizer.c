#include "tokenizer.h"
#include "atom.h"
#include "macros.h"
#include <fcntl.h>
#include <stdbool.h>
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
    die("file buffer out of bounds\n");
  }

  uint8_t b = self->buffer[self->start];
  if (b == 0) {
    die("source code contained null byte\n");
  }
  return b;
}

uint8_t file_buffer_read_byte(file_buffer_t *self) {
  uint8_t b = file_buffer_peek_byte(self);
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
    return (token_t){token_type_string, atom};

  case byte_type_token: // everything else
    buffer[0] = b;
    atom = atom_new(buffer);
    return (token_t){token_type_token, atom};

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
    return (token_t){token_type_ident, atom};

  case byte_type_number: // [0-9]
    die("todo: number parser");
    return (token_t){token_type_number, atom};
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

atom_t token_stream_ident(token_stream_t *self) {
  token_stream_whitespace(self);
  token_t token = token_stream_next(self);
  if (token.type != token_type_ident) {
    die("unexpected token type: %d\n", token.type);
  }
  return token.value;
}

void token_stream_consume_ident(token_stream_t *self, atom_t expected) {
  atom_t ident = token_stream_ident(self);
  if (!atom_eq(ident, expected)) {
    die("unexpected ident: %s != %s\n", atom_get(ident), atom_get(expected));
  }
}

void token_stream_path(token_stream_t *self) {
  token_stream_whitespace(self);

  while (true) {
    token_t token = token_stream_next(self);

    if (token.type != token_type_ident) {
      die("unexpected token type: %d\n", token.type);
    }
    eprintf("path component: %s\n", atom_get(token.value));

    token = token_stream_next(self);
    if (token.type != token_type_token || !atom_eq(token.value, atom_dot)) {
      token_stream_unshift(self, token);
      break;
    }
  }
}

void token_stream_semi(token_stream_t *self) {

  token_t token = token_stream_next(self);
  if (token.type != token_type_token) {
    die("unexpected token type: %d\n", token.type);
  }
  if (!atom_eq(token.value, atom_semi)) {
    die("unexpected token value: %s\n", atom_get(token.value));
  }
}

void token_stream_with_statement(token_stream_t *self) {
  eprintf("with statement:\n");
  token_stream_path(self);
  token_stream_semi(self);
  eprintf("\n");
}

bool token_stream_statement(token_stream_t *self);

void token_stream_procedure_statement(token_stream_t *self) {
  eprintf("procedure statement:\n");
  atom_t ident = token_stream_ident(self);
  eprintf("procedure %s\n", atom_get(ident));
  token_stream_consume_ident(self, atom_is);
  token_stream_consume_ident(self, atom_begin);
  // ----- TODO -----
  // while next_token != end {
  //  token_stream_statement();
  // }
  // ----- END OF TODO -----
  token_stream_consume_ident(self, atom_end);
  token_stream_semi(self);
  eprintf("\n");
}

bool token_stream_statement(token_stream_t *self) {
  token_stream_whitespace(self);

  token_t token = token_stream_next(self);
  if (token.type == token_type_end) {
    return false;
  }
  if (token.type != token_type_ident) {
    die("unexpected token type: %d\n", token.type);
  }

  switch (token.value.id) {
  case static_atom_with:
    token_stream_with_statement(self);
    break;
  case static_atom_procedure:
    token_stream_procedure_statement(self);
    break;
  default:
    eprintf("unknown statement: %s\n", atom_get(token.value));
    break;
  }
  return true;
}

void tokenize_file(const char *path) {
  token_stream_t token_stream = {};
  file_buffer_open(&token_stream.file_buffer, path);

  eprintf("\n\n");
  while (token_stream_statement(&token_stream)) {
  }
  eprintf("\n\n");
  eprintf("end of file\n");
  close(token_stream.file_buffer.fd);

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
