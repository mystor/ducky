use std::iter::repeat;
use std::collections::{HashMap, RingBuf};
use rusty_llvm as llvm;
use il::*;

struct SymbolTable {
    symbols: HashMap<Symbol, u64>,
    counter: u64
}

impl SymbolTable {
    fn new() -> SymbolTable {
        SymbolTable {symbols: HashMap::new(), counter: 0}
    }

    fn lookup(&mut self, s: Symbol) -> u64 {
        match self.symbols.entry(s).get() {
            Ok(v) => *v,
            Err(mut e) => {
                self.counter += 1;
                e.insert(self.counter);
                self.counter
            }
        }
    }
}

enum ValueTag {
    RECORD = 0,
    DOUBLE = 1,
    UINT32 = 2,
    BOOL = 3,
    STRING = 4,
    NULL = 5
}

#[derive(Clone)]
enum Value<'a> {
    Unk{
        ll: &'a llvm::Value
    },
    KNum{
        ll: &'a llvm::Value
    },
    KString{
        ll: &'a llvm::Value,
        len: i64
    },
    KBool{
        ll: &'a llvm::Value
    },
    KRec{
        ll: &'a llvm::Value,
        rec: Record<'a>
    },
    KNull
}

impl <'a> Value<'a> {
    fn to_unk(&self, ctx: &mut GenContext<'a>) -> Value<'a> {
        match *self {
            Value::Unk{ll} => Value::Unk{ll: ll},
            Value::KNum{ll} => {
                Value::Unk{
                    ll: ctx.ctx.const_struct(
                        &vec![ctx.ctx.int8_type().unwrap()
                              .const_int(ValueTag::DOUBLE as u64, false).unwrap(),
                              ll][], // TODO(michael): Probably wrong type
                        false).unwrap()
                }
            }
            Value::KString{ll, len:_} => {
                Value::Unk{
                    ll: ctx.ctx.const_struct(
                        &vec![ctx.ctx.int8_type().unwrap()
                              .const_int(ValueTag::STRING as u64, false).unwrap(),
                              ll][], // TODO(michael): Probably wrong type...
                        false).unwrap()
                }
            }
            Value::KBool{ll} => {
                Value::Unk{
                    ll: ctx.ctx.const_struct(
                        &vec![ctx.ctx.int8_type().unwrap()
                              .const_int(ValueTag::BOOL as u64, false).unwrap(),
                              ll][],
                        false).unwrap()
                }
            }
            Value::KRec{ll, ..} => {
                Value::Unk{
                    ll: ctx.ctx.const_struct(
                        &vec![ctx.ctx.int8_type().unwrap()
                              .const_int(ValueTag::RECORD as u64, false).unwrap(),
                              ll][],
                        false).unwrap()
                }
            },
            Value::KNull => {
                Value::Unk{
                    ll: ctx.ctx.const_struct(
                        &vec![ctx.ctx.int8_type().unwrap().const_int(ValueTag::NULL as u64, false).unwrap(),
                              ctx.ctx.int64_type().unwrap().const_int(0, false).unwrap()][],
                        false).unwrap()
                }
            }
        }
    }

    fn to_unk_ll(&self, ctx: &mut GenContext<'a>) -> &'a llvm::Value {
        if let Value::Unk{ll} = self.to_unk(ctx) {
            ll
        } else {
            panic!("to_unk doesn't work!");
        }
    }
}

struct RecDef<'a> {
    props: HashMap<Symbol, u64>,
    mthds: HashMap<Symbol, &'a llvm::Value>,
    cache: Option<&'a llvm::Value>,
}

impl <'a> RecDef<'a> {
    fn new(rec: &mut Record<'a>, ctx: &mut GenContext<'a>) -> RecDef<'a> {
        let mut rd = RecDef {
            props: HashMap::new(),
            mthds: HashMap::new(),
            cache: None
        };

        let mut offset = 0;

