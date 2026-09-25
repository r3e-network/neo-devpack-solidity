use super::*;

pub(crate) fn try_lower_native_contract_constant(
    inner: &Expression,
    member: &Identifier,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Expression::Variable(base) = inner {
        if base.name == "NativeCalls" || base.name == "NativeContracts" {
            let bytes = match member.name.as_str() {
                "NEO_CONTRACT" => Some(
                    b"\xf5\x63\xea\x40\xbc\x28\x3d\x4d\x0e\x05\xc4\x8e\xa3\x05\xb3\xf2\xa0\x73\x40\xef".to_vec(),
                ),
                "GAS_CONTRACT" => Some(
                    b"\xcf\x76\xe2\x8b\xd0\x06\x2c\x4a\x47\x8e\xe3\x55\x61\x01\x13\x19\xf3\xcf\xa4\xd2".to_vec(),
                ),
                "CONTRACT_MANAGEMENT" => Some(
                    b"\xfd\xa3\xfa\x43\x46\xea\x53\x2a\x25\x8f\xc4\x97\xdd\xad\xdb\x64\x37\xc9\xfd\xff".to_vec(),
                ),
                "POLICY_CONTRACT" => Some(
                    b"\x7b\xc6\x81\xc0\xa1\xf7\x1d\x54\x34\x57\xb6\x8b\xba\x8d\x5f\x9f\xdd\x4e\x5e\xcc".to_vec(),
                ),
                "ORACLE_CONTRACT" => Some(
                    b"\x58\x87\x17\x11\x7e\x0a\xa8\x10\x72\xaf\xab\x71\xd2\xdd\x89\xfe\x7c\x4b\x92\xfe".to_vec(),
                ),
                "ROLE_MANAGEMENT" => Some(
                    b"\xe2\x95\xe3\x91\x54\x4c\x17\x8a\xd9\x4f\x03\xec\x4d\xcd\xff\x78\x53\x4e\xcf\x49".to_vec(),
                ),
                "NOTARY_CONTRACT" => Some(
                    b"\x3b\xec\x35\x31\x11\x9b\xba\xd7\x6d\xd0\x44\x92\x0b\x0d\xe6\xc3\x19\x4f\xe1\xc1".to_vec(),
                ),
                "TREASURY_CONTRACT" => Some(
                    b"\xc1\x3a\x56\xc9\x83\x53\xa7\xea\x6a\x32\x4d\x9a\x83\x5d\x1b\x5b\xf2\x26\x63\x15".to_vec(),
                ),
                "LEDGER_CONTRACT" => Some(
                    b"\xbe\xf2\x04\x31\x40\x36\x2a\x77\xc1\x50\x99\xc7\xe6\x4c\x12\xf7\x00\xb6\x65\xda".to_vec(),
                ),
                "CRYPTO_LIB" => Some(
                    b"\x1b\xf5\x75\xab\x11\x89\x68\x84\x13\x61\x0a\x35\xa1\x28\x86\xcd\xe0\xb6\x6c\x72".to_vec(),
                ),
                "STD_LIB" => Some(
                    b"\xc0\xef\x39\xce\xe0\xe4\xe9\x25\xc6\xc2\xa0\x6a\x79\xe1\x44\x0d\xd8\x6f\xce\xac".to_vec(),
                ),
                _ => None,
            };

            if let Some(bytes) = bytes {
                instructions.push(Instruction::PushLiteral(LiteralValue::Address(bytes)));
                return Some(true);
            }
        }
    }

    None
}

pub(crate) fn try_lower_syscalls_contract_constant(
    inner: &Expression,
    member: &Identifier,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Expression::Variable(base) = inner {
        if base.name == "Syscalls" {
            let bytes = match member.name.as_str() {
                "CONTRACT_MANAGEMENT" => Some(
                    b"\xfd\xa3\xfa\x43\x46\xea\x53\x2a\x25\x8f\xc4\x97\xdd\xad\xdb\x64\x37\xc9\xfd\xff".to_vec(),
                ),
                "POLICY_CONTRACT" => Some(
                    b"\x7b\xc6\x81\xc0\xa1\xf7\x1d\x54\x34\x57\xb6\x8b\xba\x8d\x5f\x9f\xdd\x4e\x5e\xcc".to_vec(),
                ),
                "ORACLE_CONTRACT" => Some(
                    b"\x58\x87\x17\x11\x7e\x0a\xa8\x10\x72\xaf\xab\x71\xd2\xdd\x89\xfe\x7c\x4b\x92\xfe".to_vec(),
                ),
                "ROLE_MANAGEMENT" => Some(
                    b"\xe2\x95\xe3\x91\x54\x4c\x17\x8a\xd9\x4f\x03\xec\x4d\xcd\xff\x78\x53\x4e\xcf\x49".to_vec(),
                ),
                "LEDGER_CONTRACT" => Some(
                    b"\xbe\xf2\x04\x31\x40\x36\x2a\x77\xc1\x50\x99\xc7\xe6\x4c\x12\xf7\x00\xb6\x65\xda".to_vec(),
                ),
                "CRYPTO_LIB" => Some(
                    b"\x1b\xf5\x75\xab\x11\x89\x68\x84\x13\x61\x0a\x35\xa1\x28\x86\xcd\xe0\xb6\x6c\x72".to_vec(),
                ),
                "STD_LIB" => Some(
                    b"\xc0\xef\x39\xce\xe0\xe4\xe9\x25\xc6\xc2\xa0\x6a\x79\xe1\x44\x0d\xd8\x6f\xce\xac".to_vec(),
                ),
                _ => None,
            };

            if let Some(bytes) = bytes {
                instructions.push(Instruction::PushLiteral(LiteralValue::Address(bytes)));
                return Some(true);
            }
        }
    }

    None
}
