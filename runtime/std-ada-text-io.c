#include "value.h"
#include <stdio.h>

static value_t *ada_text_io_put_line(void *this, size_t argc, va_list args) {
  for (size_t i = 0; i < argc; i++) {
    value_t *arg = va_arg(args, value_t *);
    char *arg_str = value_to_string(arg);
    printf("%s\n", arg_str);
  }
  return NULL;
}

annabella_package_t *_annabella_package_Ada__Text_IO_init() {
  static annabella_package_t package = {
      "Ada.Text_IO",
  };

  switch (package.state) {
  case annabella_package_state_uninitalized:
    break;
  case annabella_package_state_initializing:
    return annabella_package_already_initializing(package.name);
  case annabella_package_state_initialized:
    return &package;
  }

  package.state = annabella_package_state_initializing;

  annabella_package_insert(
      &package, "Put_Line",
      annabella_c_function_value_new(ada_text_io_put_line, NULL));

  package.state = annabella_package_state_initialized;
  return &package;
}
