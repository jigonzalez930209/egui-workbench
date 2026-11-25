//! Cooperative background jobs for geometry, conformers, and surface generation.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};

/// A progress snapshot a job publishes for the UI.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    /// Completed units.
    pub done: usize,
    /// Total units, or `0` when the job cannot estimate it.
    pub total: usize,
    /// Short status line.
    pub message: String,
}

impl Progress {
    /// Completion in `0..=1`, or `None` when the total is unknown.
    pub fn fraction(&self) -> Option<f32> {
        (self.total > 0).then(|| (self.done as f32 / self.total as f32).clamp(0.0, 1.0))
    }
}

/// A shared cancellation flag.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// A fresh token.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ask the job to stop at its next checkpoint.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Whether cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// Why a job failed.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum JobError {
    /// The job observed a cancellation request.
    Cancelled,
    /// The job failed for another reason.
    Failed(String),
}

impl std::fmt::Display for JobError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => formatter.write_str("job cancelled"),
            Self::Failed(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for JobError {}

/// The context a running job receives.
#[derive(Clone, Debug)]
pub struct JobContext {
    token: CancellationToken,
    progress: Arc<Mutex<Progress>>,
}

impl JobContext {
    /// Whether the job should stop.
    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    /// The cancellation token, for passing into a blocking helper.
    pub fn token(&self) -> &CancellationToken {
        &self.token
    }

    /// Publish progress.
    pub fn set_progress(&self, done: usize, total: usize, message: impl Into<String>) {
        if let Ok(mut progress) = self.progress.lock() {
            *progress = Progress {
                done,
                total,
                message: message.into(),
            };
        }
    }

    /// The current progress snapshot.
    pub fn progress(&self) -> Progress {
        self.progress
            .lock()
            .map(|progress| progress.clone())
            .unwrap_or_default()
    }

    /// Return [`JobError::Cancelled`] when cancellation was requested.
    ///
    /// # Errors
    /// When the token was cancelled.
    pub fn check_cancelled(&self) -> Result<(), JobError> {
        if self.is_cancelled() {
            Err(JobError::Cancelled)
        } else {
            Ok(())
        }
    }
}

/// A unit of background work.
pub trait Job: Send + 'static {
    /// What the job produces.
    type Output: Send + 'static;

    /// Run the job. Call [`JobContext::check_cancelled`] at each checkpoint.
    ///
    /// # Errors
    /// When the job is cancelled or fails.
    fn run(self, context: &JobContext) -> Result<Self::Output, JobError>;
}

/// A handle to a running job.
#[derive(Debug)]
pub struct JobHandle<T> {
    receiver: Receiver<Result<T, JobError>>,
    token: CancellationToken,
    progress: Arc<Mutex<Progress>>,
}

impl<T> JobHandle<T> {
    /// Ask the job to stop. A job that ignores the token runs to completion.
    pub fn cancel(&self) {
        self.token.cancel();
    }

    /// Whether cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    /// The latest progress snapshot.
    pub fn progress(&self) -> Progress {
        self.progress
            .lock()
            .map(|progress| progress.clone())
            .unwrap_or_default()
    }

    /// Take the result if the job already finished.
    pub fn try_finish(&mut self) -> Option<Result<T, JobError>> {
        match self.receiver.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err(JobError::Failed(
                "the job thread stopped without a result".to_owned(),
            ))),
        }
    }

    /// Block until the job finishes.
    ///
    /// # Errors
    /// When the job was cancelled or failed.
    pub fn wait(self) -> Result<T, JobError> {
        self.receiver
            .recv()
            .unwrap_or_else(|_| Err(JobError::Failed("the job thread stopped without a result".to_owned())))
    }
}

/// Spawns jobs on detached threads.
#[derive(Clone, Debug, Default)]
pub struct JobQueue;

impl JobQueue {
    /// A queue that starts one detached thread per job.
    pub fn new() -> Self {
        Self
    }

    /// Spawn `job` and return its handle.
    pub fn spawn<J: Job>(&self, job: J) -> JobHandle<J::Output> {
        let token = CancellationToken::new();
        let progress = Arc::new(Mutex::new(Progress::default()));
        let (sender, receiver) = mpsc::channel();
        let context = JobContext {
            token: token.clone(),
            progress: Arc::clone(&progress),
        };
        let result = std::thread::Builder::new()
            .name("egui-workbench-chem-job".to_owned())
            .spawn(move || {
                let result = job.run(&context);
                let _ = sender.send(result);
            });
        if let Err(error) = result {
            eprintln!("chem job did not start: {error}");
        }
        JobHandle {
            receiver,
            token,
            progress,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct Add(i32, i32);

    impl Job for Add {
        type Output = i32;

        fn run(self, context: &JobContext) -> Result<Self::Output, JobError> {
            context.set_progress(1, 1, "adding");
            Ok(self.0 + self.1)
        }
    }

    struct Spin;

    impl Job for Spin {
        type Output = ();

        fn run(self, context: &JobContext) -> Result<Self::Output, JobError> {
            while !context.is_cancelled() {
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(JobError::Cancelled)
        }
    }

    #[test]
    fn a_job_returns_its_output_and_progress() {
        let queue = JobQueue::new();
        let mut handle = queue.spawn(Add(2, 3));
        let mut result = None;
        for _ in 0..100 {
            if let Some(value) = handle.try_finish() {
                result = Some(value);
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(result.expect("result").expect("ok"), 5);
        assert_eq!(handle.progress().fraction(), Some(1.0));
    }

    #[test]
    fn cancellation_is_cooperative() {
        let queue = JobQueue::new();
        let handle = queue.spawn(Spin);
        std::thread::sleep(Duration::from_millis(5));
        handle.cancel();
        assert_eq!(handle.wait(), Err(JobError::Cancelled));
    }
}
