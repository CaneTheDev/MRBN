// Committee selection using VRF
// This is MRBN's core innovation - random, verifiable committee selection

use super::vrf::{VrfKeypair, VrfOutput, calculate_threshold, generate_seed};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tracing::{info, warn};

/// Represents a node that can be selected for committee
#[derive(Debug, Clone)]
pub struct ValidatorNode {
    /// Node's public key (identifier)
    pub public_key: Vec<u8>,
    
    /// Node's VRF keypair
    pub vrf_keypair: VrfKeypair,
    
    /// Node's stake or reputation (for future use)
    pub stake: u64,
}

impl ValidatorNode {
    /// Create a new validator node
    pub fn new(vrf_keypair: VrfKeypair, stake: u64) -> Self {
        let public_key = vrf_keypair.public_key_bytes().to_vec();
        ValidatorNode {
            public_key,
            vrf_keypair,
            stake,
        }
    }

    /// Try to be selected for committee using VRF
    pub fn try_selection(&self, seed: &[u8; 32], threshold: &[u8; 32]) -> Result<Option<VrfOutput>> {
        // Generate VRF output
        let vrf_output = self.vrf_keypair.generate_vrf(seed)?;
        
        // Check if selected (output < threshold)
        if vrf_output.is_selected(threshold) {
            Ok(Some(vrf_output))
        } else {
            Ok(None)
        }
    }
}

/// Committee member with VRF proof
#[derive(Debug, Clone)]
pub struct CommitteeMember {
    pub public_key: Vec<u8>,
    pub vrf_output: VrfOutput,
}

impl CommitteeMember {
    /// Verify this committee member's selection
    pub fn verify(&self, seed: &[u8; 32], threshold: &[u8; 32]) -> Result<bool> {
        // Verify VRF proof
        if !self.vrf_output.verify(seed)? {
            return Ok(false);
        }
        
        // Verify selection (output < threshold)
        if !self.vrf_output.is_selected(threshold) {
            return Ok(false);
        }
        
        // Verify public key matches
        if self.vrf_output.public_key != self.public_key {
            return Ok(false);
        }
        
        Ok(true)
    }
}

/// Committee for validating a batch of transactions
#[derive(Debug, Clone)]
pub struct Committee {
    /// Committee ID (batch number)
    pub id: u64,
    
    /// Committee members
    pub members: Vec<CommitteeMember>,
    
    /// Target committee size
    pub target_size: usize,
    
    /// Selection seed
    pub seed: [u8; 32],
    
    /// Selection threshold
    pub threshold: [u8; 32],
}

impl Committee {
    /// Create a new committee
    pub fn new(id: u64, seed: [u8; 32], target_size: usize, network_size: usize) -> Self {
        let threshold = calculate_threshold(network_size, target_size);
        
        info!(
            "📋 Creating committee {} with target size {} from network of {} nodes",
            id, target_size, network_size
        );
        
        Committee {
            id,
            members: Vec::new(),
            target_size,
            seed,
            threshold,
        }
    }

    /// Add a member to the committee
    pub fn add_member(&mut self, member: CommitteeMember) -> Result<()> {
        // Verify the member's selection
        if !member.verify(&self.seed, &self.threshold)? {
            return Err(anyhow!("Invalid committee member selection"));
        }
        
        // Check for duplicates
        if self.members.iter().any(|m| m.public_key == member.public_key) {
            return Err(anyhow!("Member already in committee"));
        }
        
        self.members.push(member);
        info!("✅ Added member to committee {} (size: {})", self.id, self.members.len());
        
        Ok(())
    }

    /// Check if committee has enough members
    pub fn is_ready(&self) -> bool {
        self.members.len() >= self.min_size()
    }

    /// Get minimum committee size (at least 3 for 2/3 majority)
    pub fn min_size(&self) -> usize {
        std::cmp::max(3, self.target_size / 2)
    }

    /// Get current size
    pub fn size(&self) -> usize {
        self.members.len()
    }

