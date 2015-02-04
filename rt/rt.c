#include <limits.h> // MAX values
#include <stdlib.h> // Sized Types
#include <stdio.h>  // IO
#include <assert.h> // Assertions
#include <string.h> // memcpy
#include <gc.h>     // Garbage Collection

typedef uint32_t bool;
typedef uint64_t symbol;

typedef struct field_entry {
  symbol symbol;
  size_t offset;
} field_entry;

typedef struct mthd_entry {
  symbol symbol;
  void *fn;
} mthd_entry;

typedef struct record_def {
  uint32_t prop_size;
  uint32_t mthd_size;
  // entries...
} record_def;

typedef struct record {
  record_def *def;
  // fields...
} record;

typedef enum value_tag {
  TAG_RECORD,
  TAG_DOUBLE,
  TAG_UINT32,
  TAG_BOOL,
  TAG_STRING,
  TAG_NULL
} __attribute__((packed)) value_tag;

typedef struct value {
  value_tag tag; // TODO(michael): Use a technique like NaN-boxing or pointer tagging to make this struct be PO2 sized
  size_t value;
} value;

bool valueIsDouble(value v) {
  return v.tag == TAG_DOUBLE;
}

double valueAsDouble(value v) {
  return (double) v.value;
}

bool valueIsRecord(value v) {
  return v.tag == TAG_RECORD;
}

record *valueAsRecord(value v) {
  return (record *) v.value;
}

bool valueIsBool(value v) {
  return v.tag == TAG_BOOL;
}

bool valueAsBool(value v) {
  return (bool) v.value;
}

value getProperty(value v, symbol s) {
  assert(valueIsRecord(v));
  record *record = valueAsRecord(v);
  record_def *def = record->def;

  uint32_t size = def->prop_size;
  uint32_t idx = s % size;

  field_entry *fields = (field_entry *)(def + 1);

  while (fields[idx].symbol != s) {
    idx = (idx + 1) % size;

    // Theoretically, the property should always exist. This assert catches
    // the case when it doesn't exist for debugging purposes.
    assert(idx != s % size);
  }

  return ((value *)(record+1))[fields[idx].offset];
}

void *getMethod(value v, symbol s) {
  assert(valueIsRecord(v));
  record *record = valueAsRecord(v);
  record_def *def = record->def;

  uint32_t size = def->mthd_size; // TODO: Eww, double pointers :s
  uint32_t idx = s % size;

  // Move past the properties & header
  mthd_entry *mthds = (mthd_entry *)(((field_entry *)(def + 1)) + def->prop_size);

  while (mthds[idx].symbol != s) {
    idx = (idx + 1) % size;

    // Theoretically, the property should always exist. This assert catches
    // the case when it doesn't exist for debugging purposes.
    assert(idx != s % size);
  }

  return mthds[idx].fn;
}

value allocRecord(size_t size) {
  value v = { .tag = TAG_RECORD };
  v.value = (size_t) GC_MALLOC(size);
  return v;
};

void __ducky__main();
int main() {
  // TODO(michael): Store the cmd line arguments somewhere

  GC_INIT();

  __ducky__main();

  return 0;
}
