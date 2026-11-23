use super::ffi::{self, ArgExtension, ArgKind, InstructionKind, ModuleTpde, Type};
use super::ir::{Value, decode_const, size_of_type, vector_info};
use core::fmt::{Debug, Formatter, Result, Write};

impl Debug for ModuleTpde {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for (i, func) in self.functions.iter().enumerate() {
            FunctionPrinter::new(self, i, func).write(f)?;
            writeln!(f)?;
        }
        for (i, global) in self.globals.iter().enumerate() {
            write_global(self, f, i, global)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

fn ty_name(ty: Type) -> String {
    match ty {
        Type::Void => "void".into(),
        Type::Bool => "bool".into(),
        ty => format!("{ty:?}"),
    }
}

fn write_flags(out: &mut impl Write, flags: &ffi::Flags) -> Result {
    let ffi::Flags {
        extern_link,
        only_local,
        dso_local,
        weak_link,
        always_inline,
        never_delete,
        dead,
    } = *flags;
    for (set, name) in [
        (extern_link, "extern"),
        (only_local, "internal"),
        (weak_link, "weak"),
        (!extern_link && !dso_local, "got"),
        (always_inline, "always_inline"),
        (never_delete, "never_delete"),
        (dead, "dead"),
    ] {
        if set {
            write!(out, " !{name}")?;
        }
    }
    Ok(())
}

fn write_arg_info(out: &mut impl Write, info: &ffi::ArgInfo) -> Result {
    match info.kind {
        ArgKind::ByVal => write!(out, " !byval({}, {})", info.size, info.align)?,
        ArgKind::sRet => write!(out, " !sret({}, {})", info.size, info.align)?,
        _ => {}
    }
    match info.extension {
        ArgExtension::zExt => write!(out, " !zext"),
        ArgExtension::sExt => write!(out, " !sext"),
        _ => Ok(()),
    }
}

fn write_const(out: &mut impl Write, ty: Type, data: u128) -> Result {
    write!(out, "[{} ", ty_name(ty))?;
    match ty {
        Type::Bool => write!(out, "{}", data != 0)?,
        Type::i8 => write!(out, "{}", data as i8)?,
        Type::i16 => write!(out, "{}", data as i16)?,
        Type::i32 => write!(out, "{}", data as i32)?,
        Type::i64 => write!(out, "{}", data as i64)?,
        Type::i128 => write!(out, "{}", data as i128)?,
        Type::f32 => write!(out, "{}", f32::from_bits(data as u32))?,
        Type::f64 => write!(out, "{}", f64::from_bits(data as u64))?,
        _ => write!(out, "{data:#x}")?,
    }
    write!(out, "]")
}

fn write_const_vector(out: &mut impl Write, vec: &ffi::ConstVector) -> Result {
    let Some((elem_ty, _)) = vector_info(vec.ty) else {
        return write!(out, "[{} {:x?}]", ty_name(vec.ty), vec.data);
    };
    let elem_bytes = size_of_type(elem_ty) as usize;
    write!(out, "[{} <", ty_name(vec.ty))?;
    for (i, lane) in vec.data.chunks(elem_bytes).enumerate() {
        if i > 0 {
            write!(out, ", ")?;
        }
        let mut bytes = [0u8; 8];
        bytes[..lane.len()].copy_from_slice(lane);
        let bits = u64::from_le_bytes(bytes);
        match elem_ty {
            Type::f32 => write!(out, "{}", f32::from_bits(bits as u32))?,
            Type::f64 => write!(out, "{}", f64::from_bits(bits))?,
            _ => write!(out, "{bits}")?,
        }
    }
    write!(out, ">]")
}

fn write_module_operand(
    module: &ModuleTpde,
    out: &mut impl Write,
    slot: Value,
    referenced: &mut Vec<usize>,
) -> core::result::Result<bool, core::fmt::Error> {
    match slot {
        Value::Func(func) => {
            referenced.push(func.0);
            write!(out, "f{}", func.0)?;
        }
        Value::Global(g) => write!(out, "g{}", g.0)?,
        Value::GlobalPtr(p) => match module.global_ptrs.get(p as usize) {
            Some(gp) => write!(out, "g{}+{}", gp.global, gp.offset)?,
            None => write!(out, "gp{p}")?,
        },
        Value::Const(c) => {
            let (ty, data) = decode_const(&module.consts, c);
            write_const(out, ty, data)?;
        }
        Value::ConstVector(v) => write_const_vector(out, &module.const_vectors[v as usize])?,
        Value::Raw(r) => write!(out, "{r}")?,
        _ => return Ok(false),
    }
    Ok(true)
}

fn write_name_comment(module: &ModuleTpde, out: &mut impl Write, referenced: &[usize]) -> Result {
    let mut first = true;
    for &func in referenced {
        let Some(func) = module.functions.get(func) else {
            continue;
        };
        if func.name.is_empty() {
            continue;
        }
        write!(out, "{} @{}", if first { "   " } else { "," }, func.name)?;
        first = false;
    }
    Ok(())
}

struct FunctionPrinter<'a> {
    module: &'a ModuleTpde,
    index: usize,
    func: &'a ffi::Function,
    /// Index of the first phi of each block, used for global phi numbering
    phi_offsets: Vec<u32>,
}

impl<'a> FunctionPrinter<'a> {
    fn new(module: &'a ModuleTpde, index: usize, func: &'a ffi::Function) -> Self {
        let mut offset = 0;
        let phi_offsets = func
            .basic_blocks
            .iter()
            .map(|bb| {
                let start = offset;
                offset += bb.phi_len;
                start
            })
            .collect();
        Self {
            module,
            index,
            func,
            phi_offsets,
        }
    }

