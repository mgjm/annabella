#include "std.h"
#include "atom.h"
#include "std-ada-text-io.h"
#include "value.h"

scope_t global_scope = {};

static packet_value_t *ada() {
  packet_value_t *packet = packet_value_new();
  scope_insert(&packet->scope, atom_text_io, (value_t *)ada_text_io());
  return packet;
}

void init_global_scope() {
  if (global_scope.len > 0) {
    // already initalized
    return;
  }

  scope_insert(&global_scope, atom_ada, (value_t *)ada());
}