        for s in rec.props.keys() {
            rd.add_prop(s.clone(), offset);
            offset += 1;
        }
        for (s, ref mut m) in rec.mthds.iter_mut() {
            rd.add_mthd(s.clone(), m.get_function(ctx));
        }

        rd.gen(ctx);

        rd
    }
    //| The memory footprint of the record definition
    fn size(&self) -> u64 {
        const HEADER_SIZE: u64 = 8;
        const PROP_SIZE: u64 = 12;
        const METHOD_SIZE: u64 = 12;
        let props_size = PROP_SIZE * self.props.len() as u64;
        let mthds_size = METHOD_SIZE * self.mthds.len() as u64;

        HEADER_SIZE + props_size + mthds_size
    }

    fn add_prop(&mut self, symb: Symbol, offset: u64) {
        self.props.insert(symb, offset);
    }

    fn add_mthd(&mut self, symb: Symbol, func: &'a llvm::Value) {
        self.mthds.insert(symb, func);
    }

    fn gen(&mut self, ctx: &mut GenContext<'a>) -> &'a llvm::Value {
        if let Some(v) = self.cache {
            v
        } else {
            let i32t = ctx.ctx.int32_type().unwrap();
            let i64t = ctx.ctx.int64_type().unwrap();

            let mut vals = vec![
                i32t.const_int(self.props.len() as u64, false).unwrap(),
                i32t.const_int(self.mthds.len() as u64, false).unwrap()];

            let mut props = Vec::with_capacity(2*self.props.len());
            // Initialize to Nones
            for _ in self.props.iter() {
                props.push(None); props.push(None);
            }
            for (symb, offset) in self.props.iter() {
                let sti = ctx.symbol_table.lookup(symb.clone());
                let mut i = (sti as usize) % self.props.len();
                loop {
                    if props[2*i].is_none() {
                        props[2*i] = Some(i32t.const_int(sti, false).unwrap());
                        props[2*i+1] = Some(i64t.const_int(*offset, false).unwrap());
                        break;
                    } else {
                        i = (i+1) % self.props.len();

                        // Prevent infinite loops
                        assert!(i != (sti as usize) % self.props.len());
                    }
                }
            }

            let mut mthds = Vec::with_capacity(2*self.mthds.len());
            // Initialize to Nones
            for _ in self.mthds.iter() {
                mthds.push(None); mthds.push(None);
            }
            for (symb, func) in self.mthds.iter() {
                let sti = ctx.symbol_table.lookup(symb.clone());
                let mut i = (sti as usize) % self.mthds.len();
                loop {
                    if mthds[2*i].is_none() {
                        mthds[2*i] = Some(i32t.const_int(sti, false).unwrap());
                        mthds[2*i+1] = Some(*func);
                        break;
                    } else {
                        i = (i+1) % self.mthds.len();

                        // Prevent infinite loops
                        assert!(i != (sti as usize) % self.mthds.len());
                    }
                }
            }

            let props = props.iter().map(|x| x.unwrap());
            let mthds = mthds.iter().map(|x| x.unwrap());

            let vals: Vec<_> = vals.iter().cloned().chain(props).chain(mthds).collect();

            let cs = ctx.ctx.const_struct(&vals[], true).unwrap();
            self.cache = ctx.module.add_global(cs.type_of().unwrap(), "recorddef");
            let globl = self.cache.unwrap();

            globl.set_initializer(cs);
            globl
        }
    }
}

#[derive(Clone)]
struct Record<'a> {
    props: HashMap<Symbol, Value<'a>>,
    mthds: HashMap<Symbol, Method<'a>>,
}

impl <'a> Record<'a> {
    fn new() -> Record<'a> {
        Record{
            props: HashMap::new(),
            mthds: HashMap::new()
        }
    }

    fn add_prop(&mut self, s: Symbol, v: Value<'a>) {
        self.props.insert(s, v);
    }

    fn add_mthd(&mut self, s: Symbol, v: Method<'a>) {
        self.mthds.insert(s, v);
    }
}

