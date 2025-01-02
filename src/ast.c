#include "ast.h"
#include "atom.h"
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

void path_print(path_t *self) {
  for (size_t i = 0; i < self->len; i++) {
    eprintf("%s%s", i ? "." : "", atom_get(self->components[i]));
  }
}

void path_push(path_t *self, atom_t component) {
  if (self->len == self->cap) {
    size_t new_cap = self->cap > 0 ? self->cap * 2 : 1;
    self->components =
        realloc(self->components, new_cap * sizeof(*self->components));
    if (self->components == NULL) {
      die_errno("failed to reallocate path: %s\n");
    }
    self->cap = new_cap;
  }

  if (self->len >= self->cap) {
    die("path push out of bounds: %ld >= %ld\n", self->len, self->cap);
  }

  self->components[self->len++] = component;
}

void path_drop(path_t *self) {
  free(self->components);
  *self = (path_t){};
}

void ast_node_eval(void *_self) {
  ast_node_t *self = _self;
  self->vtable->eval(self);
}

static void with_stmt_drop(void *_self) {
  with_stmt_t *self = _self;
  path_drop(&self->path);
  free(self);
}

static void with_stmt_eval(void *_self) {
  with_stmt_t *self = _self;
  eprintf("with statement: ");
  path_print(&self->path);
  eprintf("\n");
}

static ast_node_vtable_t with_stmt_vtable = {
    "with_stmt",
    with_stmt_drop,
    with_stmt_eval,
};

with_stmt_t *with_stmt_new(path_t path) {
  with_stmt_t *self = malloc(sizeof(with_stmt_t));
  *self = (with_stmt_t){
      &with_stmt_vtable,
      path,
  };
  return self;
}

static void procedure_stmt_drop(void *_self) {
  procedure_stmt_t *self = _self;
  array_drop(&self->body);
  free(self);
}

static void procedure_stmt_eval(void *_self) {
  procedure_stmt_t *self = _self;
  eprintf("procedure statement %s:\n", atom_get(self->ident));
  for (size_t i = 0; i < self->body.len; i++) {
    ast_node_eval(self->body.data[i]);
  }
  eprintf("end of procedure %s\n", atom_get(self->ident));
}

static ast_node_vtable_t procedure_stmt_vtable = {
    "procedure_stmt",
    procedure_stmt_drop,
    procedure_stmt_eval,
};

procedure_stmt_t *procedure_stmt_new(atom_t ident, array_t body) {
  procedure_stmt_t *self = malloc(sizeof(procedure_stmt_t));
  *self = (procedure_stmt_t){
      &procedure_stmt_vtable,
      ident,
      body,
  };
  return self;
}

static void function_call_expr_drop(void *_self) {
  function_call_expr_t *self = _self;
  path_drop(&self->path);
  array_drop(&self->args);
  free(self);
}

static void function_call_expr_eval(void *_self) {
  function_call_expr_t *self = _self;
  eprintf("call: ");
  path_print(&self->path);
  eprintf("(\n");
  for (size_t i = 0; i < self->args.len; i++) {
    ast_node_eval(self->args.data[i]);
  }
  eprintf(")\n");
}

static ast_node_vtable_t function_call_expr_vtable = {
    "function_call_expr",
    function_call_expr_drop,
    function_call_expr_eval,
};

function_call_expr_t *function_call_expr_new(path_t path, array_t args) {
  function_call_expr_t *self = malloc(sizeof(function_call_expr_t));
  *self = (function_call_expr_t){
      &function_call_expr_vtable,
      path,
      args,
  };
  return self;
}

static void string_expr_drop(void *_self) {
  string_expr_t *self = _self;
  free(self);
}

static void string_expr_eval(void *_self) {
  string_expr_t *self = _self;
  eprintf("string expr: %s\n", atom_get(self->value));
}

static ast_node_vtable_t string_expr_vtable = {
    "string_expr",
    string_expr_drop,
    string_expr_eval,
};

string_expr_t *string_expr_new(atom_t value) {
  string_expr_t *self = malloc(sizeof(string_expr_t));
  *self = (string_expr_t){
      &string_expr_vtable,
      value,
  };
  return self;
}
