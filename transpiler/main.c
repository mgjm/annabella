#include "macros.h"
#include "statement.h"
#include "tokenizer.h"
#include <stdio.h>

int main(int argc, const char *const argv[]) {
  if (argc != 2) {
    die("usage: annabella ADA_SOURCE_FILE");
  }

  printf("Hello World from C\n");

  token_stream_t token_stream = token_stream_open(argv[1]);

  while (token_stream_statement(&token_stream)) {
  }

  return EXIT_SUCCESS;
}
