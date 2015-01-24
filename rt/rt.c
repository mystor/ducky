#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <gc.h>


// The storage system for values is copied from the SpiderMonkey compiler.
// Values are 64-bits, and consist of a 17-bit tag, and 47 bits of value.

typedef enum { // uint8_t
  TYPE_DOUBLE = 0x00,
  TYPE_BOOLEAN = 0x01,
  TYPE_STRING = 0x02,
  TYPE_RECORD = 0x03
} __attribute__((packed)) ValueType;

_Static_assert(sizeof(ValueType) == 1, "");

typedef enum {
  TAG_MAX_DOUBLE = 0x1FFF0, // First 13 bits set
  TAG_BOOLEAN = TAG_MAX_DOUBLE | TYPE_BOOLEAN,
  TAG_STRING = TAG_MAX_DOUBLE | TYPE_STRING,
  TAG_RECORD = TAG_MAX_DOUBLE | TYPE_RECORD
} __attribute__((packed)) ValueTag;

// Tags should fit in 4 bytes
_Static_assert(sizeof(ValueTag) == 4, "");

#define TAG_SHIFT 47
typedef enum {
  SHIFTED_TAG_MAX_DOUBLE = (((uint64_t) TAG_MAX_DOUBLE) << TAG_SHIFT) | 0xffffffff,
  SHIFTED_TAG_BOOLEAN = ((uint64_t) TAG_BOOLEAN) << TAG_SHIFT,
  SHIFTED_TAG_STRING = ((uint64_t) TAG_STRING) << TAG_SHIFT,
  SHIFTED_TAG_RECORD = ((uint64_t) TAG_RECORD) << TAG_SHIFT
} __attribute((packed)) ShiftedValueTag;

_Static_assert(sizeof(ShiftedValueTag) == 8, "");




typedef union jsval_layout
{
  uint64_t asBits;
#if !defined(_WIN64)
  /* MSVC does not pack these correctly :-( */
  struct {
    uint64_t payload47 : 47;
    JSValueTag tag : 17;
  } debugView;
#endif
  struct {
    union {
      int32_t i32;
      uint32_t u32;
      // JSWhyMagic why;
    } payload;
  } s;
  double asDouble;
  void *asPtr;
  size_t asWord;
  uintptr_t asUIntPtr;
} __attribute__((aligned (8))) jsval_layout;

typedef int32_t symbol;
// typedef void *value;

// a record_field should be 64-bits wide
// and tightly packed (ideally)
struct record_field {
  symbol symbol;
  int32_t offset; // in sizeof(size_t) units
};

// A record_def is a constant linear-probed hash table
struct record_def {
  int32_t size; // In sizeof(record_field) units
  // Offsets...
};

struct record {
  struct record_def *def;
  // fields...
};
// TODO(michael): Make this work on non-little-endian systems,
// and systems which have non-64bit pointers

enum tags {
  RECORD_TAG = 0x0,
  INT_TAG = 0x1, // TODO(michael): Unused right now
  BOOL_TAG = 0x2,
  STR_TAG = 0x4
};
#define TAG_MASK 0x7

typedef uint64_t ducky_bool;
typedef uint32_t bool;

typedef union value {
  uint64_t bytes;
  struct {
    // Filler fields (so that double_tag lines up with the high 16 bits of the double)
    uint16_t _b;
    uint32_t _a;
    uint16_t double_tag;
  } __attribute__((packed));
  ducky_bool asBool;
  double asDouble;
  struct record *asRecord;

} __attribute__((aligned (8))) value;

#define TRUE 0xffff 0000 0000 0003
#define FALSE 0xffff 0000 0000 0002

_Static_assert(sizeof(value) == 8, "Values must be 64-bits");

inline bool valueIsDouble(value v) {
  return v.double_tag != USHRT_MAX;
}

inline double valueAsDouble(value v) {
  return v.asDouble;
}

inline bool valueIsRecord(value v) {
  return !valueIsDouble(v) && (v.bytes & TAG_MASK) == RECORD_TAG;
}

inline struct record *valueAsRecord(value v) {

}

inline bool valueIsBool(value v) {
  return !valueIsDouble(v) && (v.bytes & TAG_MASK) == BOOL_TAG;
}



value get_property(value v, symbol s) {
  assert(valueIsRecord(v));


}



int main() {
  GC_INIT();

  // Allocate a record with 5 properties
  struct record *rec = GC_MALLOC(sizeof(struct record) + (5 * sizeof(value)));
  printf("%p\n", rec);

  void *randomptr = GC_MALLOC(1);
  printf("%p\n", randomptr);

  struct record *rec2 = GC_MALLOC(sizeof(struct record) + (5 * sizeof(value)));
  printf("%p\n", rec2);
}









/*
  2 types
  (a, b, ...) -> c
  { a: b, ... }
 */

/*

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
  * let c = a + b *
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
*/
