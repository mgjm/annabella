#include "tokenizer.h"
#include "keywords.h"
#include "macros.h"
#include "str.h"
#include <fcntl.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

static static_str_t read_to_string(str_t path) {
  eprintf("open file: %s\n", path);

  int fd = open(path, O_RDONLY | O_CLOEXEC);
  if (fd < 0) {
    die_errno("failed to open file: %s\n");
  }

  size_t len = lseek(fd, 0, SEEK_END);
  if (len < 0) {
    die_errno("failed to seek to end: %s\n");
  }

  static_str_t content = mmap(NULL, len, PROT_READ, MAP_PRIVATE, fd, 0);
  if (content == NULL) {
    die_errno("failed to mmap file: %s\n");
  }

  eprintf("end of file\n");
  if (close(fd) != 0) {
    die_errno("failed to close file: %s\n");
  }

  return content;
}

token_stream_t token_stream_open(str_t path) {
  static_str_t content = read_to_string(path);
  string_t buffer = calloc(strlen(content), sizeof(char));
  return (token_stream_t){content, buffer};
}

typedef enum byte_type {
  byte_type_null,         // \0
  byte_type_whitespace,   // \t \n ' '
  byte_type_double_quote, // "
  byte_type_token,        // everything else
  byte_type_ident,        // [a-z] [A-Z] _
  byte_type_number,       // [0-9]
} byte_type_t;

byte_type_t byte_type(char c) {
  if (c == 0) {
    return byte_type_null;
  } else if (c == ' ' || c == '\t' || c == '\n') {
    return byte_type_whitespace;
  } else if (c == '"') {
    return byte_type_double_quote;
  } else if ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_') {
    return byte_type_ident;
  } else if (c >= '0' && c <= '9') {
    return byte_type_number;
  } else {
    return byte_type_token;
  }
}

token_t token_stream_next(token_stream_t *self) {
  char c = *self->content++;
  switch (byte_type(c)) {
  case byte_type_null: // \0
    return (token_t){token_type_end};

  case byte_type_whitespace:
    while (byte_type(self->content[0]) == byte_type_whitespace) {
      self->content++;
    }
    return (token_t){token_type_whitespace};

  case byte_type_double_quote: {
    static_str_t start = self->content;
    bool has_escape_sequence = false;
    while (true) {
      c = self->content[0];
      switch (c) {
      case 0:
        die("unterminated string\n");
      case '"':
        self->content++;
        if (self->content[0] != '"') {
          break;
        } else {
          has_escape_sequence = true;
          // fallthrough
        }
      default:
        self->content++;
        continue;
      }
      break;
    }

    string_t string = self->buffer;
    strncpy(string, start, self->content - start - 1);
    self->buffer += self->content - start;

    if (has_escape_sequence) {
      string_t dest = strchr(string, '"');
      if (dest == NULL) {
        die("string with escape no longer contains a quote char");
      }

      string_t src = dest;
      while (*src != '\0') {
        *dest++ = *src;
        if (*src == '"') {
          if (*++src != '"') {
            die("string contained quote without another quote");
          }
        }
        src++;
      }
      *dest = '\0';
    }

    return (token_t){token_type_string, .string = string};
  }

  case byte_type_token:
    if (c == '-' && self->content[0] == '-') {
      // comment
      static_str_t nl = strchr(self->content, '\n');
      if (nl == NULL) {
        die("comment without trailing new line\n");
      }
      self->content = nl + 1;
      return token_stream_next(self);
    }
    return (token_t){token_type_token, .token = c};

  case byte_type_ident: {
    static_str_t start = self->content - 1;
    while (true) {
      switch (byte_type(self->content[0])) {
      case byte_type_ident:
      case byte_type_number:
        self->content++;
        continue;
      default:
        break;
      }
      break;
    }

    string_t string = self->buffer;
    strncpy(string, start, self->content - start);
    self->buffer += self->content - start + 1;

    keyword_t keyword = keyword_new(string);
    if (keyword != _not_a_keyword) {
      return (token_t){token_type_keyword, .keyword = keyword};
    } else {
      return (token_t){token_type_ident, .ident = string};
    }
  }

  case byte_type_number: {
    static_str_t start = self->content - 1;
    bool had_dot = false;
    while (true) {
      c = self->content[0];
      switch (byte_type(c)) {
      case byte_type_number:
        self->content++;
        continue;
      case byte_type_token:
        if (c == '.' && !had_dot) {
          had_dot = true;
          self->content++;
          continue;
        }
        break;
      default:
        break;
      }
      break;
    }

    string_t string = strndup(start, self->content - start);
    if (string == NULL) {
      die("failed to allocate number token\n");
    }

    return (token_t){token_type_number, .number = string};
  }
  }
}

char *token_to_string(token_t self) {
  char *str;
  switch (self.type) {
  case token_type_end:
    return "end token";
  case token_type_whitespace:
    return "whitespace";
  case token_type_token:
    asprintf(&str, "token '%c'", self.token);
    break;
  case token_type_keyword:
    asprintf(&str, "keyword `%s`", keyword_get(self.keyword));
    break;
  case token_type_ident:
    asprintf(&str, "ident `%s`", self.ident);
    break;
  case token_type_number:
    asprintf(&str, "number %s", self.ident);
    break;
  case token_type_string:
    asprintf(&str, "string \"%s\"", self.ident);
    break;
  }
  return str;
}
