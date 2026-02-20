# PolyKit Architecture

**Version**: 0.3.0
**Date**: February 2026
**Platform**: eStream v0.8.3
**Build Pipeline**: FastLang (.fl) → ESCIR → Rust/WASM codegen → .escd

---

## Design Principles

### 1. Push Everything into Rust/WASM

All crypto, state management, data transforms, wire protocol framing, event processing, RBAC checks, sanitization, and metering run in WASM. TypeScript exists only to mount React components to the DOM and call WASM exports. FastLang `.fl` files are the single source of truth for circuit logic -- the codegen pipeline produces Rust, WASM, and TypeScript type bindings from `.fl` source.

### 2. Zero-Linkage Privacy Isolation

Every Poly product is cryptographically and observationally isolated from every other product. A subpoena for one product cannot reveal anything about another -- not even that the user has other products.

- **SPARK derivation isolation**: Each product uses its own HKDF context (`poly-data-v1`, `poly-messenger-v1`, etc.), producing independent key pairs. No cross-product key correlation is possible.
- **Per-product identity**: Each product derives its own `user_id = SHA3-256(spark_ml_dsa_87_public_key)[0..16]` from the product-specific signing key. The same human has different, unlinkable `user_id` values in each product.
- **StreamSight isolation**: Telemetry stays within per-product lex namespaces. No cross-product aggregation of identifiable data.
- **Metering isolation**: Each product meters independently under its own `user_id`. PolyKit provides the metering _circuit_ but each app instantiates its own metering _graph_ in its own lex.
- **Billing isolation**: Subscription system uses blinded payment tokens. The billing backend cannot correlate which SPARK identity uses which products.

**Enterprise exception**: Enterprise admins can opt-in to cross-product visibility via an explicit lex bridge, gated by k-of-n admin witness attestation. The bridge is revocable. Even bridged, only org-level aggregates and RBAC policy flow across products -- individual user-level data is not cross-linked.

### 3. Compose Upstream, Don't Reinvent

