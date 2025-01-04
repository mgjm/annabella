#include "value.h"
#include "atom.h"
#include "macros.h"
#include "private.h"
#include <string.h>

static const value_vtable_t null_value_vtable;

static inline const value_vtable_t *value_vtable(value_t *self) {
  return self ? self->vtable : &null_value_vtable;
}

const char *value_class_name(value_t *self) {
  return value_vtable(self)->class_name;
}

char *value_to_string(value_t *self) {
  return value_vtable(self)->to_string(self);
}

value_t *annabella_value_call(value_t *self, size_t argc, ...) {
  va_list args;
  va_start(args, argc);
  value_t *result = value_vtable(self)->call(self, argc, args);
  va_end(args);
  return result;
}

value_t *value_try_get_by_key(value_t *self, atom_t key) {
  return value_vtable(self)->try_get_by_key(self, key);
}

value_t *value_get_by_key(value_t *self, atom_t key) {
  value_t *value = value_try_get_by_key(self, key);
  if (value == NULL) {
    die("failed to get %s from %s\n", atom_get(key), value_to_string(self));
  }
  return value;
}

value_t *annabella_value_get(value_t *self, const char *key) {
  return value_get_by_key(self, atom_new(key));
}

void annabella_value_drop(value_t *self) {
  if (self != NULL) {
    self->vtable->drop(self);
  }
}

value_t *value_call_unsupported(void *_self, size_t argc, va_list args) {
  value_t *self = _self;
  die("%s does not support calling\n", value_class_name(self));
}

value_t *value_get_by_key_unsupported(void *_self, atom_t key) {
  value_t *self = _self;
  die("%s does not support get by key\n", value_class_name(self));
}

static char *null_value_to_string(void *_self) { return strdup("<null>"); }

static const value_vtable_t null_value_vtable = {
    "null",
    NULL,
    null_value_to_string,
    value_call_unsupported,
    value_get_by_key_unsupported,
};
