---
title: "Production Readiness: Production Checklist Summary"
description: "Production Checklist Summary from Production Readiness."
---

# Production Checklist Summary

[Back to Production Readiness](/advisory-content/production-readiness)

## Overview

Use this checklist before every mainnet deployment:

- [ ] `make production-gate` passes
- [ ] Compiled with `-O3 --callt --deny-wildcard-contracts --deny-wildcard-methods`
- [ ] Manifest permissions audited -- no unnecessary wildcards
- [ ] All public methods tested on Neo-Express
- [ ] Constructor arguments tested (if applicable)
- [ ] Contract update path tested (if applicable)
- [ ] Deployed and tested on Neo N3 TestNet
- [ ] Contract hash verified with `--deployer`
- [ ] Security review completed (reentrancy, access control, input validation)
- [ ] Events verified for all state changes
- [ ] GAS costs estimated and acceptable
- [ ] Monitoring plan in place
