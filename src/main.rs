use std::env;
use std::str::FromStr;
use std::sync::Arc;

use mrbgpdv2::config::Config;
use mrbgpdv2::peer::Peer;
use mrbgpdv2::routing::LocRib;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let config = env::args().skip(1).fold("".to_owned(), |mut acc, s| {
        acc += &(s.to_owned() + " ");
        acc
    });
    let config = config.trim_end();
    let configs = vec![Config::from_str(&config).unwrap()];

    let loc_rib = Arc::new(Mutex::new(
        LocRib::new(&configs[0])
            .await
            .expect("LocRibの生成に失敗しました"),
    ));
    let mut peers: Vec<Peer> = configs
        .into_iter()
        .map(|c| Peer::new(c, Arc::clone(&loc_rib)))
        .collect();
    for peer in &mut peers {
        peer.start();
    }

    loop {
        for peer in &mut peers {
            peer.next().await;
        }
    }
}
