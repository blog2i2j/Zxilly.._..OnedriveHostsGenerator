mod utils;

use std::collections::HashSet;
use chrono::{Local, Utc};
use chrono_tz::Asia::Shanghai;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use once_cell::sync::Lazy;
use tokio::task::JoinSet;
use trust_dns_resolver::config::*;
use trust_dns_resolver::{AsyncResolver, TokioAsyncResolver};

use crate::utils::StringLine;

include!(concat!(env!("OUT_DIR"), "/domains.rs"));

static RESOLVER: Lazy<TokioAsyncResolver> = Lazy::new(|| {
    let mut options = ResolverOpts::default();
    options.ip_strategy = LookupIpStrategy::Ipv4AndIpv6;
    options.num_concurrent_reqs = 2;
    let mut config = NameServerConfigGroup::quad9_https();
    config.merge(NameServerConfigGroup::cloudflare_https());

    AsyncResolver::tokio(ResolverConfig::from_parts(None, vec![], config), options).unwrap()
});

pub async fn render(ipv4: bool, ipv6: bool, single: bool) -> String {
    let mut header = String::new();
    header.push_str_line("####### Onenote Hosts Start #######");
    header.push_str_line(
        "# This file is generated by https://github.com/Zxilly/OnedriveHostsGenerator",
    );

    let now = Utc::now().with_timezone(&Shanghai);

    header.push_str_line(&format!(
        "# Generate time: {}",
        now.format("%Y-%m-%d %H:%M:%S")
    ));

    let mut content = String::new();

    let mut v4_ips: Vec<(&str, Ipv4Addr)> = vec![];
    let mut v6_ips: Vec<(&str, Ipv6Addr)> = vec![];
    let mut unresolved_domains: Vec<&str> = vec![];

    let mut tasks = JoinSet::new();

    for domain in DOMAIN_LIST.into_iter() {
        tasks.spawn(async move {
            let ret = RESOLVER.lookup_ip(domain).await;
            (domain, match ret {
                Ok(ips) => Ok(ips),
                Err(e) => Err(e),
            })
        });
    }

    while let Some(ret) = tasks.join_next().await {
        if let Ok((domain, ret)) = ret {
            match ret {
                Ok(ips) => {
                    for ip in ips.iter() {
                        match ip {
                            IpAddr::V4(ip) => v4_ips.push((domain, ip)),
                            IpAddr::V6(ip) => v6_ips.push((domain, ip)),
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Resolve {} failed: {}", domain, e);
                    unresolved_domains.push(domain);
                }
            }
        } else {
            eprintln!("JoinError: {:?}", ret.unwrap_err());
        }
    }

    if !unresolved_domains.is_empty() {
        content.push_str_line("\n# Unresolved domains");
        for domain in unresolved_domains.into_iter() {
            content.push_str_line(&format!("# {} not resolved", domain));
        }
    }

    fn find_max_length<T: std::fmt::Display>(ips: &[(&str, T)]) -> (usize, usize) {
        let mut max_ip_len = 0;
        let mut max_domain_len = 0;
        ips.iter().for_each(|(domain, ip)| {
            let len = ip.to_string().len();
            if len > max_ip_len {
                max_ip_len = len;
            }
            let len = domain.len();
            if len > max_domain_len {
                max_domain_len = len;
            }
        });
        (max_ip_len, max_domain_len)
    }

    if ipv4 {
        content.push_str("\n# IPv4 addresses:\n");

        let mut printed_domain = HashSet::new();

        // find max length of v4 ip
        let (max_v4_ip_len, max_v4_domain_len) = find_max_length(&v4_ips);

        for (domain, ip) in v4_ips.into_iter() {
            if single && (printed_domain.contains(domain)) {
                continue;
            }
            printed_domain.insert(domain);

            content.push_str_line(&format!(
                "{:w1$} {:>w2$}",
                ip,
                domain,
                w1 = max_v4_ip_len,
                w2 = max_v4_domain_len
            ));
        }
    }

    if ipv6 {
        content.push_str_line("");
        if !v6_ips.is_empty() {
            content.push_str_line("# IPv6 addresses:");
        } else {
            content.push_str_line("# No IPv6 addresses resolved");
        }

        let mut printed_domain = HashSet::new();

        // find max length of v6 ip
        let (max_v6_ip_len, max_v6_domain_len) = find_max_length(&v6_ips);

        for (domain, ip) in v6_ips.into_iter() {
            if single && (printed_domain.contains(domain)) {
                continue;
            }
            printed_domain.insert(domain);

            content.push_str_line(&format!(
                "{:w1$} {:>w2$}",
                ip,
                domain,
                w1 = max_v6_ip_len,
                w2 = max_v6_domain_len
            ));
        }
    }

    content.push_str_line("####### Onenote Hosts End #######");

    let cost_time = Local::now().signed_duration_since(now).num_milliseconds();

    header.push_str_line(&format!("# Generate in: {} ms", cost_time));

    header.push_str_line(&content);

    header
}
