#include "atom.h"
#include "macros.h"
#include "static-atom.h"
#include <stdlib.h>
#include <string.h>

// TODO: add hashmap to speed up atom lookup

#define _STATIC_ATOM_ENTRY(name, value) [static_atom_##name] = value,
static const char *const start_entries[] = {_STATIC_ATOMS(_STATIC_ATOM_ENTRY)};

static const char **entries = NULL;
static size_t len = array_len(start_entries);
static size_t cap = array_len(start_entries);

atom_t atom_new(const char *str) {
  if (entries == NULL) {
    entries = malloc(sizeof(start_entries));
    if (!entries) {
      die_errno("failed to malloc atom entries: %s\n");
    }
    memcpy(entries, start_entries, sizeof(start_entries));
  }

  // is in entries => return
  for (size_t i = 0; i < len; i++) {
    if (strcmp(entries[i], str) == 0) {
      return (atom_t){i};
    }
  }

  if (len == cap) {
    cap = cap ? cap * 2 : 2;
    entries = realloc(entries, cap * sizeof(*entries));
    if (!entries) {
      die_errno("failed to realloc atom entries: %s\n");
    }
  }

  if (len >= cap) {
    die("atom out of bounds\n");
  }
  atom_t atom = {len};
  entries[len++] = strdup(str);
  return atom;
}

const char *atom_get(atom_t self) {
  if (self.id >= len) {
    die("unknwon atom id: %ld\n", self.id);
  }
  return entries[self.id];
}
