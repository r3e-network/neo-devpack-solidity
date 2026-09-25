#[test]
fn supported_standards_flags_nep17() {
    let build_method = |name: &str| FunctionMetadata {
        name: name.to_string(),
        neo_name: name.to_string(),
        kind: FunctionKind::Regular,
        parameters: vec![ParameterMetadata {
            name: Some("addr".to_string()),
            ty: "address".to_string(),
            neo_type: Some(NeoType::Address),
            storage: None,
        }],
        return_parameters: vec![ParameterMetadata {
            name: None,
            ty: "uint256".to_string(),
            neo_type: Some(NeoType::Integer {
                signed: false,
                bits: 256,
            }),
            storage: None,
        }],
        state_mutability: StateMutability::View,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector: [0u8; 4],
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    };

    // NEP-17 requires all 5 methods (symbol, decimals, totalSupply,
    // balanceOf, transfer) PLUS the conformant signatures: a 4-parameter
    // `transfer(from, to, amount, data)` and a 3-parameter Transfer event.
    let make_param = |name: &str, ty: &str, neo_type: NeoType| ParameterMetadata {
        name: Some(name.to_string()),
        ty: ty.to_string(),
        neo_type: Some(neo_type),
        storage: None,
    };
    let uint256 = || NeoType::Integer {
        signed: false,
        bits: 256,
    };
    let mut methods = vec![
        build_method("balanceOf"),
        build_method("totalSupply"),
        build_method("symbol"),
        build_method("decimals"),
        FunctionMetadata {
            name: "transfer".to_string(),
            neo_name: "transfer".to_string(),
            kind: FunctionKind::Regular,
            parameters: vec![
                make_param("from", "address", NeoType::Address),
                make_param("to", "address", NeoType::Address),
                make_param("amount", "uint256", uint256()),
                make_param("data", "bytes", NeoType::ByteArray { fixed_len: None }),
            ],
            return_parameters: vec![ParameterMetadata {
                name: None,
                ty: "bool".to_string(),
                neo_type: Some(NeoType::Boolean),
                storage: None,
            }],
            state_mutability: StateMutability::NonPayable,
            visibility: VisibilityKind::Public,
            offset: 0,
            body: None,
            selector: [0u8; 4],
            is_virtual: false,
            is_override: false,
            documentation: NatspecDoc::default(),
            had_modifier_epilogue: false,
        },
    ];

    let events = vec![EventMetadata {
        name: "Transfer".to_string(),
        normalized_name: "transfer".to_string(),
        parameters: vec![
            EventParameter {
                name: Some("from".to_string()),
                ty: "address".to_string(),
                indexed: true,
                neo_type: Some(NeoType::Address),
            },
            EventParameter {
                name: Some("to".to_string()),
                ty: "address".to_string(),
                indexed: true,
                neo_type: Some(NeoType::Address),
            },
            EventParameter {
                name: Some("amount".to_string()),
                ty: "uint256".to_string(),
                indexed: false,
                neo_type: Some(uint256()),
            },
        ],
        anonymous: false,
    }];

    let result = detect_supported_standards(&methods, &events);
    assert!(
        result.standards.iter().any(|s| s == "NEP-17"),
        "expected NEP-17 standard to be detected with all required methods"
    );

    // Adding extra methods should not affect detection
    methods.push(build_method("approve"));
    let result_extra = detect_supported_standards(&methods, &events);
    assert!(
        result_extra.standards.iter().any(|s| s == "NEP-17"),
        "NEP-17 should still be detected with extra methods"
    );

    // A 2-parameter ERC-20 `transfer(to, amount)` must NOT be advertised as
    // NEP-17 — wallets would invoke the 4-parameter spec signature and fail.
    let mut erc20_methods = methods.clone();
    erc20_methods.retain(|m| m.name != "transfer");
    let mut erc20_transfer = build_method("transfer");
    erc20_transfer.parameters = vec![
        make_param("to", "address", NeoType::Address),
        make_param("amount", "uint256", uint256()),
    ];
    erc20_methods.push(erc20_transfer);
    let result_erc20 = detect_supported_standards(&erc20_methods, &events);
    assert!(
        !result_erc20.standards.iter().any(|s| s == "NEP-17"),
        "2-parameter ERC-20 transfer must not be advertised as NEP-17"
    );
    assert!(
        result_erc20.diagnostics.iter().any(|d| {
            d.standard == "NEP-17"
                && d.level == StandardsDiagnosticLevel::Warning
                && d.message.contains("transfer")
        }),
        "expected a conformance warning for the ERC-20-shaped transfer"
    );
}

