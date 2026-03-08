# Two-Layer Architecture

**Date:** March 2026  
**Status:** Core Architecture Design

---

## Overview

MRBN v3.0 uses a two-layer architecture that separates coordination complexity from validation simplicity.

**Key Innovation:** Complex orchestration happens at the server layer, simple validation happens at the user layer.

---

## The Two Layers

```
┌─────────────────────────────────────────────┐
│         Layer 1: Orchestrator Nodes         │
│              (Coordination)                 │
│                                             │
│  - Track online validators                  │
│  - Distribute validation tasks              │
│  - Collect votes                            │
│  - Cross-validate results                   │
│  - Finalize transactions                    │
│  - Broadcast state updates                  │
│  - Archive history                          │
│                                             │
│  Requirements: Server ($50-500/month)       │
│  Operators: Anyone (open source)            │
│  Earn: 10% of transaction fees              │
└──────────────┬──────────────────────────────┘
               │
               │ WebSocket Connections
               │ (Bidirectional Randomness)
               │
               ↓
┌─────────────────────────────────────────────┐
│         Layer 2: Validator Nodes            │
│              (Validation)                   │
│                                             │
│  - Connect to random orchestrators          │
│  - Receive validation tasks                 │
│  - Check balance + signature                │
│  - Return vote (APPROVE/REJECT)             │
│  - Earn fees                                │
│                                             │
│  Requirements: Phone/Laptop                 │
│  Operators: Anyone (just click Start)       │
│  Earn: 90% of transaction fees (split)      │
└─────────────────────────────────────────────┘
```

---

## Layer 1: Orchestrator Nodes

### Purpose

Orchestrators handle the complex coordination logic so validators don't have to.

### Responsibilities

**1. Validator Registry**
```
Maintain list of online validators:
- Validator ID
- Connection status
- Last heartbeat time
- Geographic location
- Device type (optional)
```

**2. Task Distribution**
```
When transaction arrives:
1. Select 20 random validators (VRF)
2. Send validation task to each
3. Wait for responses (with timeout)
4. Collect votes
5. Determine result (14/20 = pass)
```

**3. Cross-Validation**
```
Coordinate with other orchestrators:
1. Broadcast result to peers
2. Verify VRF selection was random
3. Verify votes are legitimate
4. Reach consensus (3/4 orchestrators agree)
5. Finalize transaction
```

**4. State Management**
```
Maintain current account balances:
- Query interface for validators
- Update after finalized transactions
- Periodic checkpoints
- Sync with other orchestrators
```

**5. History Archival**
```
Store transaction history:
- Every finalized transaction
- Periodic snapshots
- Queryable by anyone
- Optional (not required for validation)
```

### What Orchestrators DON'T Do

**❌ They don't validate transactions** - Validators do that  
**❌ They don't control consensus** - Multiple orchestrators must agree  
**❌ They don't hold funds** - Just coordinate messages  
**❌ They don't know which validators are "theirs"** - Blind registration  

### Technical Requirements

**Hardware:**
- 4GB RAM
- 2 CPU cores
- 100GB storage
- 100Mbps internet

**Software:**
- Rust-based server
- Redis/Valkey for validator registry
- WebSocket server
- VRF implementation

**Cost:**
- $50-500/month depending on scale
- Covered by 10% fee earnings

### Who Runs Them

**Anyone can run an orchestrator:**
- Project team (initially)
- Community members
- Companies
- Institutions

**Requirements:**
- Run open-source software
- Stake small amount (1000 Kain) to prevent Sybil
- Maintain uptime
- Follow protocol

**Incentive:**
- Earn 10% of transaction fees
- At scale: $100-1000/day per orchestrator

---

## Layer 2: Validator Nodes

### Purpose

Validators perform the actual transaction validation and earn the majority of fees.

### Responsibilities

**1. Connection Management**
```
On startup:
1. Generate/load validator ID
2. Download orchestrator list
3. Select 5 random orchestrators (geo-diverse)
4. Connect via WebSocket
5. Send HELLO message
```

**2. Heartbeat**
```
Every 30 seconds:
1. Send HEARTBEAT to all 5 orchestrators
2. Receive ACK
3. If no ACK: Orchestrator is dead, replace it
```

**3. Validation**
```
When task arrives:
1. Receive transaction details
2. Query orchestrator: "What's sender's balance?"
3. Verify signature
4. Check: balance >= amount?
5. Return vote: APPROVE or REJECT
6. Earn fee (if transaction finalizes)
```

**4. Automatic Rotation**
```
Every 24 hours (background):
1. Select 2 random connections to rotate
2. Send GOODBYE to those orchestrators
3. Disconnect
4. Select 2 new random orchestrators
5. Connect and send HELLO
6. Continue earning (no downtime)
```

### What Validators DON'T Do

**❌ They don't store full blockchain** - Just current state (optional)  
**❌ They don't coordinate with each other** - Orchestrators handle that  
**❌ They don't know other validators** - Anonymous participation  
**❌ They don't need technical knowledge** - App handles everything  

