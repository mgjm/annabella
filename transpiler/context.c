#include "context.h"
#include "macros.h"
#include "str.h"
#include <stdlib.h>

void context_finalize(context_t *self) {
  if (self->value) {
    die("unused code in ctx->value: %s\n", self->value);
  }

  if (self->init) {
    eprintf("unused code in ctx->init, generating a main function\n");

    string_append(&self->functions,
                  "int main() {\n"
                  "annabella_scope_t scope = {};\n"
                  "\n"
                  "%s"
                  "\n"
                  "return 0;\n"
                  "}\n",
                  self->init);
    free(self->init);
    self->init = NULL;
  }

  if (!self->functions) {
    die("no code generated\n");
  }

  printf("%s\n", self->functions);
  free(self->functions);
}
