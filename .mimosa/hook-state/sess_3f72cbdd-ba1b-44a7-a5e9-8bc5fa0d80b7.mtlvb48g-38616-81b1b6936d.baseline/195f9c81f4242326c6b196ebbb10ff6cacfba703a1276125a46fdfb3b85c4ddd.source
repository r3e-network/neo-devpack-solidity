#[test]
fn manifest_supported_standards_match_detection() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract ERC20Like {
        mapping(address => uint256) public balances;

        event Transfer(address indexed from, address indexed to, uint256 amount);

        function symbol() public pure returns (string memory) { return "TKN"; }
        function decimals() public pure returns (uint8) { return 8; }
        function totalSupply() public view returns (uint256) { return 0; }
        function balanceOf(address account) public view returns (uint256) { return balances[account]; }

        function transfer(address from, address to, uint256 amount, bytes memory data) public returns (bool) {
            data;
            balances[from] -= amount;
            balances[to] += amount;
            emit Transfer(from, to, amount);
            return true;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let manifest = &artifacts[0].manifest;

    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supportedstandards array");
    assert!(
        standards.iter().any(|s| s.as_str() == Some("NEP-17")),
        "ERC20-like token should advertise NEP-17"
    );
}

#[test]
fn erc721_shaped_contract_does_not_advertise_nep11() {
    // A plain ERC-721 port (balanceOf + ownerOf + transferFrom) lacks most
    // of the NEP-11 mandatory surface (symbol, decimals, totalSupply,
    // tokensOf, transfer) — advertising NEP-11 would make wallets call
    // methods that do not exist. The methods still appear in the manifest;
    // only the standard claim is withheld.
    let source = r#"
    pragma solidity ^0.8.19;

    contract ERC721Like {
        mapping(uint256 => address) private owners;
        mapping(address => uint256) private balances;

        function balanceOf(address owner) public view returns (uint256) { return balances[owner]; }
        function ownerOf(uint256 tokenId) public view returns (address) { return owners[tokenId]; }

        function transferFrom(address from, address to, uint256 tokenId) public {
            owners[tokenId] = to;
            balances[from] -= 1;
            balances[to] += 1;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let manifest = &artifacts[0].manifest;

    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supportedstandards array");
    assert!(
        !standards.iter().any(|s| s.as_str() == Some("NEP-11")),
        "ERC721-shaped contract must not advertise NEP-11 (missing required methods)"
    );

    let methods = manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    let requires = ["ownerOf", "transferFrom"];
    for required in requires {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in manifest"
        );
    }
}

#[test]
fn full_nep11_surface_advertises_nep11() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract Nep11Token {
        mapping(uint256 => address) private owners;
        mapping(address => uint256) private balances;

        event Transfer(address indexed from, address indexed to, uint256 amount, bytes tokenId);

        function symbol() public pure returns (string memory) { return "NFT"; }
        function decimals() public pure returns (uint8) { return 0; }
        function totalSupply() public view returns (uint256) { return 0; }
        function balanceOf(address owner) public view returns (uint256) { return balances[owner]; }
        function tokensOf(address owner) public view returns (bytes memory) { owner; return ""; }
        function ownerOf(uint256 tokenId) public view returns (address) { return owners[tokenId]; }

        function transfer(address to, uint256 tokenId, bytes memory data) public returns (bool) {
            address from = owners[tokenId];
            owners[tokenId] = to;
            balances[from] -= 1;
            balances[to] += 1;
            emit Transfer(from, to, 1, data);
            return true;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let manifest = &artifacts[0].manifest;

    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supportedstandards array");
    assert!(
        standards.iter().any(|s| s.as_str() == Some("NEP-11")),
        "full NEP-11 surface (methods + 3-param transfer + 4-param Transfer event) \
         should advertise NEP-11, got {standards:?}"
    );
}

#[test]
fn lifecycle_and_callback_manifest_advertises_additional_neps() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract LifecycleHooks {
        function update(bytes memory nefFile, string memory manifestText, bytes memory data) public {
            nefFile; manifestText; data;
        }

        function onNEP11Payment(address from, uint256 amount, bytes memory tokenId, bytes memory data) public {
            from; amount; tokenId; data;
        }

        function onNEP17Payment(address from, uint256 amount, bytes memory data) public {
            from; amount; data;
        }

        function _deploy(bytes memory data, bool updateFlag) public {
            data; updateFlag;
        }

        function verify() public pure returns (bool) {
            return true;
        }

        function destroy() public {}
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let manifest = &artifacts[0].manifest;

    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supportedstandards array");

    for standard in ["NEP-22", "NEP-26", "NEP-27", "NEP-29", "NEP-30", "NEP-31"] {
        assert!(
            standards.iter().any(|s| s.as_str() == Some(standard)),
            "expected {standard} in supportedstandards"
        );
    }
}
