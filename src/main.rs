use inline_colorization::*;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::{
    error::Error,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    self,
    io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

#[derive(Serialize, Deserialize, Debug)]
struct Server2ClientMsg {
    env: String,
    err: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Client2ServerMsg {
    input: String,
    user_id: i32,
    timestamp: u128,
}

#[derive(Serialize, Deserialize, Debug)]
struct ClientInitMsg {
    user_id: i32,
}

static USER_ID: i32 = 1145;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Input Meerkat server address with port: ");
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    let mut terminal_reader = BufReader::new(stdin).lines();
    let addr = terminal_reader.next_line().await.unwrap().unwrap();
    let mut stream = match TcpStream::connect(addr).await {
        Ok(s) => s,
        Err(_) => panic!("connect to server fail"),
    };
    let init_msg = ClientInitMsg { user_id: USER_ID };
    let init_msg_json = serde_json::to_string(&init_msg).unwrap();
    stream.write_all(init_msg_json.as_bytes()).await.unwrap();
    loop {
        let mut buffer = vec![0; 1024];
        let n = stream.read(&mut buffer).await?;
        let received_raw = String::from_utf8_lossy(&buffer[..n]);
        let received_json = serde_json::to_string(&received_raw).unwrap();

        // let temp = r#"
        //     {
        //         "env": "aaab\nbbccc",
        //         "err": "type error"
        //     }
        // "#;

        let received: Server2ClientMsg = serde_json::from_str(&received_json).unwrap();
        if received.err != None {
            stdout
                .write_all(
                    &format!("{color_red}{}{color_reset}\n", received.err.unwrap()).as_bytes(),
                )
                .await
                .expect("tokio output error");
        }
        stdout
            .write_all(&format!("{}\n", received.env).as_bytes())
            .await
            .expect("tokio output error");
        stdout.flush().await.unwrap();
        stdout
            .write_all(&format!("\n>>> ").as_bytes())
            .await
            .expect("tokio output error");
        stdout.flush().await.unwrap();
        let mut input = String::new();
        loop {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).unwrap();
            if line.trim().is_empty() {
                break;
            }
            input.push_str(&line);
        }
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let timestamp = since_the_epoch.as_millis();
        let send = Client2ServerMsg {
            input: input,
            user_id: USER_ID,
            timestamp: timestamp,
        };
        let send_json = serde_json::to_string(&send).unwrap();
        stream.write_all(send_json.as_bytes()).await.unwrap();
    }
    Ok(())
}
