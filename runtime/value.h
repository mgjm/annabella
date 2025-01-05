#pragma once

#include "atom.h"
#include "private.h"
#include <stdarg.h>

struct value_vtable_required_end {};
static const struct value_vtable_required_end value_vtable_required_end = {};

struct value_vtable_optional_start {};

typedef struct value_vtable {
  // required: (always fill in required_end marker)
  struct {
    const char *class_name;
    void (*drop)(void *self);
    struct value_vtable_required_end _end;
  } required;

  // optional:
  struct value_vtable_optional_start _start;

  char *(*to_string)(void *self);
  // value_t *(*clone)(void *self);
  value_t *(*to_value)(void *self, scope_t *scope);
  value_t *(*call)(void *self, scope_t *scope, size_t argc, va_list args);
  value_t *(*try_get_by_key)(void *self, atom_t key);
  void (*assign)(void *self, value_t *value);
  value_t *(*default_)(void *self);
} value_vtable_t;

struct annabella_value {
  const value_vtable_t *vtable;
  size_t ref_count;
};

extern const char *value_class_name(value_t *self);
extern char *value_to_string(value_t *self);
// extern value_t *value_clone(value_t *self);
extern value_t *value_try_get_by_key(value_t *self, atom_t key);
extern value_t *value_get_by_key(value_t *self, atom_t key);

static inline value_t value_base_new(value_vtable_t *vtable) {
  return (value_t){
      vtable,
      1,
  };
}

extern value_t *value_add_ref(value_t *self);
extern void value_rm_ref(value_t *self);

extern value_t *package_value_new();
extern void package_value_set_package(value_t *_self, package_t *package);
extern value_t *package_value_get_package(value_t *_self, atom_t key);