#[test]
fn supported_standards_flags_nep24() {
    let build_method = |name: &str| FunctionMetadata {
        name: name.to_string(),
        neo_name: name.to_string(),
        kind: FunctionKind::Regular,
        parameters: vec![],
        return_parameters: vec![],
        state_mutability: StateMutability::View,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector: [0u8; 4],
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    };

    // NEP-24's single mandatory method is `royaltyInfo(tokenId,
    // royaltyToken, salePrice)` — a 3-parameter signature.
    let mut royalty_info = build_method("royaltyInfo");
    royalty_info.parameters = vec![
        ParameterMetadata {
            name: Some("tokenId".to_string()),
            ty: "bytes32".to_string(),
            neo_type: Some(NeoType::ByteArray {
                fixed_len: Some(32),
            }),
            storage: None,
        },
        ParameterMetadata {
            name: Some("royaltyToken".to_string()),
            ty: "address".to_string(),
            neo_type: Some(NeoType::Address),
            storage: None,
        },
        ParameterMetadata {
            name: Some("salePrice".to_string()),
            ty: "uint256".to_string(),
            neo_type: Some(NeoType::Integer {
                signed: false,
                bits: 256,
            }),
            storage: None,
        },
    ];
    let methods = vec![royalty_info];
    let result = detect_supported_standards(&methods, &[]);
    assert!(
        result.standards.iter().any(|s| s == "NEP-24"),
        "expected NEP-24 standard to be detected for a 3-parameter royaltyInfo"
    );

    // `tokenURI` is ERC-721 metadata, NOT a NEP-24 trigger: advertising the
    // royalty standard without royaltyInfo makes marketplaces call a
    // nonexistent method.
    let metadata_only = vec![build_method("tokenUri")];
    let result_metadata = detect_supported_standards(&metadata_only, &[]);
    assert!(
        !result_metadata.standards.iter().any(|s| s == "NEP-24"),
        "tokenURI alone must not advertise NEP-24"
    );

    // A royaltyInfo with the wrong arity must not be advertised either, but
    // should produce an informational diagnostic.
    let wrong_arity = vec![build_method("royaltyInfo")];
    let result_wrong = detect_supported_standards(&wrong_arity, &[]);
    assert!(
        !result_wrong.standards.iter().any(|s| s == "NEP-24"),
        "royaltyInfo with wrong arity must not advertise NEP-24"
    );
    assert!(
        result_wrong
            .diagnostics
            .iter()
            .any(|d| d.standard == "NEP-24" && d.message.contains("signature")),
        "expected signature diagnostic for wrong-arity royaltyInfo"
    );
}

#[test]
fn standards_not_detected_when_incomplete() {
    // Missing balanceOf should not trigger NEP-17
    let methods = vec![
        FunctionMetadata {
            name: "transfer".to_string(),
            neo_name: "transfer".to_string(),
            kind: FunctionKind::Regular,
            parameters: vec![],
            return_parameters: vec![],
            state_mutability: StateMutability::NonPayable,
            visibility: VisibilityKind::Public,
            offset: 0,
            body: None,
            selector: [0u8; 4],
            is_virtual: false,
            is_override: false,
            documentation: NatspecDoc::default(),
            had_modifier_epilogue: false,
        },
        FunctionMetadata {
            name: "totalSupply".to_string(),
            neo_name: "totalSupply".to_string(),
            kind: FunctionKind::Regular,
            parameters: vec![],
            return_parameters: vec![],
            state_mutability: StateMutability::View,
            visibility: VisibilityKind::Public,
            offset: 0,
            body: None,
            selector: [0u8; 4],
            is_virtual: false,
            is_override: false,
            documentation: NatspecDoc::default(),
            had_modifier_epilogue: false,
        },
    ];

    let result = detect_supported_standards(&methods, &[]);
    assert!(
        result.standards.is_empty(),
        "incomplete method sets should not advertise standards"
    );
}

