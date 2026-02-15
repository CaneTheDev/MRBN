# MRBN Frequently Asked Questions

## General Questions

### What is MRBN?

MRBN (Micro Resource Based Network) is a blockchain protocol that lets anyone with a laptop or desktop computer participate as a validator and earn rewards, without needing expensive hardware or large amounts of capital.

### How is MRBN different from Bitcoin?

Bitcoin requires specialized mining hardware (ASICs) and massive energy consumption. MRBN uses small amounts of CPU, RAM, and storage from regular computers, making it accessible to everyone.

### How is MRBN different from Ethereum?

Ethereum requires 32 ETH (~$50,000+) to run a validator, or you must join a staking pool. MRBN requires no capital investment—just a computer with basic specs.

### Is MRBN a cryptocurrency?

MRBN is the protocol. Kain is the first cryptocurrency built on MRBN. The network can support multiple tokens and applications.

---

## Technical Questions

### What are the minimum requirements to run a node?

- **RAM**: 1 GB
- **CPU**: Any modern processor
- **Storage**: 500 MB - 1 GB
- **Network**: Stable internet connection
- **OS**: Windows, macOS, or Linux

### How does validation work?

1. Transactions are grouped into batches
2. Random committees of 10-40 nodes are selected
3. Each committee validates their batch in parallel
4. 2/3 majority agreement approves the batch
5. Approved batches form a new block

### What is VRF?

VRF (Verifiable Random Function) is a cryptographic method that produces random, unpredictable, but verifiable outputs. MRBN uses it to randomly select validator committees in a way that no one can predict or manipulate.

### How fast are transactions?

- **Block time**: 10-30 seconds
- **Finality**: 1-5 minutes (probabilistic)
- **Throughput**: 1,000-10,000 transactions per second (target)

### Is MRBN secure?

MRBN's security comes from having many distributed validators. An attacker would need to control >66% of all nodes globally, which becomes economically irrational due to resource caps and probabilistic committee selection.

---

## Economic Questions

### How do I earn rewards?

By running a validator node. When your node is selected for a committee and successfully validates transactions, you earn a share of the gas fees.

### How much can I earn?

Earnings depend on:
- Network transaction volume
- Your node's uptime and reliability
- Number of validations you participate in

Early estimates suggest micro-income potential (similar to passive income), growing as the network scales.

### What is Kain?

Kain is the native cryptocurrency of MRBN. It's used to pay gas fees and reward validators.

### How is Kain distributed?

- Initial claim campaign (first 10%)
- Ongoing validation rewards
- Founder allocation (for development)

### Can I create my own token on MRBN?

Yes! MRBN is designed to support multiple tokens and applications, similar to Ethereum's ERC-20 tokens.

---

## Participation Questions

### Do I need technical knowledge?

No. MRBN will have a simple GUI application that anyone can download and run. Just install, create a wallet, and start validating.

### Can I run multiple nodes?

Yes, but there are diminishing returns. Each node must have its own hardware and OS instance. Running many nodes doesn't give you an unfair advantage due to resource caps and probabilistic selection.

### What if my computer is offline?

Your node simply won't be selected for committees while offline. There's no penalty, but you won't earn rewards during that time. Your reputation score may decrease slightly if you're offline frequently.

### Can I run a node on a VPS?

Yes, as long as it meets the minimum requirements and you're not running multiple virtual instances on the same physical hardware.

---

## Security Questions

### What prevents someone from running thousands of fake nodes?

1. **Resource caps**: Each node requires real CPU, RAM, and storage
2. **Probabilistic dilution**: More nodes = exponentially harder to control committees
3. **Reputation scoring**: New or unreliable nodes have lower selection probability
4. **Economic cost**: Running many nodes costs money without proportional benefit

### What if a committee is compromised?

Even if one committee is compromised, it only affects one batch of transactions. Other committees validate other batches independently. An attacker would need to compromise many committees simultaneously, which is statistically improbable.

