use crate::torrent::dht::{TrackerCommand, TrackerContext};
use crate::torrent::CompactIpAddr;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::net::{IpAddr, SocketAddr};
use std::time::Instant;

/// The minimum number of distinct reporters required for consensus
const MIN_REPORTERS: usize = 3;
/// The maximum number of observed external IP addresses to keep in memory
const MAX_OBSERVED: usize = 15;

/// Observe the external IP address of the server from received node messages.
#[derive(Debug)]
pub struct Observer {
    /// The observed external IP address of the server
    external_ip: Option<IpAddr>,
    /// The observed external IP addresses of the server (target_addr, own_addr)
    observed_addrs: HashSet<ObservedAddress>,
}

impl Observer {
    pub fn new() -> Self {
        Self {
            external_ip: None,
            observed_addrs: Default::default(),
        }
    }

    pub async fn observe(
        &mut self,
        source_addr: SocketAddr,
        ip: Option<&CompactIpAddr>,
        context: &TrackerContext,
    ) {
        self.make_observation(source_addr, ip, context).await;
        self.clean_old_observations();
    }

    async fn make_observation(
        &mut self,
        source_addr: SocketAddr,
        ip: Option<&CompactIpAddr>,
        context: &TrackerContext,
    ) {
        let ip = match ip {
            None => return,
            Some(ip) => ip,
        };

        // if we've already seen this external addr, drop all older entries for it
        // we only want the most recent observation for that IP
        self.observed_addrs.insert(ObservedAddress {
            source_addr,
            observed_addr: ip.into(),
            last_seen: Instant::now(),
        });

        // early exit when we don't have enough information to determine our external IP address
        if self.observed_addrs.len() < MIN_REPORTERS {
            return;
        }

        // check if at least `MIN_REPORTERS` remote nodes report the same external IP address
        let mut reports: HashMap<IpAddr, usize> = HashMap::new();
        for external in self.observed_addrs.iter() {
            let count = reports.entry(external.observed_addr.ip()).or_default();
            *count += 1;
        }

        if let Some(candidate) = reports
            .into_iter()
            .filter(|(_, count)| *count >= MIN_REPORTERS)
            .sorted_by(|a, b| b.1.cmp(&a.1))
            .map(|(addr, _)| addr)
            .next()
        {
            let routing_table = context.routing_table_lock().await;
            // check if the observed external IP address is different from the current stored node id ip
            if !routing_table.id.is_secure_id(&candidate) {
                self.external_ip.replace(candidate);
                let _ = context
                    .command_sender()
                    .send(TrackerCommand::UpdateExternalIp(candidate));
            }
        }
    }

    fn clean_old_observations(&mut self) {
        let total = self.observed_addrs.len();
        if total < MAX_OBSERVED {
            return;
        }

        let to_remove = total - MAX_OBSERVED;
        let observed_by_oldest = self
            .observed_addrs
            .iter()
            .sorted_by(|a, b| a.last_seen.cmp(&b.last_seen))
            .map(|addr| addr.source_addr)
            .take(to_remove)
            .collect::<Vec<_>>();

        self.observed_addrs
            .retain(|addr| !observed_by_oldest.contains(&addr.source_addr));
    }
}

#[derive(Debug)]
struct ObservedAddress {
    source_addr: SocketAddr,
    observed_addr: SocketAddr,
    last_seen: Instant,
}

impl Hash for ObservedAddress {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.source_addr.hash(state);
    }
}

impl PartialEq for ObservedAddress {
    fn eq(&self, other: &Self) -> bool {
        self.source_addr == other.source_addr
    }
}

impl Eq for ObservedAddress {}
