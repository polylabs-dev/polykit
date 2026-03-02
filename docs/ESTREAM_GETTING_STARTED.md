# Getting Started with eStream — PolyKit (Poly Labs)

> **eStream SDK**: v0.8.3 (single version model)
> **Date**: February 2026
> **Previous**: eStream v0.8.1 (see [Migration](#migrating-from-v081))

This guide covers how Poly Labs developers use PolyKit to build WASM-first apps on the eStream platform, write FastLang circuits, test locally, and deploy to the alpha-devnet.

PolyKit is the shared framework for all Poly Labs apps — Poly Data, Poly Messenger, Poly Mail, Poly VPN, Poly Pass, Poly OAuth, Poly Mind. It provides identity, PQ crypto, metering, classification, ESLite, console widgets, and sanitization as FastLang circuits compiled to WASM via the ESCIR codegen pipeline.

---

## FastLang Circuits

PolyKit defines 10 FastLang circuit files plus 4 app-level circuits and 1 platform composition in `circuits/fl/`:

| File | What It Does | Key Features |
|------|-------------|--------------|
| [`polykit_profile.fl`](../circuits/fl/polykit_profile.fl) | Shared annotation profiles | `poly_framework_standard`, `poly_framework_sensitive`, `amplify enterprise` |
| [`polykit_identity.fl`](../circuits/fl/polykit_identity.fl) | SPARK auth + HKDF + ML-DSA-87/ML-KEM-1024 | `constant_time`, `kat_vector`, `invariant`, `observe metrics` |
| [`polykit_metering.fl`](../circuits/fl/polykit_metering.fl) | 8-dimension resource metering (E/H/B/S/O/P/C/M) | `stream` + `emit`, `parallel for`, `meters`, `observe metrics` |
| [`polykit_rate_limiter.fl`](../circuits/fl/polykit_rate_limiter.fl) | Rate limiter with FSM and abuse detection | `state_machine`, `streamsight_anomaly()`, `monitor` |
| [`polykit_sanitize.fl`](../circuits/fl/polykit_sanitize.fl) | 3-stage PII/PCI/HIPAA/GDPR compliance | `@sanitize`, `li_classify`, `witness full`, `esz_emit`, `li_feed` |
| [`polykit_li_effects.fl`](../circuits/fl/polykit_li_effects.fl) | LI Effects content classification | `feedback` streams, `li_embed`/`li_classify`/`li_infer` |
| [`polykit_delta_curate.fl`](../circuits/fl/polykit_delta_curate.fl) | Field-level delta encoding with bitmask proofs | `delta_curate`, lossless ~20x compression |
| [`polykit_governance.fl`](../circuits/fl/polykit_governance.fl) | Field-level visibility control | `audience`, `field_governance`, filtered fan-out |
| [`polykit_regional_us.fl`](../circuits/fl/polykit_regional_us.fl) | US sovereignty, CCPA, SOC2 compliance | `sovereignty us`, `data_residency us`, `sub_lex fan_in/fan_out` |
| [`polykit_regional_eu.fl`](../circuits/fl/polykit_regional_eu.fl) | EU sovereignty, GDPR, EU AI Act compliance | `sovereignty eu`, `data_residency eu_only`, `field_governance` tiers |
| [`polykit_platform.fl`](../circuits/fl/polykit_platform.fl) | Platform composition connecting all circuits | `import`, `group`, `connect`, `bind` |

App-level circuits in `circuits/fl/apps/` (adapted from polylabs examples with PolyKit profiles):

| File | What It Does | Key Features |
|------|-------------|--------------|
| [`polykit_media_stream.fl`](../circuits/fl/apps/polykit_media_stream.fl) | PQ-encrypted voice/video SFU | `blind relay`, `constant_time`, `mlkem_encaps`, `aes_gcm_encrypt` |
| [`polykit_crdt_sync.fl`](../circuits/fl/apps/polykit_crdt_sync.fl) | Offline-capable CRDT merge | `offline mode crdt`, `sync eventual`, `crdt_merge` |
| [`polykit_blind_relay.fl`](../circuits/fl/apps/polykit_blind_relay.fl) | Privacy-preserving message relay with cover traffic | `blind_route`, `pad_to_size`, `generate_cover` |
| [`polykit_classified_fusion.fl`](../circuits/fl/apps/polykit_classified_fusion.fl) | SCI omniscient fusion with 4-tier fan_in | `lex` hierarchy, `li_classify`, `streamsight_anomaly` |

These circuits are the **single source of truth** — FastLang generates the Rust, which compiles to the WASM that PolyKit ships. StreamSight telemetry is inline on every circuit via `observe`/`monitor` annotations (no separate telemetry circuit).

Additionally, 4 platform-level FastLang circuits exist in the eStream SDK at `estream-fastlang/examples/polylabs/`:

| File | What It Does | Key Features |
|------|-------------|--------------|
| [`media_stream.fl`](https://github.com/polyquantum/estream-io/blob/main/crates/estream-fastlang/examples/polylabs/media_stream.fl) | Voice setup and video SFU with blind relay | `blind relay`, `constant_time`, `pad_to_size`, `aes_gcm_encrypt` |
| [`crdt_sync.fl`](https://github.com/polyquantum/estream-io/blob/main/crates/estream-fastlang/examples/polylabs/crdt_sync.fl) | Offline-capable CRDT merge with eventual sync | `offline mode crdt`, `sync eventual`, `wasm_abi`, `crdt_merge` |
| [`classified_fusion.fl`](https://github.com/polyquantum/estream-io/blob/main/crates/estream-fastlang/examples/polylabs/classified_fusion.fl) | SCI omniscient fusion with 4-tier fan_in | `lex` hierarchy, `hardware_required`, `li_classify`, `critical_path` |
| [`blind_relay.fl`](https://github.com/polyquantum/estream-io/blob/main/crates/estream-fastlang/examples/polylabs/blind_relay.fl) | Privacy-preserving message routing with cover traffic | `blind_route`, `pad_to_size`, `generate_cover`, `delay_until` |

---

## Regional Lex Architecture

PolyKit uses a multi-level lex hierarchy with regional sub-lexes for data sovereignty:

```
esn/global/org/polylabs                          <- Global org lex (aggregates)
├── esn/region/us/org/polylabs                   <- US regional (CCPA, SOC2)
│   └── sub_lex app fan_in → fan_out global      <- Only compliance_status + metrics fan up
├── esn/region/eu/org/polylabs                   <- EU regional (GDPR, EU AI Act)
│   └── sub_lex app fan_in → fan_out global      <- Only anonymous + compliance_status
└── esn/global/org/polylabs/session              <- Per-session (media, relay, CRDT)
```

- **US regional**: Enforces CCPA data subject requests, SOC2, COPPA. Raw PII stays in the US lex.
- **EU regional**: Enforces GDPR Art. 25 data protection by design. Four field governance tiers (SPECIAL_CATEGORY, PERSONAL, PSEUDONYMIZED, ANONYMOUS). Raw data never leaves the EU lex.
- **Global**: Receives only anonymized metrics and compliance status via `fan_out` with `share`/`redact`.
- **LI** stands for **Learned Intelligence** -- the `li_classify`, `li_embed`, `li_infer`, `li_train` builtins.

---

## PolyKit SDK Stack

| Layer | Component | What It Does |
|-------|-----------|--------------|
| **FastLang** | `circuits/fl/*.fl` | 8 circuits + 1 platform composition — the golden source |
| **Rust/WASM** | `polykit-core` | Thin kernel: AppContext, format helpers, error types (≤16 KB) |
| **Rust/WASM** | `polykit-eslite` | ESLite migrations, queries, sync (≤32 KB) |
| **Rust/WASM** | `polykit-console` | Widget data pipeline, event bus, RBAC, demo mode (≤32 KB) |
| **Rust/WASM** | `polykit-wasm` | wasm-bindgen shim over codegen'd circuit exports |
| **TypeScript** | `@polykit/react` | Thin DOM binding — `PolyProvider`, `useWasmSubscription`, `useWasmEmit`, `WidgetShell` |

**Key principle**: Push everything into Rust/WASM. TypeScript is ONLY a DOM binding layer. FastLang circuits define all computation; crates contain only runtime plumbing.

---

## Go-Fast Tips

1. **Never write crypto in TypeScript** — `polykit_identity.fl` handles all PQ crypto (ML-DSA-87, ML-KEM-1024, HKDF-SHA3-256) with `constant_time true`. The WASM boundary is sub-millisecond.

2. **Use annotation profiles** — Every circuit applies `profile poly_framework_standard` or `profile poly_framework_sensitive`. This gives you lex scoping, budgets, meters, StreamSight, RBAC, offline support, and WASM host imports in one line.

3. **StreamSight is inline** — Add `observe metrics: [...]` and `monitor "name" { expr }` directly on your circuit. No separate telemetry circuit needed. For anomaly detection, call `streamsight_anomaly()` and `streamsight_baseline()` in the circuit body.

4. **ESLite is your local database** — `polykit-eslite` gives you SQL-queryable, sync-capable local storage. Don't use IndexedDB directly. Use `wasm_abi [eslite_query, eslite_insert]` in your circuit annotations.

5. **CRDT sync = offline-first for free** — `crdt_sync.fl` implements mathematically proven conflict-free merge. Every Poly Labs app gets offline capability through this shared circuit.

6. **WidgetShell handles RBAC gating** — Wrap your UI in `<WidgetShell requiredRole="editor">`. PolyKit checks the WASM-side RBAC against the user's SPARK identity. The `rbac` annotation in your profile auto-generates the gate.

7. **Respect the size budget** — PolyKit targets ≤128 KB per circuit WASM, ≤512 KB total, ≤4 MB linear memory. The `budget` annotation in the profile enforces this at build time.

8. **Wire protocol only — no REST** — All Poly Labs apps use eStream's native QUIC/UDP wire protocol (per estream-io #551). Declare `stream` at file level and use `emit` in circuit bodies or `transaction` blocks.

---

## Build Pipeline

```bash
# Single command: compile .fl, generate Rust/WASM, sign, package
estream-dev build-wasm-client --from-fl circuits/fl/ --sign key.pem --enforce-budget

# Or step-by-step:

# 1. Compile FastLang to ESCIR
estream codegen compile circuits/fl/polykit_identity.fl
estream codegen compile circuits/fl/polykit_metering.fl
# ... (all .fl files)

# 2. Build WASM from codegen'd Rust
cargo build --target wasm32-unknown-unknown --release

# 3. Optimize
wasm-opt -Oz target/wasm32-unknown-unknown/release/polykit_wasm.wasm -o polykit.wasm

# 4. Package as .escd
estream-dev package-escd

# 5. Build the thin TypeScript layer
cd packages/react && npm run build
```

The pipeline: `.fl` → ESCIR → Rust codegen → `cargo build --target wasm32-unknown-unknown` → `wasm-opt -Oz` → ABI validation → ML-DSA-87 signing → `.escd` → TypeScript `.d.ts`.

---

## Testing Locally

### 1. Clone the repos

```bash
git clone https://github.com/polyquantum/estream-io.git
git clone https://github.com/polylabs-dev/polykit.git
```

### 2. Run the FastLang golden tests

```bash
cd estream-io

# Test PolyKit circuits
cargo test -p estream-fastlang -- polykit

# Test platform-level Poly Labs circuits
cargo test -p estream-fastlang -- polylabs
```

### 3. Build PolyKit WASM locally

```bash
cd ../polykit

# Build via FastLang pipeline
estream-dev build-wasm-client --from-fl circuits/fl/ --enforce-budget

# Or manually:
cargo build --target wasm32-unknown-unknown --release
wasm-opt -Oz target/wasm32-unknown-unknown/release/polykit_wasm.wasm -o polykit.wasm
```

### 4. Start a local devnet

```bash
cd ../estream-io
cargo build --release --bin estream --bin ws-edge

estream localnet start --nodes 3 --with-console
```

### 5. Deploy PolyKit circuits

```bash
# Compile and submit each circuit
estream lex compile ../polykit/circuits/fl/polykit_identity.fl
estream lex submit polykit_identity --lex esn/global/org/polylabs

estream lex compile ../polykit/circuits/fl/polykit_metering.fl
estream lex submit polykit_metering --lex esn/global/org/polylabs
```

### 6. Test from a Poly Labs app

```bash
# Emit a metering event
estream stream emit metering_events '{"user_id":"0x01","operation":"upload","dimensions":{}}' \
  --lex esn/global/org/polylabs

# Watch classification events
estream stream subscribe classification_events --lex esn/global/org/polylabs --follow
```

### 7. Formal verification

```bash
# Prove identity key isolation invariant
estream codegen smt circuits/fl/polykit_identity.fl -o identity.smt2
z3 identity.smt2
# UNSAT = key isolation invariant holds

# Prove sanitization completeness
estream codegen smt circuits/fl/polykit_sanitize.fl -o sanitize.smt2
z3 sanitize.smt2
# UNSAT = no PII leaks to output
```

### 8. Docker smoke test

```bash
docker compose -f docker/smoke-test/docker-compose.yml up --abort-on-container-exit
```

---

## Alpha-Devnet

The eStream alpha-devnet is coming online (or may already be live) at:

- **Edge**: `wss://edge-alpha-devnet.estream.dev`
- **Console**: `https://console.estream.dev`

To deploy PolyKit circuits:

```bash
estream-dev build-wasm-client --from-fl circuits/fl/ --sign $POLYLABS_KEY --enforce-budget

estream-dev deploy-escd polykit.escd --target alpha-devnet --signing-key $POLYLABS_KEY
```

---

## Migrating from v0.8.1

PolyKit v0.1.0 referenced eStream v0.8.1 with ESCIR YAML circuits. The v0.2.0 refactor migrates to FastLang-native:

| What Changed | Action Required |
|-------------|-----------------|
| **Single version model** | All eStream crates now 0.8.3. Update `estream-kernel` pins in `polykit/Cargo.toml` |
| **FastLang is canonical** | Circuit definitions migrated from `.escir.yaml` to `.fl` source in `circuits/fl/` |
| **YAML circuits archived** | Old `.escir.yaml` files moved to `circuits/legacy/` for reference |
| **Telemetry is inline** | Separate `polykit-telemetry` circuit eliminated. StreamSight via `observe`/`monitor` on every circuit |
| **Annotation profiles** | Shared `poly_framework_standard`/`poly_framework_sensitive` profiles replace repeated annotations |
| **New circuits** | `polykit_delta_curate.fl` (delta encoding) and `polykit_governance.fl` (field governance) added |
| **Platform composition** | `polykit_platform.fl` connects all circuits with `import`, `group`, `connect`, `bind` |
| **Crates slimmed** | `polykit-core` reduced to thin kernel; identity/crypto/metering logic now in `.fl` codegen |
| **WASM ABI annotation** | `wasm_abi` in FastLang generates typed host imports — defined in profile |
| **Field governance** | `field_governance` blocks control per-field visibility per audience |
| **Filtered fan-out** | Sub-lex `fan_out` with `share`/`redact` for tiered dissemination |

No breaking changes to wire protocol, `.escd` format, or WASM ABI contract.

---

## Documentation Links

| Document | Where |
|----------|-------|
| [PolyKit Architecture](./ARCHITECTURE.md) | Architecture, build pipeline, crate breakdown |
| [FastLang Refactor Plan](./FASTLANG_REFACTOR_PLAN.md) | Full refactor design with circuit code |
| [eStream Feedback](./ESTREAM_FEEDBACK.md) | PolyKit team's DX feedback to eStream core |
| [FastLang Quickstart](https://github.com/polyquantum/estream-io/blob/main/docs/guides/FASTLANG_QUICKSTART.md) | Zero to compiled circuit in 15 minutes |
| [App Developer Guide](https://github.com/polyquantum/estream-io/blob/main/docs/guides/FASTLANG_APP_GUIDE.md) | Building app circuits (Poly Labs examples included) |
| [Codegen Targets](https://github.com/polyquantum/estream-io/blob/main/docs/guides/CODEGEN_TARGETS.md) | When to use Rust vs WASM for browser/mobile |
| [Poly Labs Examples README](https://github.com/polyquantum/estream-io/blob/main/crates/estream-fastlang/examples/polylabs/README.md) | Catalog of platform-level .fl files |
| [Security Tier Selection](https://github.com/polyquantum/estream-io/blob/main/docs/guides/security-tier-selection.md) | Choosing classification tiers (PUBLIC → SOVEREIGN) |
| [WASM Client Spec (issue #550)](https://github.com/polyquantum/estream-io/issues/550) | ESCIR WASM client build pipeline specification |
| [Wire Protocol Only (issue #551)](https://github.com/polyquantum/estream-io/issues/551) | Why Poly Labs apps use wire protocol, not REST |
| [Refactor Epic](../.github/epics/EPIC_POLYKIT_FASTLANG_REFACTOR.md) | Tracking epic for this refactor |
