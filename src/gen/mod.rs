use std::ptr;
use std::collections::HashMap;
use string_cache::Atom;
use rustc_llvm as llvm;
use il;

struct GenContext {
    context: llvm::ContextRef,
    builder: llvm::BuilderRef,
    module: llvm::ModuleRef,
    named_values: HashMap<Atom, llvm::ValueRef>,
    interned_strings: HashMap<Atom, u64>,
}

impl GenContext {
    unsafe fn new() -> GenContext {
        // Create the global code generation context
        let context = llvm::LLVMContextCreate();
        
        // Create the builder, it generates llvm instructions!
        let builder = llvm::LLVMCreateBuilderInContext(context);
        
        // The module is the llvm construct that contains global stuff
        let module = llvm::LLVMModuleCreateWithNameInContext(
            "my cool language".to_c_str().as_ptr(), context); 

        // Named Values!
        let named_values = HashMap::<Atom, llvm::ValueRef>::new();
        
        GenContext {
            context: context,
            builder: builder,
            module: module,
            named_values: named_values,
            interned_strings: HashMap::new(),
        }
    }
}

pub fn gen() {
    unsafe {
        let ctx = GenContext::new();

        // We need to depend on the puts function, let's declare it!
        let puts = llvm::LLVMAddFunction(
            ctx.module, "puts".to_c_str().as_ptr(), llvm::LLVMFunctionType(
                llvm::LLVMInt32TypeInContext(ctx.context),
                [llvm::LLVMPointerType(llvm::LLVMInt8TypeInContext(ctx.context), 0)].as_ptr(),
                1,
                llvm::False
                    ));

        // Create the main function!
        let the_fn = llvm::LLVMAddFunction(
            ctx.module, "main".to_c_str().as_ptr(), llvm::LLVMFunctionType(
                llvm::LLVMVoidTypeInContext(ctx.context),
                ptr::null(),
                0,
                llvm::False
                    ));
        
        // Implement it!
        let fn_body = llvm::LLVMAppendBasicBlockInContext(
            ctx.context,
            the_fn,
            "__ducky_main_block".to_c_str().as_ptr()
                );
        
        llvm::LLVMPositionBuilderAtEnd(ctx.builder, fn_body);

        let print_string = llvm::LLVMBuildGlobalStringPtr(
            ctx.builder,
            "some_string".to_c_str().as_ptr(),
            "print_string".to_c_str().as_ptr());
        
        // Position the builder
        llvm::LLVMBuildCall(ctx.builder, puts,
                            [print_string].as_ptr(),
                            1,
                            "call_puts".to_c_str().as_ptr()
                            );
        llvm::LLVMBuildRetVoid(ctx.builder);

        llvm::LLVMDumpModule(ctx.module);
        
        llvm::LLVMWriteBitcodeToFile(ctx.module, "my_god_its_working.bc".to_c_str().as_ptr());
    }
}

/// This function gets a value representing the index of an interned string
fn get_interned_str(ctx: &mut GenContext, st: Atom) -> llvm::ValueRef {
    if let Some(index) = ctx.interned_strings.get(&st) {
        return unsafe {
            llvm::LLVMConstInt(llvm::LLVMInt64TypeInContext(ctx.context),
                               *index, llvm::False)
        }
    }

    let index = ctx.interned_strings.len().to_u64().unwrap();
    ctx.interned_strings.insert(st, index);
    unsafe {
        llvm::LLVMConstInt(llvm::LLVMInt64TypeInContext(ctx.context),
                           index, llvm::False)
    }
}


unsafe fn gen_expr(ctx: &GenContext, expr: &il::Expr) -> llvm::ValueRef {
    match *expr {
        il::Expr::Literal(ref val) => {
            match *val {
                il::Literal::Int(val) => {
                    // TODO: Implement non-small ints
                    if val > (1 << 28) - 1 || val < -(1 << 28) {
                        panic!("non-small integers haven't been implemented yet");
                    }
                    
                    println!("{:X}, {:X}", val, ((val << 3) | 1).to_u64().unwrap());

                    // Small integers will be stored inside of a pointer type
                    llvm::LLVMConstInt(llvm::LLVMInt32TypeInContext(ctx.context), ((val << 3) | 1).to_u64().unwrap(), llvm::True)
                }
                il::Literal::Bool(val) => {
                    let int_val = if val {1u64} else {0};
                    llvm::LLVMConstInt(llvm::LLVMInt32TypeInContext(ctx.context), (int_val << 3) | 2, llvm::True)
                }
                _ => unimplemented!()
            }
        }
        _ => unimplemented!()
    }
}
