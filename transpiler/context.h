#pragma once

#include "str.h"

typedef struct context {
  string_t functions;
  string_t init;
  string_t value;
} context_t;

extern void context_finalize(context_t *self);
