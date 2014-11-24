# TODO

## Parsing
- [x] Working Lexer
- [x] Working Parser
- [x] AST -> Ducky IR converter (ast == ir now - probably temporary)
- [ ] User Annotated Types
- [ ] Concrete syntax design

## Type Inference
- [x] Infer Function Composition
- [x] Let-Polymorphism
- [x] Extensible Record Types
- [x] Infer from Method Calls
- [x] Type Constructor-level Type Simplification
- [ ] Type Simplification shouldn't expand explicit identifiers
- [ ] Type Property Type Simplification
- [ ] Pretty-printed Types
- [ ] Named Structs (probably as a required "field type")
- [ ] Conditional Statements
- [ ] Type-safe Conditional Statements (Match vs types)
- [x] Primitive Record Definitions
- [ ] Ensure Correctness (many more test cases needed)
- [ ] User Annotated Types
- [ ] Mutable Records/Closures

## Optimizer
Leave for much later

## Code Generator
- [ ] Determine layout of objects in memory
- [ ] Determine layout of functions & closures in memory
- [ ] Implementation of primitive records
- [ ] Emit basic functions
- [ ] Garbage Collection
And more
