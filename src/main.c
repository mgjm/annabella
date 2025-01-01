#include "macros.h"
#include "tokenizer.h"
#include <stdio.h>

int main(int argc, const char *const argv[]) {
  if (argc != 2) {
    die("usage: annabella ADA_SOURCE_FILE");
  }

  printf("Hello World from C\n");
  tokenize_file(argv[1]);

  return EXIT_SUCCESS;
}
