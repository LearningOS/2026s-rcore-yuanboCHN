//! Synchronization and interior mutability primitives

mod condvar;
mod mutex;
mod semaphore;
mod up;

pub use condvar::Condvar;
pub use mutex::{Mutex, MutexBlocking, MutexSpin};
pub use semaphore::Semaphore;
pub use up::UPSafeCell;

use alloc::vec;
use alloc::vec::Vec;

/// Return code reported by a blocking acquire when deadlock is detected.
pub const DEADLOCK_RET: isize = -0xdead;

/// Per-process state used by the Banker's-algorithm deadlock detector.
///
/// Every semaphore is treated as a reusable resource type; its "instance
/// count" is the value passed to `semaphore_create`. Every thread is a process
/// in the Banker's algorithm. `mutex_owner` additionally records the owner of
/// each blocking mutex so that self-deadlock (locking a mutex one already
/// holds) can be detected directly.
///
/// This is stored per-process (inside the process control block) because the
/// deadlock tests are run concurrently as separate processes; a global table
/// would let their resource accounting interfere with each other.
pub struct DeadlockDetector {
    /// whether detection is enabled
    pub enabled: bool,
    /// semaphore id -> total instance count
    pub total: Vec<isize>,
    /// thread tid -> semaphore id -> currently held instances
    pub allocation: Vec<Vec<isize>>,
    /// thread tid -> semaphore id -> requested (blocked on) instances
    pub need: Vec<Vec<isize>>,
    /// mutex id -> owning tid (-1 means unlocked)
    pub mutex_owner: Vec<isize>,
}

impl DeadlockDetector {
    /// Create an empty, disabled detector.
    pub fn new() -> Self {
        DeadlockDetector {
            enabled: false,
            total: Vec::new(),
            allocation: Vec::new(),
            need: Vec::new(),
            mutex_owner: Vec::new(),
        }
    }

