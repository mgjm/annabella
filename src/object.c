#include "object.h"
#include "macros.h"
#include <stdlib.h>

void array_push(array_t *self, void *item) {
  if (self->len == self->cap) {
    size_t new_cap = self->cap > 0 ? self->cap * 2 : 1;
    self->data = realloc(self->data, new_cap * sizeof(*self->data));
    if (self->data == NULL) {
      die_errno("failed to reallocate array: %s\n");
    }
    self->cap = new_cap;
  }

  if (self->len >= self->cap) {
    die("array push out of bounds: %ld >= %ld\n", self->len, self->cap);
  }

  self->data[self->len++] = item;
}

void array_drop(array_t *self) {
  for (size_t i = 0; i < self->len; i++) {
    object_drop(self->data[i]);
  }
  free(self->data);
  *self = (array_t){};
}
