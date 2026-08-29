use super::*;
use crate::opcode::OpCode;
use crate::runtime::execution::types::stack::ByteArrayType;

impl ExecutionContext {
    pub(crate) fn execute_collection_instruction(
        &mut self,
        opcode: u8,
    ) -> Result<bool, RuntimeError> {
        const PACKMAP: u8 = OpCode::PACKMAP.byte();
        const PACKSTRUCT: u8 = OpCode::PACKSTRUCT.byte();
        const PACK: u8 = OpCode::PACK.byte();
        const UNPACK: u8 = OpCode::UNPACK.byte();
        const NEWARRAY0: u8 = OpCode::NEWARRAY0.byte();
        const NEWARRAY: u8 = OpCode::NEWARRAY.byte();
        const NEWARRAY_T: u8 = OpCode::NEWARRAY_T.byte();
        const NEWSTRUCT0: u8 = OpCode::NEWSTRUCT0.byte();
        const NEWSTRUCT: u8 = OpCode::NEWSTRUCT.byte();
        const NEWMAP: u8 = OpCode::NEWMAP.byte();
        const SIZE: u8 = OpCode::SIZE.byte();
        const HASKEY: u8 = OpCode::HASKEY.byte();
        const KEYS: u8 = OpCode::KEYS.byte();
        const VALUES: u8 = OpCode::VALUES.byte();
        const PICKITEM: u8 = OpCode::PICKITEM.byte();
        const APPEND: u8 = OpCode::APPEND.byte();
        const SETITEM: u8 = OpCode::SETITEM.byte();
        const REVERSEITEMS: u8 = OpCode::REVERSEITEMS.byte();
        const REMOVE: u8 = OpCode::REMOVE.byte();
        const CLEARITEMS: u8 = OpCode::CLEARITEMS.byte();
        const POPITEM: u8 = OpCode::POPITEM.byte();
        const ISNULL: u8 = OpCode::ISNULL.byte();
        const ISTYPE: u8 = OpCode::ISTYPE.byte();
        const CONVERT: u8 = OpCode::CONVERT.byte();
        const ABORTMSG: u8 = OpCode::ABORTMSG.byte();
        const ASSERTMSG: u8 = OpCode::ASSERTMSG.byte();

        match opcode {
            PACKMAP => {
                // PACKMAP
                self.pack_map()?;
                self.instruction_pointer += 1;
            }
            PACKSTRUCT => {
                // PACKSTRUCT (array-backed)
                self.pack_items()?;
                self.instruction_pointer += 1;
            }
            PACK => {
                // PACK
                self.pack_items()?;
                self.instruction_pointer += 1;
            }
            UNPACK => {
                // UNPACK
                self.unpack()?;
                self.instruction_pointer += 1;
            }
            NEWARRAY0 => {
                // NEWARRAY0
                self.new_array0()?;
                self.instruction_pointer += 1;
            }
            NEWARRAY => {
                // NEWARRAY
                self.new_array()?;
                self.instruction_pointer += 1;
            }
            NEWARRAY_T => {
                // NEWARRAY_T
                self.new_array()?;
                self.instruction_pointer += 1;
            }
            NEWSTRUCT0 => {
                // NEWSTRUCT0
                self.new_struct0()?;
                self.instruction_pointer += 1;
            }
            NEWSTRUCT => {
                // NEWSTRUCT
                self.new_struct()?;
                self.instruction_pointer += 1;
            }
            NEWMAP => {
                // NEWMAP
                self.new_map()?;
                self.instruction_pointer += 1;
            }
            SIZE => {
                // SIZE
                self.size_of()?;
                self.instruction_pointer += 1;
            }
            HASKEY => {
                // HASKEY
                self.haskey()?;
                self.instruction_pointer += 1;
            }
            KEYS => {
                // KEYS
                self.keys()?;
                self.instruction_pointer += 1;
            }
            VALUES => {
                // VALUES
                self.values()?;
                self.instruction_pointer += 1;
            }
            PICKITEM => {
                // PICKITEM
                self.pick_item()?;
                self.instruction_pointer += 1;
            }
            APPEND => {
                // APPEND
                self.append_item()?;
                self.instruction_pointer += 1;
            }
            SETITEM => {
                // SETITEM
                self.set_item()?;
                self.instruction_pointer += 1;
            }
            REVERSEITEMS => {
                // REVERSEITEMS
                self.reverse_items()?;
                self.instruction_pointer += 1;
            }
            REMOVE => {
                // REMOVE
                self.remove_item()?;
                self.instruction_pointer += 1;
            }
            CLEARITEMS => {
                // CLEARITEMS
                self.clear_items()?;
                self.instruction_pointer += 1;
            }
            POPITEM => {
                // POPITEM
                self.pop_item_from_collection()?;
                self.instruction_pointer += 1;
            }
            ISNULL => {
                // ISNULL
                let item = self.pop_stack()?;
                self.push_stack(StackItem::Boolean(matches!(item, StackItem::Null)))?;
                self.instruction_pointer += 1;
            }
            ISTYPE => {
                // ISTYPE (best-effort): NeoVM takes a 1-byte immediate operand (StackItemType).
                let operand_index = self.instruction_pointer as usize + 1;
                if operand_index >= self.bytecode.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "ISTYPE: insufficient bytecode for operand".to_string(),
                    });
                }
                let expected = self.bytecode[operand_index];
                let item = self.pop_stack()?;
                let matches = match expected {
                    0x00 => true, // Any
                    0x20 => matches!(item, StackItem::Boolean(_)),
                    0x21 | 0x22 => {
                        matches!(item, StackItem::Integer(_) | StackItem::UnsignedInteger(_))
                    }
                    0x28 => matches!(
                        item,
                        StackItem::ByteArray {
                            type_tag: ByteArrayType::ByteString,
                            ..
                        }
                    ),
                    0x30 => matches!(
                        item,
                        StackItem::ByteArray {
                            type_tag: ByteArrayType::Buffer,
                            ..
                        }
                    ),
                    0x40 | 0x41 => matches!(item, StackItem::Array(_)),
                    0x48 => matches!(item, StackItem::Map(_)),
                    0x60 => matches!(item, StackItem::ByteArray { data: _, .. }), // interop handles as byte tokens
                    0x80 => self.is_iterator_token(&item),
                    _ => false,
                };
                self.push_stack(StackItem::Boolean(matches))?;
                self.instruction_pointer += 2;
            }
            CONVERT => {
                // CONVERT (best-effort coercion): NeoVM takes a 1-byte immediate operand (StackItemType).
                let operand_index = self.instruction_pointer as usize + 1;
                if operand_index >= self.bytecode.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "CONVERT: insufficient bytecode for operand".to_string(),
                    });
                }
                let target_code = self.bytecode[operand_index];
                let item = self.pop_stack()?;
                let converted = self.convert_item(item, target_code)?;
                self.push_stack(converted)?;
                self.instruction_pointer += 2;
            }
            ABORTMSG => {
                // ABORTMSG
                let message = Self::stack_item_to_bytes(self.pop_stack()?);
                return Err(RuntimeError::ExecutionError {
                    message: format!("ABORTMSG: {}", String::from_utf8_lossy(&message)),
                });
            }
            ASSERTMSG => {
                // ASSERTMSG
                let message = Self::stack_item_to_bytes(self.pop_stack()?);
                let condition = self.pop_stack()?;
                if condition.is_truthy() {
                    self.instruction_pointer += 1;
                } else {
                    return Err(RuntimeError::ExecutionError {
                        message: format!("ASSERTMSG failed: {}", String::from_utf8_lossy(&message)),
                    });
                }
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
