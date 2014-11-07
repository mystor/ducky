# Type Inference Algorithm

The type inference algorithm is based off of the [Algorithm W](http://prooftoys.org/ian-grant/hm/milner-damas.pdf). for the Hindley-Milner Type System.

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
