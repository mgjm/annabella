#pragma once

#include "atom.h"

typedef struct value value_t;
typedef struct scope scope_t;

typedef struct scope_entry {
  atom_t key;
  value_t *value;
} scope_entry_t;

typedef struct scope {
  scope_t *parent;
  scope_entry_t *data;
  size_t len;
  size_t cap;
} scope_t;

extern void scope_insert(scope_t *self, atom_t key, value_t *value);

extern value_t *scope_get(scope_t *self, atom_t key);

extern void scope_drop(scope_t *self);
