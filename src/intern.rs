use std::fmt;
use std::sync::{Arc, Mutex};
use std::hash;
use std::collections::HashMap;
// This is a temporary file. At some point it would probably be nice to use a real
// string interning solution again, but I wanted to just get something simple working

lazy_static! {
    static ref HASHMAP: Mutex<HashMap<String, Arc<String>>> = Mutex::new(HashMap::new());
}

// TODO: Hash should be more efficient, because we only need to hash based on the
// reference, rather than the value.
#[derive(Clone, PartialOrd, Ord)]
pub struct Atom {
    string: Arc<String>
}


impl Atom {
    pub fn from_slice(string: &str) -> Atom {
        let mut hash_map = HASHMAP.lock().unwrap();

        if let Some(arc) = hash_map.get(string) {
            return Atom { string: arc.clone() };
        }

        // TODO: Check to see if these olympics are necessary
        let string = string.to_string();
        let arc = Arc::new(string.clone());

        hash_map.insert(string, arc.clone());
        Atom { string: arc }
    }
}

impl Str for Atom {
    fn as_slice(&self) -> &str {
        self.string.as_slice()
    }
}

impl PartialEq for Atom {
    fn eq(&self, other: &Atom) -> bool {
        &*self.string as *const _ == &*other.string as *const _
    }
}
impl Eq for Atom {}

impl <S> hash::Hash<S> for Atom where S: hash::Hasher, S: hash::Writer {
    fn hash(&self, state: &mut S) {
        (&*self.string as *const _).hash(state)
    }
}

impl fmt::String for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_slice())
    }
}

impl fmt::Show for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Atom({})", self.as_slice())
    }
}

// Comparisons with strings!
impl<'a> PartialEq<&'a str> for Atom {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_slice() == *other
    }
}
