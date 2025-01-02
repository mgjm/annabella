#include "ast.h"
#include "atom.h"
#include "macros.h"
#include "object.h"
#include "scope.h"
#include "std.h"
#include "value.h"
#include <stdlib.h>

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

value_t *ast_node_eval(void *_self, scope_t *scope) {
  ast_node_t *self = _self;
  return self->vtable->eval(self, scope);
}

static void with_stmt_drop(void *_self) {
  with_stmt_t *self = _self;
  path_drop(&self->path);
  free(self);
}

static value_t *with_stmt_eval(void *_self, scope_t *scope) {
  with_stmt_t *self = _self;

  init_global_scope();
  value_t *value = scope_try_get(&global_scope, self->path.components[0]);
  if (value != NULL) {
    scope_insert(scope, self->path.components[0], value);
  } else {
    // TODO: import from file
    if (atom_eq(self->path.components[0], atom_new("Test"))) {
      packet_value_t *test = packet_value_new();
      scope_insert(scope, atom_new("Test"), (value_t *)test);

      string_value_t *get_lang = string_value_from_ref("Hello from Ada via C");
      scope_insert(&test->scope, atom_new("GetLang"), (value_t *)get_lang);
    } else {
      die("file import not implemented");
    }
  }

  return NULL;
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

static value_t *procedure_stmt_eval(void *_self, scope_t *scope) {
  procedure_stmt_t *self = _self;
  eprintf("procedure statement %s:\n", atom_get(self->ident));
  for (size_t i = 0; i < self->body.len; i++) {
    ast_node_eval(self->body.data[i], scope);
  }
  eprintf("end of procedure %s\n", atom_get(self->ident));
  return NULL;
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

static value_t *function_call_expr_eval(void *_self, scope_t *scope) {
  function_call_expr_t *self = _self;

  // fetch function value
  value_t *value = scope_get(scope, self->path.components[0]);
  for (size_t i = 1; i < self->path.len; i++) {
    value = value_get_by_key(value, self->path.components[i]);
  }

  // collect arguments
  array_t args = {};
  for (size_t i = 0; i < self->args.len; i++) {
    value_t *arg = ast_node_eval(self->args.data[i], scope);
    eprintf("push arg: %lx\n", (size_t)arg);
    array_push(&args, arg);
  }

  // call function
  return value_call(value, &args);
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

static value_t *string_expr_eval(void *_self, scope_t *scope) {
  string_expr_t *self = _self;
  return (value_t *)string_value_from_atom(self->value);
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

static void path_expr_drop(void *_self) {
  path_expr_t *self = _self;
  path_drop(&self->path);
  free(self);
}

static value_t *path_expr_eval(void *_self, scope_t *scope) {
  path_expr_t *self = _self;

  value_t *value = scope_get(scope, self->path.components[0]);
  for (size_t i = 1; i < self->path.len; i++) {
    value = value_get_by_key(value, self->path.components[i]);
  }
  return value;
}

static ast_node_vtable_t path_expr_vtable = {
    "path_expr",
    path_expr_drop,
    path_expr_eval,
};

path_expr_t *path_expr_new(path_t path) {
  path_expr_t *self = malloc(sizeof(path_expr_t));
  *self = (path_expr_t){
      &path_expr_vtable,
      path,
  };
  return self;
}
