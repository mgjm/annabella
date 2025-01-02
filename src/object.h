#pragma once

#include <stddef.h>

typedef struct object_vtable {
  const char *class_name;
  void (*drop)(void *self);
} object_vtable_t;

typedef struct object {
  object_vtable_t *vtable;
} object_t;

static inline const char *object_class_name(object_t *self) {
  if (self == NULL) {
    return "<null>";
  }
  if (self->vtable == NULL) {
    return "<wtf: missing vtable>";
  }
  return self->vtable->class_name;
}

static inline void object_drop(object_t *self) { self->vtable->drop(self); }

typedef struct array {
  object_t **data;
  size_t len;
  size_t cap;
} array_t;

// item needs to be an instance of object_t
extern void array_push(array_t *self, void *item);

extern void array_drop(array_t *self);

static inline object_t *array_as_object(array_t *self) {
  return (object_t *)self;
};
