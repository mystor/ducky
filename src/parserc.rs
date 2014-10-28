#[deriving(Show, PartialEq, Eq)]
pub struct Pos{
    pub line: int,
    pub col: int,
}

impl Pos {
    fn next(&self, c: &char) -> Pos {
        match *c {
            '\n' => Pos{line: self.line + 1, col: 0},
            _ => Pos{line: self.line, col: self.col + 1}
        }
    }
}

#[deriving(Show)]
pub struct State<'a>(pub &'a str, pub Pos);

// pub type Parser<'a, a> = fn (State<'a>) -> Result<(a, State<'a>), String>;
pub type Parser<'a, A> = Result<(A, State<'a>), String>;

pub fn anycp<'a>(pred: |&char| -> bool, state: &State<'a>) -> Parser<'a, char> {
    let &State(s, p) = state;
    let c = s.char_at(0);
    if pred(&c) {
        Ok((c, State(s.slice_from(1), p.next(&c))))
    } else {
        Err(format!("Unexpected {} at {}", c, p))
    }
}

pub fn ab<'a>(st: &State<'a>) -> Parser<'a, String> {
    let (_, st) = try!(anycp(|c| { *c == 'a' }, st));
    let (_, st) = try!(anycp(|c| { *c == 'b' }, &st));
    Ok(("ab".to_string(), st))
}

macro_rules! parser {
    ($name:ident( $($fnarg:ident : $fnty:ty),* ) -> $resty:ty {
        $($x:ident <- $parser:ident($($arg:expr),*));*
        $res:expr
    }) => (
        fn $name<'a>($($fnarg: $fnty,)* state: &State<'a>) -> Parser<'a, $resty> {
            $(let ($x, state) = try!($parser($($arg,)*, state));)*
            Ok(($res, state))
        }
    )
}
// parser! {
//     ab() -> String {
//         a <- anycp(|c| { *c == 'a' });
//         b <- anycp(|c| { *c == 'b' });
//         "ab".to_string()
//     }
// }
