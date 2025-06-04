# Haven

> A modern replacement for dnsmasq. It not only has dns and dhcp, but also can do more. Built with Rust.

## Features

- [ ] DNS Server
  - [ ] Rule based upstream DNS Server
  - [ ] Auto import name record from DHCP
  - [ ] Support UDP, DoH
- [ ] DHCP Server
  - [ ] DHCPv4 and DHCPv6
  - [ ] IPv6 RA Support
- [ ] Packet Routing
  - [ ] Rule based routing
  - [ ] Pluginable upstream.
- [ ] Integrateable
  - [ ] Work with NetworkManager and Firewalld
  - [ ] RESTful API support
  - [ ] Database support.
  - [ ] External cache support

## Documents

Refer to [design](./docs/design.md) to visit the design of db schema and cache.
Refer to [api](./docs/api.md) to visit the API.
