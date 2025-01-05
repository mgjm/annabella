#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>

#define PUB __attribute__((visibility("default")))

typedef struct annabella_scope annabella_scope_t;
typedef struct annabella_scope_entry annabella_scope_entry_t;
typedef struct annabella_scope {
  annabella_scope_t *parent;
  annabella_scope_entry_t *data;
  size_t len;
  size_t cap;
} annabella_scope_t;

typedef struct annabella_value annabella_value_t;

typedef enum annabella_package_state {
  annabella_package_state_uninitalized,
  annabella_package_state_initializing,
  annabella_package_state_initialized,
} annabella_package_state_t;

typedef struct annabella_package {
  const char *name;
  annabella_package_state_t state;
  annabella_scope_t scope;
} annabella_package_t;

extern PUB void annabella_main_scope_init(annabella_scope_t *self);
extern PUB void annabella_package_scope_init(annabella_scope_t *self);

extern PUB void annabella_scope_insert_package(annabella_scope_t *self,
                                               annabella_package_t *package);
extern PUB void annabella_scope_insert_value(annabella_scope_t *self,
                                             const char *name,
                                             annabella_value_t *value);
extern PUB annabella_value_t *annabella_scope_get(annabella_scope_t *self,
                                                  const char *key);
extern PUB void annabella_scope_exec_main(annabella_scope_t *self);
extern PUB void annabella_scope_drop(annabella_scope_t *self);

extern PUB void annabella_value_drop(annabella_value_t *self);
extern PUB char *annabella_value_to_string(annabella_value_t *self);
extern PUB annabella_value_t *
annabella_value_to_value(annabella_value_t *self, annabella_scope_t *scope);
extern PUB annabella_value_t *annabella_value_call(annabella_value_t *self,
                                                   annabella_scope_t *scope,
                                                   size_t argc, ...);
extern PUB annabella_value_t *annabella_value_get(annabella_value_t *self,
                                                  const char *key);
extern PUB void annabella_value_assign(annabella_value_t *self,
                                       annabella_value_t *value);
extern PUB annabella_value_t *annabella_value_default(annabella_value_t *self);
extern PUB bool annabella_value_to_bool(annabella_value_t *self);

extern PUB annabella_package_t *
annabella_package_already_initializing(const char *path);

typedef size_t annabella_integer_t;

extern PUB annabella_value_t *
annabella_range_type_value(annabella_integer_t min, annabella_integer_t max);

extern PUB annabella_value_t *annabella_string_value(const char *value);

extern PUB annabella_value_t *
annabella_integer_value(annabella_integer_t number);

typedef annabella_value_t *(*annabella_function_call_t)(
    annabella_scope_t *scope, va_list args);
extern PUB annabella_value_t *
annabella_function_value(annabella_function_call_t call, size_t argc, ...);
