#include "annabella-rt.h"
#include "macros.h"
#include "private.h"
#include "value.h"
#include <stdlib.h>
#include <string.h>

typedef struct integer_value {
  annabella_value_t super;
  size_t value;
} integer_value_t;

static value_vtable_t integer_value_vtable;

static void integer_value_drop(void *_self) {
  integer_value_t *self = _self;
  free(self);
}

static char *integer_value_to_string(void *_self) {
  integer_value_t *self = _self;
  char *str;
  if (asprintf(&str, "%ld", self->value) < 0) {
    die_errno("failed to stringify integer: %s\n");
  }
  return str;
}

static void integer_value_assign(void *_self, value_t *value) {
  integer_value_t *self = _self;
  if (value->vtable != &integer_value_vtable) {
    die("integer assignment with %s %s not supported\n",
        value_class_name(value), value_to_string(value));
  }
  self->value = ((integer_value_t *)value)->value;
  value_rm_ref(&self->super);
  value_rm_ref(value);
}

static bool integer_value_to_bool(void *_self) {
  integer_value_t *self = _self;
  return self->value != 0;
}

static value_vtable_t integer_value_vtable = {
    "integer",
    integer_value_drop,
    value_vtable_required_end,

    .to_string = integer_value_to_string,
    .assign = integer_value_assign,
    .to_bool = integer_value_to_bool,
};

value_t *annabella_integer_value(integer_t value) {
  integer_value_t *self = malloc(sizeof(integer_value_t));
  *self = (integer_value_t){
      value_base_new(&integer_value_vtable),
      value,
  };
  return &self->super;
}
