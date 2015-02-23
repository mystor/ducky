use std::iter::repeat;
use std::collections::{HashMap, VecDeque};
use il::*;

#[cfg(test)]
mod test;

#[macro_use]
mod llvm;

struct SymbolTable {
    symbols: HashMap<Symbol, u64>,
    counter: u64
}

impl SymbolTable {
    unsafe fn new() -> SymbolTable {
        SymbolTable {
            symbols: HashMap::new(), counter: 0
        }
    }

    unsafe fn lookup(&mut self, s: Symbol) -> u64 {
        match self.symbols.entry(s).get() {
            Ok(v) => *v,
            Err(e) => {
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
enum Value {
    Unk{
        ll: llvm::Value
    },
    KNum{
        ll: llvm::Value
    },
    KString{
        ll: llvm::Value,
        len: i64
    },
    KBool{
        ll: llvm::Value
    },
    KRec{
        ll: llvm::Value,
        rec: Record
    },
    KNull
}

impl Value {
    unsafe fn mk_val_struct(ctx: &mut GenContext,
                     tag: ValueTag,
                     data: llvm::Value) -> Value {
        let unk = ctx.builder.build_alloca(ctx.value_type(), "unk_value_struct");

        ctx.builder.build_store(
            ctx.ctx.int8_type().const_int(tag as u64, false),
            ctx.builder.build_gep(unk, &[
                ctx.ctx.int32_type().const_int(0, false),
                ctx.ctx.int32_type().const_int(0, false)], "value_tag"));

        ctx.builder.build_store(
            data,
            ctx.builder.build_gep(unk, &[
                ctx.ctx.int32_type().const_int(0, false),
                ctx.ctx.int32_type().const_int(1, false)], "data"));

        Value::Unk{ ll: unk }
    }

    unsafe fn to_unk(&self, ctx: &mut GenContext) -> Value {
        let i64t = ctx.ctx.int64_type();

        match *self {
            Value::Unk{ll} => Value::Unk{ll: ll},
            Value::KNum{ll} => {
                let ll = ctx.bit_cast(ll, i64t);
                Value::mk_val_struct(ctx, ValueTag::DOUBLE, ll)
            }
            Value::KString{ll, len:_} => {
                let ll = ctx.bit_cast(ll, i64t);
                Value::mk_val_struct(ctx, ValueTag::STRING, ll)
            }
            Value::KBool{ll} => {
                let ll = ctx.bit_cast(ll, i64t);
                Value::mk_val_struct(ctx, ValueTag::BOOL, ll)
            }
            Value::KRec{ll, ..} => {
                let ll = ctx.bit_cast(ll, i64t);
                Value::mk_val_struct(ctx, ValueTag::RECORD, ll)
            },
            Value::KNull => {
                let zero = ctx.ctx.int64_type().const_int(0, false);
                Value::mk_val_struct(ctx, ValueTag::NULL, zero)
            }
        }
    }

    unsafe fn to_unk_ll(&self, ctx: &mut GenContext) -> llvm::Value {
        if let Value::Unk{ll} = self.to_unk(ctx) {
            ll
        } else {
            panic!("to_unk doesn't work!");
        }
    }
}

struct RecDef {
    props: HashMap<Symbol, u64>,
    mthds: HashMap<Symbol, llvm::Value>,
    cache: Option<llvm::Value>,
}

impl RecDef {
    unsafe fn new(rec: &mut Record, ctx: &mut GenContext) -> RecDef {
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
    unsafe fn size(&self) -> u64 {
        const HEADER_SIZE: u64 = 8;
        const PROP_SIZE: u64 = 12;
        const METHOD_SIZE: u64 = 12;
        let props_size = PROP_SIZE * self.props.len() as u64;
        let mthds_size = METHOD_SIZE * self.mthds.len() as u64;

        HEADER_SIZE + props_size + mthds_size
    }

    unsafe fn add_prop(&mut self, symb: Symbol, offset: u64) {
        self.props.insert(symb, offset);
    }

    unsafe fn add_mthd(&mut self, symb: Symbol, func: llvm::Value) {
        self.mthds.insert(symb, func);
    }

    unsafe fn gen(&mut self, ctx: &mut GenContext) -> llvm::Value {
         if let Some(v) = self.cache {
            v
        } else {
            let i32t = ctx.ctx.int32_type();
            let i64t = ctx.ctx.int64_type();

            let vals = vec![
                i32t.const_int(self.props.len() as u64, false),
                i32t.const_int(self.mthds.len() as u64, false)];

            let mut props = Vec::with_capacity(2 * self.props.len());
            // Initialize to Nones
            for _ in self.props.iter() {
                props.push(None); props.push(None);
            }
            for (symb, offset) in self.props.iter() {
                let sti = ctx.symbol_table.lookup(symb.clone());
                let mut i = (sti as usize) % self.props.len();
                loop {
                    if props[2*i].is_none() {
                        props[2*i] = Some(i32t.const_int(sti, false));
                        props[2*i+1] = Some(i64t.const_int(*offset, false));
                        break;
                    } else {
                        i = (i+1) % self.props.len();

                        // Prevent infinite loops
                        assert!(i != (sti as usize) % self.props.len());
                    }
                }
            }

            let mut mthds = Vec::with_capacity(2 * self.mthds.len());
            // Initialize to Nones
            for _ in self.mthds.iter() {
                mthds.push(None); mthds.push(None);
            }
            for (symb, func) in self.mthds.iter() {
                let sti = ctx.symbol_table.lookup(symb.clone());
                let mut i = (sti as usize) % self.mthds.len();
                loop {
                    if mthds[2*i].is_none() {
                        mthds[2*i] = Some(i32t.const_int(sti, false));
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

            let cs = ctx.ctx.const_struct(&vals, true);
            let globl = ctx.module.add_global(cs.type_of(), "recorddef");
            self.cache = Some(globl);

            globl.set_initializer(cs);
             globl
         }
    }
}

#[derive(Clone)]
struct Record {
    props: HashMap<Symbol, Value>,
    mthds: HashMap<Symbol, Method>,
}

impl  Record {
    unsafe fn new() -> Record {
        Record{
            props: HashMap::new(),
            mthds: HashMap::new()
        }
    }

    unsafe fn add_prop(&mut self, s: Symbol, v: Value) {
        self.props.insert(s, v);
    }

    unsafe fn add_mthd(&mut self, s: Symbol, v: Method) {
        self.mthds.insert(s, v);
    }
}

#[derive(Clone)]
struct Method {
    // record: Record,
    params: Vec<Ident>,
    body: Expr,
    implementation: Option<llvm::Value>,
    built: bool
}

impl Method {
    unsafe fn new(params: Vec<Ident>, body: Expr) -> Method {
        Method{
            params: params,
            body: body,
            implementation: None,
            built: false
        }
    }

    unsafe fn get_function(&mut self, ctx: &mut GenContext) -> llvm::Value {
        let param_tys: Vec<_> = repeat(ctx.value_type()).take(self.params.len()).collect();

        if self.implementation.is_none() {
            self.implementation = Some(ctx.module.add_function(
                "generic_function_name",
                llvm::function_type(ctx.value_type(),
                                          &param_tys,
                                          false)));
        }

        self.implementation.unwrap()
    }

    unsafe fn gen(&mut self, ctx: &mut GenContext) -> llvm::Value {
        let decl = self.get_function(ctx);
        if self.built { return decl }

        // Create the basic block for the function!
        let fn_body = ctx.ctx.append_basic_block(decl, "function_body");
        ctx.builder.position_builder_at_end(fn_body);

        // Generate the function's body
        let ret_val = gen_expr(&self.body, ctx);

        // Return the resulting value from the function
        ctx.builder.build_ret(ret_val.to_unk_ll(ctx));

        decl
    }
}

struct GenContext {
    ctx: llvm::Context,
    builder: llvm::Builder,
    module: llvm::Module,
    method_queue: VecDeque<Method>,
    symbol_table: SymbolTable,
}

macro_rules! builtin_func {
    ($rustname:ident, $cname:expr, $slf:ident, $return_ty: expr, $($pty: expr),+) => {
        // The function which will fetch it for you
        unsafe fn $rustname(&self) -> llvm::Value {
            let $slf = self;
            match self.module.get_named_function($cname) {
                Some(x) => x,
                None => {
                    let func_type = llvm::function_type($return_ty, &[$($pty),+], false);
                    let function = self.module.add_function($cname, func_type);
                    // assert_eq!(self.module.get_named_function($cname), function);

                    function
                }
            }
        }
    }
}

macro_rules! builtin_mthd {
    ($rustname:ident, $cname:expr, $($args: ident),+) => {
        unsafe fn $rustname(&self) -> llvm::Value {
            match self.module.get_named_function($cname) {
                Some(x) => x,
                None => {
                    let func_type = llvm::function_type(self.value_type(), &[$($pty),+], false)
                    let function = module.add_function($cname, func_type);
                    // assert_eq!(module.get_named_function($cname), function);

                    function
                }
            }
        }
    }
}

impl  GenContext {
    unsafe fn new() -> GenContext {
        unimplemented!()
    }

    unsafe fn value_type(&self) -> llvm::Type {
        self.ctx.struct_type(
            // TODO(michael): Non-64-bit computers
            &[self.ctx.int8_type(),
                  self.ctx.int64_type()],
            false)
    }

    unsafe fn record_def_type(&self) -> llvm::Type {
        self.ctx.struct_type(
            &[self.ctx.int32_type(),
                  self.ctx.int32_type()],
            false)
    }

    unsafe fn record_type(&self) -> llvm::Type {
        self.ctx.struct_type(
            &[self.record_def_type().pointer()],
            false)
    }

    unsafe fn symbol_type(&self) -> llvm::Type {
        self.ctx.int64_type()
    }

    builtin_func!(bi_alloc_record, "allocRecord", this,
                  this.value_type(),
                  this.ctx.int64_type());

    builtin_func!(bi_get_property, "getProperty", this,
                  this.value_type(),
                  this.value_type(), this.symbol_type());

    builtin_func!(bi_get_method, "getMethod", this,
                  this.ctx.int8_type().pointer(),
                  this.value_type(), this.symbol_type());

    unsafe fn bit_cast(&self, value: llvm::Value, ty: llvm::Type) -> llvm::Value {
        value.dump();
        ty.dump();

        self.builder.build_bit_cast(value, ty, "num_as_bytes")
    }
}

unsafe fn gen_expr(e: &Expr, ctx: &mut GenContext) -> Value {
    match *e {
        Expr::Literal(ref lit) => {
            match *lit {
                Literal::Str(ref atom) => {
                    Value::KString{
                        ll: ctx.builder.build_global_string(atom.as_slice(), "_string_"),
                        len: atom.as_slice().len() as i64
                    }
                }
                Literal::Int(i) => {
                    Value::KNum{ // TODO(michael): Real Ints maybe?
                        ll: ctx.ctx.double_type().const_real(i as f64)
                    }
                }
                Literal::Float(f) => {
                    Value::KNum{
                        ll: ctx.ctx.double_type().const_real(f)
                    }
                }
                Literal::Bool(b) => {
                    Value::KBool{
                        ll: ctx.ctx.int1_type().const_int(b as u64, false)
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
                &[ctx.ctx.int64_type().const_int(rec_def.size(), false)],
                "record");

            let zero = ctx.ctx.int64_type().const_int(0, false);
            // Set the properties!
            ctx.builder.build_store(
                rec_def.gen(ctx), // Pointer to the record definition
                ctx.builder.build_in_bounds_gep(
                    alloced_rec,
                    &[zero, zero],
                    "record_def_ptr"));

            let values_ptr = ctx.builder.build_bit_cast(
                ctx.builder.build_in_bounds_gep(
                    alloced_rec,
                    &[ctx.ctx.int64_type().const_int(1, false)],
                    "values_ptr_uncast"),
                ctx.value_type().pointer(),
                "values_ptr");

            for (symb, idx) in rec_def.props.iter() {
                ctx.builder.build_store(
                    rec.props[symb.clone()].to_unk_ll(ctx),
                    ctx.builder.build_in_bounds_gep(
                        values_ptr, &[
                            ctx.ctx.int64_type().const_int(*idx, false)
                                ], "value_ptr"));
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
                    &[ll,
                        ctx.ctx.int64_type().const_int(symbol_ll, false)],
                    "get_property")
            }
        }
        Expr::Call(ref obj, ref symb, ref args) => {
            let objv = gen_expr(&**obj, ctx);
            // TODO(michael): Directly index known types,
            // rather than performing expensive lookups

            let ll = objv.to_unk_ll(ctx);
            let symbol_ll = ctx.symbol_table.lookup(symb.clone());
            let get_method = ctx.bi_get_method();
            get_method.dump();
            ll.dump();
            ctx.ctx.int64_type().const_int(symbol_ll, false).dump();

            println!("Here");
            let method_ll = ctx.builder.build_call(
                get_method,
                &[ll,
                      ctx.ctx.int64_type().const_int(symbol_ll, false)],
                "get_method");
            println!("There");

            // Determine the function type we want
            let mut ptypes = Vec::with_capacity(args.len());
            for _ in args.iter() { ptypes.push(ctx.value_type()); }
            let ftype = llvm::function_type(
                ctx.value_type(),
                &ptypes,
                false);


            let method_ll = ctx.builder.build_bit_cast(
                method_ll,
                ftype.pointer(),
                "typed_method");

            let mut args_ll = Vec::with_capacity(args.len());
            for arg in args.iter() {
                args_ll.push(gen_expr(arg, ctx).to_unk_ll(ctx));
            }

            Value::Unk{
                ll: ctx.builder.build_call(
                    method_ll,
                    &args_ll,
                    "method_result")
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


unsafe fn gen_stmt(stmt: &Stmt, ctx: &mut GenContext) -> Value {
    match *stmt {
        Stmt::Let(ref id, ref expr) =>  {
            unimplemented!()
        }
        Stmt::Expr(ref expr) => gen_expr(expr, ctx),
        Stmt::Empty => Value::KNull
    }
}

// TODO(michael): make this actually useful
pub unsafe fn gen_code(ast: Vec<Stmt>) {
    let ctx = llvm::OwnedContext::new();
    let module = llvm::OwnedModule::new("module", *ctx);
    let builder = llvm::OwnedBuilder::new(*ctx);
    let mut method_queue = VecDeque::new();
    let mut symbol_table = SymbolTable::new();

    let mut gc = GenContext{
        ctx: *ctx,
        builder: *builder,
        module: *module,
        method_queue: method_queue,
        symbol_table: symbol_table
    };

    // TODO(michael): Global variables and more!
    // (and variables at all)

    // Create the main function!
    let main_function = module.add_function(
        "__ducky_main",
        llvm::function_type(ctx.void_type(), &[], false));

    let main_function_body = ctx.append_basic_block(main_function, "main_body");
    builder.position_builder_at_end(main_function_body);

    // Create the body for that main function!
    let random_expr = Expr::Block(ast);
    // And generate it!
    gen_expr(&random_expr, &mut gc);

    builder.build_ret_void();

    module.dump();
}
