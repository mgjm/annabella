#pragma once

#include "atom.h"
#include "private.h"

typedef struct annabella_scope_entry {
  atom_t key;
  value_t *value;
} annabella_scope_entry_t;

extern void scope_insert(scope_t *self, atom_t key, value_t *value);

extern value_t *scope_try_get(scope_t *self, atom_t key);
extern value_t *scope_get(scope_t *self, atom_t key);

extern void scope_drop(scope_t *self);
