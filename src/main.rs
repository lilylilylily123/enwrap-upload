use std::fs::{File  };
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::time;
use cdn_rust_v2::ThreadPool;

fn main() {
    setup_server();
}



fn setup_server() {
    let pool = ThreadPool::new(4);
    let listener = TcpListener::bind("0.0.0.0:1234").unwrap();
    let mut incoming = listener.incoming();
    for stream in incoming {
        let stream = stream.unwrap();

        pool.execute(move || {
            handle_conn(&stream);
        });
    }
}

fn handle_conn(mut stream: &TcpStream) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let mut line_array = Vec::new();
    for line in reader.by_ref().lines() {
        println!("{:?}", line);
        if line.as_ref().unwrap() == "\r\n\r\n" || line.as_ref().unwrap() == "" {
            break;
        }
        let line_clone = line.unwrap().clone();
        line_array.push(line_clone);

    }

    println!("{:?}", line_array);
    if line_array[0].starts_with("GET") {
        handle_get(&mut &stream);
    } else if line_array[0].starts_with("POST") && line_array[0].contains("/upload") {
        println!("POST!");
        handle_post(reader, &mut &stream, &line_array[8], &line_array[7]);
    } else {
        handle_404(&mut &stream);
    }
}


fn handle_404(mut stream: &TcpStream) {
    stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\nThis page is not available!").unwrap();
    stream.flush().unwrap();
}

fn handle_post(mut reader: BufReader<&TcpStream>, mut stream: &TcpStream, content_type: &String, content_length: &String) {
    let time_now = time::SystemTime::now();
    let content_type_line = content_type.lines().find(|line| line.starts_with("Content-Type"));
    if let Some(content_type_line) = content_type_line {
        let content_type = content_type_line
            .split(':')
            .nth(1)
            .map(|s| s.trim())
            .unwrap_or("");

        let file_extension = match content_type {
            "text/plain" => "txt",
            "text/html" => "html",
            "text/css" => "css",
            "application/javascript" => "js",
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/x-icon" => "ico",
            "application/zip" => "zip",
            "media/mp4" => "mp4",
            "application/pdf" => "pdf",
            "application/x-rar-compressed" => "rar",

            "application/x-apple-diskimage" => "dmg",
            _ => "",
        };
        println!("{}", file_extension);
        let content_length = content_length.lines().find(|line| line.starts_with("Content-Length"));
        if let Some(content_length) = content_length {
            let content_length = content_length
                .split(':')
                .nth(1)
                .map(|s| s.trim())
                .unwrap_or("");
            println!("cont length: {}", content_length);
            // let mut byte_line = Vec::new();
            let content_length = content_length.parse::<usize>().unwrap();
            let mut byte_line = vec![0; content_length];
            println!("slash 4, {}", content_length / 4);
            reader.read_exact(&mut byte_line).unwrap();
            // println!("{:?}", byte_line);
            println!("byte line length: {:?}", byte_line.len());
            if byte_line.len() == content_length {
                let mut file = File::create(format!("storage/test.{}", file_extension)).unwrap();
                if file_extension == "" {
                    let mut perms = file.metadata().unwrap().permissions();
                    perms.set_mode(0o770);
                    file.set_permissions(perms).unwrap();
                }
                file.write_all(&byte_line).unwrap();
                println!("File created");
                let time_taken = time_now.elapsed().unwrap().as_millis();
                stream.write_all(format!("HTTP/1.1 200 OK\r\n\r\nFile upload successful! Time in ms: {}", time_taken).as_ref()).unwrap();
                stream.flush().unwrap();
            } else {
                stream.write_all(b"HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\nFile upload failed!").unwrap();
                stream.flush().unwrap();
            }
        }
    }
}

fn handle_get(mut stream: &TcpStream) {
    stream.write_all(b"HTTP/1.1 200 OK\r\n\r\nCool beans").unwrap();
    stream.flush().unwrap();
}