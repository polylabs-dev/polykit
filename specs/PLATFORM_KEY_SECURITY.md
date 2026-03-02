# Platform Key Security Specification

Applies to: sdk-backend implementations (TakeTitle, TrueResolve, Thermogen Zero, etc.)

Security Level: NIST Level 5 (ML-DSA-87, ML-KEM-1024)

---

## Executive Summary

Platform backends that send messages to users via Poly Messenger must never hold complete signing keys. This specification defines the **threshold signing** and **blind relay** patterns that all sdk-backend implementations must follow.

---

## The Problem

Without TEE (Trusted Execution Environment), any key held on a server can be:
- Extracted via memory dumps
- Compromised through hypervisor attacks
- Stolen by malicious insiders

**TEE is not a solution**: AMD SEV vulnerabilities (CVE-2023-31315 "Sinkclose") proved that hardware isolation cannot be trusted.

**Implication**: All security must come from cryptographic guarantees, not hardware.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Platform Backend (TakeTitle, etc.)                   │
│                                                                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │   Service A     │  │   Service B     │  │   Service C     │         │
│  │   (Share 1)     │  │   (Share 2)     │  │   (Share 3)     │         │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘         │
│           │                    │                    │                   │
│           └──────────┬─────────┴──────────┬─────────┘                   │
│                      │                    │                              │
│                      ▼                    ▼                              │
│           ┌─────────────────────────────────────────┐                   │
│           │        Threshold Signing (2-of-3)       │                   │
│           │                                         │                   │
│           │  • No single service has complete key   │                   │
│           │  • Requires coordination to sign        │                   │
│           │  • Audit log of all signatures          │                   │
│           └─────────────────────────────────────────┘                   │
│                              │                                           │
│                              ▼                                           │
│           ┌─────────────────────────────────────────┐                   │
│           │     Message (encrypted to user)         │                   │
│           │                                         │                   │
│           │  • Platform cannot decrypt              │                   │
│           │  • Signed with threshold signature      │                   │
│           │  • User's group_public_key only         │                   │
│           └─────────────────────────────────────────┘                   │
│                              │                                           │
└──────────────────────────────┼───────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         Poly Edge (Blind Relay)                          │
│                                                                          │
│  Sees:                          Cannot:                                  │
│  • group_id                     • Decrypt payload                        │
│  • encrypted_blob               • Identify user                          │
│  • routing_metadata             • Modify content                         │
│                                 • Link to user identity                  │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
                        ┌──────────────┐
                        │ User Device  │
                        │              │
                        │ Only place   │
                        │ with private │
                        │ key          │
                        └──────────────┘
```

---

## Component Security Models

### 1. Console / Mobile App (Client-Only)

**Security Model**: All crypto happens on-device in WASM.

| Property | Implementation |
|----------|----------------|
| Key Storage | Device keychain / secure enclave |
| Crypto | poly-core-wasm (WASM, sandboxed) |
| Auth | Spark visual authentication |
| Transport | WebTransport to edge (E2E encrypted) |

**Keys never leave the device.**

```typescript
// Console pattern
const keys = await PolyCore.generateDeviceKeys();
// keys.privateKey stays in WASM memory, never exposed to JS
// keys.publicKey can be shared
```

---

### 2. sdk-backend (Threshold Signing)

**Security Model**: Platform signing key split across multiple services.

#### Threshold Configuration

| Deployment | Threshold | Services |
|------------|-----------|----------|
| Minimum | 2-of-3 | 3 independent services |
| Standard | 3-of-5 | 5 services (recommended) |
| High Security | 5-of-7 | 7 services (critical systems) |

#### Key Generation (One-Time Setup)

```rust
use estream_kernel::crypto::threshold::{ThresholdMlDsa, KeyShare};

// Generate threshold keypair
let (public_key, shares) = ThresholdMlDsa::keygen(
    threshold: 2,  // k signatures required
    total: 3,      // n total shares
)?;

// Distribute shares to services
service_a.store_share(shares[0].clone())?;
service_b.store_share(shares[1].clone())?;
service_c.store_share(shares[2].clone())?;

// Only public_key is stored centrally
platform.store_public_key(public_key)?;
```

#### Signing Protocol

```rust
// Service A receives signing request
let partial_a = service_a.partial_sign(&message)?;

// Service B receives signing request
let partial_b = service_b.partial_sign(&message)?;

// Combine partials (any service can do this)
let signature = ThresholdMlDsa::combine(&[partial_a, partial_b])?;

