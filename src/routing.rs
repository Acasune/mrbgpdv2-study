use std::net::{IpAddr, Ipv4Addr};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use anyhow::{Context, Result};
use futures::TryStreamExt;
use ipnetwork;
use rtnetlink::new_connection;

use crate::config::Config;
use crate::error::ConfigParseError;

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
