//! Thread pool that joins all thread when dropped.

#![allow(clippy::mutex_atomic)]

// NOTE: Crossbeam channels are MPMC, which means that you don't need to wrap the receiver in
// Arc<Mutex<..>>. Just clone the receiver and give it to each worker thread.
use crossbeam_channel::{unbounded, Sender, Receiver};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

struct Job(Box<dyn FnOnce() + Send + 'static>);

#[derive(Debug)]
struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

imple Worker {
    fn new(id: usize, receiver: Receiver) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();

            match job {
                Ok(Job(f)) => {
                    println!("Worker {} got a job; executing.", id);
                    f();
                }
                Err(_) => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });
    }
}

impl Drop for Worker {
    /// When dropped, the thread's `JoinHandle` must be `join`ed.  If the worker panics, then this
    /// function should panic too.  NOTE: that the thread is detached if not `join`ed explicitly.
    fn drop(&mut self) {
        println!("Shutting down worker {}", self.id);

        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

/// Internal data structure for tracking the current job status. This is shared by the worker
/// closures via `Arc` so that the workers can report to the pool that it started/finished a job.
#[derive(Debug, Default)]
struct ThreadPoolInner {
    job_count: Mutex<usize>,
    empty_condvar: Condvar,
}

impl ThreadPoolInner {
    /// Increment the job count.
    fn start_job(&self) {
        let mut count = self.job_count.lock().unwrap();
        *count += 1;
        //todo!()
    }

    /// Decrement the job count.
    fn finish_job(&self) {
        let mut count = self.job_count.lock().unwrap();
        *count -= 1;
        if count == 0 {self.empty_condvar.notify_one();}
        //todo!()
    }

    /// Wait until the job count becomes 0.
    ///
    /// NOTE: We can optimize this function by adding another field to `ThreadPoolInner`, but let's
    /// not care about that in this homework.
    fn wait_empty(&self) {
        let mut count = self.job_count.lock().unwrap();
        while count != 0 {
            count = self.empty_condvar.wait(count).unwrap();
        }
        //todo!()
    }
}

/// Thread pool.
#[derive(Debug)]
pub struct ThreadPool {
    _workers: Vec<Worker>,
    job_sender: Option<Sender<Job>>,
    pool_inner: Arc<ThreadPoolInner>,
}

impl ThreadPool {
    /// Create a new ThreadPool with `size` threads. Panics if the size is 0.
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        
        let (sender, receiver) = unbounded();

        let mut workers = Vec::with_capcity(size);

        for id in 0..size{
            workers.push(Worker::new(id, receiver.clone()))
        }

        ThreadPool {workers, sender}
        
        // todo!()
    }

    /// Execute a new job in the thread pool.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.job_sender.unwrap().send(Job(job)).unwrap();
        // todo!()
    }

    /// Block the current thread until all jobs in the pool have been executed.  NOTE: This method
    /// has nothing to do with `JoinHandle::join`.
    pub fn join(&self) {
        (*self.pool_inner).wait_empty();
        // todo!()
    }
}

impl Drop for ThreadPool {
    /// When dropped, all worker threads' `JoinHandle` must be `join`ed. If the thread panicked,
    /// then this function should panic too.
    fn drop(&mut self) {
        drop(self.job_sender.unwrap());
        //todo!()
    }
}
