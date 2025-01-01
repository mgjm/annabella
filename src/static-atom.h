#pragma once

#define _STATIC_ATOMS(callback)                                                \
  callback(empty, "") callback(with, "with") callback(dot, ".")                \
      callback(semi, ";") callback(procedure, "procedure") callback(is, "is")  \
          callback(begin, "begin") callback(end, "end")
