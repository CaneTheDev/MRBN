# MRBN Research Documents

**Status:** Research Phase - Not Official  
**Purpose:** Exploring new architectural approaches for MRBN

---

## Documents in This Folder

### 1. [CONSENSUS_MODEL_V2.md](./CONSENSUS_MODEL_V2.md)
Detailed specification of the proposed two-round cascading consensus model.

**Key Topics:**
- Core philosophy and design principles
- Mathematical security analysis
- Network growth effects
- Implementation pseudocode
- Comparison to Bitcoin

### 2. [ATTACK_SCENARIOS.md](./ATTACK_SCENARIOS.md)
Comprehensive analysis of potential attack vectors and defense mechanisms.

**Key Topics:**
- Horizontal scaling attacks
- Committee capture attacks
- IP spoofing and eclipse attacks
- DoS and collusion scenarios
- Bootstrap phase security

---

## Key Findings

### The Core Innovation

MRBN inverts the Bitcoin model:
- **Bitcoin:** One massive task, winner takes all
- **MRBN:** Millions of tiny tasks, everyone gets some

This enables:
- Low barrier to entry (any device can participate)
- Passive income for all participants
- Exponential security through network growth
- No economies of scale for large operators

### The Security Model

**Two-Round Cascading Consensus:**
- Round 1: 20 nodes, need 14 votes (70%)
- Round 2: 20 nodes, need 14 votes (70%)
- IP diversity: Max 2 nodes per /24 subnet

**Result:**
- 10% attacker: Success rate = 0.00000000000004% (impossible)
- 30% attacker: Success rate = 0.000004% (25M attempts needed)
- 50% attacker: Success rate = 0.0006% (166K attempts needed)

### The Economic Model

**Diminishing Returns for Horizontal Scaling:**
- 1 node: Earns X per day
- 1000 nodes (same datacenter): Earns ~2X per day (not 1000X)
- 1000 nodes (distributed): Earns ~30X per day (not 1000X)

**Attack Cost vs Reward:**
- Running 1000 nodes: $10,000-$50,000/month
- Revenue from 1000 nodes: $60-$120/month
- **Conclusion:** Economically irrational

---

## Open Questions

1. What is the optimal committee size? (15, 20, 25, 30 nodes?)
2. Should we use /24, /16, or ASN-level IP diversity?
3. How do we handle bootstrap phase when network is small?
4. What VRF algorithm provides best performance?
5. How should gas fees be distributed among validators?
6. What happens if committee members go offline mid-validation?

---

## Next Steps

1. ✅ Mathematical analysis (completed)
2. ⏳ Simulation of attack scenarios
3. ⏳ Prototype implementation
4. ⏳ Testnet deployment
5. ⏳ Security audit
6. ⏳ Performance benchmarking
7. ⏳ Community feedback

---

## How to Contribute

This is research-phase documentation. Feedback welcome on:
- Mathematical accuracy
- Attack vectors we haven't considered
- Implementation challenges
- Economic modeling assumptions

---

## Relationship to Official Whitepaper

These documents represent **exploratory research** and are not yet part of the official MRBN specification. The current whitepaper (Docs/WHITEPAPER.md) remains the authoritative source until this research is validated and approved.

---

**Last Updated:** March 2026  
**Status:** Active Research
