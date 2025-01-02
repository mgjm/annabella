#pragma once

#define KEYWORD_FIRST abort
#define KEYWORD_LAST xor

#define _STATIC_ATOMS(callback) callback(empty, "") /**/ \
/* keywords */ \
callback(abort, "abort") \
callback(abs, "abs") \
callback(abstract, "abstract") /* Ada 95 */ \
callback(accept, "accept") \
callback(access, "access") \
callback(aliased, "aliased") /* Ada 95 */ \
callback(all, "all") \
callback(and, "and") \
callback(array, "array") \
callback(at, "at") \
callback(begin, "begin") \
callback(body, "body") \
callback(case, "case") \
callback(constant, "constant") \
callback(declare, "declare") \
callback(delay, "delay") \
callback(delta, "delta") \
callback(digits, "digits") \
callback(do, "do") \
callback(else, "else") \
callback(elsif, "elsif") \
callback(end, "end") \
callback(entry, "entry") \
callback(exception, "exception") \
callback(exit, "exit") \
callback(for, "for") \
callback(function, "function") \
callback(generic, "generic") \
callback(goto, "goto") \
callback(if, "if") \
callback(in, "in") \
callback(interface, "interface") /* Ada 2005 */ \
callback(is, "is") \
callback(limited, "limited") \
callback(loop, "loop") \
callback(mod, "mod") \
callback(new_, "new") \
callback(not, "not") \
callback(null, "null") \
callback(of, "of") \
callback(or, "or") \
callback(others, "others") \
callback(out, "out") \
callback(overriding, "overriding") /* Ada 2005 */ \
callback(package, "package") \
callback(parallel, "parallel") /* Ada 2022 */ \
callback(pragma, "pragma") \
callback(private, "private") \
callback(procedure, "procedure") \
callback(protected, "protected") /* Ada 95 */ \
callback(raise, "raise") \
callback(range, "range") \
callback(record, "record") \
callback(rem, "rem") \
callback(renames, "renames") \
callback(requeue, "requeue") /* Ada 95 */ \
callback(return, "return") \
callback(reverse, "reverse") \
callback(select, "select") \
callback(separate, "separate") \
callback(some, "some") /* Ada 2012 */ \
callback(subtype, "subtype") \
callback(synchronized, "synchronized") /* Ada 2005 */ \
callback(tagged, "tagged") /* Ada 95 */ \
callback(task, "task") \
callback(terminate, "terminate") \
callback(then, "then") \
callback(type, "type") \
callback(until, "until") /* Ada 95 */ \
callback(use, "use") \
callback(when, "when") \
callback(while, "while") \
callback(with, "with") \
callback(xor, "xor")