    /// Verify all committee members
    pub fn verify_all(&self) -> Result<bool> {
        for member in &self.members {
            if !member.verify(&self.seed, &self.threshold)? {
                warn!("❌ Invalid committee member found");
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// Committee selection manager
pub struct CommitteeSelector {
    /// Current network size estimate
    network_size: usize,
    
    /// Target committee size per batch
    target_committee_size: usize,
    
    /// Active committees by batch ID
    committees: HashMap<u64, Committee>,
}

impl CommitteeSelector {
    /// Create a new committee selector
    pub fn new(network_size: usize, target_committee_size: usize) -> Self {
        info!(
            "🎯 Initializing committee selector: network_size={}, target_size={}",
            network_size, target_committee_size
        );
        
        CommitteeSelector {
            network_size,
            target_committee_size,
            committees: HashMap::new(),
        }
    }

    /// Update network size estimate
    pub fn update_network_size(&mut self, size: usize) {
        info!("📊 Updating network size: {} -> {}", self.network_size, size);
        self.network_size = size;
    }

    /// Create a new committee for a batch
    pub fn create_committee(&mut self, batch_id: u64, previous_block_hash: &[u8]) -> Committee {
        let seed = generate_seed(previous_block_hash, batch_id);
        let committee = Committee::new(
            batch_id,
            seed,
            self.target_committee_size,
            self.network_size,
        );
        
        self.committees.insert(batch_id, committee.clone());
        committee
    }

    /// Get committee by batch ID
    pub fn get_committee(&self, batch_id: u64) -> Option<&Committee> {
        self.committees.get(&batch_id)
    }

    /// Remove old committees (cleanup)
    pub fn cleanup_old_committees(&mut self, current_batch: u64, keep_last: u64) {
        let cutoff = current_batch.saturating_sub(keep_last);
        let before_count = self.committees.len();
        self.committees.retain(|&id, _| id >= cutoff);
        let removed = before_count - self.committees.len();
        
        // Only log if we actually removed something
        if removed > 0 {
            info!("🧹 Cleaned up {} old committees (cutoff: batch {})", removed, cutoff);
        }
    }

    /// Calculate expected committee size based on network size
    pub fn expected_committee_size(&self) -> usize {
        // Dynamic sizing: 10-40 nodes based on network size
        let size = (self.network_size as f64 * 0.1).ceil() as usize;
        size.clamp(10, 40)
    }
}

/// Run committee selection simulation
pub fn simulate_selection(
    validators: &[ValidatorNode],
    seed: &[u8; 32],
    target_size: usize,
) -> Result<Vec<CommitteeMember>> {
    let network_size = validators.len();
    let threshold = calculate_threshold(network_size, target_size);
    
    info!(
        "🎲 Simulating committee selection: {} validators, target size {}",
        network_size, target_size
    );
    
    let mut selected = Vec::new();
    
    for validator in validators {
        if let Some(vrf_output) = validator.try_selection(seed, &threshold)? {
            selected.push(CommitteeMember {
                public_key: validator.public_key.clone(),
                vrf_output,
            });
        }
    }
    
    info!("✅ Selected {} out of {} validators", selected.len(), network_size);
    
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_validators(count: usize) -> Vec<ValidatorNode> {
        (0..count)
            .map(|_| {
                let keypair = VrfKeypair::generate();
                ValidatorNode::new(keypair, 1000)
            })
            .collect()
    }

    #[test]
    fn test_validator_node_creation() {
        let keypair = VrfKeypair::generate();
        let node = ValidatorNode::new(keypair, 1000);
        
        assert_eq!(node.public_key.len(), 32);
        assert_eq!(node.stake, 1000);
    }

    #[test]
    fn test_committee_creation() {
        let seed = [1u8; 32];
        let committee = Committee::new(1, seed, 10, 100);
        
        assert_eq!(committee.id, 1);
        assert_eq!(committee.target_size, 10);
        assert_eq!(committee.members.len(), 0);
        assert!(!committee.is_ready());
    }

    #[test]
    fn test_committee_selection_simulation() {
        let validators = create_test_validators(100);
        let seed = [42u8; 32];
        
        let selected = simulate_selection(&validators, &seed, 10).unwrap();
        
        // Should select some validators (probabilistic, but with 100 validators targeting 10, should get some)
        assert!(selected.len() > 0);
        assert!(selected.len() <= validators.len());
    }

    #[test]
    fn test_committee_member_verification() {
        let validators = create_test_validators(10);
        let seed = [42u8; 32];
        
        let selected = simulate_selection(&validators, &seed, 5).unwrap();
        
        if let Some(member) = selected.first() {
            let threshold = calculate_threshold(10, 5);
            assert!(member.verify(&seed, &threshold).unwrap());
        }
    }

    #[test]
    fn test_committee_selector() {
        let mut selector = CommitteeSelector::new(100, 10);
        
        let block_hash = b"previous_block_hash_123456789012";
        let committee = selector.create_committee(1, block_hash);
        
        assert_eq!(committee.id, 1);
        assert_eq!(committee.target_size, 10);
        
        let retrieved = selector.get_committee(1);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_committee_cleanup() {
        let mut selector = CommitteeSelector::new(100, 10);
        let block_hash = b"previous_block_hash_123456789012";
        
        // Create multiple committees
        for i in 1..=10 {
            selector.create_committee(i, block_hash);
        }
        
        assert_eq!(selector.committees.len(), 10);
        
        // Cleanup old committees (keep last 5)
        selector.cleanup_old_committees(10, 5);
        
        assert!(selector.committees.len() <= 6); // Keeps committees 5-10
    }

    #[test]
    fn test_dynamic_committee_sizing() {
        let selector = CommitteeSelector::new(50, 10);
        let size = selector.expected_committee_size();
        assert!(size >= 10 && size <= 40);
        
        let large_selector = CommitteeSelector::new(1000, 10);
        let large_size = large_selector.expected_committee_size();
        assert_eq!(large_size, 40); // Capped at 40
    }
}
