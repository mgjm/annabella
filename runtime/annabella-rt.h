#pragma once

#include <stdarg.h>
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

extern PUB void annabella_scope_insert_package(annabella_scope_t *self,
                                               annabella_package_t *package);
extern PUB annabella_value_t *annabella_scope_get(annabella_scope_t *self,
                                                  const char *key);
extern PUB void annabella_scope_drop(annabella_scope_t *self);

extern PUB void annabella_value_drop(annabella_value_t *self);
extern PUB annabella_value_t *annabella_value_call(annabella_value_t *self,
                                                   size_t argc, ...);
extern PUB annabella_value_t *annabella_value_get(annabella_value_t *self,
                                                  const char *key);

extern PUB annabella_package_t *
annabella_package_already_initializing(const char *path);
extern PUB void annabella_package_insert(annabella_package_t *self,
                                         const char *key,
                                         annabella_value_t *value);

extern PUB annabella_value_t *
annabella_string_value_from_ref(const char *value);

typedef annabella_value_t *(*annabella_c_function_call_t)(void *self,
                                                          size_t argc,
                                                          va_list args);
extern PUB annabella_value_t *
annabella_c_function_value_new(annabella_c_function_call_t call, void *this);

// std library packages
extern PUB annabella_package_t *_annabella_package_Ada__Text_IO_init();