#[test]
fn near_miss_nep17_warns_on_missing_methods() {
    let build_method = |name: &str| FunctionMetadata {
        name: name.to_string(),
        neo_name: name.to_string(),
        kind: FunctionKind::Regular,
        parameters: vec![],
        return_parameters: vec![],
        state_mutability: StateMutability::View,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector: [0u8; 4],
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    };

    // 4 of 5 NEP-17 methods (missing: transfer) → near-miss warning
    let methods = vec![
        build_method("balanceOf"),
        build_method("totalSupply"),
        build_method("symbol"),
        build_method("decimals"),
    ];
    let result = detect_supported_standards(&methods, &[]);
    assert!(
        result.standards.is_empty(),
        "should not detect NEP-17 with only 4 of 5 methods"
    );
    assert!(
        result.diagnostics.iter().any(|d| {
            d.standard == "NEP-17"
                && d.level == StandardsDiagnosticLevel::Warning
                && d.message.contains("missing")
        }),
        "expected near-miss warning for NEP-17, got: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn near_miss_nep11_warns_ownerof_without_transfer() {
    let build_method = |name: &str| FunctionMetadata {
        name: name.to_string(),
        neo_name: name.to_string(),
        kind: FunctionKind::Regular,
        parameters: vec![],
        return_parameters: vec![],
        state_mutability: StateMutability::View,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector: [0u8; 4],
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    };

    // ownerOf present but most of the NEP-11 surface missing → near-miss
    // warning that lists the missing methods.
    let methods = vec![build_method("balanceOf"), build_method("ownerOf")];
    let result = detect_supported_standards(&methods, &[]);
    assert!(
        result.standards.is_empty(),
        "should not detect NEP-11 with an incomplete method set"
    );
    assert!(
        result.diagnostics.iter().any(|d| {
            d.standard == "NEP-11"
                && d.level == StandardsDiagnosticLevel::Warning
                && d.message.contains("missing")
                && d.message.contains("transfer")
        }),
        "expected near-miss warning for NEP-11 listing missing methods"
    );
}

#[test]
fn supported_standards_flags_nep22_nep26_nep27_nep29_nep30_nep31() {
    let build_method = |name: &str| FunctionMetadata {
        name: name.to_string(),
        neo_name: name.to_string(),
        kind: FunctionKind::Regular,
        parameters: vec![],
        return_parameters: vec![],
        state_mutability: StateMutability::NonPayable,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector: [0u8; 4],
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    };

    let mut update = build_method("update");
    update.parameters = vec![
        ParameterMetadata {
            name: Some("nefFile".to_string()),
            ty: "bytes".to_string(),
            neo_type: Some(NeoType::ByteArray { fixed_len: None }),
            storage: None,
        },
        ParameterMetadata {
            name: Some("manifest".to_string()),
            ty: "string".to_string(),
            neo_type: Some(NeoType::String),
            storage: None,
        },
        ParameterMetadata {
            name: Some("data".to_string()),
            ty: "Any".to_string(),
            neo_type: Some(NeoType::Any),
            storage: None,
        },
    ];

    let mut nep26 = build_method("onNEP11Payment");
    nep26.parameters = vec![
        ParameterMetadata {
            name: Some("from".to_string()),
            ty: "address".to_string(),
            neo_type: Some(NeoType::Address),
            storage: None,
        },
        ParameterMetadata {
            name: Some("amount".to_string()),
            ty: "uint256".to_string(),
            neo_type: Some(NeoType::Integer {
                signed: false,
                bits: 256,
            }),
            storage: None,
        },
        ParameterMetadata {
            name: Some("tokenId".to_string()),
            ty: "bytes".to_string(),
            neo_type: Some(NeoType::ByteArray { fixed_len: None }),
            storage: None,
        },
        ParameterMetadata {
            name: Some("data".to_string()),
            ty: "Any".to_string(),
            neo_type: Some(NeoType::Any),
            storage: None,
        },
    ];

    let mut nep27 = build_method("onNEP17Payment");
    nep27.parameters = vec![
        ParameterMetadata {
            name: Some("from".to_string()),
            ty: "address".to_string(),
            neo_type: Some(NeoType::Address),
            storage: None,
        },
        ParameterMetadata {
            name: Some("amount".to_string()),
            ty: "uint256".to_string(),
            neo_type: Some(NeoType::Integer {
                signed: false,
                bits: 256,
            }),
            storage: None,
        },
        ParameterMetadata {
            name: Some("data".to_string()),
            ty: "Any".to_string(),
            neo_type: Some(NeoType::Any),
            storage: None,
        },
    ];

    let mut nep29 = build_method("_deploy");
    nep29.parameters = vec![
        ParameterMetadata {
            name: Some("data".to_string()),
            ty: "Any".to_string(),
            neo_type: Some(NeoType::Any),
            storage: None,
        },
        ParameterMetadata {
            name: Some("update".to_string()),
            ty: "bool".to_string(),
            neo_type: Some(NeoType::Boolean),
            storage: None,
        },
    ];

    let mut nep30 = build_method("verify");
    nep30.return_parameters = vec![ParameterMetadata {
        name: None,
        ty: "bool".to_string(),
        neo_type: Some(NeoType::Boolean),
        storage: None,
    }];

    let destroy = build_method("destroy");

    let methods = vec![update, nep26, nep27, nep29, nep30, destroy];
    let result = detect_supported_standards(&methods, &[]);

    for standard in ["NEP-22", "NEP-26", "NEP-27", "NEP-29", "NEP-30", "NEP-31"] {
        assert!(
            result.standards.iter().any(|s| s == standard),
            "expected {standard} to be detected"
        );
    }
}

#[test]
fn near_miss_nep26_warns_wrong_callback_signature() {
    let method = FunctionMetadata {
        name: "onNEP11Payment".to_string(),
        neo_name: "onNEP11Payment".to_string(),
        kind: FunctionKind::Regular,
        parameters: vec![
            ParameterMetadata {
                name: Some("from".to_string()),
                ty: "address".to_string(),
                neo_type: Some(NeoType::Address),
                storage: None,
            },
            ParameterMetadata {
                name: Some("amount".to_string()),
                ty: "uint256".to_string(),
                neo_type: Some(NeoType::Integer {
                    signed: false,
                    bits: 256,
                }),
                storage: None,
            },
            ParameterMetadata {
                name: Some("tokenId".to_string()),
                ty: "bytes".to_string(),
                neo_type: Some(NeoType::ByteArray { fixed_len: None }),
                storage: None,
            },
        ],
        return_parameters: vec![],
        state_mutability: StateMutability::NonPayable,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector: [0u8; 4],
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    };

    let result = detect_supported_standards(&[method], &[]);
    assert!(
        !result.standards.iter().any(|s| s == "NEP-26"),
        "should not detect NEP-26 with wrong callback arity"
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.standard == "NEP-26" && d.message.contains("signature")),
        "expected signature warning for NEP-26"
    );
}

