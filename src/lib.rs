use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct ThreadPool {
  //threads: Vec<thread::JoinHandle<()>>,
  workers: Vec<Worker>,
  sender: mpsc::Sender<Message>,
}

trait FnBox {
  fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
  fn call_box(self: Box<F>) {
    (*self)()
  }
}

//struct Job;
type Job = Box<FnBox + Send + 'static>;

impl ThreadPool {
  /// 新しいThreadPoolを生成する。
  ///
  /// sizeがプールのスレッド数です。
  ///
  /// # パニック
  ///
  /// sizeが0なら、`new`関数はパニックします。
  ///
  /// Create a new ThreadPool.
  ///
  /// The size is the number of threads in the pool.
  ///
  /// # Panics
  ///
  /// The `new` function will panic if the size is zero.
  pub fn new(size: usize) -> ThreadPool {
    assert!(size > 0);

    let (sender, receiver) = mpsc::channel();

    let receiver = Arc::new(Mutex::new(receiver));

    //let mut threads = Vec::with_capacity(size);
    let mut workers = Vec::with_capacity(size);

    for id in 0..size {
      //workers.push(Worker::new(id, receiver));
      workers.push(Worker::new(id, Arc::clone(&receiver)));
    }

    ThreadPool { workers, sender }
  }

  pub fn execute<F>(&self, f: F)
  where
    F: FnOnce() + Send + 'static,
  {
    let job = Box::new(f);

    self.sender.send(Message::NewJob(job)).unwrap();
  }
}

impl Drop for ThreadPool {
  fn drop(&mut self) {
    println!("Sending terminate message to all workers.");

    for _ in &mut self.workers {
      //println!("Shutting down worker {}", worker.id);
      self.sender.send(Message::Terminate).unwrap();
    }

    println!("Shutting down all workers.");

    for worker in &mut self.workers {
      println!("Shutting down worker {}", worker.id);

      //worker.thread.join().unwrap();
      if let Some(thread) = worker.thread.take() {
        thread.join().unwrap();
      }
    }
  }
}

struct Worker {
  id: usize,
  //thread: thread::JoinHandle<()>,
  thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
  fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
    let thread = thread::spawn(move || {
      loop {
        let message = receiver.lock().unwrap().recv().unwrap();

        match message {
          Message::NewJob(job) => {
            println!("Worker {} got a job; executing.", id);

            //(*job)();
            job.call_box();
          }
          Message::Terminate => {
            println!("Worker {} was told to terminate.", id);

            break;
          }
        }
      }
      //receiver;
    });

    Worker {
      id,
      thread: Some(thread),
    }
  }
}

enum Message {
  NewJob(Job),
  Terminate,
}
