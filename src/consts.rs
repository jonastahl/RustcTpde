use crate::builder::Builder;
use crate::context::CodegenCx;
use crate::shared::ir::{Binding, Global, Module, Slot};
use rustc_codegen_ssa::traits::{MiscCodegenMethods, StaticBuilderMethods, StaticCodegenMethods};
use rustc_middle::mir::interpret::{read_target_uint, Allocation, ConstAllocation, InitChunk, Pointer};
use rustc_span::def_id::DefId;
use std::ops::Range;
use rustc_abi::Size;
use rustc_ast::Mutability;
use rustc_hir::attrs::Linkage;
use rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrFlags;

impl<'tcx> StaticBuilderMethods for Builder<'_, '_, 'tcx> {
    fn get_static(&mut self, def_id: DefId) -> Self::Value {
        if self.tcx.is_thread_local_static(def_id) {
            todo!("thread-local statics are not supported yet")
        }
        Slot::new_global(self.cx.get_global(def_id))
    }
}

#[derive(PartialEq)]
pub enum IsInitOrFini {
    Yes,
    No,
}
impl<'tcx> CodegenCx<'_, 'tcx> {
    pub fn const_alloc_to_tpde(
        &self,
        g: Global,
        alloc: &Allocation,
        is_init_fini: IsInitOrFini,
    ) {
        let mut module = self.module.borrow_mut();
        let dl = &self.tcx.data_layout;
        let pointer_size = dl.pointer_size().bytes() as usize;

        module.global_set_align(g, alloc.align.bytes() as u32);

        fn append_chunks_of_bytes(
            cx: &CodegenCx,
            module: &mut Module,
            global: Global,
            alloc: &Allocation,
            range: Range<usize>
        ) {
            let chunks = alloc.init_mask().range_as_init_chunks(range.clone().into());

            let max = cx.sess().opts.unstable_opts.uninit_const_chunk_threshold;
            let allow_uninit_chunks = chunks.clone().take(max.saturating_add(1)).count() <= max;

            if allow_uninit_chunks {
                for chunk in chunks {
                    match chunk {
                        InitChunk::Init(range) => {
                            let range = (range.start.bytes() as usize)..(range.end.bytes() as usize);
                            let bytes = alloc.inspect_with_uninit_and_ptr_outside_interpreter(range);
                            module.global_add_init_chunk(global, bytes);
                        },
                        InitChunk::Uninit(range) => {
                            let len = range.end.bytes() - range.start.bytes();
                            module.global_add_unit_chunk(global, len as usize);
                        }
                    };
                }
            } else {
                let bytes = alloc.inspect_with_uninit_and_ptr_outside_interpreter(range);
                module.global_add_init_chunk(global, bytes);
            }
        }

        let mut next_offset = 0;
        for &(offset, prov) in alloc.provenance().ptrs().iter() {
            let offset = offset.bytes();
            assert_eq!(offset as usize as u64, offset);
            let offset = offset as usize;
            if offset > next_offset {
                append_chunks_of_bytes(self, &mut module, g, alloc, next_offset..offset);
            }
            let ptr_offset = read_target_uint(
                dl.endian,
                alloc.inspect_with_uninit_and_ptr_outside_interpreter(
                    offset..(offset + pointer_size),
                ),
            ).expect("could not read relocation pointer") as u64;

            {
                let address_space = self.tcx.global_alloc(prov.alloc_id()).address_space(self);

                let schema = if self.sess().pointer_authentication() {
                    match is_init_fini {
                        IsInitOrFini::Yes => self.sess().pointer_authentication_init_fini(),
                        IsInitOrFini::No => self.sess().pointer_authentication_functions(),
                    }
                } else {
                    None
                };
                let ptr = Pointer::new(prov, Size::from_bytes(ptr_offset));

                drop(module);
                let ptr = self.ptr_to_backend(ptr);
                module = self.module.borrow_mut();

                if let Some(value) = module.const_data(ptr) {
                    module.global_add_init_chunk(g, &value.to_le_bytes()[..pointer_size]);
                } else {
                    module.global_add_reloc_chunk(g, offset as u32, ptr, pointer_size);
                }
            }

            next_offset = offset + pointer_size;
        }
        if alloc.len() >= next_offset {
            let range = next_offset..alloc.len();
            append_chunks_of_bytes(self, &mut module, g, alloc, range);
        }
    }
}

impl<'tcx> StaticCodegenMethods for CodegenCx<'_, 'tcx> {
    fn static_addr_of(&self, alloc: ConstAllocation<'_>, kind: Option<&str>) -> Self::Value {
        let global_name = self.generate_local_symbol_name(kind.unwrap_or("global"));
        let g = self.module.borrow_mut()
            .add_global(&global_name, Linkage::Internal, Mutability::Mut, Binding::Definition);

        self.const_alloc_to_tpde(g, alloc.inner(), IsInitOrFini::No);
        
        Slot::new_global(g)
    }

    fn codegen_static(&mut self, def_id: DefId) {
        let g = self.get_global(def_id);

        let attrs = self.tcx.codegen_fn_attrs(def_id);

        let alloc = match self.tcx.eval_static_initializer(def_id) {
            Ok(alloc) => alloc,
            Err(_) => // Error has already been reported
                return,
        }.inner();

        // some weird renaming that we possibly don't need

        if attrs.flags.contains(CodegenFnAttrFlags::THREAD_LOCAL) {
            self.module.borrow_mut().global_set_thread_local(g);
        }

        let is_init_fini = attrs
            .link_section
            .map(|link_section| {
                let s = link_section.as_str();
                if s.starts_with(".init_array") || s.starts_with(".fini_array") {
                    IsInitOrFini::Yes
                } else {
                    IsInitOrFini::No
                }
            })
            .unwrap_or(IsInitOrFini::No);
        self.const_alloc_to_tpde(g, alloc, is_init_fini);

        // more dll stuff
    }
}