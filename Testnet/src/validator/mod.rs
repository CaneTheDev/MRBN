// Validator module - resource management and validation

#![allow(dead_code)]

pub mod resources;
pub mod task_queue;

use crate::consensus::validation::{ValidationResult, ValidationTask};
use anyhow::Result;
use resources::ResourceMonitor;
use task_queue::TaskQueue;
use tracing::{info, warn};

/// Validator configuration
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Maximum RAM usage in bytes (default: 1 GB)
    pub max_ram_bytes: u64,
    
    /// Maximum concurrent validation tasks (default: 3)
    pub max_concurrent_tasks: usize,
    
    /// Maximum pending tasks in queue (default: 10)
    pub max_pending_tasks: usize,
    
    /// Timeout per validation task in seconds (default: 5)
    pub task_timeout_secs: u64,
    
    /// Maximum storage in bytes (default: 1 GB)
    pub max_storage_bytes: u64,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        ValidatorConfig {
            max_ram_bytes: 1_073_741_824,      // 1 GB
            max_concurrent_tasks: 3,
            max_pending_tasks: 10,
            task_timeout_secs: 5,
            max_storage_bytes: 1_073_741_824,  // 1 GB
        }
    }
}

/// Validator for MRBN network
pub struct Validator {
    /// Configuration
    #[allow(dead_code)]
    config: ValidatorConfig,
    
    /// Resource monitor
    resource_monitor: ResourceMonitor,
    
    /// Task queue
    task_queue: TaskQueue,
    
    /// Is validator active
    is_active: bool,
}

impl Validator {
    /// Create a new validator
    pub fn new(config: ValidatorConfig) -> Result<Self> {
        info!("🛡️ Initializing validator");
        info!("   Max RAM: {} MB", config.max_ram_bytes / 1_048_576);
        info!("   Max concurrent tasks: {}", config.max_concurrent_tasks);
        info!("   Max pending tasks: {}", config.max_pending_tasks);
        info!("   Task timeout: {}s", config.task_timeout_secs);
        
        let resource_monitor = ResourceMonitor::new(config.clone())?;
        let task_queue = TaskQueue::new(
            config.max_pending_tasks,
            config.max_concurrent_tasks,
        );
        
        Ok(Validator {
            config,
            resource_monitor,
            task_queue,
            is_active: false,
        })
    }

    /// Start the validator
    pub fn start(&mut self) -> Result<()> {
        info!("✅ Starting validator");
        
        // Check if resources are within limits
        self.resource_monitor.check_resources()?;
        
        self.is_active = true;
        Ok(())
    }

    /// Stop the validator
    pub fn stop(&mut self) {
        info!("⏸️ Stopping validator");
        self.is_active = false;
    }

    /// Check if validator is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Submit a validation task
    pub fn submit_task(&mut self, task: ValidationTask) -> Result<()> {
        if !self.is_active {
            return Err(anyhow::anyhow!("Validator is not active"));
        }

        // Check resources before accepting task
        if let Err(e) = self.resource_monitor.check_resources() {
            warn!("⚠️ Resource limit exceeded, rejecting task: {}", e);
            return Err(e);
        }

        // Add to queue
        self.task_queue.enqueue(task)?;
        
        info!("📥 Task queued (pending: {})", self.task_queue.pending_count());
        
        Ok(())
    }

    /// Process next task from queue
    pub async fn process_next_task(&mut self) -> Result<Option<ValidationResult>> {
        if !self.is_active {
            return Ok(None);
        }

        // Check if we can process more tasks
        if !self.task_queue.can_process_more() {
            return Ok(None);
        }

        // Get next task
        let task = match self.task_queue.dequeue() {
            Some(t) => t,
            None => return Ok(None),
        };

        info!("⚙️ Processing validation task for batch {}", task.batch_id);

        // Check resources before processing
        self.resource_monitor.check_resources()?;

        // Process task (simplified - in production, this would validate transactions)
        let result = self.validate_task(&task).await?;

        info!("✅ Completed validation for batch {}", task.batch_id);

        Ok(Some(result))
    }

    /// Validate a task (placeholder implementation)
    async fn validate_task(&self, task: &ValidationTask) -> Result<ValidationResult> {
        // Simulate validation work
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // In production, this would:
        // 1. Validate all transactions in the batch
        // 2. Check signatures
        // 3. Verify balances
        // 4. Check nonces
        // 5. Return validation result

        // For now, return a placeholder result
        Ok(ValidationResult {
            batch_id: task.batch_id,
            validator_public_key: vec![0u8; 32], // Placeholder
            is_valid: true,
            signature: vec![0u8; 64], // Placeholder
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            reason: None,
        })
    }

    /// Get resource usage statistics
    pub fn get_resource_stats(&mut self) -> Result<resources::ResourceStats> {
        self.resource_monitor.get_stats()
    }

    /// Get task queue statistics
    pub fn get_queue_stats(&self) -> task_queue::QueueStats {
        self.task_queue.get_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::committee::Committee;

    fn create_test_task(batch_id: u64) -> ValidationTask {
        let committee = Committee::new(batch_id, [0u8; 32], 5, 100);
        ValidationTask::new(batch_id, vec![], &committee, 30)
    }

    #[test]
    fn test_validator_creation() {
        let config = ValidatorConfig::default();
        let validator = Validator::new(config);
        assert!(validator.is_ok());
    }

    #[test]
    fn test_validator_start_stop() {
        let config = ValidatorConfig::default();
        let mut validator = Validator::new(config).unwrap();

        assert!(!validator.is_active());

        validator.start().unwrap();
        assert!(validator.is_active());

        validator.stop();
        assert!(!validator.is_active());
    }

    #[test]
    fn test_submit_task() {
        let config = ValidatorConfig::default();
        let mut validator = Validator::new(config).unwrap();

        validator.start().unwrap();

        let task = create_test_task(1);
        assert!(validator.submit_task(task).is_ok());
    }
}