    /// Enable / disable detection; enabling resets all accumulated state.
    pub fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            self.total.clear();
            self.allocation.clear();
            self.need.clear();
            self.mutex_owner.clear();
        }
    }

    /// Is detection currently enabled?
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Register a semaphore with its total instance count.
    pub fn semaphore_register(&mut self, sem_id: usize, total: isize) {
        if !self.enabled {
            return;
        }
        ensure_len(&mut self.total, sem_id + 1, 0);
        self.total[sem_id] = total;
    }

    /// Record that `tid` successfully acquired one instance of `sem_id`.
    pub fn semaphore_acquire(&mut self, tid: usize, sem_id: usize) {
        if !self.enabled || sem_id == 0 {
            return;
        }
        let n = self.total.len();
        if sem_id >= n {
            return;
        }
        ensure_len(&mut self.allocation, tid + 1, Vec::new());
        ensure_len(&mut self.need, tid + 1, Vec::new());
        ensure_len(&mut self.allocation[tid], n, 0);
        ensure_len(&mut self.need[tid], n, 0);
        if self.need[tid][sem_id] > 0 {
            self.need[tid][sem_id] -= 1;
        }
        self.allocation[tid][sem_id] += 1;
    }

    /// Record that `tid` released one instance of `sem_id`.
    pub fn semaphore_release(&mut self, tid: usize, sem_id: usize) {
        if !self.enabled || sem_id == 0 {
            return;
        }
        ensure_len(&mut self.total, sem_id + 1, 0);
        let n = self.total.len();
        ensure_len(&mut self.allocation, tid + 1, Vec::new());
        ensure_len(&mut self.need, tid + 1, Vec::new());
        ensure_len(&mut self.allocation[tid], n, 0);
        ensure_len(&mut self.need[tid], n, 0);
        if self.allocation[tid][sem_id] > 0 {
            self.allocation[tid][sem_id] -= 1;
        } else if self.need[tid][sem_id] > 0 {
            self.need[tid][sem_id] -= 1;
        }
    }

    /// `tid` would like to acquire `sem_id` but it is currently unavailable.
    /// Returns `false` if granting this request would lead to deadlock
    /// (no safe sequence exists); otherwise records the request and returns
    /// true.
    pub fn semaphore_request(&mut self, tid: usize, sem_id: usize) -> bool {
        if !self.enabled {
            return true;
        }
        // `sem_id == 0` is the barrier semaphore used for thread
        // synchronization: it is not a resource and must never participate in
        // deadlock detection.
        if sem_id == 0 {
            return true;
        }
        ensure_len(&mut self.total, sem_id + 1, 0);
        let n = self.total.len();
        ensure_len(&mut self.allocation, tid + 1, Vec::new());
        ensure_len(&mut self.need, tid + 1, Vec::new());
        for t in 0..self.allocation.len() {
            ensure_len(&mut self.allocation[t], n, 0);
            ensure_len(&mut self.need[t], n, 0);
        }
        // compute available = total - sum(allocation)
        let mut available = vec![0isize; n];
        for r in 0..n {
            let used: isize = self.allocation.iter().map(|a| a[r]).sum();
            available[r] = self.total[r] - used;
        }
        // An instance is available right now: it will be granted immediately by
        // `semaphore_acquire` after `down()` returns.
        if available[sem_id] > 0 {
            return true;
        }
        // No instance is available: the caller will block. Tentatively add the
        // request to `need` and run Banker's safety algorithm.
        self.need[tid][sem_id] += 1;
        let safe = banker_is_safe(&self.allocation, &self.need, &self.total);
        if !safe {
            self.need[tid][sem_id] -= 1;
            return false;
        }
        true
    }

    /// Register a mutex id (extend the owner table).
    pub fn mutex_register(&mut self, mutex_id: usize) {
        ensure_len(&mut self.mutex_owner, mutex_id + 1, -1);
    }

    /// Check whether `tid` may attempt to lock `mutex_id` without causing a
    /// self-deadlock. Returns `false` if the mutex is already owned by `tid`.
    pub fn mutex_try_lock(&mut self, tid: usize, mutex_id: usize) -> bool {
        ensure_len(&mut self.mutex_owner, mutex_id + 1, -1);
        if self.mutex_owner[mutex_id] == tid as isize {
            return false;
        }
        true
    }

    /// Record that `tid` successfully acquired `mutex_id`.
    pub fn mutex_acquire(&mut self, tid: usize, mutex_id: usize) {
        ensure_len(&mut self.mutex_owner, mutex_id + 1, -1);
        self.mutex_owner[mutex_id] = tid as isize;
    }

    /// Release `mutex_id` (owner becomes -1).
    pub fn mutex_unlock(&mut self, mutex_id: usize) {
        ensure_len(&mut self.mutex_owner, mutex_id + 1, -1);
        self.mutex_owner[mutex_id] = -1;
    }
}

/// Helper: ensure `v` has at least `n` entries, padding with `fill`.
fn ensure_len<T: Clone>(v: &mut Vec<T>, n: usize, fill: T) {
    while v.len() < n {
        v.push(fill.clone());
    }
}

/// Banker's safety algorithm: is there a safe completion sequence?
fn banker_is_safe(allocation: &[Vec<isize>], need: &[Vec<isize>], total: &[isize]) -> bool {
    let n_thread = allocation.len();
    let n_res = total.len();
    // available = total - sum of allocations
    let mut work = total.to_vec();
    for t in 0..n_thread {
        for r in 0..n_res {
            work[r] -= allocation[t][r];
        }
    }
    let mut finish = vec![false; n_thread];
    loop {
        let mut progressed = false;
        for t in 0..n_thread {
            if finish[t] {
                continue;
            }
            let can_finish = (0..n_res).all(|r| need[t][r] <= work[r]);
            if can_finish {
                for r in 0..n_res {
                    work[r] += allocation[t][r];
                }
                finish[t] = true;
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    finish.iter().all(|&f| f)
}
