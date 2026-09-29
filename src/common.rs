use crate::consts::IsInitOrFini;
use crate::context::CodegenCx;
use crate::shared::ir::{Binding, FullType, Slot, Type};
use rustc_abi::Size;
use rustc_codegen_ssa::traits::{ConstCodegenMethods, MiscCodegenMethods};
use rustc_data_structures::stable_hash::{StableHash, StableHasher};
use rustc_hashes::Hash128;
use rustc_hir::attrs::Linkage;
use rustc_middle::mir::interpret::{CtfeProvenance, GlobalAlloc, Pointer, Scalar};
use rustc_session::PointerAuthSchema;

impl<'tcx> ConstCodegenMethods for CodegenCx<'_, 'tcx> {
    fn const_null(&self, t: Self::Type) -> Self::Value {
        match t {
            FullType::Single(t) => self.module.borrow_mut().add_const(t, 0),
            FullType::Pair(ty_a, ty_b, o) => {
                let module = &mut self.module.borrow_mut();

                module.add_pair_consts(ty_a, ty_b, o, 0, 0)
            }
            FullType::Memory { .. } => todo!(),
        }
    }

    fn const_undef(&self, t: Self::Type) -> Self::Value {
        self.const_null(t)
    }

    fn const_poison(&self, t: Self::Type) -> Self::Value {
        self.const_null(t)
    }

    fn const_bool(&self, val: bool) -> Self::Value {
        todo!()
    }

    fn const_i8(&self, i: i8) -> Self::Value {
        todo!()
    }

    fn const_i16(&self, i: i16) -> Self::Value {
        todo!()
    }

    fn const_i32(&self, i: i32) -> Self::Value {
        todo!()
    }

    fn const_i64(&self, i: i64) -> Self::Value {
        todo!()
    }

    fn const_int(&self, t: Self::Type, i: i64) -> Self::Value {
        todo!()
    }

    fn const_u8(&self, i: u8) -> Self::Value {
        self.const_uint(FullType::Single(Type::i8), i as u64)
    }

    fn const_u32(&self, i: u32) -> Self::Value {
        todo!()
    }

    fn const_u64(&self, i: u64) -> Self::Value {
        todo!()
    }

    fn const_u128(&self, i: u128) -> Self::Value {
        todo!()
    }

    fn const_usize(&self, i: u64) -> Self::Value {
        self.module.borrow_mut().add_const(Type::i64, i as u128)
    }

    fn const_uint(&self, ty: Self::Type, i: u64) -> Self::Value {
        let FullType::Single(ty) = ty else {
            unreachable!()
        };
        self.module.borrow_mut().add_const(ty, i as u128)
    }

    fn const_uint_big(&self, ty: Self::Type, u: u128) -> Self::Value {
        let FullType::Single(ty) = ty else {
            unreachable!()
        };
        self.module.borrow_mut().add_const(ty, u)
    }

    fn const_real(&self, t: Self::Type, val: f64) -> Self::Value {
        todo!()
    }

    fn const_str(&self, s: &str) -> (Self::Value, Self::Value) {
        todo!()
    }

    fn const_struct(&self, elts: &[Self::Value], packed: bool) -> Self::Value {
        todo!()
    }

    fn const_vector(&self, elts: &[Self::Value]) -> Self::Value {
        todo!()
    }

    fn const_to_opt_uint(&self, v: Self::Value) -> Option<u64> {
        self.module.borrow().const_data(v).map(|v| v as u64)
    }

    fn const_to_opt_u128(&self, v: Self::Value, sign_ext: bool) -> Option<u128> {
        self.module.borrow().const_data(v)
    }

    fn scalar_to_backend_with_pac(
        &self,
        cv: Scalar,
        layout: rustc_abi::Scalar,
        ty: Self::Type,
        schema: Option<&PointerAuthSchema>,
    ) -> Self::Value {
        match ty {
            FullType::Single(ty) => {
                let data = match ty {
                    Type::Bool => cv.to_bool().unwrap() as u128,
                    Type::i8 => cv.to_i8().unwrap() as u128,
                    Type::i16 => cv.to_i16().unwrap() as u128,
                    Type::i32 => cv.to_i32().unwrap() as u128,
                    Type::i64 => cv.to_i64().unwrap() as u128,
                    Type::i128 => cv.to_i128().unwrap() as u128,
                    Type::ptr => {
                        let ptr = cv.to_pointer(&self.tcx.data_layout)
                            .into_pointer_or_addr();
                        return match ptr {
                            Ok(p) => self.ptr_to_backend(p),
                            Err(addr) => {
                                let addr_val = addr.bytes() as u128;
                                self.module.borrow_mut().add_const(Type::ptr, addr_val)
                            }
                        }
                    }
                    Type::f32 => cv.to_bits(cv.size()).unwrap(),
                    Type::f64 => cv.to_bits(cv.size()).unwrap(),
                    _ => todo!(),
                };
                self.module.borrow_mut().add_const(ty, data)
            }
            FullType::Pair(ty_a, ty_b, offset_b) => {
                todo!()
            }
            _ => todo!(),
        }
    }

    fn const_ptr_byte_offset(&self, val: Self::Value, offset: Size) -> Self::Value {
        self.module.borrow_mut().add_to_ptr(val, offset.bytes() as u32)
    }
}

impl<'tcx> CodegenCx<'_, 'tcx> {
    pub fn ptr_to_backend(&self, ptr: Pointer<CtfeProvenance>) -> Slot {
        let (prov, offset) = ptr.prov_and_relative_offset();
        let global_alloc = self.tcx.global_alloc(prov.alloc_id());
        let base_addr_space = global_alloc.address_space(self);
        match global_alloc {
            GlobalAlloc::Static(def_id) => {
                assert!(self.tcx.is_static(def_id));
                assert!(!self.tcx.is_thread_local_static(def_id));
                Slot::new_global(self.get_global(def_id))
            }
            // GlobalAlloc::Memory(alloc) if alloc.inner().len() == 0 => {
            //     todo!()
            //     // let val = alloc.inner().align.bytes().wrapping_add(offset.bytes());
            //     // let data = self.tcx.truncate_to_target_usize(val) as u128;
            //     // self.tpde_module.borrow_mut().add_const(Type::ptr, data)
            // }
            GlobalAlloc::Memory(alloc) => {
                let id = prov.alloc_id().0;
                let offset = offset.bytes();
                let alloc = alloc.inner();

                let name = {
                    let hash = self.tcx.with_stable_hashing_context(|mut hcx| {
                        let mut hasher = StableHasher::new();
                        alloc.stable_hash(&mut hcx, &mut hasher);
                        hasher.finish::<Hash128>()
                    });
                    format!("alloc_{hash:032x}")
                };

                let g = self.module.borrow_mut()
                    .add_global(&name,
                                Linkage::Internal,
                                alloc.mutability,
                                Binding::Definition);
                self.const_alloc_to_tpde(g, &alloc, IsInitOrFini::No);

                // TODO so far we ignore the address space

                if offset != 0 {
                    todo!()
                    // module.add_global_ptr(g, offset as u32)
                } else {
                    Slot::new_global(g)
                }
            }
            GlobalAlloc::Function { instance } => {
                self.get_fn_addr(instance, None)
            }
            // GlobalAlloc::VTable(ty, dyn_ty) => {
            //     todo!()
            // }
            // // Drop the provenance, the offset contains the bytes of the hash
            // GlobalAlloc::TypeId { .. } => self
            //     .tpde_module
            //     .borrow_mut()
            //     .add_const(Type::ptr, offset.bytes() as u128),
            _ => todo!(),
        }
    }
}
