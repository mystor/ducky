use string_cache::Atom;
use self::Token::*;

macro_rules! nom {
    ($stream:ident -> { $($patt:expr => $action:expr),+ }) => (
        $(if let Some((0, len)) = regex!($patt).find($stream) {
            let val = $stream.slice_to(len);
            $stream = $stream.slice_from(len);
            Some($action(val))
        })else+ else {
            None
        }
    )
}

#[allow(non_camel_case_types)]
#[deriving(Show)]
pub enum Token {
    // Operators
    EQ,
    LT,
    LE,
    EQEQ,
    NE,
    GE,
    GT,
    ANDAND,
    OROR,
    NOT,
    PLUS,
    MINUS,
    STAR,
    SLASH,
    PERCENT,
    CARET,
    AND,
    OR,

    // Structural
    LBRACE,
    RBRACE,
    LBRACKET,
    RBRACKET,
    LPAREN,
    RPAREN,
    DOT,
    COMMA,
    SEMI,
    COLON,
    RARROW,
    LARROW,
    FAT_ARROW,

    // Keywords
    FN,
    LET,
    TRUE,
    FALSE,
    MATCH,
    AS,
    IF,
    ELSE,

    // Literals
    LIT_INTEGER(i64),
    LIT_FLOAT(f64),
    LIT_STR(Atom),

    // Identifier
    IDENT(Atom),
}

pub fn lex(program: &str) -> Result<Vec<Token>, String> {
    let mut toks: Vec<Token> = vec![];
    let mut stream = program.clone();

    let start_len = stream.char_len();

    while stream.char_len() > 0 {
        if let Some((0, len)) = regex!(r"^\s+").find(stream) {
            // Skip all spaces
            stream = stream.slice_from(len);
        } else if let Some(tok) = nom!(stream -> {
            // Brackets, Braces, and Parens
            r"^\{" => |_| { LBRACE },
            r"^\}" => |_| { RBRACE },
            r"^\(" => |_| { LPAREN },
            r"^\)" => |_| { RPAREN },
            r"^\[" => |_| { LBRACKET },
            r"^\]" => |_| { RBRACKET },

            // Arrows
            r"^->" => |_| { RARROW },
            r"^<-" => |_| { LARROW },
            r"^=>" => |_| { FAT_ARROW },

            // Logical Operators
            r"^<=" => |_| { LE },
            r"^>=" => |_| { GE },
            r"^!=" => |_| { NE },
            r"^<"  => |_| { LT },
            r"^>"  => |_| { GT },
            r"^==" => |_| { EQEQ },
            r"^="  => |_| { EQ },
            r"^&&" => |_| { ANDAND },
            r"^&"  => |_| { AND },
            r"^\|\|" => |_| { OROR },
            r"^\|" => |_| { OR },
            r"^!"  => |_| { NOT },

            // Mathematical Operators
            r"^\+" => |_| { PLUS },
            r"^-"  => |_| { MINUS },
            r"^\*" => |_| { STAR },
            r"^/"  => |_| { SLASH },
            r"^%"  => |_| { PERCENT },
            r"^\^" => |_| { CARET },

            // Structural
            r"^\." => |_| { DOT },
            r"^,"  => |_| { COMMA },
            r"^;"  => |_| { SEMI },
            r"^:"  => |_| { COLON },

            // The interesting ones
            r"^[a-zA-Z_][a-zA-Z0-9_]*" => |v: &str| {
                match v {
                    "fn" => FN,
                    "let" => LET,
                    "true" => TRUE,
                    "false" => FALSE,
                    "match" => MATCH,
                    "as" => AS,
                    "if" => IF,
                    "else" => ELSE,
                    _ => IDENT(Atom::from_slice(v)),
                }
            },
            r#"^"([^"]|\\")""# => |v: &str| { LIT_STR(Atom::from_slice(v)) }, // TODO: Better string parsing
            r"^[0-9]*\.[0-9]+" => |v: &str| { LIT_FLOAT(from_str(v).unwrap()) },
            r"^[0-9]+" => |v: &str| { LIT_INTEGER(from_str(v).unwrap()) }
        }) {
            toks.push(tok);
        } else {
            return Err(format!("{}: {}", start_len - stream.char_len(), stream.char_at(0)));
        }
    }

    Ok(toks)
}
