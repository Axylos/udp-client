use s2n_quic::{client::Connect, Client};
use std::{error::Error, net::SocketAddr};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("connecting");
    let cert = fs::read_to_string("/tmp/cert.pem")?;
    let client = Client::builder()
        .with_tls(cert.as_str())?
        .with_io("0.0.0.0:0")?
        .start()?;

    let addr: SocketAddr = "192.168.1.139:4433".parse()?;
    let connect = Connect::new(addr).with_server_name("localhost");
    let mut connection = client.connect(connect).await?;

    // ensure the connection doesn't time out with inactivity
    connection.keep_alive(true)?;
    println!("keep alive after connect");

    // open a new stream and split the receiving and sending sides
    let stream = connection.open_bidirectional_stream().await?;
    let (mut receive_stream, mut send_stream) = stream.split();

    // spawn a task that copies responses from the server to stdout
    tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        let _ = tokio::io::copy(&mut receive_stream, &mut stdout).await;
    });

    // copy data from stdin and send it to the server
    let mut stdin = tokio::io::stdin();
    tokio::io::copy(&mut stdin, &mut send_stream).await?;

    Ok(())
}
