#include "std-ada-text-io.h"
#include "macros.h"
#include "value.h"

value_t *ada_text_io_put_line(void *this, array_t *args) {
  eprintf("put line called with %ld arguments:\n", args->len);
  for (size_t i = 0; i < args->len; i++) {
    char *arg_str = value_to_string((value_t *)args->data[i]);
    eprintf("- %s\n", arg_str);
    printf("%s\n", arg_str);
  }

  return NULL;
}
