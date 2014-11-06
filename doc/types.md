# Ducky's Type System

> The syntax in this document isn't final, Ducky is still in very early development.
> In addition, the syntax which I am using here doesn't look that good, and is probably
> ambiguous in some way. I haven't really put any effort into it.

## Goals of the Ducky Type System
- Code written in Ducky should not require explicit type declarations
- Writing code in Ducky should feel like writing code in a runtime-checked dynamic programming language, except that they 
- Ducky's type system should not prevent interaction with code written in a nominal type system

## Primitive Types
Ducky has a very small type system, which allows for massive flexibility. It consists of only two primitive types: Functions and Records.

### Functions
Functions are of the form `(Ty, Ty, ...) -> Ty`, and represent a mapping from types to types. They can have side effects. Functions are first-class in Ducky, and can be easily passed around as values.

### Records
Records contain some set of key-value pairs (`ident: Ty`), and a set of methods (`ident: (self, ...) -> Ty`). They can be written in an extensible manner: `a{ ident: Ty }` represents the record containing all of the fields and methods of `a`, with the addition of `ident: Ty`. Multiple records can be composed together with `a:b{ ident: Ty }`. In this case, `a`, `b` and `{ ident: Ty }` may share no fields or methods.


#### Named Records
Sometimes, you need to avoid Ducky's duck typing system, and prevent custom objects which look the same from being declared and passed into functions. This is mostly important for FFIs and other low level concepts which are not implemented within Ducky's structural type system, but rather in a nominal type system.

You can declare a name with the following syntax:
```
name MyType
```

Now, there is a non-exported, bottom record `#{MyType}#` which has a unique, non-copyable, type: `#MyType#`. It is impossible for user code to fake a value of type `#MyType#`, which means that any function which accepts a value of that type, must accept a value which was constructed by your code.

The name would be used:
```
type MyType = #MyType#{ some_method: (self) -> Int }

x : () -> MyType
x := fn () {
    #{MyType}#:{
        some_method: fn (self) { 5 }
    }
}
```

Right now this is really ugly, so it will probably be changed at some point when I figure out a better way to do it.

### Type Aliasing
Functions and Records may be `aliased`, permitting recursive record data structures. Below are some examples of traditional data types implemented in Ducky:

```
type Nat = { is_succ: (self) -> Bool, is_zero: (self) -> Bool }
type Succ = Nat{ of: Nat }
type Zero = Nat

type List[A] = { get_car: (self) -> Maybe[A], get_cdr: (self) -> Maybe[List[A]] }
type Cons[A] = List[A]{ car: A, cdr: List[A] }
type Zero[A] = List[A]
```

## Builtin Types
It doesn't make sense to model all numbers as recursive Records. Because of that, there are a few built-in types:
```
type Num[a] = { '+: (self, a) -> a, '-: (self, a) -> a, '*: (self, a) -> a, '/: (self, a) -> a }

type Int = [[Int]]:Num[Int]
```

Where `[[Int]]` is the internal name `Int`, which cannot be created for custom records in user code. You can't actually write `[[Int]]` as it is not exported to user code. Only the fundamental type `Int` is exported
