use inline_colorization::*;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::error::Error;
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
    }
    Ok(())
}
