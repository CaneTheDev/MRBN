// Resource monitoring and enforcement

#![allow(dead_code)]

use super::ValidatorConfig;
use anyhow::{anyhow, Result};
use sysinfo::{System, Pid};
use tracing::{info, warn};

/// Resource usage statistics
#[derive(Debug, Clone)]
pub struct ResourceStats {
    /// Current RAM usage in bytes
    pub ram_used_bytes: u64,
    
    /// RAM usage percentage
    pub ram_usage_percent: f64,
    
    /// CPU usage percentage
    pub cpu_usage_percent: f32,
    
    /// Storage used in bytes
    pub storage_used_bytes: u64,
    
    /// Storage usage percentage
    pub storage_usage_percent: f64,
}

/// Resource monitor for enforcing MRBN resource caps
pub struct ResourceMonitor {
    /// Configuration
    config: ValidatorConfig,
    
    /// System information
    system: System,
    
    /// Current process ID
    pid: Pid,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new(config: ValidatorConfig) -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_all();
        
        let pid = sysinfo::get_current_pid()
            .map_err(|e| anyhow!("Failed to get process ID: {}", e))?;
        
        info!("📊 Resource monitor initialized");
        
        Ok(ResourceMonitor {
            config,
            system,
            pid,
        })
    }

    /// Check if resources are within limits
    pub fn check_resources(&mut self) -> Result<()> {
        self.system.refresh_all();
        
        let stats = self.get_stats()?;
        
        // Check RAM limit
        if stats.ram_used_bytes > self.config.max_ram_bytes {
            warn!(
                "⚠️ RAM limit exceeded: {} MB / {} MB",
                stats.ram_used_bytes / 1_048_576,
                self.config.max_ram_bytes / 1_048_576
            );
            return Err(anyhow!(
                "RAM limit exceeded: {} MB (limit: {} MB)",
                stats.ram_used_bytes / 1_048_576,
                self.config.max_ram_bytes / 1_048_576
            ));
        }
        
        Ok(())
    }

    /// Get current resource usage statistics
    pub fn get_stats(&mut self) -> Result<ResourceStats> {
        self.system.refresh_all();
        
        // Get process information
        let process = self.system.process(self.pid)
            .ok_or_else(|| anyhow!("Failed to get process information"))?;
        
        // RAM usage (sysinfo 0.38+ returns bytes directly)
        let ram_used_bytes = process.memory();
        let ram_usage_percent = (ram_used_bytes as f64 / self.config.max_ram_bytes as f64) * 100.0;
        
        // CPU usage
        let cpu_usage_percent = process.cpu_usage();
        
        // Storage usage (placeholder - would need to check actual storage)
        let storage_used_bytes = 0; // TODO: Implement actual storage tracking
        let storage_usage_percent = (storage_used_bytes as f64 / self.config.max_storage_bytes as f64) * 100.0;
        
        Ok(ResourceStats {
            ram_used_bytes,
            ram_usage_percent,
            cpu_usage_percent,
            storage_used_bytes,
            storage_usage_percent,
        })
    }

    /// Log resource usage
    pub fn log_stats(&mut self) -> Result<()> {
        let stats = self.get_stats()?;
        
        info!(
            "📊 Resources: RAM {:.1}% ({} MB), CPU {:.1}%",
            stats.ram_usage_percent,
            stats.ram_used_bytes / 1_048_576,
            stats.cpu_usage_percent
        );
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_monitor_creation() {
        let config = ValidatorConfig::default();
        let monitor = ResourceMonitor::new(config);
        assert!(monitor.is_ok());
    }

    #[test]
    fn test_get_stats() {
        let config = ValidatorConfig::default();
        let mut monitor = ResourceMonitor::new(config).unwrap();
        
        let stats = monitor.get_stats();
        assert!(stats.is_ok());
        
        let stats = stats.unwrap();
        assert!(stats.ram_used_bytes > 0);
    }

    #[test]
    fn test_check_resources() {
        let config = ValidatorConfig::default();
        let mut monitor = ResourceMonitor::new(config).unwrap();
        
        // Should pass with default 1GB limit
        assert!(monitor.check_resources().is_ok());
    }
}
