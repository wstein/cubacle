use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, LazyLock, Mutex};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

thread_local! {
    static IN_POOL_WORKER: Cell<bool> = const { Cell::new(false) };
}

/// A small, process-lifetime worker pool used by the opt-in parallel APIs.
pub(super) struct ThreadPool {
    sender: Sender<Job>,
    worker_count: usize,
    _workers: Vec<JoinHandle<()>>,
}

impl ThreadPool {
    fn new(worker_count: usize) -> Self {
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(worker_count);

        for index in 0..worker_count {
            let receiver = Arc::clone(&receiver);
            workers.push(
                thread::Builder::new()
                    .name(format!("cubacle-{index}"))
                    .spawn(move || worker_loop(receiver))
                    .expect("failed to start cubacle worker"),
            );
        }

        Self {
            sender,
            worker_count,
            _workers: workers,
        }
    }

    pub(super) fn execute(&self, job: impl FnOnce() + Send + 'static) {
        self.sender
            .send(Box::new(job))
            .expect("cubacle worker pool unexpectedly stopped");
    }

    fn worker_count(&self) -> usize {
        self.worker_count
    }
}

fn worker_loop(receiver: Arc<Mutex<Receiver<Job>>>) {
    loop {
        let job = match receiver.lock().expect("worker queue mutex poisoned").recv() {
            Ok(job) => job,
            Err(_) => return,
        };
        IN_POOL_WORKER.with(|in_worker| in_worker.set(true));
        // The panic hook has already reported the panic; callers see a missing result.
        let _ = catch_unwind(AssertUnwindSafe(job));
        IN_POOL_WORKER.with(|in_worker| in_worker.set(false));
    }
}

fn configured_worker_count() -> usize {
    std::env::var("CUBACLE_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|&count| count > 0)
        .or_else(|| std::thread::available_parallelism().ok().map(usize::from))
        .unwrap_or(1)
}

static GLOBAL_POOL: LazyLock<ThreadPool> =
    LazyLock::new(|| ThreadPool::new(configured_worker_count()));

pub(super) fn global() -> &'static ThreadPool {
    &GLOBAL_POOL
}

pub(super) fn worker_count() -> usize {
    GLOBAL_POOL.worker_count()
}

pub(super) fn in_pool_worker() -> bool {
    IN_POOL_WORKER.with(Cell::get)
}
