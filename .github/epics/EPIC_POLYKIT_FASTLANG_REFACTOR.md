# PolyKit FastLang Refactor

> **Status**: In Progress
> **Priority**: P1
> **Estimated Effort**: ~2-3 weeks (4 phases)
> **Target**: PolyKit v0.2.0 on eStream SDK v0.8.3
> **Depends On**: [EPIC_FASTLANG_V083_RELEASE](https://github.com/polyquantum/estream-io/blob/main/.github/epics/EPIC_FASTLANG_V083_RELEASE.md), [EPIC_FASTLANG_NATIVE_TRANSITION](https://github.com/polyquantum/estream-io/blob/main/.github/epics/EPIC_FASTLANG_NATIVE_TRANSITION.md)

---

## Overview

Migrate PolyKit from ESCIR YAML circuit definitions (v0.8.1) to FastLang `.fl` source files (v0.8.3). FastLang becomes the single source of truth -- generating Rust, WASM, and TypeScript bindings from `.fl` files. The separate telemetry circuit is eliminated; StreamSight is woven inline via annotations on every circuit.

### Prior State

| Item | Status |
|------|--------|
| 6 ESCIR YAML circuit definitions | Current (v0.8.1) |
| 5 hand-written Rust crates (~2,500 lines) | Current |
| 4 FastLang polylabs examples in estream-io | Reference only |
| polykit-telemetry as separate circuit | Current (to be eliminated) |

### Success Metrics

| Metric | Target |
|--------|--------|
| All circuits defined as `.fl` source | 10 core + 4 app + 1 platform |
| Separate telemetry circuit | Eliminated (inline via annotations) |
| Hand-written Rust | ≤600 lines (kernel + shim) |
| FastLang `.fl` lines | ~800 lines total |
| ESCIR YAML files | Archived to `circuits/legacy/` |
| Build pipeline | `estream-dev build-wasm-client --from-fl` |
| StreamSight coverage | Every circuit has `observe` + `monitor` |

---

## Phase 1: Design & Scaffolding

- [x] Design document (`docs/FASTLANG_REFACTOR_PLAN.md`)
- [x] Epic created (`.github/epics/EPIC_POLYKIT_FASTLANG_REFACTOR.md`)
- [x] `circuits/fl/` directory created
- [ ] Shared annotation profile (`polykit_profile.fl`)

### Exit Criteria

- Design document reviewed and approved
- Profile file compiles with `estream codegen compile`

---

## Phase 2: Circuit Conversion (YAML to FastLang)

Convert 5 ESCIR YAML circuits to FastLang. The 6th (telemetry) is eliminated -- its functionality is distributed as inline annotations across all circuits.

- [ ] `polykit_identity.fl` -- SPARK auth, HKDF, ML-DSA-87, ML-KEM-1024
  - Replaces: `circuits/polykit-identity/circuit.escir.yaml` + `crates/polykit-core/src/identity.rs` + `crates/polykit-core/src/crypto.rs`
  - Features: `constant_time`, `kat_vector`, `invariant`, `observe metrics`, `monitor`
- [ ] `polykit_metering.fl` -- 8-dimension resource metering
  - Replaces: `circuits/polykit-metering/circuit.escir.yaml` + `crates/polykit-core/src/metering.rs`
  - Features: `stream` + `emit`, `parallel for`, `meters`, `observe metrics`
- [ ] `polykit_rate_limiter.fl` -- Rate limiter with FSM
  - Replaces: `circuits/polykit-rate-limiter/circuit.escir.yaml`
  - Features: `state_machine`, `streamsight_anomaly()` body calls, `observe metrics`, `monitor`
- [ ] `polykit_sanitize.fl` -- 3-stage compliance pipeline
  - Replaces: `circuits/polykit-sanitize/circuit.escir.yaml` + `crates/polykit-sanitize/`
  - Features: `@sanitize`, `li_classify`, `witness full`, `esz_emit`, `li_feed`, `observe metrics`, `monitor`
- [ ] `polykit_li_effects.fl` -- LI Effects classification + human-in-loop
  - Replaces: `circuits/polykit-eslm-classify/circuit.escir.yaml` (ESLM renamed to LI Effects)
  - Features: `feedback` streams, `li_embed`/`li_classify`/`li_infer`, `stream` + `emit`, `observe metrics`

### Exit Criteria

- All 5 `.fl` files compile with `estream codegen compile`
- `cargo test -p estream-fastlang -- polykit` passes
- StreamSight annotations present on every circuit

---

## Phase 3: New FastLang-Only Circuits

Circuits that leverage constructs only available in FastLang (not expressible in YAML).

- [ ] `polykit_delta_curate.fl` -- Field-level delta encoding with bitmask proofs
  - Features: `delta_curate` annotation, `invariant "lossless"`, `observe metrics`
- [ ] `polykit_governance.fl` -- Field governance with filtered fan-out + regional fan-in
  - Features: `audience` declarations, `field_governance` blocks, `sub_lex fan_out share/redact`, `sub_lex region_us fan_in`, `sub_lex region_eu fan_in`
- [ ] `polykit_regional_us.fl` -- US sovereignty and CCPA/SOC2 compliance
  - Features: `sovereignty us`, `data_residency us`, `compliance [ccpa, soc2, coppa]`, `sub_lex app fan_in`, `sub_lex global fan_out` with share/redact
- [ ] `polykit_regional_eu.fl` -- EU sovereignty and GDPR/EU AI Act compliance
  - Features: `sovereignty eu`, `data_residency eu_only`, `compliance [gdpr, eu_ai_act]`, GDPR `field_governance` tiers, `sub_lex personal/pseudonymized/anonymous fan_out`
- [ ] `polykit_platform.fl` -- Platform composition connecting all circuits
  - Features: `platform`, `import`, `group`, `connect`, `bind`, `streamsight true`, `esz_emit`, `li_feed`
  - Groups: `core`, `compliance`, `intelligence`, `regional`, `apps`

### Phase 3b: App-Level Circuits (`circuits/fl/apps/`)

Adapted from estream-io polylabs examples with PolyKit profiles and metering:

- [ ] `polykit_media_stream.fl` -- PQ-encrypted voice/video SFU
- [ ] `polykit_crdt_sync.fl` -- Offline-capable CRDT merge
- [ ] `polykit_blind_relay.fl` -- Privacy-preserving message relay with cover traffic
- [ ] `polykit_classified_fusion.fl` -- SCI omniscient fusion with 4-tier fan_in

### Exit Criteria

- All core + regional + app `.fl` files compile
- Platform composition validates all circuit connections (including regional and app groups)
- Golden tests cover new circuits
- Regional fan-in/fan-out correctly scopes data to US and EU sub-lexes

---

## Phase 4: Crate Slimming, Archive & Docs

- [ ] Slim `polykit-core` -- remove identity/crypto/metering code replaced by codegen
- [ ] Slim `polykit-wasm` -- reduce to thin shim over codegen'd circuit exports
- [ ] Archive YAML -- move `circuits/*.escir.yaml` to `circuits/legacy/`
- [ ] Update `ARCHITECTURE.md` -- reflect FastLang-native pipeline
- [ ] Update `ESTREAM_GETTING_STARTED.md` -- new build commands, circuit catalog, migration guide
- [ ] Update `Cargo.toml` -- bump eStream deps to v0.8.3, remove codegen-replaced crate code

### Exit Criteria

- `estream-dev build-wasm-client --from-fl circuits/fl/ --sign key.pem --enforce-budget` succeeds
- Hand-written Rust ≤600 lines
- All docs reference `.fl` files, not `.escir.yaml`
- No YAML circuit files outside `circuits/legacy/`

---

## Dependencies

| Dependency | Status | Notes |
|-----------|--------|-------|
| eStream SDK v0.8.3 | In Progress | FastLang compiler, codegen backends |
| FastLang `profile` support | In Progress | Shared annotation profiles |
| FastLang `composes:` support | In Progress | Circuit composition with versioning |
| FastLang `stream` + `emit` | Complete | Stream declarations and emit calls |
| ESCIR WASM Client Spec (#550) | Complete | Pipeline spec |
| Wire Protocol Only (#551) | Complete | No REST policy |
