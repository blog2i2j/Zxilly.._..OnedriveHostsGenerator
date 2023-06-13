use std::collections::HashSet;
use std::net::{Ipv4Addr, Ipv6Addr, ToSocketAddrs};
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

pub fn render(ipv4: bool, ipv6: bool) -> String {
    let mut ret = String::new();
    ret.push_str("####### Onenote Hosts Start #######\n");
    ret.push_str("# This file is generated by https://github.com/Zxilly/OnedriveHostsGenerator\n");

    let now = Local::now();

    ret.push_str(&format!("# Generate time: {}\n", now.format("%Y-%m-%d %H:%M:%S")));

    let mut v4_ips: Vec<(String, Ipv4Addr)> = vec![];
    let mut v6_ips: Vec<(String, Ipv6Addr)> = vec![];
    let mut unresolved_domains: Vec<String> = vec![];

    for domain in DOMAIN_LIST.clone().into_iter() {
        let addr = (domain.as_ref(), 0).to_socket_addrs();
        match addr {
            Ok(mut addrs) => {
                for socket_addr in addrs.by_ref() {
                    match socket_addr {
                        std::net::SocketAddr::V4(v4_addr) => {
                            v4_ips.push((domain.clone(), *v4_addr.ip()));
                        }
                        std::net::SocketAddr::V6(v6_addr) => {
                            v6_ips.push((domain.clone(), *v6_addr.ip()));
                        }
                    }
                }
            }
            Err(_) => {
                unresolved_domains.push(domain);
            }
        }
    }

    if !unresolved_domains.is_empty() {
        ret.push_str("\n# Unresolved domains:\n");
        for domain in unresolved_domains.into_iter() {
            ret.push_str(&format!("# {} not resolved\n", domain));
        }
    }

    fn find_max_length<T: std::fmt::Display>(ips: &[(String, T)]) -> (usize, usize) {
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
        ret.push_str("\n# IPv4 addresses:\n");

        // find max length of v4 ip
        let (max_v4_ip_len, max_v4_domain_len) = find_max_length(&v4_ips);

        for (domain, ip) in v4_ips.into_iter() {
            ret.push_str(&format!("{:w1$} {:>w2$}\n", ip, domain, w1 = max_v4_ip_len, w2 = max_v4_domain_len));
        }
    }

    if ipv6 {
        if !v6_ips.is_empty() {
            ret.push_str("\n# IPv6 addresses:\n");
        } else {
            ret.push_str("\n# No IPv6 addresses resolved\n");
        }

        // find max length of v6 ip
        let (max_v6_ip_len, max_v6_domain_len) = find_max_length(&v6_ips);

        for (domain, ip) in v6_ips.into_iter() {
            ret.push_str(&format!("{:w1$} {:>w2$}\n", ip, domain, w1 = max_v6_ip_len, w2 = max_v6_domain_len));
        }
    }

    ret.push_str("####### Onenote Hosts End #######\n");

    ret
}