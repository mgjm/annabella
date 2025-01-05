#include "annabella-rt.h"
#include "macros.h"
#include "private.h"
#include "scope.h"
#include "value.h"
#include <stdlib.h>
#include <string.h>

typedef struct package_value {
  value_t super;
  const char *name;
  scope_t scope;
} package_value_t;

static void package_value_drop(void *_self) {
  package_value_t *self = _self;
  // eprintf("warning: package %s dropped\n", self->name);
  annabella_scope_drop(&self->scope);
  free(self);
}

static char *package_value_to_string(void *_self) {
  package_value_t *self = _self;
  return strdup(self->name);
}

static value_t *package_value_try_get_by_key(void *_self, atom_t key) {
  package_value_t *self = _self;
  value_t *value = scope_try_get(&self->scope, key);
  value_rm_ref(&self->super);
  return value_add_ref(value);
}

static value_vtable_t package_value_vtable = {
    "package",
    package_value_drop,
    value_vtable_required_end,

    .to_string = package_value_to_string,
    .try_get_by_key = package_value_try_get_by_key,
};

value_t *package_value_new() {
  package_value_t *self = malloc(sizeof(package_value_t));
  *self = (package_value_t){
      value_base_new(&package_value_vtable),
      "<package>",
  };
  return &self->super;
}

static package_value_t *package_value_from_value(value_t *self) {
  if (self->vtable != &package_value_vtable) {
    die("package_value_set_package invoked with %s value\n",
        value_class_name(self));
  }
  return (package_value_t *)self;
}

void package_value_set_package(value_t *_self, package_t *package) {
  package_value_t *self = package_value_from_value(_self);
  self->name = package->name;
  self->scope.parent = &package->scope;
}

value_t *package_value_get_package(value_t *_self, atom_t key) {
  package_value_t *self = package_value_from_value(_self);
  value_t *value = scope_try_get(&self->scope, key);
  if (value == NULL) {
    value = package_value_new();
    scope_insert(&self->scope, key, value);
  }
  return value;
}
