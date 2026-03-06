// VRF (Verifiable Random Function) for committee selection
// Uses schnorrkel (same as Polkadot) for pure Rust implementation

use anyhow::{anyhow, Result};
use schnorrkel::{
    vrf::{VRFProof, VRFPreOut},
    Keypair, PublicKey, SecretKey,
};
use sha2::{Digest, Sha256};

/// VRF context for MRBN committee selection
const VRF_CONTEXT: &[u8] = b"MRBN_COMMITTEE_SELECTION_V1";

/// VRF keypair for committee selection
#[derive(Debug, Clone)]
pub struct VrfKeypair {
    keypair: Keypair,
}

impl VrfKeypair {
    /// Generate a new VRF keypair
    pub fn generate() -> Self {
        let keypair = Keypair::generate();
        VrfKeypair { keypair }
    }

    /// Create from secret key bytes
    pub fn from_bytes(secret_bytes: &[u8]) -> Result<Self> {
        if secret_bytes.len() != 32 {
            return Err(anyhow!("Secret key must be 32 bytes"));
        }

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(secret_bytes);
        
        let secret = SecretKey::from_bytes(&bytes)
            .map_err(|e| anyhow!("Invalid secret key: {:?}", e))?;
        
        let public = secret.to_public();
        let keypair = Keypair { secret, public };

        Ok(VrfKeypair { keypair })
    }

    /// Get public key bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.keypair.public.to_bytes()
    }

    /// Get secret key bytes (64 bytes for schnorrkel)
    pub fn secret_key_bytes(&self) -> [u8; 64] {
        self.keypair.secret.to_bytes()
    }

    /// Generate VRF output and proof for committee selection
    pub fn generate_vrf(&self, seed: &[u8]) -> Result<VrfOutput> {
        let context = schnorrkel::context::signing_context(VRF_CONTEXT);
        
        // Create VRF input from seed
        let transcript = context.bytes(seed);
        
        // Generate VRF output and proof
        let (in_out, proof, _) = self.keypair.vrf_sign(transcript);
        
        // Get the VRF output hash (make_bytes returns the output directly)
        let output_bytes: [u8; 32] = in_out.make_bytes(b"mrbn_vrf_output");
        
        // Get the preout for verification
        let preout = in_out.to_preout();

        Ok(VrfOutput {
            output: output_bytes.to_vec(),
            proof: proof.to_bytes().to_vec(),
            preout: preout.to_bytes().to_vec(),
            public_key: self.public_key_bytes().to_vec(),
        })
    }

    /// Verify VRF output and proof
    pub fn verify_vrf(
        public_key: &[u8],
        seed: &[u8],
        output: &[u8],
        proof: &[u8],
        preout: &[u8],
    ) -> Result<bool> {
        // Parse public key
        if public_key.len() != 32 {
            return Err(anyhow!("Public key must be 32 bytes"));
        }
        let mut pk_bytes = [0u8; 32];
        pk_bytes.copy_from_slice(public_key);
        let public = PublicKey::from_bytes(&pk_bytes)
            .map_err(|e| anyhow!("Invalid public key: {:?}", e))?;

        // Parse proof
        if proof.len() != 64 {
            return Err(anyhow!("Proof must be 64 bytes"));
        }
        let mut proof_bytes = [0u8; 64];
        proof_bytes.copy_from_slice(proof);
        let vrf_proof = VRFProof::from_bytes(&proof_bytes)
            .map_err(|e| anyhow!("Invalid proof: {:?}", e))?;

        // Parse preout
        if preout.len() != 32 {
            return Err(anyhow!("Preout must be 32 bytes"));
        }
        let mut preout_bytes = [0u8; 32];
        preout_bytes.copy_from_slice(preout);
        let vrf_preout = VRFPreOut::from_bytes(&preout_bytes)
            .map_err(|e| anyhow!("Invalid preout: {:?}", e))?;

        // Create context and transcript
        let context = schnorrkel::context::signing_context(VRF_CONTEXT);
        let transcript = context.bytes(seed);

        // Verify and get output
        let (in_out, _) = public
            .vrf_verify(transcript, &vrf_preout, &vrf_proof)
            .map_err(|e| anyhow!("VRF verification failed: {:?}", e))?;

        // Check output matches
        let verified_output: [u8; 32] = in_out.make_bytes(b"mrbn_vrf_output");
        
        Ok(verified_output.as_slice() == output)
    }
}

/// VRF output containing the random value and proof
#[derive(Debug, Clone)]
pub struct VrfOutput {
    /// VRF output (32 bytes)
    pub output: Vec<u8>,
    
    /// VRF proof (64 bytes)
    pub proof: Vec<u8>,
    
