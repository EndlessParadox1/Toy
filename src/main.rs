use async_std::{
    net::{TcpListener, TcpStream},
    prelude::*,
    task,
};
use chrono::{Local, SubsecRound};
use futures::stream::StreamExt;
use regex::Regex;
use std::{fs, str};

#[async_std::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();
    listener
        .incoming()
        .for_each_concurrent(None, |stream| async move {
            let stream = stream.unwrap();
            task::spawn(handle_connection(stream));
        })
        .await;
}

async fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await.unwrap();
    let request = match str::from_utf8(&buffer) {
        Ok(tmp) => tmp,
        Err(_) => return,
    };
    let re = Regex::new(r"GET\s(?P<src>.*)\sHTTP/1.1").unwrap();
    let cap = match re.captures(request) {
        Some(tmp) => tmp,
        None => return,
    };
    let src = &cap["src"];
    let date = Local::now().naive_local().round_subsecs(0).to_string();
    println!("{}, from IP: {}.", date, stream.peer_addr().unwrap().ip());
    const SUC: &str = "HTTP/1.1 200 OK";
    const FAIL: &str = "HTTP/1.1 404 NOT FOUND";
    let (status_line, contents) = if src == "/" {
        (SUC, fs::read_to_string("front/index.html").unwrap())
    } else {
        match fs::read_to_string(String::from("front/") + src.trim_start_matches('/')) {
            Ok(contents) => (SUC, contents),
            Err(_) => (FAIL, fs::read_to_string("front/404.html").unwrap()),
        }
    };
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents,
    );
    stream.write_all(response.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
}
