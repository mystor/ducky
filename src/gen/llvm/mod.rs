#![allow(dead_code)]

// This module is mostly utility functions to make working with
// llvm-c slightly nicer. The meat is in the ffi module
// I will add utility functions as I find them handy in here.

use std::ops::{Deref, DerefMut};
use self::ffi::*;


#[allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
pub mod ffi;

// Eww, I'm an awful person
// I just really don't want to constantly type this out :-/
macro_rules! cstr {
    ($s: expr) => {
        ::std::ffi::CString::new($s.as_bytes()).unwrap().as_ptr()
    }
}

macro_rules! ll_type {
    ($name:ident, $llname:ty) => {
        #[derive(Copy, Clone)]
        pub struct $name {
            ptr: $llname
        }

        impl $name {
            fn new(ll: $llname) -> $name {
                $name { ptr: ll }
            }
        }

        impl ::std::ops::Deref for $name {
            type Target = $llname;

            fn deref<'a>(&'a self) -> &'a $llname {
                &self.ptr
            }
        }

        impl ::std::ops::DerefMut for $name {
            fn deref_mut<'a>(&'a mut self) -> &'a mut $llname {
                &mut self.ptr
            }
        }
    }
}

// Some nice type aliases to make my life easier
ll_type!(MemoryBuffer, ffi::LLVMMemoryBufferRef);
ll_type!(Context, ffi::LLVMContextRef);
ll_type!(Module, ffi::LLVMModuleRef);
ll_type!(Type, ffi::LLVMTypeRef);
ll_type!(Value, ffi::LLVMValueRef);
ll_type!(BasicBlock, ffi::LLVMBasicBlockRef);
ll_type!(Builder, ffi::LLVMBuilderRef);
ll_type!(ModuleProvider, ffi::LLVMModuleProviderRef);
ll_type!(PassManager, ffi::LLVMPassManagerRef);
ll_type!(PassRegistry, ffi::LLVMPassRegistryRef);
ll_type!(Use, ffi::LLVMUseRef);
ll_type!(DiagnosticInfo, ffi::LLVMDiagnosticInfoRef);

#[allow(missing_copy_implementations)]
pub struct OwnedContext {
    ptr: Context
}

impl OwnedContext {
    pub unsafe fn new() -> OwnedContext {
        OwnedContext { ptr: Context::new(LLVMContextCreate()) }
    }
}

impl Drop for OwnedContext {
    fn drop(&mut self) {
        unsafe { LLVMContextDispose(*self.ptr) }
    }
}

impl Deref for OwnedContext {
    type Target = Context;

    fn deref(&self) -> &Context {
        &self.ptr
    }
}

impl DerefMut for OwnedContext {
    fn deref_mut(&mut self) -> &mut Context {
        &mut self.ptr
    }
}

#[allow(missing_copy_implementations)]
pub struct OwnedModule {
    ptr: Module
}

impl OwnedModule {
    pub unsafe fn new(module_id: &str, c: Context) -> OwnedModule {
        OwnedModule { ptr: Module::new(LLVMModuleCreateWithNameInContext(cstr!(module_id), *c)) }
    }
}

impl Drop for OwnedModule {
    fn drop(&mut self) {
        unsafe { LLVMDisposeModule(*self.ptr) }
    }
}

impl Deref for OwnedModule {
    type Target = Module;

    fn deref(&self) -> &Module {
        &self.ptr
    }
}

impl DerefMut for OwnedModule {
    fn deref_mut(&mut self) -> &mut Module {
        &mut self.ptr
    }
}

#[allow(missing_copy_implementations)]
pub struct OwnedBuilder {
    ptr: Builder
}

impl OwnedBuilder {
    pub unsafe fn new(c: Context) -> OwnedBuilder {
        OwnedBuilder { ptr: Builder::new(LLVMCreateBuilderInContext(*c)) }
    }
}

impl Drop for OwnedBuilder {
    fn drop(&mut self) {
        unsafe { LLVMDisposeBuilder(*self.ptr) }
    }
}

impl Deref for OwnedBuilder {
    type Target = Builder;

    fn deref(&self) -> &Builder {
        &self.ptr
    }
}

impl DerefMut for OwnedBuilder {
    fn deref_mut(&mut self) -> &mut Builder {
        &mut self.ptr
    }
}

// These are a minimal set of wrappers around the llvm functions
// which are intended to make me using them easier. I am not wrapping
// all of llvm, nor do I intend to (that would be a much bigger project).
// In addition, these functions are still considered unsafe, as I am not
// encoding lifetimes using rust's lifetime syntax, meaning that it is
// probably possible to break memory safety using them.

impl Context {
    pub unsafe fn int1_type(self) -> Type {
        Type::new(LLVMInt1TypeInContext(*self))
    }

    pub unsafe fn int8_type(self) -> Type {
        Type::new(LLVMInt8TypeInContext(*self))
    }

    pub unsafe fn int16_type(self) -> Type {
        Type::new(LLVMInt16TypeInContext(*self))
    }

    pub unsafe fn int32_type(self) -> Type {
        Type::new(LLVMInt32TypeInContext(*self))
    }

    pub unsafe fn int64_type(self) -> Type {
        Type::new(LLVMInt64TypeInContext(*self))
    }

