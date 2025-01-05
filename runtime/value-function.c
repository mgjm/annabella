#include "annabella-rt.h"
#include "macros.h"
#include "value.h"
#include <stdlib.h>
#include <string.h>

typedef struct function_value {
  value_t super;
  annabella_function_call_t call;
  size_t argc;
} function_value_t;

static void function_value_drop(void *_self) {
  function_value_t *self = _self;
  // functions live for ever
}

static value_t *function_value_to_value(void *_self, scope_t *scope) {
  function_value_t *self = _self;
  return annabella_value_call(&self->super, scope, 0);
}

static value_t *function_value_call(void *_self, scope_t *scope, size_t argc,
                                    va_list args) {
  function_value_t *self = _self;
  if (self->argc != argc) {
    die("number of arguments does not match %ld != %ld\n", argc, self->argc);
  }
  return self->call(scope, args);
}

static value_vtable_t function_value_vtable = {
    "function_value",
    function_value_drop,
    value_vtable_required_end,

    .to_value = function_value_to_value,
    .call = function_value_call,
};

value_t *annabella_function_value(annabella_function_call_t call, size_t argc,
                                  ...) {
  function_value_t *self = malloc(sizeof(function_value_t));
  *self = (function_value_t){
      &function_value_vtable,
      call,
      argc,
  };
  return &self->super;
}
