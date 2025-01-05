#include "macros.h"
#include "value.h"
#include <stdlib.h>
#include <string.h>

typedef struct string_value {
  annabella_value_t super;
  char *value;
} string_value_t;

static value_vtable_t string_value_vtable;

static void string_value_drop(void *_self) {
  string_value_t *self = _self;
  free(self->value);
  free(self);
}

static char *string_value_to_string(void *_self) {
  string_value_t *self = _self;
  return strdup(self->value);
}

static void string_value_assign(void *_self, value_t *value) {
  string_value_t *self = _self;
  if (value->vtable != &string_value_vtable) {
    die("string assignment with %s %s not supported\n", value_class_name(value),
        value_to_string(value));
  }
  self->value = strdup(((string_value_t *)value)->value);
  value_rm_ref(&self->super);
  value_rm_ref(value);
}

static value_vtable_t string_value_vtable = {
    "string",
    string_value_drop,
    value_vtable_required_end,

    .to_string = string_value_to_string,
    .assign = string_value_assign,
};

value_t *string_value_from_owned(char *value) {
  string_value_t *self = malloc(sizeof(string_value_t));
  *self = (string_value_t){
      value_base_new(&string_value_vtable),
      value,
  };
  return &self->super;
}

value_t *string_value_from_ref(const char *value) {
  return string_value_from_owned(strdup(value));
}

value_t *string_value_from_atom(atom_t value) {
  return string_value_from_ref(atom_get(value));
}

annabella_value_t *annabella_string_value(const char *value) {
  return string_value_from_ref(value);
}
