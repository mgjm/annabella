#include "package.h"
#include "macros.h"

package_t *annabella_package_already_initializing(const char *name) {
  die("circular ada package initialization detected in: %s\n", name);
}