#[derive(Clone)]
struct Method<'a> {
    // record: Record<'a>,
    params: Vec<Ident>,
    body: Expr,
    implementation: Option<&'a llvm::Value>
}

impl<'a> Method<'a> {
    fn new(params: Vec<Ident>, body: Expr) -> Method<'a> {
        Method{
            params: params,
            body: body,
            implementation: None
        }
    }

    fn get_function(&mut self, ctx: &mut GenContext<'a>) -> &'a llvm::Value {
        let param_tys: Vec<_> = repeat(ctx.value_type()).take(self.params.len()).collect();

        if self.implementation.is_none() {
            self.implementation = ctx.module.add_function(
                "generic_function_name",
                llvm::Type::function_type(ctx.value_type(),
                                          &param_tys[],
                                          false).unwrap());
        }

        self.implementation.unwrap()
    }
}

struct GenContext<'a> {
    ctx: &'a llvm::Context,
    builder: &'a llvm::Builder,
    module: &'a llvm::Module,
    method_queue: RingBuf<Method<'a>>,
    symbol_table: SymbolTable,
}

impl <'a> GenContext<'a> {
    fn new() -> GenContext<'a> {
        unimplemented!()
    }

    fn value_type(&self) -> &'a llvm::Type {
        self.ctx.struct_type(
            // TODO(michael): Non-64-bit computers
            &vec![self.ctx.int8_type().unwrap(),
                  self.ctx.int64_type().unwrap()][],
            false).unwrap()
    }

    fn bi_alloc_record(&self) -> &'a llvm::Value {
        self.module.get_named_function("allocRecord").unwrap()
    }

    fn bi_get_property(&self) -> &'a llvm::Value {
        self.module.get_named_function("getProperty").unwrap()
    }

    fn bi_get_method(&self) -> &'a llvm::Value {
        self.module.get_named_function("getMethod").unwrap()
    }
}