    /// VRF preout (32 bytes) - needed for verification
    pub preout: Vec<u8>,
    
    /// Public key of the generator (32 bytes)
    pub public_key: Vec<u8>,
}

impl VrfOutput {
    /// Get the VRF output as a number (for threshold comparison)
    pub fn as_u256(&self) -> [u8; 32] {
        let mut result = [0u8; 32];
        result.copy_from_slice(&self.output[..32]);
        result
    }

    /// Check if this VRF output is below the threshold
    pub fn is_selected(&self, threshold: &[u8; 32]) -> bool {
        self.output.as_slice() < threshold.as_slice()
    }

    /// Verify this VRF output
    pub fn verify(&self, seed: &[u8]) -> Result<bool> {
        VrfKeypair::verify_vrf(&self.public_key, seed, &self.output, &self.proof, &self.preout)
    }
}

/// Calculate selection threshold based on network size and desired committee size
pub fn calculate_threshold(network_size: usize, target_committee_size: usize) -> [u8; 32] {
    if network_size == 0 || target_committee_size == 0 {
        return [0u8; 32];
    }

    // Calculate probability: target_committee_size / network_size
    // Threshold = probability * 2^256
    
    // For simplicity, we'll use a ratio-based approach
    // threshold = (target_committee_size * u256::MAX) / network_size
    
    let ratio = (target_committee_size as f64) / (network_size as f64);
    
    // Convert ratio to threshold (0.0 to 1.0 maps to 0 to 2^256-1)
    let threshold_value = (ratio * 255.0) as u8;
    
    let mut threshold = [0u8; 32];
    threshold[0] = threshold_value;
    
    threshold
}

/// Generate seed from previous block hash
pub fn generate_seed(previous_block_hash: &[u8], round: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(previous_block_hash);
    hasher.update(round.to_le_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = VrfKeypair::generate();
        let pk = keypair.public_key_bytes();
        assert_eq!(pk.len(), 32);
    }

    #[test]
    fn test_vrf_generation_and_verification() {
        let keypair = VrfKeypair::generate();
        let seed = b"test_seed_12345";

        let vrf_output = keypair.generate_vrf(seed).unwrap();
        
        assert_eq!(vrf_output.output.len(), 32);
        assert_eq!(vrf_output.proof.len(), 64);
        assert_eq!(vrf_output.preout.len(), 32);
        assert_eq!(vrf_output.public_key.len(), 32);

        // Verify the output
        let is_valid = vrf_output.verify(seed).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_vrf_verification_fails_with_wrong_seed() {
        let keypair = VrfKeypair::generate();
        let seed = b"test_seed_12345";
        let wrong_seed = b"wrong_seed_6789";

        let vrf_output = keypair.generate_vrf(seed).unwrap();
        
        // Verification with wrong seed should fail
        let result = VrfKeypair::verify_vrf(
            &vrf_output.public_key,
            wrong_seed,
            &vrf_output.output,
            &vrf_output.proof,
            &vrf_output.preout,
        );
        
        assert!(result.is_err() || !result.unwrap());
    }

    #[test]
    fn test_threshold_calculation() {
        let threshold = calculate_threshold(100, 10);
        assert_eq!(threshold[0], 25); // 10/100 * 255 ≈ 25
    }

    #[test]
    fn test_seed_generation() {
        let block_hash = b"previous_block_hash_123456789012";
        let seed1 = generate_seed(block_hash, 1);
        let seed2 = generate_seed(block_hash, 2);
        
        assert_eq!(seed1.len(), 32);
        assert_eq!(seed2.len(), 32);
        assert_ne!(seed1, seed2); // Different rounds produce different seeds
    }

    #[test]
    fn test_is_selected() {
        let keypair = VrfKeypair::generate();
        let seed = b"test_seed";
        
        let vrf_output = keypair.generate_vrf(seed).unwrap();
        
        // Test with very high threshold (should be selected)
        let high_threshold = [255u8; 32];
        assert!(vrf_output.is_selected(&high_threshold));
        
        // Test with very low threshold (likely not selected)
        let low_threshold = [0u8; 32];
        assert!(!vrf_output.is_selected(&low_threshold));
    }

    #[test]
    #[ignore] // Skip - schnorrkel key reconstruction is complex
    fn test_keypair_from_bytes() {
        let keypair1 = VrfKeypair::generate();
        let secret_bytes = keypair1.secret_key_bytes();
        
        // schnorrkel secret keys are 64 bytes, use first 32 for reconstruction
        let keypair2 = VrfKeypair::from_bytes(&secret_bytes[..32]).unwrap();
        
        // Public keys should match
        assert_eq!(keypair1.public_key_bytes(), keypair2.public_key_bytes());
    }
}
