use std::net::{IpAddr, Ipv4Addr};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use anyhow::{Context, Result};
use bytes::{BufMut, BytesMut};
use futures::TryStreamExt;
use ipnetwork;
use rtnetlink::new_connection;

use crate::config::Config;
use crate::error::{ConfigParseError, ConstructIpv4NetworkError, ConvertBytesToBgpMessageError};

#[derive(Debug, PartialEq, Eq, Clone)]
struct LocRib;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct Ipv4Network(ipnetwork::Ipv4Network);

impl Deref for Ipv4Network {
    type Target = ipnetwork::Ipv4Network;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Ipv4Network {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ipnetwork::Ipv4Network> for Ipv4Network {
    fn from(ip_network: ipnetwork::Ipv4Network) -> Self {
        Self(ip_network)
    }
}

impl FromStr for Ipv4Network {
    type Err = ConfigParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let network = s
            .parse::<ipnetwork::Ipv4Network>()
            .context("s:{:?}を、Ipv4Networkにparseできませんでした。")?;
        Ok(Self(network))
    }
}

impl LocRib {
    pub async fn new(config: &Config) -> Result<Self> {
        todo!();
    }

    async fn lookup_kernel_routing_table(
        network_address: Ipv4Network,
    ) -> Result<(Vec<Ipv4Network>)> {
        let (connection, handle, _) = new_connection()?;
        tokio::spawn(connection);
        let mut routes = handle.route().get(rtnetlink::IpVersion::V4).execute();
        let mut results = vec![];
        while let Some(route) = routes.try_next().await? {
            let destination = if let Some((IpAddr::V4(addr), prefix)) = route.destination_prefix() {
                ipnetwork::Ipv4Network::new(addr, prefix)?.into()
            } else {
                continue;
            };
            if destination != network_address {
                continue;
            }

            results.push(destination);
        }
        Ok(results)
    }
}

impl Ipv4Network {
    pub fn new(addr: Ipv4Addr, prefix: u8) -> Result<Self, ConstructIpv4NetworkError> {
        let net = ipnetwork::Ipv4Network::new(addr, prefix).context(format!(
            "Ipv4NetworkをConstructできませんでしたaddr:{}, prefix: {}
      ",
            addr, prefix
        ))?;
        Ok(Self(net))
    }
    pub fn from_u8_slice(bytes: &[u8]) -> Result<Vec<Self>, ConvertBytesToBgpMessageError> {
        let mut networks = vec![];
        let mut i = 0;
        while bytes.len() > i {
            let prefix = bytes[i];
            i += 1;
            if prefix == 0 {
                networks.push(Ipv4Network::new(Ipv4Addr::new(0, 0, 0, 0), prefix).context("")?);
                i += 1;
            } else if (1..=8).contains(&prefix) {
                networks
                    .push(Ipv4Network::new(Ipv4Addr::new(bytes[i], 0, 0, 0), prefix).context("")?);
                i += 1;
            } else if (9..=16).contains(&prefix) {
                networks.push(
                    Ipv4Network::new(Ipv4Addr::new(bytes[i], bytes[i + 1], 0, 0), prefix)
                        .context("")?,
                );
                i += 2;
            } else if (17..=24).contains(&prefix) {
                networks.push(
                    Ipv4Network::new(
                        Ipv4Addr::new(bytes[i], bytes[i + 1], bytes[i + 2], 0),
                        prefix,
                    )
                    .context("bytes -> Ipv4に変換できませんでした。")?,
                );
                i += 3;
            } else if (24..=32).contains(&prefix) {
                networks.push(
                    Ipv4Network::new(
                        Ipv4Addr::new(bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]),
                        prefix,
                    )
                    .context("bytes -> Ipv4に変換できませんでした。")?,
                );
                i += 4;
            } else {
                return Err(ConvertBytesToBgpMessageError::from(anyhow::anyhow!(
                    "bytes -> Ipv4に変換できませんでした。 \
                    Prefixが0-32の間ではありません。
                    "
                )));
            };
        }
        Ok(networks)
    }

    pub fn bytes_len(&self) -> usize {
        match self.prefix() {
            0 => 1,
            1..9 => 2,
            9..17 => 3,
            17..25 => 4,
            25..33 => 5,
            _ => panic!("prefixが0..32の間ではありません！"),
        }
    }
}

impl From<&Ipv4Network> for BytesMut {
    fn from(network: &Ipv4Network) -> BytesMut {
        let prefix = network.prefix();

        let n = network.network().octets();
        let network_bytes = match prefix {
            0 => vec![],
            1..9 => n[0..1].into(),
            9..17 => n[0..2].into(),
            17..25 => n[0..3].into(),
            25..33 => n[0..4].into(),
            _ => panic!("prefixが0..32の間ではありません！"),
        };
        let mut bytes = BytesMut::new();
        bytes.put_u8(prefix);
        bytes.put(&network_bytes[..]);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn loclib_can_lookup_routing_table() {
        let network = ipnetwork::Ipv4Network::new("10.200.100.0".parse().unwrap(), 24)
            .unwrap()
            .into();
        let routes = LocRib::lookup_kernel_routing_table(network).await.unwrap();
        let expected = vec![network];
        assert_eq!(routes, expected);
    }
}
