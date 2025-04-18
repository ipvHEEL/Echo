use anyhow::Result;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};
async fn main() -> Result<()> {#[tokio::main]
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Сервер запущен на 0.0.0.0:8080");

    let (tx, _rx) = broadcast::channel::<(String, std::net::SocketAddr)>(10);

    loop {
        let (socket, addr) = listener.accept().await?;
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            handle_connection(socket, addr, tx, rx).await;
        });
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    addr: std::net::SocketAddr,
    tx: broadcast::Sender<(String, std::net::SocketAddr)>,
    mut rx: broadcast::Receiver<(String, std::net::SocketAddr)>,
) {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut send_task = tokio::spawn(async move {
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // Соединение закрыто
                Ok(_) => {
                    let msg = line.trim().to_string();
                    if !msg.is_empty() {
                        tx.send((msg, addr)).unwrap();
                    }
                }
                Err(_) => break,
            }
        }
    });
    let mut recv_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok((msg, other_addr)) => {
                    if addr != other_addr {
                        writer.write_all(format!("{}: {}\n", other_addr, msg).as_bytes()).await.unwrap();
                    }
                }
                Err(_) => break,
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    println!("Клиент {} отключился", addr);
}
