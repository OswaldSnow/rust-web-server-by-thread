use std::{fs, io::{BufRead, BufReader, Write}, net::TcpStream, sync::{mpsc, Arc, Mutex}, thread::{self}, time::Duration};

#[allow(unused)]
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            println!("thread {id} is waiting");

            let message = receiver.lock().unwrap().recv();
            
            match message {
                Ok(job) => job(),
                Err(_) => break
            };
        });

        // 此种方式无法实现多线程处理请求 会卡住
        // let thread = thread::spawn(move || {
        //     while let Ok(job) = receiver.lock().unwrap().recv() {
        //         println!("Worker {id} got a job; executing.");

        //         job();
        //     }
        // });

        Worker { id, thread: Some(thread)}
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[allow(unused)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        
        let mut workers = Vec::with_capacity(4);

        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..4 {
            workers.push(Worker::new(id, receiver.clone()));
        }

        ThreadPool { workers, sender: Some(sender) }
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

pub fn handle_connection(mut stream: TcpStream) {
   
    let buf_reader = BufReader::new(&mut stream);
    
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, res_file_name) = match &request_line[..] {
        line if line.eq_ignore_ascii_case("GET / HTTP/1.1") => {
            ("HTTP/1.1 200 OK", "hello.html")
        },
        line if line.eq_ignore_ascii_case("GET /sleep HTTP/1.1") => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "sleep.html")
        },
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(res_file_name).unwrap();
    let len = contents.len();

    let response_info = 
        format!("{status_line}\r\nContent-length: {len}\r\n\r\n\n{contents}");

    stream.write_all(response_info.as_bytes()).unwrap();
}