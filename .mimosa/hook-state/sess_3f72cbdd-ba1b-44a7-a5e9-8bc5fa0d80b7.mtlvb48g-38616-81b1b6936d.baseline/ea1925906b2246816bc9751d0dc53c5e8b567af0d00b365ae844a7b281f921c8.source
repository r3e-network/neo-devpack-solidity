use super::*;

/// Disassemble NeoVM bytecode into a human-readable textual listing.
///
/// This function is total: any byte sequence (including malformed or
/// truncated bytecode) produces some output without panicking. That
/// invariant is fuzzed by `fuzz/fuzz_targets/fuzz_target_disasm.rs`.
pub fn disassemble_neovm_bytecode(bytecode: &[u8]) -> String {
    let width = if bytecode.len() <= 0xFFFF { 4 } else { 6 };
    let mut out = String::new();
    let mut pc: usize = 0;

    while pc < bytecode.len() {
        let offset = pc;
        let opcode = bytecode[pc];
        pc += 1;

        let opname = crate::runtime::spec::opcode_name(opcode)
            .map(str::to_string)
            .unwrap_or_else(|| format!("OP_{opcode:02X}"));

        out.push_str(&format!("{offset:0width$X}: {opname}"));

        match opcode {
            // Push immediates
            0x00 => {
                // PUSHINT8
                if let Some(v) = take_i8(bytecode, &mut pc) {
                    out.push_str(&format!(" {v}"));
                } else {
                    out.push_str(" <unexpected EOF>");
                    break;
                }
            }
            0x01 => {
                // PUSHINT16
                if let Some(v) = take_i16(bytecode, &mut pc) {
                    out.push_str(&format!(" {v}"));
                } else {
                    out.push_str(" <unexpected EOF>");
                    break;
                }
            }
            0x02 => {
                // PUSHINT32
                if let Some(v) = take_i32(bytecode, &mut pc) {
                    out.push_str(&format!(" {v}"));
                } else {
                    out.push_str(" <unexpected EOF>");
                    break;
                }
            }
            0x03 => {
                // PUSHINT64
                if let Some(v) = take_i64(bytecode, &mut pc) {
                    out.push_str(&format!(" {v}"));
                } else {
                    out.push_str(" <unexpected EOF>");
                    break;
                }
            }
            0x04 => {
                // PUSHINT128 (raw LE bytes)
                if let Some(bytes) = take(bytecode, &mut pc, 16) {
                    out.push_str(&format!(" 0x{}", hex::encode(bytes)));
                } else {
                    out.push_str(" <unexpected EOF>");
                    break;
                }
            }
            0x05 => {
                // PUSHINT256 (raw LE bytes)
                if let Some(bytes) = take(bytecode, &mut pc, 32) {
                    out.push_str(&format!(" 0x{}", hex::encode(bytes)));
                } else {
                    out.push_str(" <unexpected EOF>");
                    break;
                }
            }
            0x0A => {
                // PUSHA — signed offset relative to this opcode (Pointer target).
                if let Some(raw) = take_u32(bytecode, &mut pc) {
                    out.push_str(&format!(" {}", raw as i32));
                } else {
                    out.push_str(" <unexpected EOF>");
                    break;
                }
            }
            0x0C => {
                // PUSHDATA1
                let Some(len) = take_u8(bytecode, &mut pc) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                let Some(data) = take(bytecode, &mut pc, len as usize) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                out.push_str(&format!(" len={len} 0x{}", hex::encode(data)));
            }
            0x0D => {
                // PUSHDATA2
                let Some(len) = take_u16(bytecode, &mut pc) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                let Some(data) = take(bytecode, &mut pc, len as usize) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                out.push_str(&format!(" len={len} 0x{}", hex::encode(data)));
            }
            0x0E => {
                // PUSHDATA4
                let Some(len) = take_u32(bytecode, &mut pc) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                let Some(data) = take(bytecode, &mut pc, len as usize) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                out.push_str(&format!(" len={len} 0x{}", hex::encode(data)));
            }

            // Flow control (wide forms used by compiler)
            0x23 // JMP_L
            | 0x25 // JMPIF_L
            | 0x27 // JMPIFNOT_L
            | 0x29 // JMPEQ_L
            | 0x2B // JMPNE_L
            | 0x2D // JMPGT_L
            | 0x2F // JMPGE_L
            | 0x31 // JMPLT_L
            | 0x33 // JMPLE_L
            | 0x35 // CALL_L
            | 0x3E // ENDTRY_L
            => {
                let Some(rel) = take_i32(bytecode, &mut pc) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                out.push_str(&format!(" -> {}", fmt_target(bytecode.len(), offset, rel)));
            }
            0x3C => {
                // TRY_L CatchOffset(int) FinallyOffset(int)
                let Some(catch_rel) = take_i32(bytecode, &mut pc) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                let Some(finally_rel) = take_i32(bytecode, &mut pc) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                if catch_rel != 0 {
                    out.push_str(&format!(
                        " catch={}",
                        fmt_target(bytecode.len(), offset, catch_rel)
                    ));
                }
                if finally_rel != 0 {
                    out.push_str(&format!(
                        " finally={}",
                        fmt_target(bytecode.len(), offset, finally_rel)
                    ));
                }
            }
            0x3B => {
                // TRY CatchOffset(sbyte) FinallyOffset(sbyte)
                let Some(catch_rel) = take_i8(bytecode, &mut pc) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                let Some(finally_rel) = take_i8(bytecode, &mut pc) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                if catch_rel != 0 {
                    out.push_str(&format!(
                        " catch={}",
                        fmt_target(bytecode.len(), offset, catch_rel as i32)
                    ));
                }
                if finally_rel != 0 {
                    out.push_str(&format!(
                        " finally={}",
                        fmt_target(bytecode.len(), offset, finally_rel as i32)
                    ));
                }
            }
            0x22 // JMP
            | 0x24 // JMPIF
            | 0x26 // JMPIFNOT
            | 0x28 // JMPEQ
            | 0x2A // JMPNE
            | 0x2C // JMPGT
            | 0x2E // JMPGE
            | 0x30 // JMPLT
            | 0x32 // JMPLE
            | 0x34 // CALL
            | 0x3D // ENDTRY
            => {
                let Some(rel) = take_i8(bytecode, &mut pc) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                out.push_str(&format!(
                    " -> {}",
                    fmt_target(bytecode.len(), offset, rel as i32)
                ));
            }

            // Slots / indices
            0x56 => {
                // INITSSLOT static_slots
                let Some(count) = take_u8(bytecode, &mut pc) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                out.push_str(&format!(" {count}"));
            }
            0x57 => {
                // INITSLOT locals,args
                let Some(locals) = take_u8(bytecode, &mut pc) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                let Some(args) = take_u8(bytecode, &mut pc) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                out.push_str(&format!(" locals={locals} args={args}"));
            }
            0x5F | 0x67 | 0x6F | 0x77 | 0x7F | 0x87 => {
                // LDSFLD/STSFLD/LDLOC/STLOC/LDARG/STARG (generic index form)
                let Some(index) = take_u8(bytecode, &mut pc) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                out.push_str(&format!(" {index}"));
            }

            // Syscall
            0x41 => {
                let Some(id_bytes) = take(bytecode, &mut pc, 4) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                let id = [id_bytes[0], id_bytes[1], id_bytes[2], id_bytes[3]];
                if let Some(name) = crate::runtime::spec::syscall_name(&id) {
                    out.push_str(&format!(" {name}"));
                } else {
                    out.push_str(&format!(" 0x{}", hex::encode(id)));
                }
            }

            // CALLT — calls a method token by 2-byte (u16 LE) token index. The
            // operand MUST be consumed or every following instruction mis-aligns.
            0x37 => {
                let Some(idx_bytes) = take(bytecode, &mut pc, 2) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                let idx = u16::from_le_bytes([idx_bytes[0], idx_bytes[1]]);
                out.push_str(&format!(" token={idx}"));
            }

            0xDB | 0xD9 => {
                // CONVERT (0xDB) / ISTYPE (0xD9) take a mandatory 1-byte
                // StackItemType operand. Without consuming it the operand byte
                // was re-decoded as the next opcode, misaligning every
                // subsequent instruction's offset in the listing (the compiler
                // emits 0xDB 0x21 for essentially every storage-int load).
                let Some(type_byte) = take(bytecode, &mut pc, 1) else {
                    out.push_str(" <unexpected EOF>");
                    break;
                };
                out.push_str(&format!(" type=0x{:02x}", type_byte[0]));
            }

            _ => {}
        }

        out.push('\n');
    }

    out
}
