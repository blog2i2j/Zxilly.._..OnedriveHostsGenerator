use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use chrono::Local;
use once_cell::sync::Lazy;

fn sort_domain(domain_list: Vec<&str>) -> Vec<String> {
    // Sort domain list by domain root name, then by domain name
    let domain_list: HashSet<&str> = domain_list.into_iter().collect();
    let mut domain_list: Vec<String> = domain_list.into_iter().map(String::from).collect();

    domain_list.sort_by(|a, b| {
        let a_parts: Vec<&str> = a.split('.').rev().collect();
        let a = a_parts.join(".");

        let b_parts: Vec<&str> = b.split('.').rev().collect();
        let b = b_parts.join(".");

        a.cmp(&b)
    });

    domain_list
}

static DOMAIN_LIST: Lazy<Vec<String>> = Lazy::new(|| {
    let domain_list = include_str!("../domains.txt");
    let domain_list: Vec<&str> = domain_list.split('\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    sort_domain(domain_list)
});

pub fn render() -> String {
    let mut ret = String::new();
    ret.push_str("####### Onenote Hosts Start #######\n");
    ret.push_str("# This file is generated by https://github.com/Zxilly/OnedriveHostsGenerator\n");

    let now = Local::now();

    ret.push_str(&format!("# Generate time: {:?}\n", now.to_string()));

    let mut v4_ips: HashSet<(String, Ipv4Addr)> = HashSet::new();
    let mut v6_ips: HashSet<(String, Ipv6Addr)> = HashSet::new();
    let mut unresolved_domains: HashSet<String> = HashSet::new();

    for domain in DOMAIN_LIST.clone().into_iter() {
        let addr = (domain.as_ref(), 0).to_socket_addrs();
        match addr {
            Ok(mut addrs) => {
                for socket_addr in addrs.by_ref() {
                    match socket_addr {
                        std::net::SocketAddr::V4(v4_addr) => {
                            v4_ips.insert((domain.clone(), *v4_addr.ip()));
                        }
                        std::net::SocketAddr::V6(v6_addr) => {
                            v6_ips.insert((domain.clone(), *v6_addr.ip()));
                        }
                    }
                }
            }
            Err(_) => {
                unresolved_domains.insert(domain);
            }
        }
    }

    if !unresolved_domains.is_empty() {
        ret.push_str("\n# Unresolved domains:\n");
        for domain in unresolved_domains.into_iter() {
            ret.push_str(&format!("# {} not resolved\n", domain));
        }
    }

    ret.push_str("\n# IPv4 addresses:\n");

    fn find_max_length<T: std::fmt::Display>(ips: &HashSet<(String, T)>) -> usize {
        let mut max_ip_len = 0;
        ips.iter().for_each(|(_, ip)| {
            let len = ip.to_string().len();
            if len > max_ip_len {
                max_ip_len = len;
            }
        });
        max_ip_len
    }

    // find max length of v4 ip
    let max_v4_ip_len = find_max_length(&v4_ips);

    for (domain, ip) in v4_ips.into_iter() {
        ret.push_str(&format!("{:width$} {}\n", ip, domain, width = max_v4_ip_len));
    }

    if !v6_ips.is_empty() {
        ret.push_str("\n# IPv6 addresses:\n");
    }

    // find max length of v6 ip
    find_max_length(&v6_ips);

    for (domain, ip) in v6_ips.into_iter() {
        ret.push_str(&format!("{:width$} {}\n", ip, domain, width = max_v4_ip_len));
    }

    ret.push_str("####### Onenote Hosts End #######\n");

    ret
}