### Can MRBN be 51% attacked?

An attacker would need to control >66% of all global nodes (not just hash power or stake). Due to resource caps, this requires controlling thousands of physical machines worldwide, making it economically irrational.

---

## Development Questions

### When will MRBN launch?

Development roadmap:
- **Stage 1** (Current): Desktop client development
- **Stage 2**: Testnet and claim campaign
- **Stage 3**: Mainnet launch
- **Stage 4**: Mobile apps and ecosystem expansion

### Is MRBN open source?

Yes! MRBN is licensed under MIT License, the same as Bitcoin. Anyone can read, modify, and redistribute the code.

### How can I contribute?

- Code contributions (Rust, GUI development)
- Documentation and tutorials
- Testing and bug reports
- Community support
- Research and analysis

See [CONTRIBUTING.md](../CONTRIBUTING.md) for details.

### What programming language is MRBN written in?

The core protocol is written in Rust for performance and safety. The GUI uses Tauri or egui for cross-platform compatibility.

---

## Comparison Questions

### Why not just use Bitcoin?

Bitcoin mining is dominated by industrial operations. Average people can't meaningfully participate or earn rewards. MRBN is designed specifically for grassroots participation.

### Why not just stake Ethereum?

Ethereum requires 32 ETH (~$50,000+) to run a validator. Most people don't have that capital. MRBN requires no capital investment.

### Why not join a mining/staking pool?

Pools concentrate power and take fees. MRBN eliminates the need for pools by making individual participation viable and rewarding.

---

## Philosophical Questions

### What is the goal of MRBN?

To create a truly decentralized network where anyone, anywhere, with basic hardware can participate meaningfully and earn rewards. To prove that blockchain security doesn't require capital concentration or industrial hardware.

### Who controls MRBN?

No one. Like Bitcoin, MRBN is designed to be community-controlled. The protocol is open source, and development is transparent.

### Is MRBN trying to replace Bitcoin or Ethereum?

No. MRBN addresses a different problem: accessibility and fairness. Bitcoin is the gold standard for store of value. Ethereum is the leader in smart contracts. MRBN focuses on grassroots participation.

---

## Practical Questions

### Where can I download the MRBN client?

*Coming soon - Stage 1 development in progress*

### Where can I buy Kain?

Kain is not yet available. Initial distribution will be through a claim campaign and validation rewards after mainnet launch.

### Is there a testnet?

Not yet. Testnet will launch in Stage 2 of development.

### How can I stay updated?

*Community channels and newsletter coming soon*

---

## Concerns and Criticisms

### Isn't this just another altcoin?

MRBN introduces novel concepts not found in existing blockchains: resource-capped validation, VRF-based committee selection, and micro-task parallelism. It's not a fork or copy of existing chains.

### How do you prevent centralization over time?

Resource caps prevent vertical scaling. Probabilistic selection prevents predictable control. Reputation scoring prevents Sybil attacks. These mechanisms are built into the protocol, not just social norms.

### What if large players still dominate?

Even if someone runs 1,000 nodes, they can't control committees due to probabilistic dilution. The probability of controlling a committee decreases exponentially: P = (X%)^k, where X is their node share and k is committee size.

### Is this sustainable long-term?

Yes. As transaction volume grows, gas fees increase, making validation more rewarding. The network scales naturally with adoption.

---

## Getting Started

### I want to participate. What should I do?

1. Star the GitHub repository to stay updated
2. Join community channels (coming soon)
3. Read the documentation
4. Wait for Stage 1 client release
5. Download, install, and start validating!

### I'm a developer. How can I help?

Check out [CONTRIBUTING.md](../CONTRIBUTING.md) and the GitHub issues. We need help with:
- Core protocol implementation
- GUI development
- Testing and QA
- Documentation
- Security analysis

---

**Have more questions?**

*Community channels and support coming soon*

---

**Last Updated**: February 2026  
**Version**: 0.2
