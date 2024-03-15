use std::{collections::hash_map::Entry, ffi::c_uint};

use llvm_sys::{
    core::{
        LLVMArrayType2, LLVMDoubleTypeInContext, LLVMFP128TypeInContext, LLVMFloatTypeInContext,
        LLVMHalfTypeInContext, LLVMIntTypeInContext, LLVMPointerTypeInContext,
        LLVMStructTypeInContext, LLVMVectorType,
    },
    target::LLVMABISizeOfType,
    LLVMType,
};
use naga::{
    AddressSpace, ArraySize, Handle, Scalar, ScalarKind, Span, StructMember, Type, TypeInner,
};

use crate::codegen::error::Info;

use super::{error::Error, Generator};

impl Generator {
    pub(super) fn eval_scalar_type(
        &mut self,
        span: Span,
        scalar: Scalar,
    ) -> Result<*mut LLVMType, Error> {
        match scalar {
            Scalar {
                kind: ScalarKind::Bool,
                ..
            } => unsafe { Ok(LLVMIntTypeInContext(self.module.context, 1)) },
            Scalar {
                kind: ScalarKind::Float,
                width,
            } => match width {
                2 => unsafe { Ok(LLVMHalfTypeInContext(self.module.context)) },
                4 => unsafe { Ok(LLVMFloatTypeInContext(self.module.context)) },
                8 => unsafe { Ok(LLVMDoubleTypeInContext(self.module.context)) },
                16 => unsafe { Ok(LLVMFP128TypeInContext(self.module.context)) },
                size => Err(Error {
                    info: Info::InvalidFloatSize(size),
                    span,
                }),
            },
            Scalar {
                kind: ScalarKind::Sint | ScalarKind::Uint,
                width,
            } => unsafe {
                Ok(LLVMIntTypeInContext(
                    self.module.context,
                    (width * 8) as c_uint,
                ))
            },
            other @ Scalar { .. } => Err(Error {
                info: Info::IllegalScalar(other),
                span,
            }),
        }
    }

    fn eval_type(&mut self, span: Span, ty: Handle<Type>) -> Result<*mut LLVMType, Error> {
        let inner = &self.module.src.types[ty].inner;
        match inner {
            &TypeInner::Atomic(scalar) | &TypeInner::Scalar(scalar) => {
                self.eval_scalar_type(span, scalar)
            }
            &TypeInner::Vector { size, scalar } => {
                let inner = self.eval_scalar_type(span, scalar)?;
                unsafe { Ok(LLVMVectorType(inner, (size as u32) as c_uint)) }
            }
            &TypeInner::Matrix {
                columns,
                rows,
                scalar,
            } => {
                let inner = self.eval_scalar_type(span, scalar)?;
                unsafe { Ok(LLVMVectorType(inner, rows as c_uint * columns as c_uint)) }
            }
            &TypeInner::Pointer { .. } | TypeInner::ValuePointer { .. } => unsafe {
                Ok(LLVMPointerTypeInContext(self.module.context, 0))
            },
            // TODO: correct array layout (with `stride`)
            &TypeInner::Array { base, size, .. } => {
                let base = self.eval_cached_type(span, base)?;
                if let ArraySize::Constant(size) = size {
                    unsafe { Ok(LLVMArrayType2(base, size.get() as u64)) }
                } else {
                    unsafe { Ok(LLVMArrayType2(base, 0)) }
                }
            }
            TypeInner::Struct { members, .. } => {
                let len = members.len();
                let mut offset = 0;
                let mut llvm_members = Vec::with_capacity(len);
                let mut map = Vec::new();
                let mut idx = 0;
                for StructMember {
                    ty: member_ty,
                    offset: target_offset,
                    ..
                } in members.clone()
                {
                    if offset < target_offset {
                        llvm_members.push(unsafe {
                            LLVMIntTypeInContext(self.module.context, (target_offset - offset) * 8)
                        });
                        offset = target_offset;
                        idx += 1;
                    }
                    let ty = self.eval_cached_type(span, member_ty)?;
                    llvm_members.push(ty);
                    map.push(idx);
                    idx += 1;
                    offset +=
                        (unsafe { LLVMABISizeOfType(self.target_info.data_layout, ty) }) as u32;
                }
                self.struct_maps.insert(ty, map);
                unsafe {
                    Ok(LLVMStructTypeInContext(
                        self.module.context,
                        llvm_members.as_ptr().cast_mut(),
                        llvm_members.len() as c_uint,
                        1,
                    ))
                }
            }
            &TypeInner::Image { .. } => todo!("images"),
            &TypeInner::Sampler { .. } => todo!("samplers"),
            &TypeInner::AccelerationStructure => todo!("acceleration structures"),
            &TypeInner::RayQuery => todo!("ray queries"),
            &TypeInner::BindingArray { .. } => todo!("binding arrays"),
        }
    }

