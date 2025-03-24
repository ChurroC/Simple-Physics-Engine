use std::{thread, sync::mpsc, sync::Arc, sync::Mutex};

// Change JobT<T> to Job to indicate it's for void functions
type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            workers.push(Worker::new(Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender
        }
    }
    
    // The were makes sure that the generic is not a reference but a value
    pub fn execute<F, T>(&self, f: F) -> mpsc::Receiver<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (response_tx, response_rx) = mpsc::channel();
        
        // Our job just executes f and sends its result, but doesn't return anything itself
        let job = Box::new(move || {
            let result = f();
            response_tx.send(result).unwrap();
        });

        self.sender.send(Message::NewJob(job)).unwrap();
        
        response_rx
    }
}

impl Default for ThreadPool {
    fn default() -> Self {
        // Create a default threadpool with a reasonable number of threads
        // Using num_cpus for a sensible default based on the system
        let num_threads = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4); // Default to 4 if we can't detect

        Self::new(num_threads)
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}


struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    job();
                }
                Message::Terminate => {
                    break;
                }
            }
        });

        Worker { 
            thread: Some(thread)
        }
    }
}