# Type Inference Algorithm

The type inference algorithm is based off of the [Algorithm W](http://prooftoys.org/ian-grant/hm/milner-damas.pdf). for the Hindley-Milner Type System.

## Preconditions
Before the type inference algorithms are run, we have done some preprocessing. These are the
assumptions which we are going to make about the state of the system at this time:

- All identifiers are unique, and are assigned exactly once. There is no shadowing. This was done in an earlier step.
- Every identifier has been assigned a unique unbounded type variable.
- There is a source of unlimited unique type variables

## Unify

> Proposition 3 (Robinson). There is an algorithm U which, given a pair of types, either
> returns a substitution V or fails; further
> 
> (i) If U(τ,τ_0) returns V , then V unifies τ and τ_0, i.e. V τ = τ_0.
> 
> (ii) If S unifies τ and τ_0 then U(τ,τ_0)returns some V and there is another substitution R such that S = RV .
>
> Moreover, V involves only variables in τ and τ_0.


```
unify (Name a) b =
    if let Some(ty) = lookup a {
        unify ty b
    } else {
        b
    }
unify a (Name b) =
    if let Some(ty) = lookup b {
        unify a ty
    } else {
        a // Check this is right
    }
unify (Fn (argsa...) -> resa) (Fn (argsb...) -> resb) =
    if argsa.len() != argsb.len() { Err("Unable to unify") }

    subst = emptySet;

    for (arga, argb) in zip argsa argsb {
        subst.add(try!(unify (subst arga) argb)) // Can I subst on argb? & will it ever matter?
    }

    subst.add(try!(unify (subst resa) resb))
unify (Rec namesa reca) (Rec namesb recb) =
    // I'm pretty sure that this current pseudocode implementation is horrifically wrong,
    // but I will come back to it later and possibly clean it up
    subst = emptySet;

    // Can only have at most one unbound item?
    aunbound = None
    bunbound = None
    for namea in namesa {
        if let Some(ty) = lookup namea {
            // TODO: Require present on both if not mergeable
            try!(merge_into reca ty) // Add the fields into the type rec. Not sure if sufficient?
        } else {
            if aunbound == None {
                aunbound = Some(namea)
            } else {
                Err("Multiple unbound names in record")
            }
        }
    }

    for nameb in namesb {
        if let Some(ty) = lookup nameb {
            // TODO: Require present on both if not mergeable
            try!(merge_into reca ty) // Add the fields into the type rec. Not sure if sufficient? Maybe needs a fancier solver... darn
        } else {
            if aunbound == None {
                aunbound = Some(nameb)
            } else {
                Err("Multiple unbound names in record")
            }
        }
    }

    // Unify the intersection
    intersection = intersect reca recb
    for (tya, tyb) in zip reca recb { subst.add(try!(unify (subst tya) tyb)) }

    // The elements potentially in both
    both_unlisted = if bunbound != None && aunbound != None {
        SomeNewIdentifier()
    } else { None }

    // Handle elements only in one set or the other
    only_a = only_in_first reca recb
    if bunbound == None && only_a != emptySet { Err() }
    subst.add(bunbound => Rec [both_unlisted] only_a)

    only_b = only_in_first recb reca
    if aunbound == None && only_b != emptySet { Err() }
    subst.add(aunbound => Rec [both_unlisted] only_b)

    subst

```

### Problems
- Recursive records could lead to infinite loop. Detect infinite loops & allow for early abortion. Detect if two types are different ASAP.

## The Inference Algorithm
Algorithm W makes some assumptions which are difficult to cope with in our language. For example, the primitive algorithm doesn't permit mutual recursion, which is very valuable with our weird type system.

Algorithm W is as follows (mostly, this was copy pasted from a PDF, so the formatting is off):
    W (A,e) = (S,τ) where

    (i) If e is x and there is an assumption x :∀α_1,...,α_nτ' in A then S = Id2 and τ = [βi /αi]τ' where the β_i s are new.

    (ii) If e is e1 e2 then let W(A,e_2) = (S_1 ,τ_2) and W (S_1A,e_2) = (S_2 ,τ2) and U(S_2τ_1,τ2 → β) = V where β is new; then S = V S_2 S_1 and τ = V β.

    (iii) If e is λx.e1 then let β be a new type variable and W(A_x ∪{x :β},e_1) = (S_1 ,τ_1); then S = S_1 and τ = S_1β → τ_1.

    (iv) If e is let x = e_1 in e_2 then let W (A,e_1) = (S_1 ,τ_2) and W (S_1Ax∪{x : S_1A(τ_1)},e_2) = (S_2 ,τ_2); then S = S_2 S_1 and τ = τ_2.

```
algW env (Ident x) =
    (emptySet, (lookup env x).unwrap)
algW env (App e1 e2) =
    let (S1, T1) = algW(env, e1)
    let (S2, T2) = algW(S1(env), e2)
    let V = unify(S1(T1), T2 -> newVariableBeta)
    (V(S2(S1)), V(newVariableBeta))
algW env (Abs x e) =
    let B = newVariableBeta // Actually unnecessary because already done for all variables
    let (S1, T1) = algW(env, e)
    (S1, (lookup S1(env) x) -> T1)
// Notably missing: methods, record literals, record extensions

// Not really in algW as it doesn't have a type... and isn't an expression... 
algW env (Let x e) =
    let (S1, T1) = algW(env, e)
    // Unify the type for x into x. If this fails, we're screwed
    let V = unify((lookup S1(env) x), T1)
    (V(S1), _)
```