    pub unsafe fn int_type(self, n: u32) -> Type {
        Type::new(LLVMIntTypeInContext(*self, n))
    }

    pub unsafe fn float_type(self) -> Type {
        Type::new(LLVMFloatTypeInContext(*self))
    }

    pub unsafe fn double_type(self) -> Type {
        Type::new(LLVMDoubleTypeInContext(*self))
    }

    pub unsafe fn struct_type(self, element_types: &[Type], packed: bool) -> Type {
        Type::new(LLVMStructTypeInContext(*self, element_types.as_ptr() as *mut _, element_types.len() as u32, packed as LLVMBool))
    }

    pub unsafe fn void_type(self) -> Type {
        Type::new(LLVMVoidTypeInContext(*self))
    }

    pub unsafe fn const_struct(self, constant_vals: &[Value],  packed: bool) -> Value {
        Value::new(LLVMConstStructInContext(*self, constant_vals.as_ptr() as *mut _, constant_vals.len() as u32, packed as LLVMBool))
    }

    pub unsafe fn append_basic_block(self, fun: Value, name: &str) -> BasicBlock {
        BasicBlock::new(LLVMAppendBasicBlockInContext(*self, *fun, cstr!(name)))
    }
}

impl Module {
    pub unsafe fn add_global(self, ty: Type, name: &str) -> Value {
        Value::new(LLVMAddGlobal(*self, *ty, cstr!(name)))
    }

    pub unsafe fn add_function(self, name: &str, ty: Type) -> Value {
        Value::new(LLVMAddFunction(*self, cstr!(name), *ty))
    }

    pub unsafe fn get_named_function(self, name: &str) -> Option<Value> {
        let func = LLVMGetNamedFunction(*self, cstr!(name));
        if func.is_null() { None } else { Some(Value::new(func)) }
    }

    pub unsafe fn dump(self) {
        LLVMDumpModule(*self);
    }
}

impl Type {
    pub unsafe fn pointer_with_address_space(self, address_space: u32) -> Type {
        Type::new(LLVMPointerType(*self, address_space))
    }

    pub unsafe fn pointer(self) -> Type {
        self.pointer_with_address_space(0)
    }

    pub unsafe fn const_int(self, n: u64, sign_extend: bool) -> Value {
        Value::new(LLVMConstInt(*self, n, sign_extend as LLVMBool))
    }

    pub unsafe fn const_real(self, n: f64) -> Value {
        Value::new(LLVMConstReal(*self, n))
    }

    pub unsafe fn dump(self) {
        LLVMDumpType(*self);
    }
}

impl Value {
    pub unsafe fn set_initializer(self, init: Value) {
        LLVMSetInitializer(*self, *init)
    }

    pub unsafe fn type_of(self) -> Type {
        Type::new(LLVMTypeOf(*self))
    }

    pub unsafe fn dump(self) {
        LLVMDumpValue(*self);
    }
}

impl Builder {
    pub unsafe fn build_ret_void(self) -> Value {
        Value::new(LLVMBuildRetVoid(*self))
    }
    pub unsafe fn build_ret(self, v: Value) -> Value {
        Value::new(LLVMBuildRet(*self, *v))
    }
    pub unsafe fn build_alloca(self, ty: Type, name: &str) -> Value {
        Value::new(LLVMBuildAlloca(*self, *ty, cstr!(name)))
    }

    pub unsafe fn build_store(self, val: Value, ptr: Value) -> Value {
        Value::new(LLVMBuildStore(*self, *val, *ptr))
    }

    pub unsafe fn build_load(self, ptr: Value, name: &str) -> Value {
        Value::new(LLVMBuildLoad(*self, *ptr, cstr!(name)))
    }

    pub unsafe fn build_gep(self, ptr: Value, indices: &[Value], name: &str) -> Value {
        Value::new(LLVMBuildGEP(*self, *ptr, indices.as_ptr() as *mut _, indices.len() as u32, cstr!(name)))
    }

    pub unsafe fn build_in_bounds_gep(self, ptr: Value, indices: &[Value], name: &str) -> Value {
        Value::new(LLVMBuildInBoundsGEP(*self, *ptr, indices.as_ptr() as *mut _, indices.len() as u32, cstr!(name)))
    }

    pub unsafe fn build_bit_cast(self, val: Value, ty: Type, name: &str) -> Value {
        Value::new(LLVMBuildBitCast(*self, *val, *ty, cstr!(name)))
    }

    pub unsafe fn build_global_string(self, string: &str, name: &str) -> Value {
        Value::new(LLVMBuildGlobalString(*self, cstr!(string), cstr!(name)))
    }

    pub unsafe fn build_call(self, fun: Value, args: &[Value], name: &str) -> Value {
        Value::new(LLVMBuildCall(*self, *fun, args.as_ptr() as *mut _, args.len() as u32, cstr!(name)))
    }

    pub unsafe fn position_builder_at_end(self, block: BasicBlock) {
        LLVMPositionBuilderAtEnd(*self, *block)
    }
}

pub unsafe fn function_type(return_type: Type, param_types: &[Type], is_var_arg: bool) -> Type {
    Type::new(LLVMFunctionType(*return_type, param_types.as_ptr() as *mut _, param_types.len() as u32, is_var_arg as LLVMBool))
}
