#include "annabella-rt.h"
#include "macros.h"
#include "private.h"
#include "value.h"
#include <stdarg.h>
#include <stdlib.h>
#include <string.h>

typedef struct function_value {
  value_t super;
  annabella_function_call_t call;
  scope_t *scope;
  size_t argc;
  value_t **args;
} function_value_t;

static void function_value_drop(void *_self) {
  function_value_t *self = _self;
  for (size_t i = 0; i < self->argc; i++) {
    value_rm_ref(self->args[i]);
  }
  free(self->args);
  free(self);
}

static value_t *function_value_to_value(void *_self, scope_t *scope) {
  function_value_t *self = _self;
  return annabella_value_call(&self->super, 0);
}

static value_t *function_value_call(void *_self, size_t argc, va_list args) {
  function_value_t *self = _self;
  if (self->argc != argc) {
    die("number of arguments does not match %ld != %ld\n", argc, self->argc);
  }
  return self->call(self->scope, args);
}

static value_vtable_t function_value_vtable = {
    "function_value",
    function_value_drop,
    value_vtable_required_end,

    .to_value = function_value_to_value,
    .call = function_value_call,
};

value_t *annabella_function_value(annabella_function_call_t call,
                                  scope_t *scope, size_t argc, ...) {
  va_list args = {};
  va_start(args, argc);
  value_t **args_array = NULL;
  if (argc != 0) {
    args_array = malloc(argc * sizeof(*args_array));
    for (size_t i = 0; i < argc; i++) {
      value_t *arg = va_arg(args, value_t *);
      args_array[i] = arg;
    }
  }
  va_end(args);

  function_value_t *self = malloc(sizeof(function_value_t));
  *self = (function_value_t){
      value_base_new(&function_value_vtable), call, scope, argc, args_array,
  };
  return &self->super;
}
