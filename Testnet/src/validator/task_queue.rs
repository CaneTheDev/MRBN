// Task queue for managing validation tasks

#![allow(dead_code)]

use crate::consensus::validation::ValidationTask;
use anyhow::{anyhow, Result};
use std::collections::VecDeque;
use tracing::info;

/// Task queue statistics
#[derive(Debug, Clone)]
pub struct QueueStats {
    /// Number of pending tasks
    pub pending_count: usize,
    
    /// Number of active tasks
    pub active_count: usize,
    
    /// Total tasks processed
    pub total_processed: u64,
    
    /// Total tasks rejected
    pub total_rejected: u64,
}

/// Task queue for validation tasks
pub struct TaskQueue {
    /// Pending tasks
    pending: VecDeque<ValidationTask>,
    
    /// Maximum pending tasks
    max_pending: usize,
    
    /// Active task count
    active_count: usize,
    
    /// Maximum concurrent tasks
    max_concurrent: usize,
    
    /// Total tasks processed
    total_processed: u64,
    
    /// Total tasks rejected
    total_rejected: u64,
}

impl TaskQueue {
    /// Create a new task queue
    pub fn new(max_pending: usize, max_concurrent: usize) -> Self {
        info!(
            "📋 Task queue initialized: max_pending={}, max_concurrent={}",
            max_pending, max_concurrent
        );
        
        TaskQueue {
            pending: VecDeque::new(),
            max_pending,
            active_count: 0,
            max_concurrent,
            total_processed: 0,
            total_rejected: 0,
        }
    }

    /// Add a task to the queue
    pub fn enqueue(&mut self, task: ValidationTask) -> Result<()> {
        if self.pending.len() >= self.max_pending {
            self.total_rejected += 1;
            return Err(anyhow!(
                "Task queue full: {} pending tasks (max: {})",
                self.pending.len(),
                self.max_pending
            ));
        }

        self.pending.push_back(task);
        Ok(())
    }

    /// Get next task from queue
    pub fn dequeue(&mut self) -> Option<ValidationTask> {
        if !self.can_process_more() {
            return None;
        }

        if let Some(task) = self.pending.pop_front() {
            self.active_count += 1;
            Some(task)
        } else {
            None
        }
    }

    /// Mark a task as completed
    pub fn complete_task(&mut self) {
        if self.active_count > 0 {
            self.active_count -= 1;
            self.total_processed += 1;
        }
    }

    /// Check if we can process more tasks
    pub fn can_process_more(&self) -> bool {
        self.active_count < self.max_concurrent && !self.pending.is_empty()
    }

    /// Get number of pending tasks
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get number of active tasks
    pub fn active_count(&self) -> usize {
        self.active_count
    }

    /// Get queue statistics
    pub fn get_stats(&self) -> QueueStats {
        QueueStats {
            pending_count: self.pending.len(),
            active_count: self.active_count,
            total_processed: self.total_processed,
            total_rejected: self.total_rejected,
        }
    }

    /// Clear all pending tasks
    pub fn clear(&mut self) {
        self.pending.clear();
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
    fn test_task_queue_creation() {
        let queue = TaskQueue::new(10, 3);
        assert_eq!(queue.pending_count(), 0);
        assert_eq!(queue.active_count(), 0);
    }

    #[test]
    fn test_enqueue_dequeue() {
        let mut queue = TaskQueue::new(10, 3);

        let task = create_test_task(1);
        assert!(queue.enqueue(task).is_ok());
        assert_eq!(queue.pending_count(), 1);

        let task = queue.dequeue();
        assert!(task.is_some());
        assert_eq!(queue.pending_count(), 0);
        assert_eq!(queue.active_count(), 1);
    }

    #[test]
    fn test_queue_full() {
        let mut queue = TaskQueue::new(2, 3);

        // Fill queue
        queue.enqueue(create_test_task(1)).unwrap();
        queue.enqueue(create_test_task(2)).unwrap();

        // Should reject
        let result = queue.enqueue(create_test_task(3));
        assert!(result.is_err());
    }

    #[test]
    fn test_max_concurrent() {
        let mut queue = TaskQueue::new(10, 2);

        // Add tasks
        queue.enqueue(create_test_task(1)).unwrap();
        queue.enqueue(create_test_task(2)).unwrap();
        queue.enqueue(create_test_task(3)).unwrap();

        // Dequeue up to max concurrent
        assert!(queue.dequeue().is_some());
        assert!(queue.dequeue().is_some());
        assert!(queue.dequeue().is_none()); // Max concurrent reached

        // Complete one task
        queue.complete_task();

        // Should be able to dequeue again
        assert!(queue.dequeue().is_some());
    }
}
