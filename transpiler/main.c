#include "ast-node.h"
#include "macros.h"
#include "token-stream.h"
#include "tokenizer.h"
#include <stdio.h>

int main(int argc, const char *const argv[]) {
  if (argc != 2) {
    die("usage: annabella ADA_SOURCE_FILE");
  }

  printf("Hello World from C\n");

  token_stream_t token_stream = token_stream_open(argv[1]);

  ast_node_t *stmt;
  while (!token_stream_is_end(&token_stream)) {
    ast_node_t *stmt = token_stream_stmt(&token_stream);
    eprintf("\n");
    ast_node_debug(stmt);
  }

  return EXIT_SUCCESS;
}
