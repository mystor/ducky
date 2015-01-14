use std::str::FromStr;
use intern::Atom;
use self::Token::*;

macro_rules! nom {
    ($stream:ident | $var: ident | -> { $($patt:expr => $action:expr),+ }) => (
        $(if let Some((0, len)) = regex!($patt).find($stream) {
            let $var = $stream.slice_to(len);
            $stream = $stream.slice_from(len);
            Some($action)
        })else+ else {
            None
        }
    )
}

#[allow(non_camel_case_types)]
#[derive(Show)]
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

    let start_len = stream.len(); // TODO(michael): Figure out why char_len doesn't work anymore

    while stream.len() > 0 {
        if let Some((0, len)) = regex!(r"^\s+").find(stream) {
            // Skip all spaces
            stream = stream.slice_from(len);
        } else if let Some(tok) = nom!(stream |_v| -> {
            // Brackets, Braces, and Parens
            r"^\{" => { LBRACE },
            r"^\}" => { RBRACE },
            r"^\(" => { LPAREN },
            r"^\)" => { RPAREN },
            r"^\[" => { LBRACKET },
            r"^\]" => { RBRACKET },

            // Arrows
            r"^->" => { RARROW },
            r"^<-" => { LARROW },
            r"^=>" => { FAT_ARROW },

            // Logical Operators
            r"^<=" => { LE },
            r"^>=" => { GE },
            r"^!=" => { NE },
            r"^<"  => { LT },
            r"^>"  => { GT },
            r"^==" => { EQEQ },
            r"^="  => { EQ },
            r"^&&" => { ANDAND },
            r"^&"  => { AND },
            r"^\|\|" => { OROR },
            r"^\|" => { OR },
            r"^!"  => { NOT },

            // Mathematical Operators
            r"^\+" => { PLUS },
            r"^-"  => { MINUS },
            r"^\*" => { STAR },
            r"^/"  => { SLASH },
            r"^%"  => { PERCENT },
            r"^\^" => { CARET },

            // Structural
            r"^\." => { DOT },
            r"^,"  => { COMMA },
            r"^;"  => { SEMI },
            r"^:"  => { COLON },

            // The interesting ones
            r"^[a-zA-Z_][a-zA-Z0-9_]*" => {
                match _v {
                    "fn" => FN,
                    "let" => LET,
                    "true" => TRUE,
                    "false" => FALSE,
                    "if" => IF,
                    "else" => ELSE,
                    _ => IDENT(Atom::from_slice(_v)),
                }
            },
            r#"^"([^"]|\\")""# => { LIT_STR(Atom::from_slice(_v)) }, // TODO: Better string parsing
            r"^[0-9]*\.[0-9]+" => { LIT_FLOAT(FromStr::from_str(_v).unwrap()) },
            r"^[0-9]+" => { LIT_INTEGER(FromStr::from_str(_v).unwrap()) }
        }) {
            toks.push(tok);
        } else {
            return Err(format!("{}: {}", start_len - stream.len(), stream.char_at(0)));
        }
    }

    Ok(toks)
}
