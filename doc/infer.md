# Type Inference Algorithm

The type inference algorithm is based off of the [Algorithm W](http://prooftoys.org/ian-grant/hm/milner-damas.pdf). for the Hindley-Milner Type System.

## Preconditions
Before the type inference algorithms are run, we have done some preprocessing. These are the
assumptions which we are going to make about the state of the system at this time:

- All identifiers are unique, and are assigned exactly once. There is no shadowing. This was done in an earlier step.
- Every identifier has been assigned a unique unbounded type variable.
- There is a source of unlimited unique type variables


## Algorithm
Currently the most correct implementation of the algorithm is the one implemented in `src/infer.rs`. Hopefully a formal definition will be located here at some point in the future, and will be verified for correctness, but right now I don't have that.
