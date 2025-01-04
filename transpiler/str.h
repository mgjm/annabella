#pragma once

#include "macros.h"
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>

// valid for ever, don't call free
typedef const char *static_str_t;

// allocated string, free after use
typedef char *string_t;

// valid during function call, will be freed later, don't call free
typedef const char *str_t;

static inline void __attribute__((format(printf, 2, 3)))
string_append(string_t *str, str_t fmt, ...) {
  va_list args;

  if (*str == NULL) {
    va_start(args, fmt);
    if (vasprintf(str, fmt, args) < 0) {
      die_errno("failed to asprintf: %s\n");
    }
    va_end(args);
    return;
  }

  va_start(args, fmt);
  size_t new_len = vsnprintf(NULL, 0, fmt, args) + 1;
  if (new_len < 0) {
    die_errno("failed to calculate length of printf: %s\n");
  }
  va_end(args);

  size_t old_len = strlen(*str);
  string_t new = realloc(*str, old_len + new_len);
  if (new == NULL) {
    die_errno("failed to reallocate for printf: %s\n");
  }
  *str = new;
  va_start(args, fmt);
  if (vsnprintf(*str + old_len, new_len, fmt, args) < 0) {
    die_errno("failed to printf: %s\n");
  }
  va_end(args);
}