fn gen_expr<'a>(e: &Expr, ctx: &mut GenContext<'a>) -> Value<'a> {
    match *e {
        Expr::Literal(ref lit) => {
            match *lit {
                Literal::Str(ref atom) => {
                    Value::KString{
                        ll: ctx.builder.build_global_string(atom.as_slice(), "_string_").unwrap(),
                        len: atom.as_slice().len() as i64
                    }
                }
                Literal::Int(i) => {
                    Value::KNum{ // TODO(michael): Real Ints maybe?
                        ll: ctx.ctx.float_type().unwrap().const_real(i as f64).unwrap()
                    }
                }
                Literal::Float(f) => {
                    Value::KNum{
                        ll: ctx.ctx.float_type().unwrap().const_real(f).unwrap()
                    }
                }
                Literal::Bool(b) => {
                    Value::KBool{
                        ll: ctx.ctx.int1_type().unwrap().const_int(b as u64, false).unwrap()
                    }
                }
            }
        }
        Expr::Ident(ref id) => {
            // TODO(michael): Add environments, and allow looking up vars
            Value::Unk{
                ll: unimplemented!()
            }
        }
        Expr::Rec(ref props) => {
            // Create the record object
            let mut rec = Record::new();
            for prop in props.iter() {
                match *prop {
                    Prop::Val(ref s, ref expr) => {
                        let value = gen_expr(expr, ctx);
                        rec.add_prop(s.clone(), value);
                    }
                    Prop::Method(ref s, ref args, ref body) => {
                        let mthd = Method::new(args.clone(), body.clone());
                        rec.add_mthd(s.clone(), mthd);
                    }
                }
            }

            // Create the record definition object
            let mut rec_def = RecDef::new(&mut rec, ctx);

            // Allocate the record
            let alloced_rec = ctx.builder.build_call(
                ctx.bi_alloc_record(),
                &vec![ctx.ctx.int64_type().unwrap().const_int(rec_def.size(), false).unwrap()][],
                "record").unwrap();

            let zero = ctx.ctx.int64_type().unwrap().const_int(0, false).unwrap();
            // Set the properties!
            ctx.builder.build_store(
                rec_def.gen(ctx), // Pointer to the record definition
                ctx.builder.build_in_bounds_gep(
                    alloced_rec,
                    &vec![zero, zero][],
                    "record_def_ptr").unwrap());

            let values_ptr = ctx.builder.build_bit_cast(
                ctx.builder.build_in_bounds_gep(
                    alloced_rec,
                    &vec![ctx.ctx.int64_type().unwrap().const_int(1, false).unwrap()][],
                    "values_ptr_uncast").unwrap(),
                ctx.value_type().pointer_type(0).unwrap(),
                "values_ptr").unwrap();

            for (symb, idx) in rec_def.props.iter() {
                ctx.builder.build_store(
                    rec.props[symb.clone()].to_unk_ll(ctx),
                    ctx.builder.build_in_bounds_gep(
                        values_ptr, &vec![
                            ctx.ctx.int64_type().unwrap().const_int(*idx, false).unwrap()
                                ][], "value_ptr").unwrap());
            }

            Value::KRec{ll: alloced_rec, rec: rec}
        }
        Expr::Member(ref obj, ref symb) => {
            let objv = gen_expr(&**obj, ctx);
            // TODO(michael): Directly index known types,
            // rather than performing expensive lookups

            let ll = objv.to_unk_ll(ctx);
            let symbol_ll = ctx.symbol_table.lookup(symb.clone());
            Value::Unk{
                ll: ctx.builder.build_call(
                    ctx.bi_get_property(),
                    &vec![ll,
                        ctx.ctx.int64_type().unwrap().const_int(symbol_ll, false).unwrap()][],
                    "get_property").unwrap()
            }
        }
        Expr::Call(ref obj, ref symb, ref args) => {
            let objv = gen_expr(&**obj, ctx);
            // TODO(michael): Directly index known types,
            // rather than performing expensive lookups

            let ll = objv.to_unk_ll(ctx);
            let symbol_ll = ctx.symbol_table.lookup(symb.clone());
            let method_ll = ctx.builder.build_call(
                ctx.bi_get_method(),
                &vec![ll,
                      ctx.ctx.int64_type().unwrap().const_int(symbol_ll, false).unwrap()][],
                "get_method").unwrap();

            // Determine the function type we want
            let mut ptypes = Vec::with_capacity(args.len());
            for _ in args.iter() { ptypes.push(ctx.value_type()); }
            let ftype = llvm::Type::function_type(
                ctx.value_type(),
                &ptypes[],
                false).unwrap();


            let method_ll = ctx.builder.build_bit_cast(
                method_ll,
                ftype.pointer_type(0).unwrap(),
                "typed_method").unwrap();

            let mut args_ll = Vec::with_capacity(args.len());
            for arg in args.iter() {
                args_ll.push(gen_expr(arg, ctx).to_unk_ll(ctx));
            }

            Value::Unk{
                ll: ctx.builder.build_call(
                    method_ll,
                    &args_ll[],
                    "method_result").unwrap()
            }
        }
        Expr::Block(ref body) => {
            let mut val = Value::KNull;
            for stmt in body.iter() {
                val = gen_stmt(stmt, ctx);
            }

            val
        }
        Expr::If(ref cond, ref cons, ref alt) => {
            unimplemented!()
        }
    }
}


fn gen_stmt<'a>(stmt: &Stmt, ctx: &mut GenContext<'a>) -> Value<'a> {
    match *stmt {
        Stmt::Let(ref id, ref expr) =>  {
            unimplemented!()
        }
        Stmt::Expr(ref expr) => gen_expr(expr, ctx),
        Stmt::Empty => Value::KNull
    }
}
