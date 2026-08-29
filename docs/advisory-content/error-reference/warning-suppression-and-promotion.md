---
title: "Error Reference: Warning Suppression and Promotion"
description: "Warning Suppression and Promotion from Error Reference."
---

# Warning Suppression and Promotion

[Back to Error Reference](/advisory-content/error-reference)

## Suppress warnings by code prefix

```bash
# Suppress all unused-variable warnings (E2012)
neo-solc contract.sol --Wno E2012 -I devpack -o build/

# Suppress all semantic warnings (E2xxx)
neo-solc contract.sol --Wno E2 -I devpack -o build/
```

## Promote warnings to errors

```bash
# Treat all security warnings as errors
neo-solc contract.sol --Werror E5 -I devpack -o build/

# Treat specific warning as error
neo-solc contract.sol --Werror E5001 -I devpack -o build/
```

## Combine suppression and promotion

```bash
# Errors on security, suppress unused variables
neo-solc contract.sol --Werror E5 --Wno E2012 -I devpack -o build/
```