    fn eval_pointer_type(
        &mut self,
        span: Span,
        ty: Handle<Type>,
    ) -> Result<Option<*mut LLVMType>, Error> {
        let ty = &self.module.src.types[ty].inner;
        match ty {
            &TypeInner::Matrix {
                columns,
                rows,
                scalar,
            } => {
                let inner = self.eval_scalar_type(span, scalar)?;
                unsafe {
                    Ok(Some(LLVMArrayType2(
                        LLVMVectorType(inner, rows as c_uint),
                        columns as u64,
                    )))
                }
            }
            _ => Ok(None),
        }
    }

    pub(super) fn eval_cached_pointer_type(
        &mut self,
        span: Span,
        ty: Handle<Type>,
    ) -> Result<*mut LLVMType, Error> {
        match self.pointer_type_cache.entry(ty) {
            Entry::Occupied(entry) => Ok(*entry.get()),
            Entry::Vacant(_) => {
                if let Some(generated) = self.eval_pointer_type(span, ty)? {
                    Ok(*self.pointer_type_cache.entry(ty).or_insert(generated))
                } else {
                    match self.type_cache.entry(ty) {
                        Entry::Occupied(entry) => Ok(*entry.get()),
                        Entry::Vacant(_) => {
                            let generated = self.eval_type(span, ty)?;
                            Ok(*self.pointer_type_cache.entry(ty).or_insert(generated))
                        }
                    }
                }
            }
        }
    }

    pub(super) fn eval_cached_type(
        &mut self,
        span: Span,
        ty: Handle<Type>,
    ) -> Result<*mut LLVMType, Error> {
        match self.type_cache.entry(ty) {
            Entry::Occupied(entry) => Ok(*entry.get()),
            Entry::Vacant(_) => {
                let generated = self.eval_type(span, ty)?;
                Ok(*self.type_cache.entry(ty).or_insert(generated))
            }
        }
    }

    pub(super) fn get_pointee(&mut self, ty: Handle<Type>) -> Option<(Handle<Type>, AddressSpace)> {
        match self.module.src.types[ty].inner {
            TypeInner::Pointer { base, space } => Some((base, space)),
            TypeInner::ValuePointer {
                size: Some(size),
                scalar,
                space,
            } => Some((
                self.module.src.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Vector { size, scalar },
                    },
                    self.module.src.types.get_span(ty),
                ),
                space,
            )),
            TypeInner::ValuePointer { scalar, space, .. } => Some((
                self.module.src.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Scalar(scalar),
                    },
                    self.module.src.types.get_span(ty),
                ),
                space,
            )),
            _ => None,
        }
    }

    pub(super) fn item_type(
        &mut self,
        span: Span,
        ty: Handle<Type>,
        index: Option<usize>,
    ) -> Result<Handle<Type>, Error> {
        match &self.module.src.types[ty].inner {
            &TypeInner::Vector { scalar, .. } => Ok(self.module.src.types.insert(
                Type {
                    name: None,
                    inner: TypeInner::Scalar(scalar),
                },
                self.module.src.types.get_span(ty),
            )),
            &TypeInner::Matrix { rows, scalar, .. } => Ok(self.module.src.types.insert(
                Type {
                    name: None,
                    inner: TypeInner::Vector { size: rows, scalar },
                },
                self.module.src.types.get_span(ty),
            )),
            &TypeInner::Array { base, .. } => Ok(base),
            TypeInner::Struct { members, .. } if index.is_some() => Ok(members[index.unwrap()].ty),
            TypeInner::Struct { .. } => Err(Error {
                info: Info::NonConstantIndex,
                span,
            }),
            _ => Err(Error {
                info: Info::NotIndexable(ty),
                span,
            }),
        }
    }
}
