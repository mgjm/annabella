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

extern atom_t atom_new(const char *str);

extern const char *atom_get(atom_t self);

static inline bool atom_eq(atom_t a, atom_t b) { return a.id == b.id; }
