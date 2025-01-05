#include "annabella-rt.h"
#include "macros.h"
#include "private.h"
#include "value.h"
#include <stdlib.h>
#include <string.h>

typedef struct bool_value {
  annabella_value_t super;
  bool value;
} bool_value_t;

static value_vtable_t bool_value_vtable;

static void bool_value_drop(void *_self) {
  bool_value_t *self = _self;
  free(self);
}

static char *bool_value_to_string(void *_self) {
  bool_value_t *self = _self;
  return strdup(self->value ? "true" : "false");
}

static void bool_value_assign(void *_self, value_t *value) {
  bool_value_t *self = _self;
  if (value->vtable != &bool_value_vtable) {
    die("bool assignment with %s %s not supported\n", value_class_name(value),
        value_to_string(value));
  }
  self->value = ((bool_value_t *)value)->value;
  value_rm_ref(&self->super);
  value_rm_ref(value);
}

static bool bool_value_to_bool(void *_self) {
  bool_value_t *self = _self;
  return self->value;
}

static value_t *bool_value_cmp(void *_self, cmp_op_t op, value_t *_rhs) {
  bool_value_t *self = _self;
  if (_rhs->vtable != &bool_value_vtable) {
    die("bool assignment with %s %s not supported\n", value_class_name(_rhs),
        value_to_string(_rhs));
  }
  bool_value_t *rhs = (bool_value_t *)_rhs;

  bool result;
  switch (op) {
  case annabella_cmp_op_equal:
    result = self->value == rhs->value;
    break;
  case annabella_cmp_op_not_equal:
    result = self->value != rhs->value;
    break;
  case annabella_cmp_op_less:
    result = self->value < rhs->value;
    break;
  case annabella_cmp_op_less_or_equal:
    result = self->value <= rhs->value;
    break;
  case annabella_cmp_op_greater:
    result = self->value > rhs->value;
    break;
  case annabella_cmp_op_greater_or_equal:
    result = self->value >= rhs->value;
    break;
  default:
    die("unknown comparision operator: %d\n", op);
  }

  value_rm_ref(&self->super);
  value_rm_ref(&rhs->super);
  return annabella_bool_value(result);
}

static value_vtable_t bool_value_vtable = {
    "bool",
    bool_value_drop,
    value_vtable_required_end,

    .to_string = bool_value_to_string,
    .assign = bool_value_assign,
    .to_bool = bool_value_to_bool,
    .cmp = bool_value_cmp,
};

value_t *annabella_bool_value(bool value) {
  bool_value_t *self = malloc(sizeof(bool_value_t));
  *self = (bool_value_t){
      value_base_new(&bool_value_vtable),
      value,
  };
  return &self->super;
}
