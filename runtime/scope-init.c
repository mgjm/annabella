#include "annabella-rt.h"
#include "private.h"
#include "value.h"
#include <stdlib.h>

typedef struct string_type_value {
  value_t super;
} string_type_value_t;

static value_vtable_t string_type_value_vtable;

static void string_type_value_drop(void *_self) {
  string_type_value_t *self = _self;
  free(self);
}

static value_t *string_type_value_default(void *_self) {
  string_type_value_t *self = _self;
  value_rm_ref(&self->super);
  return annabella_string_value("");
}

static value_vtable_t string_type_value_vtable = {
    "string_type",
    string_type_value_drop,
    value_vtable_required_end,

    .default_ = string_type_value_default,
};

static value_t *annabella_string_type_value() {
  string_type_value_t *self = malloc(sizeof(string_type_value_t));
  *self = (string_type_value_t){
      value_base_new(&string_type_value_vtable),
  };
  return &self->super;
}

static void scope_init_common(scope_t *self) {

  annabella_scope_insert_value(self, "String", annabella_string_type_value());
}

void annabella_main_scope_init(scope_t *self) { scope_init_common(self); }

void annabella_package_scope_init(scope_t *self) { scope_init_common(self); }
