#include "value.h"
#include "atom.h"
#include "macros.h"
#include "object.h"
#include "scope.h"
#include <stdlib.h>
#include <string.h>

static value_t *value_call_unsupported(void *_self, array_t *args) {
  object_t *self = _self;
  die("%s does not support calling\n", object_class_name(self));
}

static value_t *value_get_by_key_unsupported(void *_self, atom_t key) {
  object_t *self = _self;
  die("%s does not support get by key\n", object_class_name(self));
}

static void packet_value_drop(void *_self) {
  packet_value_t *self = _self;
  scope_drop(&self->scope);
  free(self);
}

static char *packet_value_to_string(void *_self) { return strdup("<packet>"); }

static value_t *packet_value_get_by_key(void *_self, atom_t key) {
  packet_value_t *self = _self;
  return scope_get(&self->scope, key);
}

static value_vtable_t packet_value_vtable = {
    "packet_value",
    //
    packet_value_drop,
    packet_value_to_string,
    value_call_unsupported,
    packet_value_get_by_key,
};

packet_value_t *packet_value_new() {
  packet_value_t *self = malloc(sizeof(packet_value_t));
  *self = (packet_value_t){
      &packet_value_vtable,
  };
  return self;
}

static void c_function_value_drop(void *_self) {
  c_function_value_t *self = _self;
  free(self);
}

static char *c_function_value_to_string(void *_self) {
  return strdup("<c_function>");
}

static value_t *c_function_value_call(void *_self, array_t *args) {
  c_function_value_t *self = _self;
  return self->call(self->this, args);
}

static value_vtable_t c_function_value_vtable = {
    "c_function_value",
    //
    c_function_value_drop,
    c_function_value_to_string,
    c_function_value_call,
    value_get_by_key_unsupported,
};

c_function_value_t *c_function_value_new(c_function_call_t call, void *this) {
  c_function_value_t *self = malloc(sizeof(c_function_value_t));
  *self = (c_function_value_t){
      &c_function_value_vtable,
      call,
      this,
  };
  return self;
}

static void string_value_drop(void *_self) {
  string_value_t *self = _self;
  free(self);
}

static char *string_value_to_string(void *_self) {
  string_value_t *self = _self;
  return strdup(self->value);
}

static value_vtable_t string_value_vtable = {
    "string_value",
    //
    string_value_drop,
    string_value_to_string,
    value_call_unsupported,
    value_get_by_key_unsupported,
};

string_value_t *string_value_from_owned(char *value) {
  string_value_t *self = malloc(sizeof(string_value_t));
  *self = (string_value_t){
      &string_value_vtable,
      value,
  };
  return self;
}

string_value_t *string_value_from_ref(const char *value) {
  return string_value_from_owned(strdup(value));
}

string_value_t *string_value_from_atom(atom_t value) {
  return string_value_from_ref(atom_get(value));
}
