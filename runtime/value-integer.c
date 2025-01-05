#include "macros.h"
#include "private.h"
#include "value.h"
#include <stdlib.h>
#include <string.h>

typedef struct integer_value {
  annabella_value_t super;
  size_t value;
} integer_value_t;

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

static value_vtable_t integer_value_vtable = {
    "integer",
    integer_value_drop,
    value_vtable_required_end,

    .to_string = integer_value_to_string,
};

value_t *annabella_integer_value(integer_t value) {
  integer_value_t *self = malloc(sizeof(integer_value_t));
  *self = (integer_value_t){
      &integer_value_vtable,
      value,
  };
  return &self->super;
}
