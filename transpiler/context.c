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
                  "annabella_scope_t main_scope = {};\n"
                  "annabella_scope_t *scope = &main_scope;\n"
                  "\n"
                  "annabella_main_scope_init(scope);\n"
                  "\n"
                  "%s"
                  "\n"
                  "annabella_scope_exec_main(scope);\n"
                  "\n"
                  "annabella_scope_drop(scope);\n"
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
