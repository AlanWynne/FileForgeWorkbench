//! Workflow persistence and checkpointing.
//!
//! Supports serialization of workflow state for pause/resume and
//! application restart survival for long-running operations.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::WorkflowError;
use crate::progress::WorkflowExecutionId;
use crate::state::WorkflowState;

/// Current schema version for checkpoint files.
pub const CHECKPOINT_SCHEMA_VERSION: u32 = 1;

/// A serialized snapshot of a workflow's execution state for resumption.
///
/// Addresses: Requirement 7, criteria 1/2/3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Unique execution ID.
    pub execution_id: WorkflowExecutionId,
    /// Workflow name (for looking up the definition on resume).
    pub workflow_name: String,
    /// The full workflow state at checkpoint time.
    pub state: WorkflowState,
    /// Timestamp when checkpoint was created (RFC 3339).
    pub created_at: String,
    /// Schema version (for detecting incompatible checkpoints).
    pub schema_version: u32,
}

impl Checkpoint {
    /// Creates a new checkpoint from the current workflow state.
    pub fn from_state(state: &WorkflowState) -> Self {
        Self {
            execution_id: state.execution_id.clone(),
            workflow_name: state.workflow_name.clone(),
            state: state.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            schema_version: CHECKPOINT_SCHEMA_VERSION,
        }
    }
}

/// Manages workflow checkpoint persistence: save, load, cleanup.
///
/// Addresses: Requirement 7, criteria 2/3/4/6/7
pub struct CheckpointManager {
    storage_directory: PathBuf,
}

impl CheckpointManager {
    /// Creates a new checkpoint manager with the given storage directory.
    pub fn new(storage_directory: PathBuf) -> Self {
        Self { storage_directory }
    }

    /// Returns the storage directory.
    pub fn storage_directory(&self) -> &PathBuf {
        &self.storage_directory
    }

