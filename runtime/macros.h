#pragma once

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define eprintf(...) fprintf(stderr, __VA_ARGS__)

#define die(...)                                                               \
  {                                                                            \
    eprintf(__VA_ARGS__);                                                      \
    exit(EXIT_FAILURE);                                                        \
  }

#define die_errno(...) die(__VA_ARGS__, strerror(errno))

#define array_len(arr) (sizeof(arr) / sizeof(*arr))