PolyKit composes eStream platform graphs rather than duplicating them. The RBAC model, org hierarchy, and lifecycle state machines come from eStream's production `.fl` files.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│  Browser / React Native                                          │
│                                                                  │
│  @polykit/react (thin TS)                                        │
│  ├── PolyProvider          loads polykit.wasm, inits SPARK       │
│  ├── useWasmSubscription   subscribes via WASM wire client       │
│  ├── useWasmEmit           emits via WASM wire client            │
│  └── WidgetShell           layout + RBAC gate                    │
│                                                                  │
│  TS does: DOM mount, canvas render, browser events               │
│  TS does NOT: crypto, state, data transforms, wire framing       │
├──────────────────────────────────────────────────────────────────┤
│  polykit.wasm (FastLang → ESCIR → Rust/WASM codegen)            │
│                                                                  │
│  circuits/fl/*.fl      10 circuits + 3 graph circuits            │
│  circuits/fl/apps/*.fl 4 app-level circuits                      │
│  polykit-core          thin kernel (AppContext, helpers)          │
│  polykit-eslite        migrations, queries, sync                 │
│  polykit-console       event bus, widget data, demo, RBAC        │
│  polykit-graph         graph/DAG runtime helpers for WASM        │
├──────────────────────────────────────────────────────────────────┤
│  eStream Wire Protocol (UDP :5000 / WebTransport :4433)          │
└──────────────────────────────────────────────────────────────────┘
```

## Build Pipeline

FastLang-native build (v0.8.3):

1. FastLang circuits (`circuits/fl/*.fl`) → parse, type-check, lower to ESCIR
2. ESCIR optimization passes (O2)
3. Rust codegen (auto-generated from ESCIR)
4. `cargo build --target wasm32-unknown-unknown` (LTO, opt-level=z)
5. `wasm-opt -Oz` (dead code elimination)
6. ABI validation (required: `evaluate`; optional: `alloc`, `dealloc`, `circuit_name`, `circuit_version`)
7. Size budget check (≤128 KB/circuit, ≤512 KB total, ≤4 MB linear memory)
8. ML-DSA-87 signing → `.wasm.sig`
9. `.escd` packaging (manifest.json + .wasm + .wasm.sig)
10. TypeScript `.d.ts` generation

Single command:

```bash
estream-dev build-wasm-client --from-fl circuits/fl/ --sign key.pem --enforce-budget
```

---

## eStream Upstream Compositions

PolyKit composes these eStream platform graphs directly rather than building its own. Each app that uses PolyKit gets these capabilities through circuit composition.

### RBAC (`estream/circuits/graphs/rbac.fl`)

Provides the `graph rbac` with `RoleNode`, `PermissionSet`, `RoleAssignment`, `InheritanceEdge`. Key circuits:

- `check_permission(rbac_graph, principal, scope_id, permission_bit) -> bool` — single-clock permission check that walks the inheritance chain
- `resolve_permissions(rbac_graph, principal, scope_id) -> u64` — effective permission bitmask (inherited + direct)
- `assign_role` / `revoke_role` — role lifecycle with PoVC attestation
- `propagate_permissions` — single-clock propagation through hierarchy
- `expire_stale` — automatic expiry of time-bounded assignments
- `audit_principal` — full assignment history for compliance

PolyKit wraps `check_permission()` and `resolve_permissions()` into the `poly_framework_standard` profile so every app circuit automatically resolves permissions against the upstream RBAC graph. Per-product RBAC graphs are independent (zero-linkage); enterprise lex bridges can optionally connect them.

### Group Hierarchy (`estream/circuits/graphs/group_hierarchy.fl`)

Provides `graph group_hierarchy` with `OrgNode`, `GroupNode`, `RepoNode`, `ContainmentEdge`. Used by the `polylabs` central monorepo for enterprise org structure. PolyKit apps do not directly compose this -- it is consumed at the `polylabs` backend level.

### Lifecycle State Machines (`estream/circuits/graphs/issue_tracking.fl`)

The `state_machine` pattern with `guard`, `li_anomaly_detection`, `persistence wal`, and terminal states is reused across Poly apps for any entity lifecycle (file upload states, message delivery, subscription changes). PolyKit provides a `state_machine` annotation profile that apps can extend.

---

## FastLang Circuits

### Shared Circuits (`circuits/fl/`)

| Circuit | File | Purpose | Key Features |
|---------|------|---------|-------------|
| Identity | `polykit_identity.fl` | SPARK auth, HKDF, ML-DSA-87/ML-KEM-1024 | `constant_time`, `kat_vector`, `invariant`, mutates `user_graph` |
| Metering | `polykit_metering.fl` | 8-dimension resource metering (E/H/B/S/O/P/C/M) | `stream` + `emit`, `parallel for`, `meters` |
| Rate Limiter | `polykit_rate_limiter.fl` | FIFO rate limiter with backpressure | `state_machine`, `streamsight_anomaly()` |
| Sanitize | `polykit_sanitize.fl` | 3-stage PII/PCI/HIPAA/GDPR compliance | `@sanitize`, `li_classify`, `witness full` |
| LI Effects | `polykit_li_effects.fl` | LI Effects content classification | `feedback` streams, `li_embed`/`li_infer` |
| Delta Curate | `polykit_delta_curate.fl` | Field-level delta encoding with bitmask proofs | `delta_curate`, lossless ~20x compression |
| Governance | `polykit_governance.fl` | RBAC gate composing eStream `rbac.fl` | Wraps `check_permission`, `resolve_permissions` |
| Regional US | `polykit_regional_us.fl` | US sovereignty, CCPA, SOC2 compliance | `sovereignty us`, `data_residency us` |
| Regional EU | `polykit_regional_eu.fl` | EU sovereignty, GDPR, EU AI Act compliance | `sovereignty eu`, `data_residency eu_only` |
| Platform | `polykit_platform.fl` | Composition connecting all circuits + graphs | `import`, `group`, `connect`, `bind` |

### Graph Circuits (`circuits/fl/graphs/`)

| Graph | File | Type | Purpose |
|-------|------|------|---------|
| User Graph | `polykit_user_graph.fl` | `graph` | SPARK identity + device mesh per product |
| Metering Graph | `polykit_metering_graph.fl` | `graph` | Per-app 8D usage tracking as relational model |
| Subscription Lifecycle | `polykit_subscription_lifecycle.fl` | `state_machine` | Tier changes, trials, cancellations |

### App-Level Circuits (`circuits/fl/apps/`)

| Circuit | File | Purpose | Key Features |
|---------|------|---------|-------------|
| Media Stream | `polykit_media_stream.fl` | PQ-encrypted voice/video SFU | `blind relay`, `constant_time`, `mlkem_encaps`, `aes_gcm_encrypt` |
| CRDT Sync | `polykit_crdt_sync.fl` | Offline-capable CRDT merge | `offline mode crdt`, `sync eventual`, `crdt_merge` |
| Blind Relay | `polykit_blind_relay.fl` | Privacy-preserving message relay with cover traffic | `blind_route`, `pad_to_size`, `generate_cover` |
| Classified Fusion | `polykit_classified_fusion.fl` | SCI omniscient fusion with 4-tier fan_in | `lex` hierarchy, `li_classify`, `streamsight_anomaly` |

StreamSight telemetry is inline on every circuit via `observe metrics`, `monitor`, and `streamsight_anomaly()`/`streamsight_baseline()` builtins. There is no separate telemetry circuit.

---

## Graph/DAG Constructs

### User Graph (`polykit_user_graph.fl`)

Per-product identity and device mesh. Each Poly product instantiates its own `user_graph` with its own HKDF-derived user IDs (zero-linkage).

```fastlang
type UserNode = struct {
    user_id: bytes(16),
    signing_pubkey: bytes(2592),
    encryption_pubkey: bytes(1568),
    created_at: u64,
}

type DeviceNode = struct {
    device_id: bytes(16),
    platform: u8,
    etfa_fingerprint: bytes(32),
    security_tier: u8,
    registered_at: u64,
}

type GuardianNode = struct {
    guardian_id: bytes(16),
    threshold_k: u8,
    threshold_n: u8,
    activated_at: u64,
}

type OwnsDeviceEdge = struct {
    registered_at: u64,
    last_seen_ns: u64,
}

type TrustsUserEdge = struct {
    trust_level: u8,
    verified_at: u64,
}

type GuardianEdge = struct {
    share_index: u8,
    granted_at: u64,
}

graph user_graph {
    node UserNode
    node DeviceNode
    node GuardianNode
    edge OwnsDeviceEdge
    edge TrustsUserEdge
    edge GuardianEdge

    overlay security_tier: u8 curate delta_curate
    overlay last_active_ns: u64 bitmask delta_curate
    overlay trust_score: u32 bitmask delta_curate
    overlay device_health: u8 curate delta_curate
    overlay guardian_status: u8 curate

    storage csr {
        hot @bram,
        warm @ddr,
        cold @nvme,
    }

    ai_feed identity_anomaly

    observe user_graph: [security_tier, last_active_ns, device_health] threshold: {
        anomaly_score 0.9
        baseline_window 300
    }
}

series identity_series: user_graph
    merkle_chain true
    lattice_imprint true
    witness_attest true
```

Key circuits: `register_user`, `register_device`, `add_guardian`, `verify_trust`, `revoke_device`, `guardian_recovery`.

### Metering Graph (`polykit_metering_graph.fl`)

Per-app isolated metering. Each product instantiates its own `metering_graph` under its own lex namespace. No cross-app linkage.

```fastlang
type AppResourceNode = struct {
    resource_id: bytes(16),
    resource_type: u8,
    name: string,
}

type UserMeterNode = struct {
    user_id: bytes(16),
    billing_period_start: u64,
    tier: u8,
}

type ConsumptionEdge = struct {
    energy: u64,
    hardware: u64,
    bandwidth: u64,
    storage: u64,
    operations: u64,
    priority: u64,
    creation: u64,
    memory: u64,
    timestamp: u64,
}

graph metering_graph {
    node AppResourceNode
    node UserMeterNode
    edge ConsumptionEdge

    overlay energy_total: u64 bitmask delta_curate
    overlay bandwidth_total: u64 bitmask delta_curate
    overlay storage_total: u64 bitmask delta_curate
    overlay operations_total: u64 bitmask delta_curate
    overlay tier_limit_pct: u32 bitmask delta_curate
    overlay billing_period_usage: u64 bitmask delta_curate

    storage csr {
        hot @bram,
        warm @ddr,
        cold @nvme,
    }

    observe metering_graph: [tier_limit_pct, billing_period_usage] threshold: {
        anomaly_score 0.8
        baseline_window 60
    }
}

series metering_series: metering_graph
    merkle_chain true
    lattice_imprint true
    witness_attest true
```

Key circuits: `record_usage`, `check_tier_limit`, `aggregate_billing_period`, `reset_billing_period`.

### Subscription Lifecycle (`polykit_subscription_lifecycle.fl`)

Reusable state machine for tier changes, modeled after `issue_tracking.fl` lifecycle pattern.

```fastlang
state_machine subscription_lifecycle {
    initial FREE
    persistence wal
    terminal [CANCELLED]
    li_anomaly_detection true

    FREE -> TRIAL when trial_started guard valid_payment_method
    FREE -> PREMIUM when upgrade guard payment_confirmed
    TRIAL -> PREMIUM when trial_converted guard payment_confirmed
    TRIAL -> FREE when trial_expired
    PREMIUM -> PRO when upgrade guard payment_confirmed
    PREMIUM -> FREE when downgrade guard billing_period_end
    PRO -> ENTERPRISE when upgrade guard admin_approved
    PRO -> PREMIUM when downgrade guard billing_period_end
    ENTERPRISE -> PRO when downgrade guard admin_approved
    PREMIUM -> CANCELLED when cancel guard billing_period_end
    PRO -> CANCELLED when cancel guard billing_period_end
    ENTERPRISE -> CANCELLED when cancel guard admin_approved
}
```

---

## Shared Annotation Profiles

Defined in `circuits/fl/polykit_profile.fl`:

- **`poly_framework_standard`** — base profile with lex, precision, budget, meters, observe, streamsight, offline, wasm_abi. Composes eStream `check_permission()` from `rbac.fl` for RBAC gating.
- **`poly_framework_sensitive`** — inherits standard, adds `constant_time`, `sanitize`, `witness threshold(3,5)`

Every circuit applies one of these profiles, eliminating 5-7 repeated annotations per circuit.

---

## Lex Hierarchy

```
esn/global/org/polylabs                          <- Global org lex (aggregates)
├── esn/region/us/org/polylabs                   <- US regional lex (CCPA, SOC2)
│   ├── sub_lex app fan_in                       <- Per-app sub-lex
│   └── sub_lex global fan_out                   <- Only compliance_status, metrics fan up
├── esn/region/eu/org/polylabs                   <- EU regional lex (GDPR, EU AI Act)
│   ├── sub_lex app fan_in
│   ├── sub_lex personal/pseudonymized/anonymous fan_out
│   └── sub_lex global fan_out                   <- Only anonymous + compliance_status
└── esn/global/org/polylabs/session              <- Per-session sub-lex (media, relay, CRDT)
```

**Zero-linkage enforcement**: Each product's lex subtree (`esn/.../polylabs/data`, `esn/.../polylabs/messenger`, etc.) is completely isolated. The global org lex receives only anonymized compliance metrics -- never user IDs, never product-usage correlation. Raw PII stays within regional sub-lexes.

**Enterprise bridge**: When an enterprise admin opts in, a `lex_bridge` is established between product sub-lexes, gated by k-of-n admin witness attestation. The bridge allows org-level aggregates and RBAC policy to flow across products. Individual user-level data remains per-product.

---

## Crates

| Crate | Purpose | Size Target |
|-------|---------|-------------|
| polykit-core | Thin kernel: AppContext, format helpers, error types | ≤16 KB |
| polykit-eslite | Migration runner, schema DSL, query engine, sync | ≤32 KB |
| polykit-console | Event bus, widget data pipeline, demo fixtures, RBAC | ≤32 KB |
| polykit-graph | Graph/DAG runtime helpers: CSR operations, overlay reads, series queries | ≤32 KB |
| polykit-wasm | wasm-bindgen shim over codegen'd circuit exports | Overhead only |

Most computation logic lives in `.fl` circuits; crates contain only runtime plumbing that can't be expressed as circuits.

---

## How Apps Use PolyKit

Apps compose PolyKit `.fl` circuits and extend with domain-specific circuits and graphs:

```fastlang
circuit polydata_upload(user_id: bytes(16), file: bytes) -> bytes(32)
    profile poly_framework_standard
    composes: [polykit_identity, polykit_metering, polykit_sanitize]
    lex esn/global/org/polylabs/data
    observe metrics: [uploads, file_size_avg]
{
    let sanitized = sanitize(file)
    let metered = record_usage(user_id, "upload", dims)
    sha3_256(sanitized)
}
```

Each app instantiates its own `user_graph` and `metering_graph` in its own lex namespace:

```fastlang
// In polydata — isolated identity
graph polydata_users: user_graph {
    lex esn/global/org/polylabs/data/identity
}

// In polymessenger — separate, unlinkable identity
graph polymessenger_users: user_graph {
    lex esn/global/org/polylabs/messenger/identity
}
```

The same human has different `user_id` values in each product. No circuit, stream, or query can correlate them.

```tsx
import { PolyProvider } from '@polykit/react';

export default function App() {
  return (
    <PolyProvider wasm="/pkg/polydata.wasm" hkdfContext="poly-data-v1">
      <WidgetGrid />
    </PolyProvider>
  );
}
```

---

## Zero-Linkage Implementation

### Per-Product Isolation Boundaries

| Boundary | Mechanism | Enforcement |
|----------|-----------|-------------|
| Identity | Separate HKDF context per product | SPARK derivation in WASM; different `user_id` per product |
| Keys | Independent ML-DSA-87 + ML-KEM-1024 per product | HKDF produces orthogonal key material |
| StreamSight | Per-product lex namespace | Lex governance; no cross-product telemetry aggregation |
| Metering | Per-product `metering_graph` instance | Separate graph, separate lex, separate `user_id` |
| Billing | Blinded payment tokens | Payment backend sees token, not SPARK identity or product |
| RBAC | Per-product `rbac` graph instance | Separate RBAC graph per product; enterprise bridge is opt-in |
| ESLite | Per-product local database namespace | `/polydata/*`, `/polymessenger/*` — no shared tables |

### Enterprise Lex Bridge

When an enterprise admin opts in to cross-product visibility:

```fastlang
lex_bridge polylabs_enterprise {
    source esn/global/org/polylabs/data
    source esn/global/org/polylabs/messenger
    target esn/global/org/polylabs/admin

    witness_attest true
    witness_k 3
    witness_n 5

    allowed_fields [org_id, seat_count, storage_aggregate, compliance_status]
    denied_fields [user_id, file_id, message_id, content]

    revocable true
    audit_stream esn/global/org/polylabs/admin/bridge_audit
}
```

The bridge is:
- Gated by k-of-n admin witness attestation (3-of-5 admin keys must sign)
- Limited to org-level aggregate fields only
- Explicitly denies user-level identifiers and content
- Revocable at any time
- Audited via its own tamper-proof series

---

## Related Documents

| Document | Purpose |
|----------|---------|
| [FASTLANG_REFACTOR_PLAN.md](FASTLANG_REFACTOR_PLAN.md) | Full refactor design with circuit code |
| [ESTREAM_FEEDBACK.md](ESTREAM_FEEDBACK.md) | Running feedback on eStream platform |
| [ESTREAM_GETTING_STARTED.md](ESTREAM_GETTING_STARTED.md) | Getting started with eStream + PolyKit |
| [FastLang Spec](https://github.com/polyquantum/estream/blob/main/specs/protocol/FASTLANG_SPEC.md) | Canonical FastLang language specification |
| [Graph Spec](https://github.com/polyquantum/estream/blob/main/specs/protocol/GRAPH_SPEC.md) | Graph/DAG construct specification |
| [ESCIR WASM Client Spec](https://github.com/polyquantum/estream/blob/main/specs/architecture/ESCIR_WASM_CLIENT_SPEC.md) | Full WASM client specification |
| [rbac.fl](https://github.com/polyquantum/estream/blob/main/circuits/graphs/rbac.fl) | Upstream RBAC graph (composed by PolyKit) |
| [group_hierarchy.fl](https://github.com/polyquantum/estream/blob/main/circuits/graphs/group_hierarchy.fl) | Upstream org hierarchy graph |
| [Epic](../.github/epics/EPIC_POLYKIT_FASTLANG_REFACTOR.md) | Tracking epic for this refactor |
