use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    thread::{self, JoinHandle},
    time,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, rx: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let job = rx.lock().unwrap().recv();

                match job {
                    Ok(job) => {
                        println!("Worker {} on the job, executing...", id);
                        job();
                    }
                    Err(_) => {
                        println!("Worker {} shutting down.", id);
                        break;
                    }
                }
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }
}

struct ThreadPool {
    sender: Option<Sender<Job>>,
    workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn new(thread_num: usize) -> Self {
        let (tx, rx) = channel();

        let rx = Arc::new(Mutex::new(rx));

        let mut workers = Vec::with_capacity(thread_num);

        for id in 0..thread_num {
            workers.push(Worker::new(id, rx.clone()));
        }

        Self {
            workers,
            sender: Some(tx),
        }
    }

    pub fn spawn(&self, job: Job) {
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

fn main() {
    let pool = ThreadPool::new(4);

    for i in 0..7 {
        pool.spawn(Box::new(move || {
            println!("Task {} running on thread {:?}", i, thread::current().id());
            thread::sleep(time::Duration::from_millis(100));
            println!("Task {} complete", i);
        }));
    }
}
