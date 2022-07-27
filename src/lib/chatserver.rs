use tokio::{net::TcpListener, io::{AsyncWriteExt, BufReader, AsyncBufReadExt}, sync::broadcast};

async fn clean_line(l: String) -> String {
    let mut line = l;
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    line
}

async fn update_sessions (sessions: &mut Vec<String>, message_type:MessageType, param:String){
    match message_type {
        MessageType::JOIN => {
            sessions.push(param);
        },
        MessageType::LEAVE => {
            sessions.retain(|x| x != &param);
        },
        _ => unimplemented!(),
    }
}

#[derive(Debug, Copy, Clone)]
enum MessageType {
    MSG,
    JOIN,
    LEAVE,
}

pub struct ChatServer {
    host: String,
    port: u16,
    sessions: Vec<String>,
}

impl ChatServer {
    pub fn new(host: String, port: u16) -> ChatServer {
        ChatServer { host, port, sessions:Vec::new() }
    }

    pub async fn serve(self) {
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port)).await.unwrap();
        let (tx, _rx) = broadcast::channel(10);
        let sessions:Vec<String> = Vec::new();
        loop {
            let (mut socket, addr) = listener.accept().await.unwrap();
            let chat_handle = format!("yourname_{}", &addr);
            let tx = tx.clone();
            let mut rx = tx.subscribe();
            let mut sessions = sessions.clone();
            tokio::spawn(async move{
                let (read_s, mut write_s) = socket.split();
                let mut reader = BufReader::new(read_s);
                let mut line = format!("{} joined.\r\n", chat_handle);
                let msg = format!("welcome {}.\r\n", chat_handle);
                write_s.write_all(msg.as_bytes()).await.unwrap();
                tx.send((line.clone(), addr, MessageType::JOIN, chat_handle.clone())).unwrap();
                println!("CHATLOG: {} joined.", chat_handle);
                loop {
                    tokio::select! {
                        _ = reader.read_line(&mut line) => {
                            let cleaned_line = clean_line(line.clone()).await;
                            if cleaned_line.chars().count() == 2 && cleaned_line.chars().nth(0).unwrap() == '/'{
                                let command = cleaned_line.chars().nth(1).unwrap();
                                match command {
                                    'q' => {
                                        let msg = format!("bye {}", chat_handle);
                                        let line = format!("{} left the chat.\r\n", chat_handle);
                                        write_s.write_all(msg.as_bytes()).await.unwrap();
                                        tx.send((line.clone(), addr, MessageType::LEAVE, chat_handle.clone())).unwrap();
                                        println!("CHATLOG: {} left the chat", chat_handle);
                                        break;
                                    },
                                    'l' => {
                                        for s in sessions.iter() {
                                            println!("SESSION: {}", s);
                                        }
                                    },
                                    _ => {
                                        let msg = format!("no such command '{}'\r\n", command);
                                        write_s.write_all(msg.as_bytes()).await.unwrap();
                                    }
                                }
                            } else {
                                tx.send((format!("{}: {}", chat_handle, line.clone()), addr, MessageType::MSG, String::new())).unwrap();
                                line.clear();
                            }
                        }
                        result = rx.recv() => {
                            let (msg, other_addr, message_type, param) = result.unwrap();
                            match message_type {
                                MessageType::MSG => {
                                    if addr != other_addr {
                                        write_s.write_all(msg.as_bytes()).await.unwrap();
                                    }
                                },
                                _ => {
                                    update_sessions(&mut sessions, message_type, param).await;
                                }
                            }
                        }
                    }
                }
            });
        }
    }
}