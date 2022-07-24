use tokio::{net::TcpListener, io::{AsyncWriteExt, BufReader, AsyncBufReadExt}, sync::broadcast};

pub struct ChatServer {
    host: String,
    port: u16,
}

impl ChatServer {
    pub fn new(host: String, port: u16) -> ChatServer {
        ChatServer { host, port }
    }

    pub async fn serve(self) {
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port)).await.unwrap();
        let (tx, _rx) = broadcast::channel(10);
        loop {
            let (mut socket, addr) = listener.accept().await.unwrap();
            let chat_handle = format!("yourname_{}", &addr);
            let tx = tx.clone();
            let mut rx = tx.subscribe();
            tokio::spawn(async move{
                let (read_s, mut write_s) = socket.split();
                let mut reader = BufReader::new(read_s);
                let mut line = format!("{} joined.\r\n", chat_handle);
                let msg = format!("welcome {}.\r\n", chat_handle);
                write_s.write_all(msg.as_bytes()).await.unwrap();
                tx.send((line.clone(), addr)).unwrap();
                loop {
                    tokio::select! {
                        result = reader.read_line(&mut line) => {
                            if result.unwrap() == 0 {
                                break;
                            }
                            tx.send((format!("{}: {}", chat_handle, line.clone()), addr)).unwrap();
                            line.clear();
                        }
                        result = rx.recv() => {
                            let (msg, other_addr) = result.unwrap();
                            if addr != other_addr {
                                write_s.write_all(msg.as_bytes()).await.unwrap();
                            }
                        }
                    }
                }
            });
        }
    }
}
