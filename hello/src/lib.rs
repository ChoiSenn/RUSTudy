use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

// ThreadPoll에 대한 new 연관함수 필요.
impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        // 0을 수신하면 패닉.
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        // item을 size만큼 담을 수 있는 새 벡터 생성.
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    // execute 메서드 필요.
    // execute에서 얻은 클로저를 사용하여 새 Job 인스턴스를 생성한 후,
    // 해당 작업을 채널 단말로 보낸다.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

// 스레드 풀 Drop 구현.
// 풀이 버려지면 모든 스레드가 조인되어서 작업 완료를 보장해야 한다.
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // 워커 스레드 조인 전에 명시적으로 sender 버리기
        drop(self.sender.take());
        // 스레드풀 workers에 각각 종료.
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
