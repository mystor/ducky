macro_rules! nom {
    ($stream:ident -> { $($patt:expr => $action:expr),+ }) => (
        {
            $(if let Some((0, len)) = regex!($patt).find($stream) {
                let val = $stream.slice_to(len);
                $stream = $stream.slice_from(len);
                Some($action(val))
            })else+ else {
                None
            }
        }
    )
}


#[deriving(Show)]
pub enum Token {
    Lbrace,
    Rbrace,
    Lparen,
    Rparen,
    Lsqb,
    Rsqb,
    Equals,
    Comma,
    Dot,
    Colon,
    SemiColon,
    Times,
    Plus,
    Pipe,
    Ident(String),
    Int(int)
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
            r"^\{" => |_| { Lbrace },
            r"^\}" => |_| { Rbrace },
            r"^\(" => |_| { Lparen },
            r"^\)" => |_| { Rparen },
            r"^\[" => |_| { Lsqb },
            r"^\]" => |_| { Rsqb },
            r"^="  => |_| { Equals },
            r"^,"  => |_| { Comma },
            r"^\." => |_| { Dot },
            r"^:"  => |_| { Colon },
            r"^;"  => |_| { SemiColon },
            r"^\*" => |_| { Times },
            r"^\+" => |_| { Plus },
            r"^\|" => |_| { Pipe },
            r"^[a-zA-Z_][a-zA-Z0-9_]*" => |v:&str| { Ident(v.to_string()) },
            r"^[0-9]+" => |v:&str| { Int(from_str(v).unwrap()) }
        }) {
            toks.push(tok);
        } else {
            return Err(format!("{}: {}", start_len - stream.char_len(), stream.char_at(0)));
        }
    }
    
    Ok(toks)
}
