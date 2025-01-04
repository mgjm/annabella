#include "ast-node.h"
#include "macros.h"
#include "str.h"
#include "token-stream.h"
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct ast_path {
  ast_node_t super;
  static_str_t *comonents;
  size_t len;
} ast_path_t;

static void ast_path_drop(void *_self) {
  ast_path_t *self = _self;
  free(self->comonents);
  free(self);
}

static void ast_path_to_string(void *_self, string_t *str) {
  ast_path_t *self = _self;

  for (size_t i = 0; i < self->len; i++) {
    if (i != 0) {
      string_append(str, ".");
    }
    string_append(str, "%s", self->comonents[i]);
  }
}

static void ast_path_generate(void *_self, context_t *ctx) {
  ast_path_t *self = _self;
  for (size_t i = 1; i < self->len; i++) {
    string_append(&ctx->value, "annabella_value_get(");
  }
  string_append(&ctx->value, "annabella_scope_get(scope, \"%s\")",
                self->comonents[0]);
  for (size_t i = 1; i < self->len; i++) {
    string_append(&ctx->value, ", \"%s\")", self->comonents[i]);
  }
}

static const ast_node_vtable_t ast_path_vtable = {
    "path",
    ast_path_drop,
    ast_path_to_string,
    ast_path_generate,
};

void ast_path_generate_init_fn_name(ast_node_t *_self, string_t *str) {
  if (_self->vtable != &ast_path_vtable) {
    die("generate_init_fn_name called on a %s (expected path)\n",
        ast_node_class_name(_self));
  }

  ast_path_t *self = (ast_path_t *)_self;
  for (size_t i = 0; i < self->len; i++) {
    if (i != 0) {
      string_append(str, "__");
    }
    string_append(str, "%s", self->comonents[i]);
  }
}

ast_node_t *token_stream_path(token_stream_t *self) {
  static_str_t *components = NULL;
  size_t len = 0;
  size_t cap = 0;

  bool expect_dot = false;
  while (true) {
    if (expect_dot) {
      if (!token_stream_consume_if_token(self, '.')) {
        break;
      }
    }
    expect_dot = true;
    static_str_t component = token_stream_ident(self);

    if (len == cap) {
      if (cap > 0) {
        cap *= 2;
      } else {
        cap = 1;
      }
      components = realloc(components, cap * sizeof(*components));
      if (components == NULL) {
        die_errno("failed to reallic ast node array: %s\n");
      }
    }

    if (len >= cap) {
      die("ast array out of bounds: %ld >= %ld\n", len, cap);
    }

    components[len++] = component;
  }

  if (len == 0) {
    die("unreachable: path len 0\n");
  }

  ast_path_t *path = malloc(sizeof(*path));
  *path = (ast_path_t){
      &ast_path_vtable,
      components,
      len,
  };
  return &path->super;
}

void token_stream_path_eq(token_stream_t *self, ast_node_t *_path) {
  if (_path->vtable != &ast_path_vtable) {
    die("generate_init_fn_name called on a %s (expected path)\n",
        ast_node_class_name(_path));
  }

  ast_path_t *path = (ast_path_t *)_path;
  for (size_t i = 0; i < path->len; i++) {
    if (i != 0) {
      token_stream_token(self, '.');
    }
    token_stream_ident_eq(self, path->comonents[i]);
  }
}
