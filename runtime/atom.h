#pragma once

#include <stdbool.h>
#include <stddef.h>

typedef struct atom {
  size_t id;
} atom_t;

extern atom_t atom_new(const char *str);

extern const char *atom_get(atom_t self);

static inline bool atom_eq(atom_t a, atom_t b) { return a.id == b.id; }
