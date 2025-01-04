#include "value.h"
#include <stdlib.h>
#include <string.h>

typedef struct c_function_value {
  value_t super;
  annabella_c_function_call_t call;
  void *this;
} c_function_value_t;

static void c_function_value_drop(void *_self) {
  c_function_value_t *self = _self;
  free(self);
}

static char *c_function_value_to_string(void *_self) {
  return strdup("<c_function>");
}

static value_t *c_function_value_call(void *_self, size_t argc, va_list args) {
  c_function_value_t *self = _self;
  return self->call(self->this, argc, args);
}

static value_vtable_t c_function_value_vtable = {
    "c_function_value",
    //
    c_function_value_drop,
    c_function_value_to_string,
    c_function_value_call,
    value_get_by_key_unsupported,
};

value_t *annabella_c_function_value_new(annabella_c_function_call_t call,
                                        void *this) {
  c_function_value_t *self = malloc(sizeof(c_function_value_t));
  *self = (c_function_value_t){
      &c_function_value_vtable,
      call,
      this,
  };
  return &self->super;
}