    /// Saves a workflow checkpoint to storage.
    ///
    /// Addresses: Requirement 7, criterion 2
    pub async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<(), WorkflowError> {
        let path = self.checkpoint_path(&checkpoint.execution_id);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(WorkflowError::Io)?;
        }
        let json = serde_json::to_string_pretty(checkpoint).map_err(|e| {
            WorkflowError::CheckpointError {
                operation: "serialize".to_string(),
                execution_id: checkpoint.execution_id.0.clone(),
                description: e.to_string(),
            }
        })?;
        tokio::fs::write(&path, json)
            .await
            .map_err(WorkflowError::Io)?;
        Ok(())
    }

    /// Loads a checkpoint by execution ID.
    ///
    /// Addresses: Requirement 7, criterion 5
    pub async fn load_checkpoint(
        &self,
        execution_id: &WorkflowExecutionId,
    ) -> Result<Option<Checkpoint>, WorkflowError> {
        let path = self.checkpoint_path(execution_id);
        if !path.exists() {
            return Ok(None);
        }
        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(WorkflowError::Io)?;
        let checkpoint: Checkpoint =
            serde_json::from_str(&content).map_err(|e| WorkflowError::CheckpointError {
                operation: "deserialize".to_string(),
                execution_id: execution_id.0.clone(),
                description: e.to_string(),
            })?;

        // Check schema version
        if checkpoint.schema_version != CHECKPOINT_SCHEMA_VERSION {
            return Err(WorkflowError::CheckpointSchemaMismatch {
                execution_id: execution_id.0.clone(),
                expected: CHECKPOINT_SCHEMA_VERSION,
                found: checkpoint.schema_version,
            });
        }

        Ok(Some(checkpoint))
    }

    /// Scans for all incomplete (resumable) checkpoints.
    ///
    /// Addresses: Requirement 7, criterion 4
    pub async fn scan_resumable(&self) -> Result<Vec<Checkpoint>, WorkflowError> {
        let mut results = Vec::new();
        if !self.storage_directory.exists() {
            return Ok(results);
        }

        let mut entries = tokio::fs::read_dir(&self.storage_directory)
            .await
            .map_err(WorkflowError::Io)?;

        while let Some(entry) = entries.next_entry().await.map_err(WorkflowError::Io)? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => {
                        if let Ok(checkpoint) = serde_json::from_str::<Checkpoint>(&content) {
                            if checkpoint.schema_version == CHECKPOINT_SCHEMA_VERSION {
                                results.push(checkpoint);
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
        }

        Ok(results)
    }

    /// Removes a checkpoint (after successful completion).
    ///
    /// Addresses: Requirement 7, criterion 7
    pub async fn remove_checkpoint(
        &self,
        execution_id: &WorkflowExecutionId,
    ) -> Result<(), WorkflowError> {
        let path = self.checkpoint_path(execution_id);
        if path.exists() {
            tokio::fs::remove_file(&path)
                .await
                .map_err(WorkflowError::Io)?;
        }
        Ok(())
    }

    /// Cleans up expired checkpoints older than the retention period.
    ///
    /// Addresses: Requirement 7, criterion 7
    pub async fn cleanup_expired(&self, retention_days: u32) -> Result<u32, WorkflowError> {
        let mut removed = 0u32;
        if !self.storage_directory.exists() {
            return Ok(0);
        }

        let cutoff = chrono::Utc::now() - chrono::Duration::days(i64::from(retention_days));

        let mut entries = tokio::fs::read_dir(&self.storage_directory)
            .await
            .map_err(WorkflowError::Io)?;

        while let Some(entry) = entries.next_entry().await.map_err(WorkflowError::Io)? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                if let Ok(checkpoint) = serde_json::from_str::<Checkpoint>(&content) {
                    if let Ok(created) =
                        chrono::DateTime::parse_from_rfc3339(&checkpoint.created_at)
                    {
                        if created < cutoff {
                            let _ = tokio::fs::remove_file(&path).await;
                            removed += 1;
                        }
                    }
                }
            }
        }

        Ok(removed)
    }

    /// Returns the file path for a checkpoint.
    fn checkpoint_path(&self, execution_id: &WorkflowExecutionId) -> PathBuf {
        self.storage_directory
            .join(format!("{}.json", execution_id.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::WorkflowContext;
    use crate::state::WorkflowState;

    fn make_test_state() -> WorkflowState {
        WorkflowState::new(
            WorkflowExecutionId("test-123".to_string()),
            "test-wf".to_string(),
            "step1".to_string(),
            vec!["step1".to_string(), "step2".to_string()],
            WorkflowContext::new(),
        )
    }

    // Validates: Requirement 7.2 — checkpoint serialization

    #[test]
    fn checkpoint_from_state_has_correct_fields() {
        let state = make_test_state();
        let cp = Checkpoint::from_state(&state);
        assert_eq!(cp.execution_id.0, "test-123");
        assert_eq!(cp.workflow_name, "test-wf");
        assert_eq!(cp.schema_version, CHECKPOINT_SCHEMA_VERSION);
    }

    #[test]
    fn checkpoint_serialization_round_trip() {
        let state = make_test_state();
        let cp = Checkpoint::from_state(&state);
        let json = serde_json::to_string(&cp).expect("serialize");
        let restored: Checkpoint = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.execution_id, cp.execution_id);
        assert_eq!(restored.workflow_name, cp.workflow_name);
        assert_eq!(restored.schema_version, cp.schema_version);
    }

    // Validates: Requirement 7.3 — checkpoint storage

    #[tokio::test]
    async fn save_and_load_checkpoint() {
        let dir = tempfile::tempdir().unwrap();
        let manager = CheckpointManager::new(dir.path().to_path_buf());

        let state = make_test_state();
        let cp = Checkpoint::from_state(&state);

        manager.save_checkpoint(&cp).await.unwrap();

        let loaded = manager
            .load_checkpoint(&WorkflowExecutionId("test-123".to_string()))
            .await
            .unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.execution_id.0, "test-123");
    }

    #[tokio::test]
    async fn load_nonexistent_checkpoint_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let manager = CheckpointManager::new(dir.path().to_path_buf());

        let loaded = manager
            .load_checkpoint(&WorkflowExecutionId("no-such-id".to_string()))
            .await
            .unwrap();
        assert!(loaded.is_none());
    }

    // Validates: Requirement 7.7 — checkpoint cleanup

    #[tokio::test]
    async fn remove_checkpoint_deletes_file() {
        let dir = tempfile::tempdir().unwrap();
        let manager = CheckpointManager::new(dir.path().to_path_buf());

        let state = make_test_state();
        let cp = Checkpoint::from_state(&state);
        manager.save_checkpoint(&cp).await.unwrap();
        manager
            .remove_checkpoint(&WorkflowExecutionId("test-123".to_string()))
            .await
            .unwrap();

        let loaded = manager
            .load_checkpoint(&WorkflowExecutionId("test-123".to_string()))
            .await
            .unwrap();
        assert!(loaded.is_none());
    }

    // Validates: Requirement 7.4 — scan resumable

    #[tokio::test]
    async fn scan_resumable_finds_checkpoints() {
        let dir = tempfile::tempdir().unwrap();
        let manager = CheckpointManager::new(dir.path().to_path_buf());

        let state1 = WorkflowState::new(
            WorkflowExecutionId("id-1".to_string()),
            "wf".to_string(),
            "s".to_string(),
            vec!["s".to_string()],
            WorkflowContext::new(),
        );
        let state2 = WorkflowState::new(
            WorkflowExecutionId("id-2".to_string()),
            "wf".to_string(),
            "s".to_string(),
            vec!["s".to_string()],
            WorkflowContext::new(),
        );

        manager
            .save_checkpoint(&Checkpoint::from_state(&state1))
            .await
            .unwrap();
        manager
            .save_checkpoint(&Checkpoint::from_state(&state2))
            .await
            .unwrap();

        let resumable = manager.scan_resumable().await.unwrap();
        assert_eq!(resumable.len(), 2);
    }

    #[tokio::test]
    async fn scan_resumable_empty_dir_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let manager = CheckpointManager::new(dir.path().to_path_buf());
        let resumable = manager.scan_resumable().await.unwrap();
        assert!(resumable.is_empty());
    }
}
