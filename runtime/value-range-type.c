#include "annabella-rt.h"
#include "macros.h"
#include "private.h"
#include "value.h"
#include <stdlib.h>
#include <string.h>

typedef struct range_type_value {
  annabella_value_t super;
  size_t min;
  size_t max;
} range_type_value_t;

static value_vtable_t range_type_value_vtable;

static void range_type_value_drop(void *_self) {
  range_type_value_t *self = _self;
  free(self);
}

static char *range_type_value_to_string(void *_self) {
  range_type_value_t *self = _self;
  char *str;
  if (asprintf(&str, "range %ld .. %ld", self->min, self->max) < 0) {
    die_errno("failed to stringify range_type: %s\n");
  }
  return str;
}

static value_t *range_type_value_default(void *_self) {
  range_type_value_t *self = _self;
  value_rm_ref(&self->super);
  return annabella_integer_value(0);
}

static value_vtable_t range_type_value_vtable = {
    "range_type",
    range_type_value_drop,
    value_vtable_required_end,

    .to_string = range_type_value_to_string,
    .default_ = range_type_value_default,
};

value_t *annabella_range_type_value(integer_t min, integer_t max) {
  range_type_value_t *self = malloc(sizeof(range_type_value_t));
  *self = (range_type_value_t){
      value_base_new(&range_type_value_vtable),
      min,
  };
  return &self->super;
}
