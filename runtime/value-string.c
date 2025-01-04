#include "value.h"
#include <stdlib.h>
#include <string.h>

typedef struct string_value {
  annabella_value_t super;
  char *value;
} string_value_t;

static void string_value_drop(void *_self) {
  string_value_t *self = _self;
  free(self);
}

static char *string_value_to_string(void *_self) {
  string_value_t *self = _self;
  return strdup(self->value);
}

static value_vtable_t string_value_vtable = {
    "string",
    string_value_drop,
    string_value_to_string,
    value_call_unsupported,
    value_get_by_key_unsupported,
};

value_t *annabella_string_value_from_owned(char *value) {
  string_value_t *self = malloc(sizeof(string_value_t));
  *self = (string_value_t){
      &string_value_vtable,
      value,
  };
  return &self->super;
}

value_t *annabella_string_value_from_ref(const char *value) {
  return annabella_string_value_from_owned(strdup(value));
}

value_t *annabella_string_value_from_atom(atom_t value) {
  return annabella_string_value_from_ref(atom_get(value));
}