    fn phi(&self, block: u32, index: u32) -> Option<&ffi::Phi> {
        let bb = self.func.basic_blocks.get(block as usize)?;
        self.func.phis.get((bb.phi_start + index) as usize)
    }

    fn type_of(&self, slot: Value) -> Option<Type> {
        Some(match slot {
            Value::Slot(_, v) => self.func.slots.get(v as usize)?.ty,
            Value::Phi(_, block, index) => self.phi(block, index)?.ty,
            Value::Const(c) => decode_const(&self.module.consts, c).0,
            Value::ConstVector(v) => self.module.const_vectors.get(v as usize)?.ty,
            Value::Alloc(_) | Value::Func(_) | Value::Global(_) | Value::GlobalPtr(_) => Type::ptr,
            Value::Raw(_) | Value::Pair(_) => return None,
        })
    }

    fn write_operand(
        &self,
        out: &mut impl Write,
        slot: Value,
        referenced: &mut Vec<usize>,
    ) -> Result {
        if write_module_operand(self.module, out, slot, referenced)? {
            return Ok(());
        }
        match slot {
            Value::Slot(_, v) => write!(out, "s{v}"),
            Value::Alloc(a) => write!(out, "a{a}"),
            Value::Phi(_, block, index) => match self.phi_offsets.get(block as usize) {
                Some(offset) => write!(out, "p{}", offset + index),
                None => write!(out, "p(bb{block}, {index})"),
            },
            _ => write!(out, "{slot:?}"),
        }
    }

    fn write_typed(&self, out: &mut impl Write, slot: Value, referenced: &mut Vec<usize>) -> Result {
        self.write_operand(out, slot, referenced)?;
        match self.type_of(slot) {
            Some(ty) => write!(out, ": {}", ty_name(ty)),
            None => Ok(()),
        }
    }

    /// Return types are not stored, so derive them from the first `Ret` instruction.
    fn return_types(&self) -> Option<Vec<Type>> {
        let ret = self
            .func
            .instructions
            .iter()
            .find(|inst| inst.kind == InstructionKind::Ret)?;
        let start = ret.ops_start as usize;
        self.func.operands[start..start + ret.ops_len as usize]
            .iter()
            .map(|&op| self.type_of(Value::from_ffi(op)))
            .collect()
    }

