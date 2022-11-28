use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::GreprMatch;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(n: usize, matches: Arc<Mutex<Vec<GreprMatch>>>) -> ThreadPool {
        assert!(n > 0);

        let (sender, reciever) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(reciever));

        let mut workers = Vec::with_capacity(n);

        for id in 0..n {
            workers.push(Worker::new(id, Arc::clone(&receiver), Arc::clone(&matches)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&Arc<Mutex<Vec<GreprMatch>>>) + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
        matches: Arc<Mutex<Vec<GreprMatch>>>,
    ) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();
            match job {
                Ok(j) => j(&matches),
                Err(_) => continue,
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

type Job = Box<dyn FnOnce(&Arc<Mutex<Vec<GreprMatch>>>) + Send + 'static>;