#[test]
fn near_miss_nep30_warns_non_boolean_verify() {
    let method = FunctionMetadata {
        name: "verify".to_string(),
        neo_name: "verify".to_string(),
        kind: FunctionKind::Regular,
        parameters: vec![],
        return_parameters: vec![ParameterMetadata {
            name: None,
            ty: "uint256".to_string(),
            neo_type: Some(NeoType::Integer {
                signed: false,
                bits: 256,
            }),
            storage: None,
        }],
        state_mutability: StateMutability::View,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector: [0u8; 4],
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    };

    let result = detect_supported_standards(&[method], &[]);
    assert!(
        !result.standards.iter().any(|s| s == "NEP-30"),
        "should not detect NEP-30 when verify does not return bool"
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.standard == "NEP-30" && d.message.contains("return")),
        "expected return-type warning for NEP-30"
    );
}

#[test]
fn supported_standards_flags_nep22_and_nep31_from_update_destroy() {
    let update = FunctionMetadata {
        name: "update".to_string(),
        neo_name: "update".to_string(),
        kind: FunctionKind::Regular,
        parameters: vec![
            ParameterMetadata {
                name: Some("nefFile".to_string()),
                ty: "bytes".to_string(),
                neo_type: Some(NeoType::ByteArray { fixed_len: None }),
                storage: None,
            },
            ParameterMetadata {
                name: Some("manifest".to_string()),
                ty: "string".to_string(),
                neo_type: Some(NeoType::String),
                storage: None,
            },
            ParameterMetadata {
                name: Some("data".to_string()),
                ty: "Any".to_string(),
                neo_type: Some(NeoType::Any),
                storage: None,
            },
        ],
        return_parameters: vec![],
        state_mutability: StateMutability::NonPayable,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector: [0u8; 4],
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    };

    let destroy = FunctionMetadata {
        name: "destroy".to_string(),
        neo_name: "destroy".to_string(),
        kind: FunctionKind::Regular,
        parameters: vec![],
        return_parameters: vec![],
        state_mutability: StateMutability::NonPayable,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector: [0u8; 4],
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    };

    let methods = vec![update, destroy];
    let result = detect_supported_standards(&methods, &[]);
    assert!(
        result.standards.iter().any(|s| s == "NEP-22"),
        "expected NEP-22 to be detected when update() is present with 3 parameters"
    );
    assert!(
        result.standards.iter().any(|s| s == "NEP-31"),
        "expected NEP-31 to be detected when destroy() is present"
    );
    assert!(
        !result.standards.iter().any(|s| s == "NEP-26"),
        "NEP-26 must only be detected from onNEP11Payment callback signature"
    );
}

