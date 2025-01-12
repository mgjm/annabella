#include "value.h"
#include "atom.h"
#include "macros.h"
#include "private.h"
#include <limits.h>
#include <stdarg.h>
#include <string.h>

static const value_vtable_t null_value_vtable;

static inline const value_vtable_t *value_vtable(value_t *self) {
  return self ? self->vtable : &null_value_vtable;
}

const char *value_class_name(value_t *self) {
  return value_vtable(self)->required.class_name;
}

static char *value_to_string_unsupported(void *_self) {
  value_t *self = _self;
  return strdup(value_class_name(self));
}

char *value_to_string(value_t *self) {
  return (value_vtable(self)->to_string ?: value_to_string_unsupported)(self);
}

char *annabella_value_to_string(annabella_value_t *self) {
  return value_to_string(self);
}

// static value_t *value_clone_unimplemted(void *_self) {
//   value_t *self = _self;
//   return self;
// }

// value_t *value_clone(value_t *self) {
//   return (value_vtable(self)->clone ?: value_clone_unimplemted)(self);
// }

static value_t *value_to_value_unimplemted(void *_self, scope_t *scope) {
  value_t *self = _self;
  return self;
}

value_t *annabella_value_to_value(value_t *self, scope_t *scope) {
  return (value_vtable(self)->to_value ?: value_to_value_unimplemted)(self,
                                                                      scope);
}

static value_t *value_call_unsupported(void *_self, size_t argc, va_list args) {
  value_t *self = _self;
  die("%s does not support calling\n", value_class_name(self));
}

value_t *annabella_value_call(value_t *self, size_t argc, ...) {
  va_list args;
  va_start(args, argc);
  value_t *result =
      (value_vtable(self)->call ?: value_call_unsupported)(self, argc, args);
  va_end(args);
  return result;
}

static value_t *value_try_get_by_key_unsupported(void *_self, atom_t key) {
  value_t *self = _self;
  die("%s does not support get by key\n", value_class_name(self));
}

value_t *value_try_get_by_key(value_t *self, atom_t key) {
  return (value_vtable(self)->try_get_by_key
              ?: value_try_get_by_key_unsupported)(self, key);
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

static void value_assign_unsupported(void *_self, value_t *value) {
  value_t *self = _self;
  die("%s does not support assignment\n", value_class_name(self));
}

void annabella_value_assign(value_t *self, value_t *value) {
  return (value_vtable(self)->assign ?: value_assign_unsupported)(self, value);
}

static value_t *value_default_unsupported(void *_self) {
  value_t *self = _self;
  die("%s does not support createing a default value\n",
      value_class_name(self));
}

value_t *annabella_value_default(value_t *self) {
  return (value_vtable(self)->default_ ?: value_default_unsupported)(self);
}

static bool value_to_bool_unsupported(void *_self) {
  value_t *self = _self;
  die("%s does not support being used as a condition\n",
      value_class_name(self));
}

bool annabella_value_to_bool(value_t *self) {
  return (value_vtable(self)->to_bool ?: value_to_bool_unsupported)(self);
}

static value_t *value_cmp_unsupported(void *_self, cmp_op_t op, value_t *rhs) {
  value_t *self = _self;
  die("%s does not support comparison\n", value_class_name(self));
}

value_t *annabella_value_cmp(value_t *self, cmp_op_t op, value_t *rhs) {
  return (value_vtable(self)->cmp ?: value_cmp_unsupported)(self, op, rhs);
}

void annabella_value_drop(value_t *self) { value_rm_ref(self); }

value_t *value_add_ref(value_t *self) {
  if (self == NULL) {
    return self;
  }

  if (self->ref_count++ >= SSIZE_MAX) {
    die("too many references to: %s\n", value_class_name(self));
  }
  // eprintf("ref++ %s: %ld (0x%lx)\n", value_class_name(self), self->ref_count,
  //         (size_t)self);
  return self;
}

void value_rm_ref(value_t *self) {
  if (self == NULL) {
    return;
  }

  if (self->ref_count >= SSIZE_MAX) {
    die("too many references to: %s\n", value_class_name(self));
  }
  if (self->ref_count == 0) {
    die("no references left to remove: %s\n", value_class_name(self));
  }

  self->ref_count--;
  // eprintf("ref-- %s: %ld (0x%lx)\n", value_class_name(self), self->ref_count,
  //         (size_t)self);
  if (self->ref_count == 0) {
    self->vtable->required.drop(self);
  }
}

static const value_vtable_t null_value_vtable = {
    "null",
    NULL, // never used, annabella_value_drop has an null check
    value_vtable_required_end,
};
