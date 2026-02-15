use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};
use hello::ThreadPool;

fn main() {
    // TcpListener를 이용하여 해당 주소에서 TCP 연결 수신 대기
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    // TcpListener의 incoming 메서드는 스트림 시퀀스를 제공하는 반환자를 반환.
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        // // 새 스레드를 생성한 다음 새 스레드의 클로저에서 코드를 실행.
        // thread::spawn(|| {
        //     handle_connection(stream);
        // });
        // 스레드 대신 스레드 풀 생성. 
        pool.execute(|| {
            handle_connection(stream);
        });
    }
    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    // stream에 대한 가변 참조자를 감싼 BufReader 인스턴스 생성.
    // BufReader는 std::io::Read 트레이트 메서드에 대한 호출을 관리하는 것으로 버퍼링을 추가.
    let buf_reader = BufReader::new(&mut stream);
    // // 서버로 보낸 요청의 라인을 수집하기 위해서 http_request 변수 생성.
    // let request_line: Vec<_> = buf_reader
    //     .lines()  // line 메서드는 새로운 줄 바꿈 바이트가 발견될 떄마다 데이터 스트림을 분할함으로써 Result<String, std::io::Error>의 반복자 반환.
    //     .map(|result| result.unwrap())  // String을 얻기 위해 각 Result 매핑 및 unwrap
    //     .take_while(|line| !line.is_empty())  // 빈 문자 나올때까지 확인.
    //     .collect();
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    
    // format!을 사용하여 파일의 내용을 성공 응답의 본문으로 간주.
    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
    );

    stream.write_all(response.as_bytes()).unwrap();
}