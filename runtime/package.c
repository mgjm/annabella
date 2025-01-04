#include "package.h"
#include "macros.h"
#include "scope.h"

package_t *annabella_package_already_initializing(const char *name) {
  die("circular ada package initialization detected in: %s\n", name);
}

extern void annabella_package_insert(annabella_package_t *self, const char *key,
                                     annabella_value_t *value) {
  scope_insert(&self->scope, atom_new(key), value);
}
