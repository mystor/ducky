/* #![allow(unused_variables, dead_code)]

use std::ptr;
use std::collections::HashMap;
use intern::Atom;
use rustc_llvm as llvm;
use il::*;

pub struct GenContext {
    context: llvm::ContextRef,
    builder: llvm::BuilderRef,
    module: llvm::ModuleRef,
    named_values: HashMap<Ident, llvm::ValueRef>,
    interned_strings: HashMap<Atom, u64>,
}

impl GenContext {
    pub unsafe fn new() -> GenContext {
        // Create the global code generation context
        let context = llvm::LLVMContextCreate();

        // Create the builder, it generates llvm instructions!
        let builder = llvm::LLVMCreateBuilderInContext(context);

        // The module is the llvm construct that contains global stuff
        let module = llvm::LLVMModuleCreateWithNameInContext(
            "my cool language".to_c_str().as_ptr(), context);

        GenContext {
            context: context,
            builder: builder,
            module: module,
            named_values: HashMap::new(),
            interned_strings: HashMap::new(),
        }
    }

    pub unsafe fn enter_anon_fn(&self) {
        // Create the main function!
        let the_fn = llvm::LLVMAddFunction(
            self.module, "main".to_c_str().as_ptr(), llvm::LLVMFunctionType(
                llvm::LLVMVoidTypeInContext(self.context),
                ptr::null(),
                0,
                llvm::False
                    ));

        // Implement it!
        let fn_body = llvm::LLVMAppendBasicBlockInContext(
            self.context,
            the_fn,
            "__ducky_main_block".to_c_str().as_ptr()
                );

        llvm::LLVMPositionBuilderAtEnd(self.builder, fn_body);
    }

    pub unsafe fn dump(&self) {
        llvm::LLVMDumpModule(self.module);
    }

    pub unsafe fn pass(&self) {
        // let pmb = llvm::LLVMPassManagerBuilderCreate();
        // llvm::LLVMPassManagerBuilderSetOptLevel(pmb, 3);
        let pm = llvm::LLVMCreatePassManager();
        // llvm::LLVMPassManagerBuilderPopulateModulePassManager(pmb, pm);
        llvm::LLVMAddFunctionInliningPass(pm);
        println!("{}", llvm::LLVMRunPassManager(pm, self.module));
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


pub unsafe fn gen_expr(ctx: &mut GenContext, expr: &Expr) -> Result<llvm::ValueRef, String> {
    debug!("This far");
    match *expr {
        Expr::Literal(ref val) => {
            match *val {
                Literal::Int(val) => {
                    debug!("Literal::Int");
                    // TODO: Implement non-small ints
                    if val > (1 << 28) - 1 || val < -(1 << 28) {
                        panic!("non-small integers haven't been implemented yet");
                    }

                    println!("{:X}, {:X}", val, ((val << 3) | 1).to_u64().unwrap());

                    // Small integers will be stored inside of a pointer type
                    Ok(llvm::LLVMBuildIntToPtr(
                        ctx.builder,
                        llvm::LLVMConstInt(llvm::LLVMInt32TypeInContext(ctx.context),
                                           ((val << 3) | 1).to_u64().unwrap(), llvm::True),
                        llvm::LLVMPointerType(llvm::LLVMInt8TypeInContext(ctx.context),
                                              0),
                        "literal".to_c_str().as_ptr()))
                }
                Literal::Bool(val) => {
                    debug!("Literal::Bool");
                    let int_val = if val {1u64} else {0};
                    Ok(llvm::LLVMConstInt(llvm::LLVMInt32TypeInContext(ctx.context), (int_val << 3) | 2, llvm::True))
                }
                _ => unimplemented!()
            }
        }
        Expr::Ident(ref ident) => {
            debug!("Expr::Ident");
            match ctx.named_values.get(ident) {
                Some(val) => Ok(*val),
                None => Err(format!("Unable to look up type for ident: {}", ident)),
            }
        }
        Expr::Rec(ref props) => {
            unimplemented!()
        }
        Expr::Member(box ref expr, ref symbol) => {
            unimplemented!()
        }
        Expr::Call(box ref expr, ref symbol, ref args) => {
            unimplemented!()
        }
        // Expr::Call(ref call) => {
        //     debug!("Expr::Call");
        //     match *call {
        //         Call::Fn(box ref callee, ref args) => {
        //             // All callable objects are closures. Closures all follow exactly
        //             // the same implementation form:
        //             // {
        //             //   map: _____fn_map_____
        //             //   fn: function*
        //             //   env: environment
        //             // }
        //             let void_ptr_ty = llvm::LLVMPointerType(
        //                 llvm::LLVMInt8TypeInContext(ctx.context),
        //                 0);

        //             let callee : llvm::ValueRef = try!(gen_expr(ctx, callee));

        //             // let args : Vec<llvm::ValueRef> =
        //             let args : Vec<_> = try!(
        //                 args.iter().map(|arg| gen_expr(ctx, arg)).collect());


        //             debug!("Woop?");
        //             let follow = [llvm::LLVMConstInt(llvm::LLVMInt32TypeInContext(ctx.context),
        //                                               0,
        //                                               llvm::True),
        //                           llvm::LLVMConstInt(llvm::LLVMInt32TypeInContext(ctx.context),
        //                                              1,
        //                                              llvm::True)];

        //             // @TODO: This is probably wrong
        //             // I have no idea if I'm using GEP right...
        //             debug!("Here....");
        //             let fn_ptr = llvm::LLVMBuildGEP(ctx.builder,
        //                                             callee,
        //                                             follow.as_ptr(),
        //                                             follow.len() as u32,
        //                                             "fn_ptr".to_c_str().as_ptr());
        //             let fn_ptr = llvm::LLVMBuildLoad(ctx.builder,
        //                                              fn_ptr,
        //                                              "loaded_fn_ptr".to_c_str().as_ptr());
        //             debug!("Or There....");

        //             let follow2 = [llvm::LLVMConstInt(llvm::LLVMInt32TypeInContext(ctx.context),
        //                                               0,
        //                                               llvm::True),
        //                            llvm::LLVMConstInt(llvm::LLVMInt32TypeInContext(ctx.context),
        //                                               2,
        //                                               llvm::True)];
        //             let env_ptr = llvm::LLVMBuildGEP(ctx.builder,
        //                                              callee,
        //                                              follow2.as_ptr(),
        //                                              follow2.len() as u32,
        //                                              "env_ptr".to_c_str().as_ptr());
        //             let env_ptr = llvm::LLVMBuildLoad(ctx.builder,
        //                                               env_ptr,
        //                                               "loaded_env_ptr".to_c_str().as_ptr());

        //             debug!("Shoop?!?!");
        //             // Add the environment as an implcit first argument
        //             let mut true_args : Vec<llvm::ValueRef> = vec![env_ptr];
        //             true_args.push_all(args.as_slice());

        //             llvm::LLVMDumpModule(ctx.module);
        //             for arg in true_args.iter() { llvm::LLVMDumpValue(*arg); }
        //             llvm::LLVMDumpValue(fn_ptr);

        //             let mut arg_tys = Vec::with_capacity(true_args.len());
        //             for _ in true_args.iter() { arg_tys.push(void_ptr_ty); }

        //             let fn_ty = llvm::LLVMPointerType(
        //                 llvm::LLVMFunctionType(void_ptr_ty,
        //                                        arg_tys.as_ptr(),
        //                                        arg_tys.len() as u32,
        //                                        llvm::False),
        //                 0);

        //             let pointer_cast = llvm::LLVMBuildPointerCast(ctx.builder,
        //                                                           fn_ptr,
        //                                                           fn_ty,
        //                                                           "closure_void".to_c_str().as_ptr());

        //             debug!("pointer is cast!");

        //             llvm::LLVMDumpValue(pointer_cast);
        //             llvm::LLVMDumpValue(env_ptr);
        //             for arg in true_args.iter() { llvm::LLVMDumpValue(*arg); }
        //             // And call the function pointer!
        //             Ok(llvm::LLVMBuildCall(ctx.builder,
        //                                    pointer_cast,
        //                                    true_args.as_ptr(),
        //                                    true_args.len() as u32,
        //                                    "the_call_thing".to_c_str().as_ptr()))
        //         }
        //         _ => unimplemented!()
        //     }
        // }
        // Expr::Fn(ref params, box ref body) => {
        //     debug!("Expr::Fn");
        //     // @TODO: Closures! Environments! Oh my!

        //     let void_ptr_ty = llvm::LLVMPointerType(
        //         llvm::LLVMInt8TypeInContext(ctx.context),
        //         0);

        //     let mut param_tys = Vec::with_capacity(params.len() + 1);
        //     param_tys.push(void_ptr_ty);
        //     for _ in params.iter() { param_tys.push(void_ptr_ty) }

        //     debug!("FnType");
        //     let fn_ty = llvm::LLVMFunctionType(void_ptr_ty,
        //                                        param_tys.as_slice().as_ptr(),
        //                                        param_tys.len() as u32,
        //                                        llvm::False);


        //     debug!("FnDecl");
        //     let fn_decl = llvm::LLVMAddFunction(ctx.module,
        //                                         "anon_fn".to_c_str().as_ptr(),
        //                                         fn_ty);


        //     debug!("FnBody");
        //     let fn_body = llvm::LLVMAppendBasicBlockInContext(
        //         ctx.context,
        //         fn_decl,
        //         "anon_fn_body".to_c_str().as_ptr()
        //             );

        //     debug!("GenBody");
        //     {
        //         // Generate the code inside of the new body
        //         let new_builder = llvm::LLVMCreateBuilderInContext(ctx.context);
        //         llvm::LLVMPositionBuilderAtEnd(new_builder, fn_body);

        //         let restore = ctx.builder;
        //         ctx.builder = new_builder;

        //         let expr = try!(gen_expr(ctx, body));

        //         llvm::LLVMBuildRet(ctx.builder, expr);

        //         ctx.builder = restore;

        //         // llvm::LLVMDisposeBuilder(new_builder);
        //     }

        //     debug!("ClosureMake");
        //     llvm::LLVMDumpModule(ctx.module);
        //     let vec_ty = llvm::LLVMRustArrayType(void_ptr_ty, 3);

        //     debug!("Array type Built");
        //     let closure = llvm::LLVMBuildMalloc(ctx.builder,
        //                                         vec_ty,
        //                                         "closure".to_c_str().as_ptr());

        //     debug!("Follow _THIS_");

        //     let follow = [llvm::LLVMConstInt(llvm::LLVMInt32TypeInContext(ctx.context),
        //                                      0,
        //                                      llvm::True),
        //                   llvm::LLVMConstInt(llvm::LLVMInt32TypeInContext(ctx.context),
        //                                      1,
        //                                      llvm::True)];

        //     debug!("Who Builds Stores anyways?!");
        //     let pointer_cast = llvm::LLVMBuildPointerCast(ctx.builder,
        //                                                   fn_decl,
        //                                                   void_ptr_ty,
        //                                                   "closure_void".to_c_str().as_ptr());

        //     let gep = llvm::LLVMBuildGEP(ctx.builder,
        //                                  closure,
        //                                  follow.as_ptr(),
        //                                  follow.len() as u32,
        //                                  "fn_ptr".to_c_str().as_ptr());
        //     llvm::LLVMDumpModule(ctx.module);
        //     llvm::LLVMDumpValue(pointer_cast);
        //     llvm::LLVMDumpValue(gep);

        //     debug!("With That Shit");
        //     llvm::LLVMBuildStore(ctx.builder,
        //                          pointer_cast, gep
        //                          );

        //     debug!("WAT???W?W?");

        //     llvm::LLVMDumpModule(ctx.module);
        //     Ok(closure)
        // }
        _ => unimplemented!()
    }
}
*/
