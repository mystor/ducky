#include <stdio.h>
#include <stdlib.h>

/*
  2 types
  (a, b, ...) -> c
  { a: b, ... }
 */

// NOTE: Right now, ducky doesn't have a module system and can't really handle
// linking with other things. This runtime file will be used for implementing
// important mechanics stuff
// Changes may need to be made, as lots of these functions should probably
// be inlined into the code

typedef struct string_struct {
  // @TODO: Strings should probably have lazily computed hash
  unsigned int length;
  char *chars;
} *string;

typedef unsigned int istring;
istring next_istring = 0;
unsigned int intern_capacity = 0;
string *interned_strings; // @TODO: Capacity!

typedef struct map_struct {
  unsigned int length;
  istring *names;
} *map;

int _string_eq(string a, string b) {
  if (a->length != b->length) return 0;

  unsigned int i, length = a->length;
  char *ap = a->chars, *bp = b->chars;
  for (i = 0; i < length; i++)
    if (*ap++ != *bp++) return 0;

  return 1;
}

istring _get_istring(string str) {
  unsigned int i;
  for (i = 0; i < next_istring; i++) {
    if (_string_eq(interned_strings[i], str))
      return i;
  }

  if (next_istring >= intern_capacity) {
    intern_capacity *= 2;
    interned_strings = realloc(interned_strings, intern_capacity * 2);

    if (interned_strings == 0) // FUUU
      exit(100); // SHITTTT
  }

  interned_strings[next_istring] = str;
  return next_istring++;
}

// A value is a block of memory, containing only pointers
// The first value in the block of memory is a pointer to the
// _map which describes the type in the block of memory.
// After that memory, the data stored varies depending on the type of value
// 
// If the type is a function (lowest bit is 1), then *value is two words wide,
// and contains a function pointer in the second word.
//
// If the type is a record (lowest bit is 0), then *value is a list of properties
// of the object. The map object contains a list of interned strings, corresponding
// to the names of each of the properties on the object.
// 
// If the type is a double, the *value is two words wide, and the second word contains
// the IEEE double percision floating point number
typedef void **val;

typedef struct list_struct {
  unsigned int count;
  unsigned int capacity;
  val *items;
} *list;

enum value_kind {
  ptr = 0,
  smi = 1,
  str = 2,
  lst = 3,
  bool = 4
};

int main() {
  val a = (void*) ((10 << 3) + 1);
  val b = (void*) ((20 << 3) + 1);

  val c;
  /* let c = a + b */
  switch ((int) a & 7) {
  case smi:
    // I'm assuming that b must also be a smi... probably can't do that, can I
    c = (void*) (((int) a + (int) b) ^ 3);
    break;
  case str:
    // TODO: Implement a + b for strings
    return 101;
  case lst:
    // TODO: Implement
    return 101;
  case ptr:
    {
      map tymap = (map) (a[0]);
      
      istring prop = 0; // Inlined istring for +!
      
      // Get the index to look into
      // TODO: This could fail...
      unsigned int offset;
      for (offset = 0; offset < tymap->length; offset++) {
        if ((tymap->names)[offset] == prop)
          break;
      }
      
      // Get the property, and cast it to an array of function pointers
      void *(**clos) (void *, void *, void *) = a[offset];

      // The second element is where the function is stored, let's call it!
      c = clos[1](clos, // The closure
                  a,    // implicit self argument
                  b);   // Actual arguments
    }
    break;
  }
  
  printf("%lld\n", (long long int) c >> 3);
  
  return 0;
}
