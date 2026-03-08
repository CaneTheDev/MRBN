# Device Fingerprinting System

**Status:** Research Phase  
**Date:** March 2026  
**Purpose:** Generate unique, stable device identifiers for Sybil resistance

---

## Overview

Each device generates a unique fingerprint based on hardware characteristics. This fingerprint:
- Is stable (doesn't change unless hardware changes)
- Is unique (virtually impossible to duplicate)
- Is short (12 hex characters for easy storage)
- Is deterministic (same hardware = same fingerprint)
- Is one-way (can't reverse engineer hardware from fingerprint)

---

## Hardware Identifiers

### Primary Identifiers (Stable & Hard to Fake)

1. **MAC Address**
   - Network interface hardware address
   - Format: `00:1B:44:11:3A:B7`
   - Stability: High (unless network card replaced)
   - Uniqueness: High (globally unique)

2. **CPU ID**
   - Processor serial number
   - Format: `BFEBFBFF000906E9`
   - Stability: Very High (never changes)
   - Uniqueness: Very High (unique per CPU)

3. **Motherboard Serial**
   - Motherboard manufacturer serial
   - Format: `L1HF65E00X9`
   - Stability: Very High (never changes)
   - Uniqueness: High (unique per board)

4. **Disk Serial Number**
   - Primary storage device serial
   - Format: `S2X5NY0K123456`
   - Stability: High (unless disk replaced)
   - Uniqueness: Very High (unique per disk)

5. **System UUID**
   - BIOS/UEFI unique identifier
   - Format: `4C4C4544-0050-5A10-8033`
   - Stability: Very High (set by manufacturer)
   - Uniqueness: Very High (globally unique)

### Secondary Identifiers (Optional, for additional entropy)

6. **GPU Serial** (if available)
7. **RAM Serial** (if available)
8. **BIOS Version + Date**

---

## Fingerprint Generation Algorithm

### Step 1: Collect Hardware Information

```rust
struct HardwareInfo {
    mac_address: String,      // "00:1B:44:11:3A:B7"
    cpu_id: String,           // "BFEBFBFF000906E9"
    motherboard_serial: String, // "L1HF65E00X9"
    disk_serial: String,      // "S2X5NY0K123456"
    system_uuid: String,      // "4C4C4544-0050-5A10-8033"
}

fn collect_hardware_info() -> HardwareInfo {
    HardwareInfo {
        mac_address: get_mac_address(),
        cpu_id: get_cpu_id(),
        motherboard_serial: get_motherboard_serial(),
        disk_serial: get_disk_serial(),
        system_uuid: get_system_uuid(),
    }
}
```

### Step 2: Normalize and Concatenate

```rust
fn normalize_hardware_info(info: HardwareInfo) -> String {
    // Remove special characters, convert to lowercase
    let mac = info.mac_address.replace(":", "").to_lowercase();
    let cpu = info.cpu_id.to_lowercase();
    let mobo = info.motherboard_serial.to_lowercase();
    let disk = info.disk_serial.to_lowercase();
    let uuid = info.system_uuid.replace("-", "").to_lowercase();
    
    // Concatenate with separator
    format!("{}|{}|{}|{}|{}", mac, cpu, mobo, disk, uuid)
}
```

### Step 3: Hash with SHA-256

```rust
use sha2::{Sha256, Digest};

fn hash_hardware_info(normalized: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let result = hasher.finalize();
    
    // Convert to hex string
    format!("{:x}", result)
}
```

### Step 4: Detect Device Type and Generate Fingerprint

```rust
fn generate_device_fingerprint(info: HardwareInfo) -> String {
    // 1. Detect device type
    let device_type = detect_device_type(&info);
    
    // 2. Generate hash
    let normalized = normalize_hardware_info(info);
    let hash = hash_hardware_info(normalized);
    
    // 3. Take first 15 characters (60 bits) + 1 type character = 16 total
    // This gives us 2^60 = 1.15 quintillion possible IDs per device type
    let hash_part = &hash[..15];
    
    // 4. Prepend device type code
    format!("{}{}", device_type, hash_part)
}

fn detect_device_type(info: &HardwareInfo) -> char {
    // Check for VM/Cloud first (highest priority detection)
    if is_virtual_machine(info) {
        return '9'; // Virtual Machine
    }
    
    if is_cloud_instance(info) {
        return 'A'; // Cloud Instance (AWS, Azure, GCP)
    }
    
    if is_container(info) {
        return 'B'; // Container (Docker, Kubernetes)
    }
    
    // Physical device detection
    let cpu_brand = get_cpu_brand();
    let form_factor = get_form_factor();
    let cpu_cores = get_cpu_cores();
    let ram_size = get_ram_size_gb();
    let has_battery = has_battery();
    
    // Mobile phone (ARM processor, small screen)
    if cpu_brand.contains("ARM") && form_factor == "Mobile" {
        return '3';
    }
    
    // Tablet (ARM processor, medium screen)
    if cpu_brand.contains("ARM") && form_factor == "Tablet" {
        return '4';
    }
    
    // Laptop (has battery, portable)
    if has_battery && form_factor == "Portable" {
        return '7';
    }
    
    // Server (high core count, ECC RAM, no GPU)
    if cpu_cores >= 16 && has_ecc_ram() {
        return '1';
    }
    
    // Workstation (high-end desktop, professional GPU)
    if cpu_cores >= 8 && ram_size >= 32 {
        return '2';
    }
    
    // Embedded device (Raspberry Pi, IoT - low specs)
    if cpu_cores <= 4 && ram_size <= 4 {
        return '6';
    }
    
    // Default: Desktop PC
    '5'
}
```

### Complete Example

```
Input:
  MAC: "00:1B:44:11:3A:B7"
  CPU: "BFEBFBFF000906E9" (Intel Core i7)
  Motherboard: "L1HF65E00X9"
  Disk: "S2X5NY0K123456"
  UUID: "4C4C4544-0050-5A10-8033"
  Has Battery: No
  CPU Cores: 8
  RAM: 16GB

Device Type Detection:
  → Not VM, Not Cloud, Not Container
  → No battery → Not laptop
  → 8 cores, 16GB RAM → Desktop PC
  → Device Type Code: '5'

Normalized:
  "001b44113ab7|bfebfbff000906e9|l1hf65e00x9|s2x5ny0k123456|4c4c45440050"

SHA-256 Hash:
  "a3f5e8c9d2b1f4e7a6c8d9e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1"

Device Fingerprint:
  "5a3f5e8c9d2b1f4e7" (16 characters)
   ↑ Desktop PC
    └──────────────┘ Unique hardware hash
```

---

## Device Type Codes (First Character)

```
Character | Device Type           | Priority | Notes
----------|----------------------|----------|------------------
1         | Server               | Medium   | Physical datacenter hardware
2         | Workstation          | High     | High-end desktop
3         | Mobile Phone         | High     | iOS/Android phone
4         | Tablet               | High     | iPad, Android tablet
5         | Desktop PC           | High     | Consumer desktop
6         | Embedded Device      | Medium   | Raspberry Pi, IoT
7         | Laptop               | High     | Consumer laptop
8         | Reserved             | -        | Future use
9         | Virtual Machine      | Low      | VMware, VirtualBox, etc.
A         | Cloud Instance       | Low      | AWS, Azure, GCP
B         | Container            | Low      | Docker, Kubernetes
C-F       | Reserved             | -        | Future use
```

## Instant Device Type Recognition

```rust
fn get_device_type_from_fingerprint(fingerprint: &str) -> DeviceType {
    match fingerprint.chars().next().unwrap() {
        '1' => DeviceType::Server,
        '2' => DeviceType::Workstation,
        '3' => DeviceType::MobilePhone,
        '4' => DeviceType::Tablet,
        '5' => DeviceType::Desktop,
        '6' => DeviceType::Embedded,
        '7' => DeviceType::Laptop,
        '9' => DeviceType::VirtualMachine,
        'A' | 'a' => DeviceType::CloudInstance,
        'B' | 'b' => DeviceType::Container,
        _ => DeviceType::Unknown,
    }
}

fn is_physical_device(fingerprint: &str) -> bool {
    let first_char = fingerprint.chars().next().unwrap();
    matches!(first_char, '1'..='7') // Physical devices: 1-7
}

fn is_virtual_device(fingerprint: &str) -> bool {
    let first_char = fingerprint.chars().next().unwrap();
    matches!(first_char, '9' | 'A' | 'a' | 'B' | 'b') // Virtual: 9, A, B
}
```

## Collision Probability

With 16 hex characters (1 type + 15 hash = 64 bits):
- Total possible IDs per device type: 2^60 = 1,152,921,504,606,846,976 (1.15 quintillion)
- Total possible IDs across all types: 16 × 2^60 = 18.4 quintillion

**Collision Analysis:**
- For 1 billion devices: Collision probability ≈ 0.00000000043%
- For 10 billion devices: Collision probability ≈ 0.0000000043%
- For 1 trillion devices: Collision probability ≈ 0.000043%

**Conclusion:** We will never run out of IDs. Even with 1 trillion devices, collision probability is negligible.

---

## Handling Hardware Changes

### Scenario 1: Minor Change (e.g., RAM upgrade)
- Fingerprint remains the same (RAM not in primary identifiers)
- Node continues with same identity

### Scenario 2: Major Change (e.g., new motherboard)
- Fingerprint changes
- Node must re-register as new device
- Old fingerprint becomes inactive

### Scenario 3: Partial Change (e.g., new disk)
- Fingerprint changes
- Use "fuzzy matching" to detect similar devices
- Allow migration with proof of ownership

### Migration Mechanism

```rust
fn allow_migration(old_fingerprint: String, new_fingerprint: String) -> bool {
    // Check if at least 3 out of 5 identifiers match
    let old_info = get_stored_hardware_info(old_fingerprint);
    let new_info = collect_hardware_info();
    
    let matches = count_matching_identifiers(old_info, new_info);
    
    if matches >= 3 {
        // Allow migration with cryptographic proof
        return verify_ownership_signature(old_fingerprint);
    }
    
    false
}
```

---

## Virtual Machine Detection

### The Problem
VMs can fake hardware identifiers, allowing one physical machine to appear as multiple devices.

### Detection Methods

#### 1. Hypervisor Detection
```rust
fn detect_vm() -> bool {
    // Check for VM-specific strings in hardware info
    let cpu_brand = get_cpu_brand();
    let bios_vendor = get_bios_vendor();
    
    let vm_indicators = [
        "VMware", "VirtualBox", "QEMU", "Xen",
        "Hyper-V", "KVM", "Parallels"
    ];
    
    for indicator in vm_indicators {
        if cpu_brand.contains(indicator) || bios_vendor.contains(indicator) {
            return true;
        }
    }
    
    false
}
```

#### 2. Timing Analysis
```rust
fn detect_vm_timing() -> bool {
    // VMs have different CPU timing characteristics
    let start = precise_time_ns();
    
    // Perform CPU-intensive operation
    for _ in 0..1000000 {
        // Busy loop
    }
    
    let duration = precise_time_ns() - start;
    
    // VMs typically have higher variance in timing
    let variance = measure_timing_variance(10);
    
    variance > THRESHOLD
}
```

#### 3. Hardware Consistency Checks
```rust
fn check_hardware_consistency() -> bool {
    // VMs often have suspicious hardware combinations
    let cpu_cores = get_cpu_cores();
    let ram_size = get_ram_size();
    let disk_size = get_disk_size();
    
    // Example: 64 cores with 2GB RAM is suspicious
    if cpu_cores > 32 && ram_size < 4_000_000_000 {
        return false; // Likely VM
    }
    
    true
}
```

### VM Handling Strategy

**Option 1: Allow but Flag**
- VMs are allowed to participate
- Flagged as "virtual" in node registry
- Lower selection probability in committees (e.g., 50% of physical machines)

**Option 2: Require Proof of Physical Hardware**
- Nodes must pass VM detection tests
- VMs are rejected during registration

**Option 3: Economic Disincentive**
- VMs can participate but earn less
- Physical machines get 2x selection probability

**Recommended:** Option 1 (Allow but Flag)
- Doesn't exclude legitimate VPS users
- Still provides Sybil resistance through IP diversity
- Balances accessibility with security

---

## Privacy Considerations

### Concern
Hardware fingerprints could be used to track users across different applications.

### Solution: Salted Hashing

```rust
fn generate_device_fingerprint_with_salt(info: HardwareInfo, salt: &str) -> String {
    let normalized = normalize_hardware_info(info);
    let salted = format!("{}|{}", normalized, salt);
    let hash = hash_hardware_info(salted);
    
    hash[..12].to_string()
}

// Use MRBN-specific salt
const MRBN_SALT: &str = "MRBN_NETWORK_2026";

let fingerprint = generate_device_fingerprint_with_salt(info, MRBN_SALT);
```

**Result:** Fingerprint is unique to MRBN and cannot be correlated with other applications.

---

## Implementation Roadmap

### Phase 1: Basic Fingerprinting
- Implement hardware info collection (MAC, CPU, Disk, UUID)
- Generate SHA-256 hash
- Store 12-character fingerprint

### Phase 2: VM Detection
- Add hypervisor detection
- Implement timing analysis
- Flag VMs in node registry

### Phase 3: Migration Support
- Allow hardware upgrades with proof of ownership
- Implement fuzzy matching for partial changes

### Phase 4: Privacy Enhancement
- Add salted hashing
- Implement zero-knowledge proofs for fingerprint verification

---

## Platform-Specific Implementation

### Windows
```rust
// MAC Address
use pnet::datalink;
let interfaces = datalink::interfaces();
let mac = interfaces[0].mac.unwrap();

// CPU ID
use raw_cpuid::CpuId;
let cpuid = CpuId::new();
let cpu_id = cpuid.get_processor_serial().unwrap();

// System UUID
use winreg::RegKey;
let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
let uuid = hklm.open_subkey("SOFTWARE\\Microsoft\\Cryptography")
    .unwrap()
    .get_value("MachineGuid")
    .unwrap();
```

### Linux
```rust
// MAC Address
let mac = std::fs::read_to_string("/sys/class/net/eth0/address")?;

// CPU ID
let cpuinfo = std::fs::read_to_string("/proc/cpuinfo")?;
// Parse for "processor" field

// System UUID
let uuid = std::fs::read_to_string("/sys/class/dmi/id/product_uuid")?;

// Disk Serial
let disk_serial = std::fs::read_to_string("/sys/block/sda/device/serial")?;
```

### macOS
```rust
// Use system_profiler command
let output = Command::new("system_profiler")
    .arg("SPHardwareDataType")
    .output()?;

// Parse output for:
// - Hardware UUID
// - Serial Number
// - MAC Address (from networksetup)
```

---

## Integration with Consensus Model

### Node Registration

```rust
fn register_node() -> Result<NodeId> {
    // 1. Collect hardware info
    let hw_info = collect_hardware_info();
    
    // 2. Generate fingerprint
    let fingerprint = generate_device_fingerprint_with_salt(hw_info, MRBN_SALT);
    
    // 3. Check if fingerprint already exists
    if node_registry.contains(fingerprint) {
        return Err("Device already registered");
    }
    
    // 4. Detect VM
    let is_vm = detect_vm();
    
    // 5. Register node
    let node = Node {
        fingerprint,
        ip_address: get_public_ip(),
        is_virtual: is_vm,
        registered_at: current_timestamp(),
        reputation: 0,
    };
    
    node_registry.insert(node);
    
    Ok(node.fingerprint)
}
```

### Committee Selection with Instant Device Type Recognition

```rust
fn select_committee_with_fingerprint(
    size: int,
    seed: Hash,
    exclude: Vec<Node>
) -> Vec<Node> {
    let mut committee = vec![];
    let mut ip_subnets = HashMap::new();
    let mut fingerprints = HashSet::new();
    
    for node in all_nodes.shuffle_with_vrf(seed) {
        if exclude.contains(node) { continue; }
        
        // Check IP diversity (max 2 per /24 subnet)
        let subnet = node.ip.subnet_24();
        if ip_subnets.get(subnet) >= 2 { continue; }
        
        // Check fingerprint uniqueness (prevent same device with different IPs)
        if fingerprints.contains(&node.fingerprint) { continue; }
        
        // INSTANT device type check - just look at first character!
        let first_char = node.fingerprint.chars().next().unwrap();
        
        // Lower probability for virtual devices (9, A, B)
        if matches!(first_char, '9' | 'A' | 'a' | 'B' | 'b') {
            if random() > 0.5 { continue; } // 50% selection rate for VMs
        }
        
        // Prioritize physical devices (1-7)
        // They get 100% selection rate
        
        committee.push(node);
        ip_subnets[subnet] += 1;
        fingerprints.insert(node.fingerprint.clone());
        
        if committee.len() == size { break; }
    }
    
    committee
}
```

### Performance: Zero Database Queries

```rust
// OLD WAY (requires database query)
let device_info = database.query(
    "SELECT type, is_vm FROM nodes WHERE fingerprint = ?", 
    fingerprint
);
if device_info.is_vm { ... }

// NEW WAY (instant, no database)
if fingerprint.starts_with('9') || 
   fingerprint.starts_with('A') || 
   fingerprint.starts_with('B') {
    // It's a virtual device
}

// Even simpler
let first_char = fingerprint.chars().next().unwrap();
let is_physical = ('1'..='7').contains(&first_char);
```

**Performance gain:**
- Database query: ~1-10ms
- Character check: ~0.000001ms (1 nanosecond)
- **10,000x faster**
```

---

## Security Analysis

### Attack: Fake Multiple Fingerprints

**Method:** Attacker modifies hardware info before hashing

**Defense:**
- Hardware info is collected by trusted node software
- Cryptographic signing of hardware info
- Network verification through challenges

### Attack: Fingerprint Collision

**Method:** Attacker generates colliding fingerprint through brute force

**Defense:**
- 48-bit space = 281 trillion possibilities
- Brute force requires 2^47 attempts on average
- Computationally infeasible

### Attack: VM Farm

**Method:** Attacker runs 1000 VMs with different fingerprints

**Defense:**
- VM detection flags virtual machines
- IP diversity limits VMs from same datacenter
- Lower selection probability for VMs

---

## Conclusion

Device fingerprinting provides an additional layer of Sybil resistance:

1. **Unique identification** of physical hardware
2. **Stable identity** across sessions
3. **VM detection** to prevent easy scaling
4. **Privacy-preserving** through salted hashing
5. **Collision-resistant** with 48-bit ID space

Combined with IP diversity and two-round consensus, this creates a robust defense against Sybil attacks.

---

**Document Status:** Research Phase  
**Next Steps:** Prototype implementation and testing across platforms