    fn write(&self, f: &mut Formatter<'_>) -> Result {
        let func = self.func;
        let mut referenced = vec![];

        write!(f, "[f{}] {}(", self.index, func.name)?;
        for (i, info) in func.args.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            match func.slots.get(i) {
                Some(slot) => write!(f, "s{i}: {}", ty_name(slot.ty))?,
                None => write!(f, "s{i}: ?")?,
            }
            write_arg_info(f, info)?;
        }
        write!(f, ")")?;
        if func.has_ret {
            match self.return_types().as_deref() {
                Some([ty]) => write!(f, " -> {}", ty_name(*ty))?,
                Some(tys) if !tys.is_empty() => {
                    let names: Vec<_> = tys.iter().map(|ty| ty_name(*ty)).collect();
                    write!(f, " -> ({})", names.join(", "))?
                }
                _ => write!(f, " -> ?")?,
            }
        }
        write_flags(f, &func.flags)?;

        if func.basic_blocks.is_empty() {
            return writeln!(f);
        }
        writeln!(f, " {{")?;

        if func.has_personality {
            let mut line = String::new();
            write!(line, "  personality: ")?;
            let personality = Value::Func(super::ir::Function(func.personality as usize));
            self.write_operand(&mut line, personality, &mut referenced)?;
            write_name_comment(self.module, &mut line, &referenced)?;
            referenced.clear();
            writeln!(f, "{line}")?;
        }

        if !func.allocas.is_empty() {
            writeln!(f, "  allocas:")?;
            for (i, alloca) in func.allocas.iter().enumerate() {
                writeln!(
                    f,
                    "    a{i} = size: {}, align: {}",
                    alloca.size, alloca.align
                )?;
            }
        }

        if !func.callee_infos.is_empty() {
            writeln!(f, "  callee infos:")?;
            for (i, callee) in func.callee_infos.iter().enumerate() {
                let mut args = Vec::with_capacity(callee.info.len());
                for info in &callee.info {
                    let mut s = String::new();
                    write_arg_info(&mut s, info)?;
                    args.push(if s.is_empty() {
                        "_".to_string()
                    } else {
                        s.trim_start().to_string()
                    });
                }
                writeln!(f, "    ci{i} = ({})", args.join(", "))?;
            }
        }

        writeln!(f, "  basic blocks:")?;
        for (index, block) in func.basic_blocks.iter().enumerate() {
            self.write_block(f, index as u32, block)?;
        }
        writeln!(f, "}}")
    }

