#pragma once

#include "static-atom.h"
#include <stdbool.h>
#include <stddef.h>

typedef struct atom {
  size_t id;
} atom_t;

#define _STATIC_ATOM_ENUM(name, value) static_atom_##name,
enum _static_atom { _STATIC_ATOMS(_STATIC_ATOM_ENUM) };

#define _STATIC_ATOM_CONST(name, value)                                        \
  static const atom_t atom_##name = {static_atom_##name};
_STATIC_ATOMS(_STATIC_ATOM_CONST)

#define _STATIC_ATOM_(ident) static_atom_##ident
#define STATIC_ATOM_(ident) _STATIC_ATOM_(ident)

static inline bool atom_is_keyword(atom_t self) {
  return STATIC_ATOM_(KEYWORD_FIRST) <= self.id &&
         self.id <= STATIC_ATOM_(KEYWORD_LAST);
}

extern atom_t atom_new(const char *str);

extern const char *atom_get(atom_t self);

static inline bool atom_eq(atom_t a, atom_t b) { return a.id == b.id; }
