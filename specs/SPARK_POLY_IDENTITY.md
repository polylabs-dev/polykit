# Spark-Derived Poly Identity

> **Problem**: How does a user see Poly messages in the eStream Console without a separate login?  
> **Solution**: Spark authentication produces deterministic keys that work for both eStream and Poly.  
> **See also**: [ESTREAM_WALLET.md](./ESTREAM_WALLET.md) for full wallet spec

---

## The Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         User's Browser                                   │
│                                                                          │
│  1. USER SCANS SPARK                                                    │
│     ┌─────────────┐                                                     │
│     │  ░░░▓▓░░░░  │  ← Visual Spark (unique to user)                   │
│     │  ▓░░░░▓▓░░  │                                                     │
│     │  ░▓▓░░░░▓░  │                                                     │
│     └─────────────┘                                                     │
│           │                                                              │
│           ▼                                                              │
│  2. SPARK → SEED DERIVATION (in WASM)                                   │
│     ┌─────────────────────────────────────────────────────────────────┐ │
│     │  spark_image + user_secret → master_seed                        │ │
│     │                                                                  │ │
│     │  master_seed is NEVER exposed to JavaScript                     │ │
│     │  Lives only in WASM memory, zeroed on close                     │ │
│     └─────────────────────────────────────────────────────────────────┘ │
│           │                                                              │
│           ▼                                                              │
│  3. CONTEXT-SPECIFIC KEY DERIVATION                                     │
│     ┌─────────────────────────────────────────────────────────────────┐ │
│     │                                                                  │ │
│     │  estream_keys = HKDF(master_seed, "estream-console-v1")         │ │
│     │  poly_keys    = HKDF(master_seed, "poly-messenger-v1")          │ │
│     │  taketitle_keys = HKDF(master_seed, "io.taketitle-v1")          │ │
│     │                                                                  │ │
│     │  Same Spark → Same keys across sessions                         │ │
│     │  Different apps → Different derived keys (isolation)            │ │
│     └─────────────────────────────────────────────────────────────────┘ │
│           │                                                              │
│           ├─────────────────────────┬───────────────────────────────────┤
│           ▼                         ▼                                    │
│  ┌─────────────────────┐   ┌─────────────────────┐                      │
│  │  eStream Console    │   │  Poly Messages      │                      │
│  │                     │   │                     │                      │
│  │  • WebTransport auth│   │  • Decrypt messages │                      │
│  │  • Subscribe topics │   │  • Sign outgoing    │                      │
│  │  • View topology    │   │  • ESLite storage   │                      │
│  └─────────────────────┘   └─────────────────────┘                      │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Why This Works

### Same Identity, No Separate Login

```typescript
// User authenticates ONCE with Spark
const spark = await SparkAuth.authenticate({
  image: sparkImage,
  challenge: serverChallenge,
});

// Derive context-specific keys (happens in WASM)
const estreamIdentity = spark.deriveIdentity('estream-console');
const polyIdentity = spark.deriveIdentity('poly-messenger');

// Both are deterministic from same Spark
// User has ONE identity across all eStream apps
```

### Deterministic = Portable

```
If you scan the same Spark:
  → You get the same keys
  → On any device
  → In any browser
  → Across sessions

Your identity IS your Spark.
```

### Context Isolation

```
estream_keys ≠ poly_keys ≠ taketitle_keys

Even though derived from same seed:
  → Can't use eStream key to decrypt Poly messages
  → Can't use Poly key to auth to TakeTitle
  → Compromise of one app doesn't compromise others
```

---

## Implementation

### Spark Auth Library

```typescript
// @estream/spark-auth
export class SparkAuth {
  private masterSeed: WasmSecretHandle; // Never exposed to JS
  
  static async authenticate(options: {
    image: ImageData,
    challenge: Uint8Array,
  }): Promise<SparkAuth> {
    // 1. Extract Spark DNA from image (in WASM)
    const dna = await SparkDNA.extract(options.image);
    
    // 2. Derive master seed (in WASM)
    const masterSeed = await SparkKDF.derive(dna, options.challenge);
    
    // 3. Return auth handle (seed stays in WASM)
    return new SparkAuth(masterSeed);
  }
  
  deriveIdentity(context: string): Identity {
    // Derive context-specific keys (in WASM)
    return this.masterSeed.deriveIdentity(context);
  }
}
```

### eStream Console Usage

