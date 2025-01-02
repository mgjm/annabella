
#include "scope.h"
#include "atom.h"
#include "macros.h"
#include "object.h"
#include <stdlib.h>

void scope_insert(scope_t *self, atom_t key, value_t *value) {
  for (size_t i = 0; i < self->len; i++) {
    if (atom_eq(self->data[i].key, key)) {
      die("item already defined in scope: %s\n", atom_get(key));
    }
  }

  if (self->len == self->cap) {
    size_t new_cap = self->cap > 0 ? self->cap * 2 : 1;
    self->data = realloc(self->data, new_cap * sizeof(*self->data));
    if (self->data == NULL) {
      die_errno("failed to reallocate scope: %s\n");
    }
    self->cap = new_cap;
  }

  if (self->len >= self->cap) {
    die("scope insert out of bounds: %ld >= %ld\n", self->len, self->cap);
  }

  self->data[self->len++] = (scope_entry_t){key, value};
}

value_t *scope_try_get(scope_t *self, atom_t key) {
  while (self != NULL) {
    for (size_t i = 0; i < self->len; i++) {
      scope_entry_t *entry = &self->data[i];
      if (atom_eq(entry->key, key)) {
        return entry->value;
      }
    }
    self = self->parent;
  }
  return NULL;
}

value_t *scope_get(scope_t *self, atom_t key) {
  value_t *value = scope_try_get(self, key);
  if (value == NULL) {
    for (size_t i = 0; i < self->len; i++) {
      scope_entry_t *entry = &self->data[i];
    }
    die("failed to get %s from scope\n", atom_get(key));
  }
  return value;
}

void scope_drop(scope_t *self) {
  for (size_t i = 0; i < self->len; i++) {
    object_drop((object_t *)self->data[i].value);
  }
  free(self->data);
  *self = (scope_t){};
}