### Technical Requirements

**Hardware:**
- 500MB RAM
- Any CPU
- 100MB storage
- Internet connection

**Software:**
- Desktop app (Windows/Mac/Linux)
- Mobile app (iOS/Android)
- Runs in background
- Auto-updates

**Cost:**
- $0 (just electricity)
- ~$1-5/month in electricity

### Who Runs Them

**Anyone:**
- Students
- Workers
- Stay-at-home parents
- Anyone wanting passive income

**Requirements:**
- Download app
- Click "Start"
- That's it

**Incentive:**
- Earn 90% of transaction fees
- At scale: $10-50/month per validator

---

## Communication Protocol

### Validator → Orchestrator Messages

**HELLO (on connection)**
```json
{
  "type": "HELLO",
  "validator_id": "5a3f5e8c9d2b1f4e7",
  "timestamp": 1234567890,
  "version": "3.0.0"
}
```

**HEARTBEAT (every 30 seconds)**
```json
{
  "type": "HEARTBEAT",
  "validator_id": "5a3f5e8c9d2b1f4e7"
}
```

**VOTE (validation result)**
```json
{
  "type": "VOTE",
  "task_id": "tx_abc123",
  "validator_id": "5a3f5e8c9d2b1f4e7",
  "vote": "APPROVE",
  "timestamp": 1234567890
}
```

**GOODBYE (when rotating)**
```json
{
  "type": "GOODBYE",
  "validator_id": "5a3f5e8c9d2b1f4e7",
  "reason": "rotation"
}
```

### Orchestrator → Validator Messages

**ACK (heartbeat response)**
```json
{
  "type": "ACK"
}
```

**TASK (validation request)**
```json
{
  "type": "TASK",
  "task_id": "tx_abc123",
  "transaction": {
    "from": "0x123abc",
    "to": "0x456def",
    "amount": 100,
    "token": "Kain",
    "signature": "0xabc123..."
  },
  "timeout": 5000
}
```

**BALANCE_QUERY_RESPONSE**
```json
{
  "type": "BALANCE_RESPONSE",
  "account": "0x123abc",
  "balance": 500,
  "token": "Kain"
}
```

**STATE_UPDATE (after finalization)**
```json
{
  "type": "STATE_UPDATE",
  "updates": [
    {"account": "0x123abc", "balance": 400},
    {"account": "0x456def", "balance": 600}
  ]
}
```

---

## Why This Architecture Works

### Separation of Concerns

**Orchestrators handle:**
- Complexity (coordination, consensus, state)
- Reliability (always online, good hardware)
- Performance (fast queries, low latency)

**Validators handle:**
- Simplicity (just validate one transaction)
- Accessibility (works on any device)
- Democracy (anyone can participate)

### Scalability

**As network grows:**
- More validators → More orchestrators needed
- Orchestrators scale horizontally (add more servers)
- Validators stay lightweight (constant resources)
- No single bottleneck

### Security

**Attack resistance:**
- Validators randomly select orchestrators (can't be controlled)
- Orchestrators randomly select validators (can't favor their own)
- Multiple orchestrators must agree (Byzantine fault tolerance)
- Constant rotation (topology always shifting)

### Deployability

**Works everywhere:**
- Orchestrators: Any cloud platform (WebSocket support)
- Validators: Any device (phones, laptops, tablets)
- No P2P complexity
- No NAT traversal issues

---

## Comparison to Other Architectures

### vs Pure P2P (Bitcoin, v1.0)

| Aspect | P2P | Two-Layer |
|--------|-----|-----------|
| Deployment | Impossible (NAT) | Easy (WebSocket) |
| Mobile support | No | Yes |
| Complexity | Distributed everywhere | Contained in orchestrators |
| Accessibility | Low | High |

### vs Pure Client-Server (Traditional)

| Aspect | Client-Server | Two-Layer |
|--------|---------------|-----------|
| Centralization | Single server | Multiple orchestrators |
| Single point of failure | Yes | No |
| Censorship resistance | Low | High |
| Trust model | Trust server | Trust consensus |

### vs Federated (Ripple, Stellar)

| Aspect | Federated | Two-Layer |
|--------|-----------|-----------|
| Validator requirements | Server | Phone/Laptop |
| Who can validate | Institutions | Anyone |
| Earnings | No fees | 90% of fees |
| Accessibility | Low | High |

---

## Conclusion

The two-layer architecture achieves:

- ✅ Complexity where it belongs (orchestrators)
- ✅ Simplicity where it matters (validators)
- ✅ Scalability (horizontal scaling)
- ✅ Security (bidirectional randomness)
- ✅ Accessibility (anyone can validate)
- ✅ Deployability (works everywhere)

**This is the foundation of MRBN v3.0.**

---

**Status:** Core architecture design complete.  
**Next:** Read 05_BIDIRECTIONAL_RANDOMNESS.md for the security innovation.