// Verify
ThresholdMlDsa::verify(&platform_public_key, &message, &signature)?;
```

#### Audit Requirements

Every signature MUST emit an audit event:

```rust
pub struct SigningAuditEvent {
    pub timestamp: u64,
    pub message_hash: [u8; 32],
    pub participating_services: Vec<ServiceId>,
    pub signature_hash: [u8; 32],
    pub request_origin: String,
}
```

---

### 3. Edge Nodes (Blind Relay)

**Security Model**: Zero knowledge of payload content.

| Edge Sees | Edge Cannot See |
|-----------|-----------------|
| `group_id` | User's Poly identity |
| Encrypted blob | Message content |
| Routing metadata | Sender identity |
| Rate limit counters | Decrypted payload |

```rust
// Edge processing
pub async fn relay_message(envelope: EncryptedEnvelope) -> Result<()> {
    // Extract routing info (outer layer only)
    let group_id = envelope.routing.group_id;
    
    // Rate limit check
    rate_limiter.check(&group_id)?;
    
    // Forward without decryption
    regional_node.forward(envelope).await?;
    
    // Cannot log: message content, user identity, etc.
    metrics.increment("messages_relayed");
    
    Ok(())
}
```

---

## Message Flow

### Platform → User (Notification)

```
1. Platform creates notification
   ├── title: "Investment Confirmed"
   ├── body: "Your $5,000 investment..."
   └── action: "taketitle://investment/inv_123"

2. Platform encrypts to user's group_public_key
   └── Only user can decrypt (platform cannot)

3. Platform signs with threshold signature
   ├── Service A provides partial_sig_a
   ├── Service B provides partial_sig_b
   └── Combined: valid ML-DSA-87 signature

4. Message sent via Poly Edge
   ├── Edge sees: group_id, encrypted_blob
   ├── Edge cannot: decrypt, identify user
   └── Edge routes to regional node

5. User device receives
   ├── Decrypts with group_private_key
   ├── Verifies platform signature
   └── Displays notification
```

### User → Platform (Reply)

```
1. User composes reply in Poly app
   └── "Confirm my investment"

2. User encrypts to platform's public key
   └── Platform group_public_key

3. User signs with device key
   └── ML-DSA-87 signature

4. Message sent via Poly Edge
   └── Same blind relay pattern

5. Platform receives
   ├── Any service can decrypt (threshold not needed for decryption)
   ├── Verifies user's signature
   └── Processes action
```

---

## Key Material Locations

| Key Type | Location | Persistence |
|----------|----------|-------------|
| User device keys | Device keychain | Permanent |
| User group keys | Device keychain | Per-platform |
| Platform public key | Database | Permanent |
| Platform shares | Each service's vault | Ephemeral |
| Edge routing keys | Worker memory | Ephemeral |

### Ephemeral Vault for Shares

Each service uses the eStream Vault pattern:

```rust
use estream_ops::vault::EphemeralVault;

// Service startup
let vault = EphemeralVault::new(&node_secret, Environment::Production);

// Store share (encrypted in memory, never on disk)
vault.store(
    CredentialType::ThresholdShare,
    "platform_signing_share",
    share_bytes,
    Duration::from_secs(86400), // 24h TTL, re-fetch from KMS
)?;

// On shutdown: automatic zeroing
drop(vault); // All secrets wiped
```

---

## Security Properties

### Forward Secrecy
- Each message uses fresh encryption
- Compromise of one message doesn't affect others

### No Single Point of Failure
- k-of-n threshold requires multiple service compromise
- Each share is useless alone

### Blind Relay
- Edge/servers cannot read message content
- Cannot link messages to user identity

### Audit Trail
- All signatures logged with participating services
- Immutable audit log via ESLite

### Compromise Recovery
- Rotate shares without changing public key
- Revoke compromised service's share

---

## Implementation Checklist

### For New sdk-backend Implementations

- [ ] Generate threshold keypair (2-of-3 minimum)
- [ ] Deploy shares to independent services
- [ ] Implement partial signing endpoint
- [ ] Implement signature combination
- [ ] Store only group_public_key for users (from blind connection)
- [ ] Never store user's Poly identity
- [ ] Emit audit events for all signatures
- [ ] Use Ephemeral Vault for share storage
- [ ] Set TTL on shares (re-fetch from KMS)

### For Edge Deployment

- [ ] No decryption capability
- [ ] Rate limiting by group_id only
- [ ] No logging of payload content
- [ ] Ephemeral worker memory only

---

## References

- [PQ_PRIVACY_PATTERNS.md](./crypto/PQ_PRIVACY_PATTERNS.md) - Threshold ML-DSA implementation
- [ESTREAM_VAULT.md](./ESTREAM_VAULT.md) - Ephemeral credential storage
- [EDGE_SLEEVE.md](./EDGE_SLEEVE.md) - Blind relay pattern
- [BLIND_CONNECTION_PATTERN.md](../../polymessenger-app/docs/BLIND_CONNECTION_PATTERN.md) - User/platform linking

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-15 | Initial specification |
