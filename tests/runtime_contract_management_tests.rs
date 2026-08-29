use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;
use sha2::{Digest, Sha256};

fn syscall_id(name: &str) -> [u8; 4] {
    let mut h = Sha256::new();
    h.update(name.as_bytes());
    let d = h.finalize();
    [d[0], d[1], d[2], d[3]]
}

fn push_data(script: &mut Vec<u8>, data: &[u8]) {
    assert!(
        data.len() <= u8::MAX as usize,
        "push_data only supports PUSHDATA1 lengths"
    );
    script.push(0x0C); // PUSHDATA1
    script.push(data.len() as u8);
    script.extend_from_slice(data);
}

// ContractManagement native contract hash in NeoVM stack byte order (UInt160 little-endian).
const CONTRACT_MANAGEMENT_HASH_LE: [u8; 20] = [
    0xFD, 0xA3, 0xFA, 0x43, 0x46, 0xEA, 0x53, 0x2A, 0x25, 0x8F, 0xC4, 0x97, 0xDD, 0xAD, 0xDB, 0x64,
    0x37, 0xC9, 0xFD, 0xFF,
];

#[test]
fn deploy_and_update_contract_tracks_state() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");

    let call_id = syscall_id("System.Contract.Call");

    // Deploy: ContractManagement.deploy(nef, manifest)
    let mut deploy = Vec::new();
    // Stack order for System.Contract.Call: [args, flags, method, hash]
    push_data(&mut deploy, &[0xAA, 0xAB]); // nef bytes
    push_data(&mut deploy, &[0xBA, 0xBB]); // manifest bytes
    deploy.push(0x12); // PUSH2 (count=2)
    deploy.push(0xC0); // PACK -> params array
    deploy.push(0x1F); // PUSH15 (CallFlags.All = 0x0F)
    push_data(&mut deploy, b"deploy");
    push_data(&mut deploy, &CONTRACT_MANAGEMENT_HASH_LE);
    deploy.push(0x41); // SYSCALL
    deploy.extend_from_slice(&call_id);
    deploy.push(0x40); // RET

    ctx.initialize(&deploy, &[]).expect("init");
    while !ctx.step().expect("step").halted {}
    assert_eq!(ctx.contract_registry_len(), 1, "one contract registered");
    let hashes = ctx.contract_hashes();
    let hash = hashes.first().expect("hash");
    assert_eq!(ctx.contract_update_counter(hash), Some(0));

    // Update with new nef/manifest
    let mut update = Vec::new();
    push_data(&mut update, &[0xCC, 0xCD]); // nef bytes
    push_data(&mut update, &[0xDC, 0xDD]); // manifest bytes
    update.push(0x12); // PUSH2 (count=2)
    update.push(0xC0); // PACK -> params array
    update.push(0x1F); // PUSH15 (CallFlags.All)
    push_data(&mut update, b"update");
    push_data(&mut update, &CONTRACT_MANAGEMENT_HASH_LE);
    update.push(0x41); // SYSCALL
    update.extend_from_slice(&call_id);
    update.push(0x40); // RET
    ctx.initialize(&update, &[]).expect("init");
    while !ctx.step().expect("step").halted {}
    assert_eq!(ctx.contract_registry_len(), 1);
    assert_eq!(ctx.contract_update_counter(hash), Some(1));
}

#[test]
fn multiple_deploys_create_separate_contracts() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");

    let call_id = syscall_id("System.Contract.Call");

    // Deploy first contract
    let mut deploy1 = Vec::new();
    push_data(&mut deploy1, &[0x01, 0x01]); // nef bytes
    push_data(&mut deploy1, &[0x02, 0x02]); // manifest bytes
    deploy1.push(0x12); // PUSH2
    deploy1.push(0xC0); // PACK -> params array
    deploy1.push(0x1F); // PUSH15 (CallFlags.All)
    push_data(&mut deploy1, b"deploy");
    push_data(&mut deploy1, &CONTRACT_MANAGEMENT_HASH_LE);
    deploy1.push(0x41); // SYSCALL
    deploy1.extend_from_slice(&call_id);
    deploy1.push(0x40); // RET
    ctx.initialize(&deploy1, &[]).expect("init");
    while !ctx.step().expect("step").halted {}
    assert_eq!(ctx.contract_registry_len(), 1);

    // Deploy second contract with different bytes
    let mut deploy2 = Vec::new();
    push_data(&mut deploy2, &[0x03, 0x03]); // nef bytes
    push_data(&mut deploy2, &[0x04, 0x04]); // manifest bytes
    deploy2.push(0x12); // PUSH2
    deploy2.push(0xC0); // PACK -> params array
    deploy2.push(0x1F); // PUSH15 (CallFlags.All)
    push_data(&mut deploy2, b"deploy");
    push_data(&mut deploy2, &CONTRACT_MANAGEMENT_HASH_LE);
    deploy2.push(0x41); // SYSCALL
    deploy2.extend_from_slice(&call_id);
    deploy2.push(0x40); // RET
    ctx.initialize(&deploy2, &[]).expect("init");
    while !ctx.step().expect("step").halted {}
    assert_eq!(ctx.contract_registry_len(), 2, "should have two contracts");

    // Verify they have different hashes
    let hashes = ctx.contract_hashes();
    assert_eq!(hashes.len(), 2);
    assert_ne!(
        hashes[0], hashes[1],
        "contracts should have different hashes"
    );
}

#[test]
fn update_increments_counter_multiple_times() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");

    let call_id = syscall_id("System.Contract.Call");

    // Deploy (ContractManagement.deploy)
    let mut deploy = Vec::new();
    push_data(&mut deploy, &[0xAA, 0xAB]); // nef bytes
    push_data(&mut deploy, &[0xBA, 0xBB]); // manifest bytes
    deploy.push(0x12); // PUSH2
    deploy.push(0xC0); // PACK -> params array
    deploy.push(0x1F); // PUSH15 (CallFlags.All)
    push_data(&mut deploy, b"deploy");
    push_data(&mut deploy, &CONTRACT_MANAGEMENT_HASH_LE);
    deploy.push(0x41); // SYSCALL
    deploy.extend_from_slice(&call_id);
    deploy.push(0x40); // RET
    ctx.initialize(&deploy, &[]).expect("init");
    while !ctx.step().expect("step").halted {}

    let hashes = ctx.contract_hashes();
    let hash = hashes.first().expect("hash");
    assert_eq!(ctx.contract_update_counter(hash), Some(0));

    // Update multiple times
    for i in 1..=3 {
        let mut update = Vec::new();
        push_data(&mut update, &[0xCC, i as u8]); // nef bytes
        push_data(&mut update, &[0xDC, i as u8]); // manifest bytes
        update.push(0x12); // PUSH2
        update.push(0xC0); // PACK -> params array
        update.push(0x1F); // PUSH15 (CallFlags.All)
        push_data(&mut update, b"update");
        push_data(&mut update, &CONTRACT_MANAGEMENT_HASH_LE);
        update.push(0x41); // SYSCALL
        update.extend_from_slice(&call_id);
        update.push(0x40); // RET
        ctx.initialize(&update, &[]).expect("init");
        while !ctx.step().expect("step").halted {}
        assert_eq!(
            ctx.contract_update_counter(hash),
            Some(i),
            "update counter should be {i} after {i} updates"
        );
    }
}