```typescript
// console/src/App.tsx
import { SparkAuth } from '@estream/spark-auth';
import { EstreamClient } from '@estream/sdk-browser';
import { PolyInbox } from '@poly/sdk-browser';

function App() {
  const [spark, setSpark] = useState<SparkAuth | null>(null);
  
  const handleSparkAuth = async (image: ImageData) => {
    const auth = await SparkAuth.authenticate({ image, challenge });
    setSpark(auth);
    
    // eStream connection with Spark identity
    const estreamIdentity = auth.deriveIdentity('estream-console');
    await estreamClient.connect(estreamIdentity);
    
    // Poly inbox with same Spark (different derived keys)
    const polyIdentity = auth.deriveIdentity('poly-messenger');
    await polyInbox.initialize(polyIdentity);
  };
  
  return (
    <Layout>
      {!spark ? (
        <SparkScanner onScan={handleSparkAuth} />
      ) : (
        <>
          <TopologyView />
          <MetricsView />
          <PolyInbox /> {/* Messages from eStream platform */}
        </>
      )}
    </Layout>
  );
}
```

### Poly Inbox Component

```tsx
// @poly/sdk-browser
export function PolyInbox() {
  const { identity } = useSparkAuth();
  
  // Messages filtered to platform context
  const { messages } = usePolyMessages({
    identity,
    filter: { platform: 'estream' }, // Only eStream messages
  });
  
  return (
    <div className="poly-inbox">
      <h3>Messages from eStream</h3>
      
      {messages.map(msg => (
        <Message key={msg.id}>
          <MessageTitle>{msg.notification.title}</MessageTitle>
          <MessageBody>{msg.notification.body}</MessageBody>
          {msg.action && (
            <MessageAction onClick={() => handleAction(msg.action)}>
              {msg.action.label}
            </MessageAction>
          )}
        </Message>
      ))}
    </div>
  );
}
```

---

## Security Properties

### 1. Spark Never Leaves Device
```
Spark image → WASM processing → Keys
                    ↑
                    └── Never exposed to JavaScript
                        Never sent to server
                        Never stored
```

### 2. Challenge-Response Prevents Replay
```
Server sends random challenge
Client signs challenge with derived key
Server verifies signature

→ Can't replay old auth
→ Can't impersonate without Spark
```

### 3. Context Isolation
```
Same master_seed, different keys:

HKDF(seed, "estream-console-v1")  → for eStream
HKDF(seed, "poly-messenger-v1")   → for Poly
HKDF(seed, "io.taketitle-v1")     → for TakeTitle

Compromise of TakeTitle keys:
  ✗ Cannot decrypt eStream messages
  ✗ Cannot auth to Poly
  ✓ Only TakeTitle affected
```

### 4. Forward Secrecy (Optional)
```
For extra security, derive ephemeral session keys:

session_key = HKDF(context_key, session_id)

Each session uses fresh keys
Compromise of one session ≠ compromise of others
```

---

## Message Types in eStream Console

What messages would eStream platform send to users?

```typescript
type EstreamPlatformMessage = 
  | { type: 'circuit_deployed', circuitId: string, name: string }
  | { type: 'governance_proposal', proposalId: string, title: string }
  | { type: 'security_alert', severity: 'low' | 'medium' | 'high', message: string }
  | { type: 'billing_notice', amount: number, dueDate: string }
  | { type: 'system_maintenance', startTime: string, duration: string };
```

---

## Relationship to Blind Connection

**Important distinction**:

| Scenario | Pattern |
|----------|---------|
| eStream Console showing eStream messages | Direct Spark-derived Poly identity |
| TakeTitle showing TakeTitle messages | Blind Connection (TakeTitle doesn't know Poly ID) |
| Poly Messenger showing all messages | User's primary Poly identity |

**Why the difference?**

- eStream Console IS the user's eStream identity - no hiding needed
- TakeTitle is a third party - blind connection protects user's Poly ID
- Poly Messenger is the unified inbox - sees everything

---

## Implementation Status

| Component | Status |
|-----------|--------|
| SparkAuth library | 🔄 In progress |
| WASM key derivation | ✅ In poly-core-wasm |
| Context isolation | ✅ Designed |
| PolyInbox component | 📋 Planned |
| ESLite-wasm storage | ✅ In estream-io |

---

## Next Steps

1. **Extract SparkAuth** from console into `@estream/spark-auth`
2. **Add context derivation** to Spark DNA extraction
3. **Create PolyInbox** component for console
4. **Wire up** eStream platform messages
