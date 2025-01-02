#pragma once
// class Value:
//   drop()
//   call(args) -> Value
//   callUnaryOperator(op) -> Value // -5
//   callBinaryOperator(op, rhs) -> Value // 2 + 3
//   callMethod(name, args) -> Value
//   getField(name) -> Value
//   setField(name, value)
//
// StringValue:
// AtomStringValue:      // maybe?
// AllocatedStringValue: // maybe?
// IntValue:
// FloatValue:
// FunctionValue: // function definition
// ClassValue: // class definition
// ObjectValue: // instance of a class

#include "atom.h"
#include "object.h"

#include "scope.h"
#include <string.h>

typedef struct value_vtable {
  object_vtable_t object;
  char *(*to_string)(void *self);
  value_t *(*call)(void *self, array_t *args);
  value_t *(*get_by_key)(void *self, atom_t key);
} value_vtable_t;

typedef struct value {
  value_vtable_t *vtable;
} value_t;

static inline char *value_to_string(value_t *self) {
  if (self == NULL) {
    return strdup("<null>");
  }
  return self->vtable->to_string(self);
}

static inline value_t *value_call(value_t *self, array_t *args) {
  return self->vtable->call(self, args);
}

static inline value_t *value_get_by_key(value_t *self, atom_t key) {
  return self->vtable->get_by_key(self, key);
}

typedef struct packet_value {
  value_vtable_t *vtable;
  scope_t scope;
} packet_value_t;

extern packet_value_t *packet_value_new();

typedef value_t *(*c_function_call_t)(void *self, array_t *args);

typedef struct c_function_value {
  value_vtable_t *vtable;
  c_function_call_t call;
  void *this;
} c_function_value_t;

extern c_function_value_t *c_function_value_new(c_function_call_t call,
                                                void *this);

typedef struct string_value {
  value_vtable_t *vtable;
  char *value;
} string_value_t;

extern string_value_t *string_value_from_owned(char *value);
extern string_value_t *string_value_from_ref(const char *value);
extern string_value_t *string_value_from_atom(atom_t value);
