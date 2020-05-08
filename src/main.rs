mod item;
mod progress;

use futures::stream::TryStreamExt;
use tokio::io::{self, BufReader, BufWriter};
use tokio::prelude::*;
use tokio_util::compat::FuturesAsyncReadCompatExt;

use item::Item;
use progress::Progress;

#[tokio::main]
async fn main() {
    let pg = Progress::new(io::sink());
    let pg_clone = pg.clone();

    let download_handle = tokio::task::spawn(async move {
        let url = "https://releases.ubuntu.com/20.04/ubuntu-20.04-desktop-amd64.iso";
        let res = reqwest::get(url).await.unwrap();

        let mut stream = res
            .bytes_stream()
            .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
            .into_async_read()
            .compat();

        let (mut reader, mut writer) = (BufReader::new(stream), BufWriter::new(pg_clone));
        io::copy(&mut reader, &mut writer).await.unwrap();
    });

    tokio::task::spawn(async move {
        loop {
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            println!("{:?}", pg.to_size().await);
        }
    });

    download_handle.await.unwrap();
}