#[test]
fn supported_standards_update_without_nep22_signature_is_not_detected() {
    let update = FunctionMetadata {
        name: "update".to_string(),
        neo_name: "update".to_string(),
        kind: FunctionKind::Regular,
        parameters: vec![],
        return_parameters: vec![],
        state_mutability: StateMutability::NonPayable,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector: [0u8; 4],
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    };
    let methods = vec![update];
    let result = detect_supported_standards(&methods, &[]);
    assert!(
        !result.standards.iter().any(|s| s == "NEP-22"),
        "NEP-22 must not be detected for update() with wrong arity"
    );
}

#[test]
fn event_validation_warns_missing_transfer_event() {
    let build_method = |name: &str| FunctionMetadata {
        name: name.to_string(),
        neo_name: name.to_string(),
        kind: FunctionKind::Regular,
        parameters: vec![],
        return_parameters: vec![],
        state_mutability: StateMutability::View,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector: [0u8; 4],
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    };

    // All NEP-17 method names present but no Transfer event → NOT detected
    // (the manifest must never claim a standard the contract cannot fulfil)
    // and a conformance warning explains what is missing.
    let methods = vec![
        build_method("balanceOf"),
        build_method("totalSupply"),
        build_method("symbol"),
        build_method("decimals"),
        build_method("transfer"),
    ];
    let result = detect_supported_standards(&methods, &[]);
    assert!(
        !result.standards.iter().any(|s| s == "NEP-17"),
        "NEP-17 must not be claimed without a conformant Transfer event"
    );
    assert!(
        result.diagnostics.iter().any(|d| {
            d.standard == "NEP-17"
                && d.level == StandardsDiagnosticLevel::Warning
                && d.message.contains("Transfer")
        }),
        "expected warning about missing Transfer event"
    );
}

#[test]
fn event_validation_accepts_correct_transfer_event() {
    let build_method = |name: &str| FunctionMetadata {
        name: name.to_string(),
        neo_name: name.to_string(),
        kind: FunctionKind::Regular,
        parameters: vec![],
        return_parameters: vec![],
        state_mutability: StateMutability::View,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector: [0u8; 4],
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    };

    // Conformant 4-parameter transfer(from, to, amount, data).
    let mut transfer = build_method("transfer");
    transfer.parameters = (0..4)
        .map(|i| ParameterMetadata {
            name: Some(format!("p{i}")),
            ty: "bytes".to_string(),
            neo_type: Some(NeoType::ByteArray { fixed_len: None }),
            storage: None,
        })
        .collect();

    let methods = vec![
        build_method("balanceOf"),
        build_method("totalSupply"),
        build_method("symbol"),
        build_method("decimals"),
        transfer,
    ];

    // Correct NEP-17 Transfer event: 3 parameters
    let events = vec![EventMetadata {
        name: "Transfer".to_string(),
        normalized_name: "transfer".to_string(),
        parameters: vec![
            EventParameter {
                name: Some("from".to_string()),
                ty: "address".to_string(),
                indexed: true,
                neo_type: Some(NeoType::Address),
            },
            EventParameter {
                name: Some("to".to_string()),
                ty: "address".to_string(),
                indexed: true,
                neo_type: Some(NeoType::Address),
            },
            EventParameter {
                name: Some("value".to_string()),
                ty: "uint256".to_string(),
                indexed: false,
                neo_type: Some(NeoType::Integer {
                    signed: false,
                    bits: 256,
                }),
            },
        ],
        anonymous: false,
    }];

    let result = detect_supported_standards(&methods, &events);
    assert!(
        result.standards.iter().any(|s| s == "NEP-17"),
        "NEP-17 should be detected"
    );
    // No Transfer-event warning when event is correct
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|d| { d.standard == "NEP-17" && d.message.contains("Transfer") }),
        "should not warn about Transfer event when it has correct param count"
    );
}
