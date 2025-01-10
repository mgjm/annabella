#include "scope.h"
#include "annabella-rt.h"
#include "atom.h"
#include "macros.h"
#include "private.h"
#include "value.h"
#include <stdlib.h>
#include <string.h>

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

static value_t *scope_or_value_get_or_insert_package(scope_t *self,
                                                     value_t *value,
                                                     atom_t key) {
  if (value == NULL) {
    value = scope_try_get(self, key);
    if (value == NULL) {
      value = package_value_new();
      scope_insert(self, key, value);
    }
    return value;
  } else {
    return package_value_get_package(value, key);
  }
}

void annabella_scope_insert_package(scope_t *self, package_t *package) {
  char buffer[128];
  const char *start = package->name;
  const char *end;

  value_t *value = NULL;

  while ((end = strchr(start, '.'))) {
    size_t len = end - start;
    if (len >= array_len(buffer)) {
      die("package path component too long: %s\n", package->name);
    }
    strncpy(buffer, start, len);
    buffer[len] = '\0';
    start += len + 1;

    value = scope_or_value_get_or_insert_package(self, value, atom_new(buffer));
  }
  value = scope_or_value_get_or_insert_package(self, value, atom_new(start));

  package_value_set_package(value, package);
}

void annabella_scope_insert_value(annabella_scope_t *self, const char *name,
                                  annabella_value_t *value) {
  scope_insert(self, atom_new(name), value);
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
      eprintf("- %s: %s\n", atom_get(entry->key),
              value_class_name(entry->value));
    }
    die("failed to get %s from scope (%ld entries)\n", atom_get(key),
        self->len);
  }
  return value;
}

value_t *annabella_scope_get(scope_t *self, const char *key) {
  return value_add_ref(scope_get(self, atom_new(key)));
}

void annabella_scope_exec_main(annabella_scope_t *self) {
  annabella_value_drop(
      annabella_value_call(self->data[self->len - 1].value, 0));
}

void annabella_scope_drop(scope_t *self) {
  for (size_t i = 0; i < self->len; i++) {
    annabella_value_drop(self->data[i].value);
  }
  free(self->data);
  *self = (scope_t){};
}
