//! Initiator pool — manages a configurable set of async worker slots.

use crate::error::JesError;
use crate::model::{InitiatorId, InitiatorStatus, JobId};

/// A single initiator (worker slot) in the pool.
///
/// Validates: Requirement 4
#[derive(Debug)]
pub struct Initiator {
    /// Unique initiator identifier.
    pub id: InitiatorId,
    /// Current status.
    pub status: InitiatorStatus,
    /// Currently assigned job (if Active or Draining).
    pub current_job: Option<JobId>,
    /// Number of jobs completed by this initiator.
    pub jobs_completed: u64,
    /// Last error message (if Failed).
    pub last_error: Option<String>,
}

impl Initiator {
    fn new(id: InitiatorId) -> Self {
        Self {
            id,
            status: InitiatorStatus::Idle,
            current_job: None,
            jobs_completed: 0,
            last_error: None,
        }
    }
}

/// Manages a configurable pool of initiator workers.
///
/// Validates: Requirement 4
pub struct InitiatorPool {
    initiators: Vec<Initiator>,
    capacity: usize,
}

impl InitiatorPool {
    /// Creates a new pool with the specified capacity.
    ///
    /// Validates: Requirement 4 AC 1
    pub fn new(capacity: usize) -> Self {
        let initiators = (1..=capacity)
            .map(|i| Initiator::new(InitiatorId::new(i as u32)))
            .collect();
        Self {
            initiators,
            capacity,
        }
    }

    /// Returns the ID of the first available (Idle) initiator.
    ///
    /// Validates: Requirement 3 AC 7
    pub fn get_available(&self) -> Option<InitiatorId> {
        self.initiators
            .iter()
            .find(|i| i.status == InitiatorStatus::Idle)
            .map(|i| i.id)
    }

    /// Assigns a job to a specific initiator, marking it Active.
    ///
    /// Validates: Requirement 3 AC 6
    pub fn assign(&mut self, id: InitiatorId, job_id: JobId) -> Result<(), JesError> {
        let initiator = self
            .initiators
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or_else(|| JesError::Internal(format!("initiator {id} not found")))?;

        if initiator.status != InitiatorStatus::Idle {
            return Err(JesError::Internal(format!(
                "initiator {id} is not idle (status: {})",
                initiator.status
            )));
        }

        initiator.status = InitiatorStatus::Active;
        initiator.current_job = Some(job_id);
        Ok(())
    }

    /// Releases an initiator back to Idle after job completion.
    ///
    /// Validates: Requirement 6 AC 5
    pub fn release(&mut self, id: InitiatorId) -> Result<(), JesError> {
        let initiator = self
            .initiators
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or_else(|| JesError::Internal(format!("initiator {id} not found")))?;

        initiator.jobs_completed += 1;
        initiator.current_job = None;
        initiator.status = InitiatorStatus::Idle;
        Ok(())
    }

    /// Marks an initiator as Failed.
    ///
    /// Validates: Requirement 4 AC 8
    pub fn mark_failed(&mut self, id: InitiatorId, reason: String) -> Result<(), JesError> {
        let initiator = self
            .initiators
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or_else(|| JesError::Internal(format!("initiator {id} not found")))?;

        initiator.status = InitiatorStatus::Failed;
        initiator.current_job = None;
        initiator.last_error = Some(reason);
        Ok(())
    }

    /// Starts a specific initiator (Stopped → Idle).
    ///
    /// Validates: Requirement 4 AC 4
    pub fn start_initiator(&mut self, id: InitiatorId) -> Result<(), JesError> {
        let initiator = self
            .initiators
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or_else(|| JesError::Internal(format!("initiator {id} not found")))?;

        initiator.status = InitiatorStatus::Idle;
        Ok(())
    }

    /// Stops a specific initiator (marks as Stopping; caller must wait for job completion).
    ///
    /// Validates: Requirement 4 AC 5
    pub fn stop_initiator(&mut self, id: InitiatorId) -> Result<(), JesError> {
        let initiator = self
            .initiators
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or_else(|| JesError::Internal(format!("initiator {id} not found")))?;

        match initiator.status {
            InitiatorStatus::Idle => initiator.status = InitiatorStatus::Stopped,
            InitiatorStatus::Active => initiator.status = InitiatorStatus::Stopping,
            _ => {}
        }
        Ok(())
    }

