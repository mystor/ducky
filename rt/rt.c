#include <limits.h> // MAX values
#include <stdlib.h> // Sized Types
#include <stdio.h>  // IO
#include <assert.h> // Assertions
#include <string.h> // memcpy
#include <gc.h>     // Garbage Collection

// The storage system for values is copied from the SpiderMonkey jit.
// Values are 64-bits, and consist of a 17-bit tag, and 47 bits of value.
//
// TODO(michael): Make this work on non-little-endian systems,
// and systems which have non-64bit pointers

typedef enum { // uint8_t
  TYPE_DOUBLE = 0x00,
  TYPE_BOOLEAN = 0x01,
  TYPE_STRING = 0x02,
  TYPE_RECORD = 0x03
} __attribute__((packed)) ValueType;

_Static_assert(sizeof(ValueType) == sizeof(uint8_t), "Types are 8 bits wide");

typedef enum {
  TAG_MAX_DOUBLE = 0x1FFF0, // First 13 bits set
  TAG_BOOLEAN = TAG_MAX_DOUBLE | TYPE_BOOLEAN,
  TAG_STRING = TAG_MAX_DOUBLE | TYPE_STRING,
  TAG_RECORD = TAG_MAX_DOUBLE | TYPE_RECORD
} __attribute__((packed)) ValueTag;

// Tags should fit in 4 bytes
_Static_assert(sizeof(ValueTag) == sizeof(uint32_t), "Tags are 32 bits wide");

#define TAG_SHIFT 47
#define TAG_MASK ((uint64_t) UINT_MAX << TAG_SHIFT)
typedef enum {
  SHIFTED_TAG_MAX_DOUBLE = (((uint64_t) TAG_MAX_DOUBLE) << TAG_SHIFT) | 0xffffffff, // TODO(michael): Investigate
  SHIFTED_TAG_BOOLEAN = ((uint64_t) TAG_BOOLEAN) << TAG_SHIFT,
  SHIFTED_TAG_STRING = ((uint64_t) TAG_STRING) << TAG_SHIFT,
  SHIFTED_TAG_RECORD = ((uint64_t) TAG_RECORD) << TAG_SHIFT
} __attribute((packed)) ShiftedValueTag;

_Static_assert(sizeof(ShiftedValueTag) == sizeof(uint64_t), "Shifted tags are 64-bits wide");

// The actual union which holds the value
typedef union
{
  uint64_t asBits;
#if !defined(_WIN64)
  /* MSVC does not pack these correctly :-( */
  struct {
    uint64_t payload47 : 47;
    ValueTag tag : 17;
  } debugView;
#endif
  struct {
    union {
      int32_t i32;
      uint32_t u32;
    } payload;
  } s;
  double asDouble;
  struct record *asPtr;
  size_t asWord;
  uintptr_t asUIntPtr;
} __attribute__((aligned (8))) value;

_Static_assert(sizeof(value) == sizeof(uint64_t), "Values are 64-bits wide");

typedef int32_t symbol;

// a record_field should be 64-bits wide
// and tightly packed (ideally)
struct record_field {
  symbol symbol;
  int32_t offset; // in sizeof(size_t) units
};

// TODO(michael): See if these can be made a PO2 size
union record_def {
  struct {
    symbol symbol;
    uint64_t offset;
  } prop;
  struct {
    symbol symbol;
    void *fn;
  } mthd;
  struct {
    uint32_t prop_size;
    uint32_t mthd_size;
  } header;
};

struct record {
  union record_def *def;
  // fields...
};

typedef uint32_t bool;

#define TRUE 0xffff 0000 0000 0003
#define FALSE 0xffff 0000 0000 0002

_Static_assert(sizeof(value) == 8, "Values must be 64-bits");

inline bool valueIsDouble(value v) {
  return v.asBits <= SHIFTED_TAG_MAX_DOUBLE;
  // return v.double_tag != USHRT_MAX;
}

inline double valueAsDouble(value v) {
  return v.asDouble;
}

inline bool valueIsRecord(value v) {
  return (v.asBits & TAG_MASK) == SHIFTED_TAG_RECORD;
}

inline struct record *valueAsRecord(value v) {
  return (struct record *)(v.asWord & (~TAG_MASK));
}

inline bool valueIsBool(value v) {
  return (v.asBits & TAG_MASK) == SHIFTED_TAG_BOOLEAN;
}

inline bool valueAsBool(value v) {
  return v.s.payload.u32; // Tag mask doesn't cover this part of the value
}

inline value getProperty(value v, symbol s) {
  assert(valueIsRecord(v));
  struct record *record = valueAsRecord(v);
  union record_def *def = record->def;

  uint32_t size = def->header.prop_size; // TODO: Eww, double pointers :s
  uint32_t idx = s % size;
  def++;

  while ((def+idx)->prop.symbol != s) {
    idx = (idx + 1) % size;

    // Theoretically, the property should always exist. This assert catches
    // the case when it doesn't exist for debugging purposes.
    assert(idx != s % size);
  }

  return ((value *)(record+1))[(def+idx)->prop.offset];
}

void *getClosure(value v) {
  assert(valueIsRecord(v));
  struct record *record = valueAsRecord(v);
  assert(record->def->header.mthd_size);

  return record+1;
}

inline void *getMethod(value v, symbol s) {
  assert(valueIsRecord(v));
  struct record *record = valueAsRecord(v);
  union record_def *def = record->def;

  uint32_t size = def->header.mthd_size; // TODO: Eww, double pointers :s
  uint32_t idx = s % size;

  // Move past the properties & header
  def += def->header.prop_size + 1;

  while ((def+idx)->mthd.symbol != s) {
    idx = (idx + 1) % size;

    // Theoretically, the property should always exist. This assert catches
    // the case when it doesn't exist for debugging purposes.
    assert(idx != s % size);
  }

  return (def+idx)->mthd.fn;
}

inline value allocRecord(union record_def *def, value *props) {
  value v;

  uint32_t prop_count = def->header.prop_size;
  v.asPtr = GC_MALLOC(sizeof(struct record) + (prop_count * sizeof(value)));

  // TODO(michael): Gracefully handle failed allocations
  assert(v.asPtr != 0);

  v.asPtr->def = def;
  memcpy(v.asPtr+1, props, prop_count * sizeof(value));

  // Tag the value
  v.asBits &= SHIFTED_TAG_RECORD;

  return v;
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
