use std::net::TcpListener;

use rust_web_server_by_thread::{handle_connection, ThreadPool};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // 初始化线程池
    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        // 单线程处理请求
        // handle_connection(stream);

        // 每一个请求使用单独的线程进行处理
        // thread::spawn(|| handle_connection(stream));

        // 使用线程池处理请求
        pool.execute(|| handle_connection(stream));
    }
}