    /// Drains a specific initiator (finishes current job, accepts no new work).
    ///
    /// Validates: Requirement 4 AC 6
    pub fn drain_initiator(&mut self, id: InitiatorId) -> Result<(), JesError> {
        let initiator = self
            .initiators
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or_else(|| JesError::Internal(format!("initiator {id} not found")))?;

        if initiator.status == InitiatorStatus::Active {
            initiator.status = InitiatorStatus::Draining;
        }
        Ok(())
    }

    /// Returns the status of all initiators.
    ///
    /// Validates: Requirement 4 AC 3
    pub fn status_all(&self) -> &[Initiator] {
        &self.initiators
    }

    /// Returns the configured capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the count of currently active initiators.
    pub fn active_count(&self) -> usize {
        self.initiators
            .iter()
            .filter(|i| {
                matches!(
                    i.status,
                    InitiatorStatus::Active | InitiatorStatus::Draining
                )
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool_has_all_idle_initiators() {
        // Validates: Requirement 4 AC 1
        let pool = InitiatorPool::new(3);
        assert_eq!(pool.capacity(), 3);
        assert_eq!(pool.status_all().len(), 3);
        for init in pool.status_all() {
            assert_eq!(init.status, InitiatorStatus::Idle);
        }
    }

    #[test]
    fn get_available_returns_idle_initiator() {
        let pool = InitiatorPool::new(2);
        let id = pool.get_available();
        assert!(id.is_some());
    }

    #[test]
    fn assign_marks_initiator_active() {
        // Validates: Requirement 3 AC 6
        let mut pool = InitiatorPool::new(2);
        let id = pool.get_available().unwrap();
        pool.assign(id, JobId::new(1)).unwrap();
        let init = pool.status_all().iter().find(|i| i.id == id).unwrap();
        assert_eq!(init.status, InitiatorStatus::Active);
        assert_eq!(init.current_job, Some(JobId::new(1)));
    }

    #[test]
    fn release_returns_initiator_to_idle() {
        // Validates: Requirement 6 AC 5
        let mut pool = InitiatorPool::new(2);
        let id = pool.get_available().unwrap();
        pool.assign(id, JobId::new(1)).unwrap();
        pool.release(id).unwrap();
        let init = pool.status_all().iter().find(|i| i.id == id).unwrap();
        assert_eq!(init.status, InitiatorStatus::Idle);
        assert_eq!(init.current_job, None);
        assert_eq!(init.jobs_completed, 1);
    }

    #[test]
    fn no_available_when_all_active() {
        // Validates: Requirement 3 AC 7
        let mut pool = InitiatorPool::new(2);
        let id1 = pool.get_available().unwrap();
        pool.assign(id1, JobId::new(1)).unwrap();
        let id2 = pool.get_available().unwrap();
        pool.assign(id2, JobId::new(2)).unwrap();
        assert!(pool.get_available().is_none());
    }

    #[test]
    fn mark_failed_sets_failed_status() {
        // Validates: Requirement 4 AC 8
        let mut pool = InitiatorPool::new(1);
        let id = pool.get_available().unwrap();
        pool.assign(id, JobId::new(1)).unwrap();
        pool.mark_failed(id, "unrecoverable error".to_string())
            .unwrap();
        let init = pool.status_all().iter().find(|i| i.id == id).unwrap();
        assert_eq!(init.status, InitiatorStatus::Failed);
        assert!(init.last_error.is_some());
    }

    #[test]
    fn drain_active_initiator_sets_draining() {
        // Validates: Requirement 4 AC 6
        let mut pool = InitiatorPool::new(1);
        let id = pool.get_available().unwrap();
        pool.assign(id, JobId::new(1)).unwrap();
        pool.drain_initiator(id).unwrap();
        let init = pool.status_all().iter().find(|i| i.id == id).unwrap();
        assert_eq!(init.status, InitiatorStatus::Draining);
    }

    #[test]
    fn active_count_tracks_correctly() {
        let mut pool = InitiatorPool::new(3);
        assert_eq!(pool.active_count(), 0);
        let id1 = pool.get_available().unwrap();
        pool.assign(id1, JobId::new(1)).unwrap();
        assert_eq!(pool.active_count(), 1);
        let id2 = pool.get_available().unwrap();
        pool.assign(id2, JobId::new(2)).unwrap();
        assert_eq!(pool.active_count(), 2);
        pool.release(id1).unwrap();
        assert_eq!(pool.active_count(), 1);
    }
}
