#![allow(dead_code)]
use intern::Atom;
use std::collections::{HashMap, BTreeMap};
use il::*;

/// A MethodImpl represents the implementation of a method. Each MethodImpl
/// has at most one associated RecordImpl which the MethodImpl is implemented on.
/// when the MethodImpl is called, an object with that impl is passed in.
///
/// I'm not quite sure what the best way to do that circular reference is, so I'm
/// just not doing it right now.
///
/// In addition, the MethodImpl has a bunch of specializations which are implemented on it
#[derive(Clone)]
struct MethodImpl {
    /// The parameters which are required for the method
    params: Vec<Ident>,
    /// The body of the method, woop!
    body: Expr,

    specializations: HashMap<Vec<ValImpl>, MethodSpec>
}

/// Its a value in our language! Aren't you excited!
#[derive(Clone)]
enum ValImpl {
    /// A struct representing a record, very handy!
    Rec(RecordImpl),
    /// A discriminated union, they are distinguished by arbitrary identifiers!
    /// its great! It will be implemented by some sort of structure with an int,
    /// and a union of pointers or some shit. Maybe we'll put the other values inline
    /// or something like that, but there will definitely be a ton of pointer stuff
    Union(Vec<ValImpl>),
    /// An integer (woah, we have real numbers in this language!)
    Int,
    /// A floating point number (crazy, we have numbers which aren't ints!)
    Float,
    /// Strings, woo! (not interned :( )
    String,
    /// Bool! Yeah! Booyeah! Boom! Headshot!
    Bool,
}

impl ValImpl {
}

/// A RecordImpl represents an implementation of a record in memory.
/// Two RecordImpls can have the same type, but will create different
/// specializations of a function which they are called with!
///
/// There is one RecordImpl for every set of methods and prop types.
/// an environment is also passed in. Currently I'm making it a RecordImpl,
/// but I may switch it to something else.
#[derive(Clone)]
struct RecordImpl {
    /// We need some arbitrary identifier thing so that we can do enums easily and stuff
    /// It should start at like 8 or something so that I can have reserved numbers
    /// for Ints/Floats/Strings/Bools/whateverelsestrikesmyfancy!
    /// Maybe I can do this in that hashmap thing? That could be cool.
    /// It would be nice if this didn't have to be in the RecordImpl possibly,
    /// but I'm not really sure
    some_arbitrary_identifier_thing: i32,
    // TODO: Should this be here even?
    environment: Option<Box<RecordImpl>>,
    methods: BTreeMap<Symbol, MethodImpl>,
    props: BTreeMap<Symbol, ValImpl>,
}

impl RecordImpl {

}

// WHat is this even? Who knows! I will implement it later
#[derive(Clone)]
struct MethodSpec {
    return_valimpl: ValImpl,
}



/// ExprImpls are kinda like exprs in the main language, except they
/// come with more implementation details! such as the ValImpl which
/// is associated with the type they produced! Woop!
enum ExprImpl {
    /// These are the literal types, we split them out here rather than putting them
    /// like they are in Expr for no good reason.
    StringLiteral(Atom),
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),

    /// We need to have like a refernce to a value which we are looking up... I'm not sure
    /// maybe... umm... I'm really really not sure
    /// it'll be fun
    Ident(bool, i32),

    Rec{
        rimpl: RecordImpl,
        props: BTreeMap<Symbol, ExprImpl>,
    },

    /// Members! Woop! SHould we distinguish between members of unions and members of
    /// recs? No lets just do an if statement thingymabob
    Member{
        rimpl: RecordImpl,
        record: Box<ExprImpl>,
        symb: Symbol,
    },

    /// Calls need to know the particular methodspec which they need
    Call{
        mspec: MethodSpec, // This shiuld probably be different
        args: Vec<Box<ExprImpl>>, // These are the args, first must be "this"
    },



}

impl ExprImpl {
    fn valimpl(&self) -> ValImpl {
        match *self {
            // The literals have nice literal values! Very nice! woop!
            // Its fancy, because we now don't have to do weird pointer shit!
            // We'll probably end up needing fishy union things in the fishy
            // fancy llvm stuff, but whatever
            ExprImpl::StringLiteral(_) => ValImpl::String,
            ExprImpl::IntLiteral(_) => ValImpl::Int,
            ExprImpl::FloatLiteral(_) => ValImpl::Float,
            ExprImpl::BoolLiteral(_) => ValImpl::Bool,

            ExprImpl::Rec{ref rimpl, ..} => ValImpl::Rec(rimpl.clone()),

            // We s
            ExprImpl::Ident(..) => panic!("Shit, who even knows!?!?!?!"),
            ExprImpl::Member{ref rimpl, ref symb, ..} => {
                let props = &rimpl.props;
                if let Some(val) = props.get(symb) {val.clone()} else { panic!("FUCK"); }
            },
            ExprImpl::Call{ref mspec, ..} => mspec.return_valimpl.clone(),

        }
    }

    // TODO: Implement this sort of stuff
    unsafe fn build_ir(&self) {
    }
}

fn foo() {
    let mut hm = BTreeMap::new();
    hm.insert("Hello, World", "World!");
    hm.get(&"Hello, World");
}


/// This is the state object. Its like mutable and stuff. It'll be fun!
struct SpecState {
    foo: i32
}

fn specialize_expr(st: &mut SpecState, expr: &Expr) -> ExprImpl {
    match *expr {
        Expr::Literal(Literal::Str(ref atom)) => {
            // For now, we don't have interning (because that's like complicated)
            // in the target language, so let's just be lazy!
            unimplemented!()
        }
        _ => unimplemented!()
    }
}
