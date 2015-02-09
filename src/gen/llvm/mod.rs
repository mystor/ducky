use std::mem;
use std::str::from_c_str;
use std::ffi::CString;
use libc::*;

// This is a really dumb idea
// These are llvm bindings. They are only what I need.
// I'm going to be really really lazy at adding new ones
// Hopefully they work

// The types
enum Type {}
impl Type {
    fn as_mut_ptr(&self) -> *mut Type {
        mem::transmute(self);
    }

    fn function_type(return_type: &Type, arg_types: &[&Type], varargs: bool) {
        unsafe {
            LLVMFunctionType(
                return_type.as_mut_ptr(),
                )

        }
    }
}

enum Value {}
enum Context {}
impl Drop for Context {
    fn drop(&mut self) {
        unsafe { LLVMContextDispose(self.as_mut_ptr()); }
    }
}
enum Module {}
impl Drop for Module {
    fn drop(&mut self) {
        unsafe { LLVMDisposeModule(self.as_mut_ptr()); }
    }
}
enum Builder {}
impl Drop for Builder {
    fn drop(&mut self) {
        unsafe { LLVMDisposeBuilder(self.as_mut_ptr()); }
    }
}

#[link(name="llvm")]
extern {
    // Contexts
    fn LLVMContextDispose(context: *mut Context);

    // Modules
    fn LLVMDisposeModule(module: *mut Module);

    // Builders
    fn LLVMDisposeBuilder(builder: *mut Builder);
}