    fn write_block(&self, f: &mut Formatter<'_>, index: u32, block: &ffi::BasicBlock) -> Result {
        let func = self.func;
        let mut referenced = vec![];
        writeln!(f, "    bb{index}:")?;

        let phi_start = block.phi_start as usize;
        for (i, phi) in func.phis[phi_start..phi_start + block.phi_len as usize]
            .iter()
            .enumerate()
        {
            let mut line = String::from("      phi [");
            let start = phi.incoming_start as usize;
            let incoming = &func.operands[start..start + phi.incoming_len as usize * 2];
            for (j, pair) in incoming.chunks(2).enumerate() {
                if j != 0 {
                    line.push_str(", ");
                }
                match Value::from_ffi(pair[0]) {
                    Value::Raw(bb) => write!(line, "(bb{bb}, ")?,
                    other => write!(line, "({other:?}, ")?,
                }
                self.write_operand(&mut line, Value::from_ffi(pair[1]), &mut referenced)?;
                line.push(')');
            }
            write!(line, "] -> ")?;
            self.write_typed(
                &mut line,
                Value::Phi(super::ir::Function(0), index, i as u32),
                &mut referenced,
            )?;
            write_name_comment(self.module, &mut line, &referenced)?;
            referenced.clear();
            writeln!(f, "{line}")?;
        }

        let start = block.insts_start as usize;
        let insts = &func.instructions[start..start + block.insts_len as usize];
        let mut iter = insts.iter().peekable();
        while let Some(inst) = iter.next() {
            let mut line = String::new();
            write!(line, "      <{:?}> (", inst.kind)?;
            let ops_start = inst.ops_start as usize;
            let ops = &func.operands[ops_start..ops_start + inst.ops_len as usize];
            let is_call = matches!(inst.kind, InstructionKind::Call | InstructionKind::Invoke);
            for (i, &op) in ops.iter().enumerate() {
                if i != 0 {
                    line.push_str(", ");
                }
                let slot = Value::from_ffi(op);
                // indirect calls reference their callee info as second operand
                if is_call
                    && i == 1
                    && !matches!(Value::from_ffi(ops[0]), Value::Func(_))
                    && let Value::Raw(ci) = slot
                {
                    write!(line, "ci{ci}")?;
                    continue;
                }
                self.write_operand(&mut line, slot, &mut referenced)?;
            }
            line.push(')');

            if inst.has_result {
                line.push_str(" -> ");
                self.write_typed(&mut line, Value::from_ffi(inst.result), &mut referenced)?;
                // Merge the additional return value of pair returns into this line
                while let Some(add) = iter.next_if(|i| i.kind == InstructionKind::AddRet) {
                    line.push_str(", ");
                    self.write_typed(&mut line, Value::from_ffi(add.result), &mut referenced)?;
                }
            }

            write_name_comment(self.module, &mut line, &referenced)?;
            referenced.clear();
            writeln!(f, "{line}")?;
        }

        let successors = &func.successors[block.successors()];
        if !successors.is_empty() {
            let names: Vec<_> = successors.iter().map(|s| format!("bb{s}")).collect();
            writeln!(f, "      -> {}", names.join(", "))?;
        }
        Ok(())
    }
}

fn write_global(
    module: &ModuleTpde,
    f: &mut Formatter<'_>,
    index: usize,
    global: &ffi::Global,
) -> Result {
    write!(f, "[g{index}]")?;
    if !global.name.is_empty() {
        write!(f, " {}", global.name)?;
    }
    write!(
        f,
        " size: {} bytes, align: {}, {}",
        global.size,
        global.align,
        if global.init { "init" } else { "uninit" }
    )?;
    if !global.read_only {
        write!(f, " !mut")?;
    }
    if global.thread_loc {
        write!(f, " !tls")?;
    }
    write_flags(f, &global.flags)?;

    if global.data.is_empty() && global.relocations.is_empty() {
        return writeln!(f);
    }
    writeln!(f, " {{")?;

    for (i, chunk) in global.data.chunks(16).enumerate() {
        write!(f, "  {:04x}  ", i * 16)?;
        for b in chunk {
            write!(f, "{b:02x} ")?;
        }
        for _ in chunk.len()..16 {
            write!(f, "   ")?;
        }
        write!(f, " |")?;
        for &b in chunk {
            let c = if b.is_ascii_graphic() || b == b' ' {
                b as char
            } else {
                '.'
            };
            write!(f, "{c}")?;
        }
        writeln!(f, "|")?;
    }

    if !global.relocations.is_empty() {
        writeln!(f)?;
        writeln!(f, "  relocations:")?;
        for reloc in &global.relocations {
            let mut line = String::new();
            let mut referenced = vec![];
            write!(line, "    {:#x} ", reloc.offset)?;
            let slot = Value::from_ffi(reloc.slot);
            if !write_module_operand(module, &mut line, slot, &mut referenced)? {
                write!(line, "{slot:?}")?;
            }
            write_name_comment(module, &mut line, &referenced)?;
            writeln!(f, "{line}")?;
        }
    }
    writeln!(f, "}}")
}
