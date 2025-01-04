#include "ast-node.h"
#include "macros.h"
#include <stdlib.h>

void ast_node_array_push(ast_node_array_t *self, ast_node_t *node) {
  if (self->len == self->cap) {
    size_t cap = self->cap > 0 ? self->cap * 2 : 1;
    self->nodes = realloc(self->nodes, cap * sizeof(*self->nodes));
    if (self->nodes == NULL) {
      die_errno("failed to reallic ast node array: %s\n");
    }
    self->cap = cap;
  }

  if (self->len >= self->cap) {
    die("ast array out of bounds: %ld >= %ld\n", self->len, self->cap);
  }

  self->nodes[self->len++] = node;
}

void ast_node_array_drop(ast_node_array_t *self) {
  for (size_t i = 0; i < self->len; i++) {
    ast_node_drop(self->nodes[i]);
  }
  *self = (ast_node_array_t){};
}
