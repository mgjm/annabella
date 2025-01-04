#include "keywords.h"
#include "macros.h"

#define _KEYWORDS_STR(name, value) [keyword_##name] = #name,
static const char *const keywords[] = {[_not_a_keyword] = "<not a keyword>",
                                       _KEYWORDS(_KEYWORDS_STR)};

static const keyword_t keywords_len = array_len(keywords);

// TODO: implement binary search, the keywords are already sorted

keyword_t keyword_new(const char *str) {
  // start at one, zero is reserved NULL (not a keyword)
  for (keyword_t i = 1; i < keywords_len; i++) {
    if (strcmp(keywords[i], str) == 0) {
      return i;
    }
  }

  return _not_a_keyword;
}

const char *keyword_get(keyword_t self) {
  if (self >= keywords_len) {
    die("keyword out of bounds: %d\n", self);
  }
  return keywords[self];
}
