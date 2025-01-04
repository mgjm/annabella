#pragma once

#include "atom.h"
#include "private.h"
#include <stdarg.h>

typedef struct value_vtable {
  const char *class_name;
  void (*drop)(void *self);
  char *(*to_string)(void *self);
  value_t *(*call)(void *self, size_t argc, va_list args);
  value_t *(*try_get_by_key)(void *self, atom_t key);
} value_vtable_t;

struct annabella_value {
  const value_vtable_t *vtable;
};

extern const char *value_class_name(value_t *self);
extern char *value_to_string(value_t *self);
extern value_t *value_try_get_by_key(value_t *self, atom_t key);
extern value_t *value_get_by_key(value_t *self, atom_t key);

extern value_t *value_call_unsupported(void *self, size_t argc, va_list args);
extern value_t *value_get_by_key_unsupported(void *self, atom_t key);

extern value_t *package_value_new();
extern void package_value_set_package(value_t *_self, package_t *package);
extern value_t *package_value_get_package(value_t *_self, atom_t key);
