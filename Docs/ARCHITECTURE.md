# MRBN Architecture

**Technical Architecture and System Design**

Version 0.2 | February 2026

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Network Layer](#network-layer)
3. [Consensus Layer](#consensus-layer)
4. [Validation Layer](#validation-layer)
5. [Storage Layer](#storage-layer)
6. [Economic Layer](#economic-layer)
7. [Client Architecture](#client-architecture)
8. [Protocol Specifications](#protocol-specifications)

---

## System Overview

MRBN consists of five primary layers:

```
┌─────────────────────────────────────┐
│      Application Layer              │  (Wallets, DApps, Tokens)
├─────────────────────────────────────┤
│      Economic Layer                 │  (Gas, Rewards, Kain)
├─────────────────────────────────────┤
│      Validation Layer               │  (Committees, Micro-tasks)
├─────────────────────────────────────┤
│      Consensus Layer                │  (VRF, Byzantine Agreement)
├─────────────────────────────────────┤
│      Network Layer                  │  (P2P, Discovery, Sync)
└─────────────────────────────────────┘
```

---

## Network Layer

### Peer-to-Peer Communication

**Protocol**: Custom P2P over TCP/IP with TLS 1.3 encryption

**Node Discovery**:
- Bootstrap nodes (initial hardcoded list)
- DHT-based peer discovery
- Peer exchange protocol
- Geographic diversity preference

**Message Types**:
- `HELLO`: Node introduction and capability exchange
- `PEER_LIST`: Share known peers
- `TX_BROADCAST`: Transaction propagation
- `BLOCK_ANNOUNCE`: New block notification
- `COMMITTEE_SYNC`: Committee membership updates
- `VALIDATION_REQUEST`: Micro-validation task assignment
- `VALIDATION_RESULT`: Validation outcome submission

### Network Topology

- **Target**: 10,000+ globally distributed nodes
- **Connections per node**: 8-32 peers
- **Latency tolerance**: <5 seconds for global propagation
- **Bandwidth**: ~10-50 KB/s average per node

---

## Consensus Layer

### VRF-Based Committee Selection

**Algorithm**:
```
1. Previous block hash → seed
2. Each node computes: VRF_output = VRF(seed, node_private_key)
3. If VRF_output < threshold:
   - Node is selected for committee
4. Committee size: 10-40 nodes per batch
5. Selection is verifiable by all nodes
```

**VRF Properties**:
- Unpredictable until seed is known
- Verifiable by anyone with node's public key
- Unique output per node per seed
- Collision-resistant

### Byzantine Agreement

**Micro-Committee Consensus**:
- Each committee validates a transaction batch
- 2/3 majority required for approval
- Parallel committees process different batches
- Aggregated results form the block

**Finality**:
- Probabilistic finality after 1 block
- Practical finality after 6 blocks
- Absolute finality through checkpointing (future)

---

## Validation Layer

### Transaction Micro-Validation

**Process Flow**:
```
1. Transaction pool receives new transactions
2. Transactions grouped into batches (10-100 tx per batch)
3. VRF selects committee for each batch
4. Committee members validate in parallel:
   - Signature verification
   - Balance checks
   - Nonce validation
   - Gas limit verification
5. Committee members submit results
6. 2/3 agreement → batch approved
7. Approved batches → new block
```

### Resource Management

**Per-Validation Task**:
- Max CPU time: 100-500ms
- Max RAM: 100 MB
- Max network: 10 KB upload/download
- Timeout: 5 seconds

**Node Capacity**:
- Concurrent validations: 1-3 tasks
- Queue depth: 10 pending tasks
- Rejection if overloaded

---

## Storage Layer

### Distributed Block Storage

**Block Structure**:
```json
{
  "block_number": 12345,
  "previous_hash": "0x...",
  "timestamp": 1738886400,
  "transactions": [...],
  "committee_signatures": [...],
  "merkle_root": "0x...",
  "size": 5120
}
```

**Storage Strategy**:
- Each node stores: 500 MB - 1 GB of blocks
- Blocks distributed across network (sharding)
- Redundancy factor: 10-20 copies per block
- Nodes can query any holder for historical data

**Pruning**:
- Nodes can prune old blocks after threshold
- State snapshots for quick sync
- Archive nodes (optional) store full history

### State Management

**Account State**:
```json
{
  "address": "0x...",
  "balance": 1000000,
  "nonce": 42,
  "code_hash": "0x..." // for smart contracts (future)
}
```

**State Storage**:
- Merkle Patricia Trie (or similar)
- State root in each block
- Incremental state updates
- Snapshot every N blocks

---

## Economic Layer

### Gas Mechanism

**Gas Calculation**:
```
gas_cost = base_fee + (computational_complexity * cpu_factor)
                    + (storage_size * storage_factor)
                    + (network_bytes * network_factor)
```

**Fee Distribution**:
- 80% to validators (split by contribution)
- 10% to block proposer
- 10% burned (deflationary mechanism)

### Reward Distribution

**Per Validation**:
```
validator_reward = (gas_fees * validator_share) 
                 * (1 + reputation_bonus)
                 * (1 - latency_penalty)
```

**Reputation Scoring**:
- Correct validations: +1 point
- Incorrect validations: -10 points
- Timeout/offline: -5 points
- Reputation affects selection probability (±20%)

---

## Client Architecture

### Desktop Client (Stage 1)

**Technology Stack**:
- **Backend**: Rust (tokio async runtime)
- **GUI**: Tauri or egui
- **Platforms**: Windows, macOS, Linux

**Components**:
```
┌─────────────────────────────────────┐
│         GUI Layer (Tauri)           │
├─────────────────────────────────────┤
│      Application Logic (Rust)       │
│  - Wallet management                │
│  - Transaction creation             │
│  - Node monitoring                  │
├─────────────────────────────────────┤
│      Validator Node (Rust)          │
│  - P2P networking                   │
│  - VRF computation                  │
│  - Transaction validation           │
│  - Block storage                    │
├─────────────────────────────────────┤
│      Crypto Library                 │
│  - Ed25519 signatures               │
│  - VRF implementation               │
│  - Hash functions                   │
└─────────────────────────────────────┘
```

**Resource Enforcement**:
- OS-level resource monitoring
- Automatic throttling if limits exceeded
- Graceful degradation under load

---

## Protocol Specifications

### Transaction Format

```json
{
  "version": 1,
  "from": "0x...",
  "to": "0x...",
  "value": 1000,
  "gas_limit": 21000,
  "gas_price": 10,
  "nonce": 42,
  "data": "0x...",
  "signature": "0x..."
}
```

### Block Format

```json
{
  "version": 1,
  "block_number": 12345,
  "timestamp": 1738886400,
  "previous_hash": "0x...",
  "merkle_root": "0x...",
  "state_root": "0x...",
  "transactions": [...],
  "committee_info": {
    "seed": "0x...",
    "members": ["node_id_1", "node_id_2", ...],
    "signatures": [...]
  }
}
```

### Network Protocol

**Message Format**:
```json
{
  "version": 1,
  "type": "TX_BROADCAST",
  "timestamp": 1738886400,
  "payload": {...},
  "signature": "0x..."
}
```

---

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Transaction throughput | 1,000-10,000 TPS | Depends on network size |
| Block time | 10-30 seconds | Adjustable |
| Finality time | 1-5 minutes | Probabilistic |
| Node sync time | <1 hour | For new nodes |
| Validation latency | <1 second | Per micro-task |
| Network overhead | <50 KB/s | Per node average |

---

## Future Enhancements

### Smart Contracts (Stage 3+)
- WebAssembly VM for contract execution
- Gas metering for contract calls
- Contract state storage

### Cross-Chain Bridges (Stage 4+)
- Atomic swaps with other chains
- Wrapped token support
- Interoperability protocols

### Privacy Features (Research)
- Zero-knowledge proofs for private transactions
- Confidential balances
- Anonymous committee selection

---

## Implementation Notes

### Critical Path for MVP

1. P2P networking layer
2. VRF committee selection
3. Basic transaction validation
4. Block creation and storage
5. Wallet functionality
6. GUI client

### Testing Strategy

- Unit tests for all core components
- Integration tests for network protocols
- Simulation testing with 100-1000 virtual nodes
- Testnet deployment before mainnet
- Stress testing under adversarial conditions

---

**Status**: Architectural specification for MRBN v0.2  
**Next Steps**: Begin implementation of core networking and consensus layers
