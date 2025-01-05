#include "ast-node.h"
#include "macros.h"
#include "str.h"
#include "token-stream.h"
#include "tokenizer.h"
#include <stdio.h>

int main(int argc, const char *const argv[]) {
  if (argc != 2) {
    die("usage: annabella ADA_SOURCE_FILE");
  }

  token_stream_t token_stream = token_stream_open(argv[1]);

  ast_node_array_t stmts = {};

  while (!token_stream_is_end(&token_stream)) {
    ast_node_t *stmt = token_stream_stmt(&token_stream);
    ast_node_array_push(&stmts, stmt);
    // eprintf("\n");
    // ast_node_debug(stmt);
  }

  // {
  //   string_t str = NULL;
  //   ast_node_array_to_string_lines(&stmts, &str);
  //   eprintf("%s", str);
  //   free(str);
  // }

  context_t ctx = {};
  string_append(&ctx.functions, "#include \"annabella-rt.h\"\n"
                                "\n");

  for (size_t i = 0; i < stmts.len; i++) {
    ast_node_generate(stmts.nodes[i], &ctx);
    if (i == 42) {
      break;
    }
  }
  context_finalize(&ctx);

  return EXIT_SUCCESS;
}
