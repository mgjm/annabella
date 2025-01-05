#include "annabella-rt.h"
#include <stdio.h>
#include <stdlib.h>

static annabella_value_t *__Put_Line(annabella_scope_t *parent_scope,
                                     va_list args) {

  annabella_scope_t function_scope = {parent_scope};
  annabella_scope_t *scope = &function_scope;

  annabella_value_t *msg = va_arg(args, annabella_value_t *);
  char *str = annabella_value_to_string(msg);
  printf("%s\n", str);
  free(str);

  annabella_scope_drop(scope);
  return 0;
}

annabella_package_t *_annabella_package_Ada__Text_IO_init() {
  static annabella_package_t package = {
      "Ada.Text_IO",
  };
  annabella_scope_t *scope = &package.scope;

  switch (package.state) {
  case annabella_package_state_uninitalized:
    break;
  case annabella_package_state_initializing:
    return annabella_package_already_initializing(package.name);
  case annabella_package_state_initialized:
    return &package;
  }

  package.state = annabella_package_state_initializing;

  annabella_scope_insert_value(
      scope, "Put_Line",
      annabella_function_value(__Put_Line, 1, "TODO: String type"));

  package.state = annabella_package_state_initialized;
  return &package;
}

annabella_package_t *_annabella_package_Interfaces_init() {
  static annabella_package_t package = {
      "Interfaces",
  };
  annabella_scope_t *scope = &package.scope;

  switch (package.state) {
  case annabella_package_state_uninitalized:
    break;
  case annabella_package_state_initializing:
    return annabella_package_already_initializing(package.name);
  case annabella_package_state_initialized:
    return &package;
  }

  package.state = annabella_package_state_initializing;

  package.state = annabella_package_state_initialized;
  return &package;
}
