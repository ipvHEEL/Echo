use anyhow::Result;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Подключение к серверу");
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Подключено к серверу");
    let (tx, mut rx) = mpsc::channel::<String>(10);
    let stdin_task = tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            reader.read_line(&mut line).await.unwrap();
            let msg = line.trim().to_string();
            if !msg.is_empty() {
                tx.send(msg).await.unwrap();
            }
        }
    });

    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                writer.write_all(format!("{}\n", msg).as_bytes()).await?;
            }
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        println!("Сервер закрыл соединение");
                        break;
                    }
                    Ok(_) => {
                        print!("{}", line);
                        line.clear();
                    }
                    Err(e) => {
                        eprintln!("Ошибка чтения: {}", e);
                        break;
                    }
                }
            }
        }
    }

    stdin_task.abort();
    Ok(())
}
