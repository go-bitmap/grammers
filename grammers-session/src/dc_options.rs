// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2-0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::net::{Ipv4Addr, SocketAddrV4, SocketAddrV6};

use crate::types::DcOption;

pub(crate) const DEFAULT_DC: i32 = 2;

const PRODUCTION_PORT: u16 = 443;
const TEST_PORT: u16 = 35349;

fn ipv4(a: u8, b: u8, c: u8, d: u8, port: u16) -> SocketAddrV4 {
    SocketAddrV4::new(Ipv4Addr::new(a, b, c, d), port)
}

fn ipv6(a: u8, b: u8, c: u8, d: u8, port: u16) -> SocketAddrV6 {
    SocketAddrV6::new(ipv4(a, b, c, d, port).ip().to_ipv6_compatible(), port, 0, 0)
}

/// Hardcoded known `static` options from `functions::help::GetConfig`.
///
/// Returns production server addresses (port 443) when `is_test` is `false`,
/// or test server addresses (port 35349) when `is_test` is `true`.
pub fn known_dc_options(is_test: bool) -> [DcOption; 5] {
    let port = if is_test { TEST_PORT } else { PRODUCTION_PORT };

    if is_test {
        [
            DcOption {
                id: 1,
                ipv4: ipv4(47, 237, 112, 224, port),
                ipv6: ipv6(47, 237, 112, 224, port),
                auth_key: None,
            },
            DcOption {
                id: 2,
                ipv4: ipv4(47, 237, 112, 224, port),
                ipv6: ipv6(47, 237, 112, 224, port),
                auth_key: None,
            },
            DcOption {
                id: 3,
                ipv4: ipv4(47, 237, 112, 224, port),
                ipv6: ipv6(47, 237, 112, 224, port),
                auth_key: None,
            },
            DcOption {
                id: 4,
                ipv4: ipv4(47, 237, 112, 224, port),
                ipv6: ipv6(47, 237, 112, 224, port),
                auth_key: None,
            },
            DcOption {
                id: 5,
                ipv4: ipv4(47, 237, 112, 224, port),
                ipv6: ipv6(47, 237, 112, 224, port),
                auth_key: None,
            },
        ]
    } else {
        [
            DcOption {
                id: 1,
                ipv4: ipv4(149, 154, 175, 53, port),
                ipv6: ipv6(149, 154, 175, 53, port),
                auth_key: None,
            },
            DcOption {
                id: 2,
                ipv4: ipv4(149, 154, 167, 41, port),
                ipv6: ipv6(149, 154, 167, 41, port),
                auth_key: None,
            },
            DcOption {
                id: 3,
                ipv4: ipv4(149, 154, 175, 100, port),
                ipv6: ipv6(149, 154, 175, 100, port),
                auth_key: None,
            },
            DcOption {
                id: 4,
                ipv4: ipv4(149, 154, 167, 92, port),
                ipv6: ipv6(149, 154, 167, 92, port),
                auth_key: None,
            },
            DcOption {
                id: 5,
                ipv4: ipv4(91, 108, 56, 104, port),
                ipv6: ipv6(91, 108, 56, 104, port),
                auth_key: None,
            },
        ]
    }
}

/// Default DC options for production environment.
/// This is kept for backward compatibility where environment is not available.
pub(crate) const KNOWN_DC_OPTIONS: [DcOption; 5] = [
    DcOption {
        id: 1,
        ipv4: SocketAddrV4::new(Ipv4Addr::new(149, 154, 175, 53), PRODUCTION_PORT),
        ipv6: SocketAddrV6::new(
            Ipv4Addr::new(149, 154, 175, 53).to_ipv6_compatible(),
            PRODUCTION_PORT,
            0,
            0,
        ),
        auth_key: None,
    },
    DcOption {
        id: 2,
        ipv4: SocketAddrV4::new(Ipv4Addr::new(149, 154, 167, 41), PRODUCTION_PORT),
        ipv6: SocketAddrV6::new(
            Ipv4Addr::new(149, 154, 167, 41).to_ipv6_compatible(),
            PRODUCTION_PORT,
            0,
            0,
        ),
        auth_key: None,
    },
    DcOption {
        id: 3,
        ipv4: SocketAddrV4::new(Ipv4Addr::new(149, 154, 175, 100), PRODUCTION_PORT),
        ipv6: SocketAddrV6::new(
            Ipv4Addr::new(149, 154, 175, 100).to_ipv6_compatible(),
            PRODUCTION_PORT,
            0,
            0,
        ),
        auth_key: None,
    },
    DcOption {
        id: 4,
        ipv4: SocketAddrV4::new(Ipv4Addr::new(149, 154, 167, 92), PRODUCTION_PORT),
        ipv6: SocketAddrV6::new(
            Ipv4Addr::new(149, 154, 167, 92).to_ipv6_compatible(),
            PRODUCTION_PORT,
            0,
            0,
        ),
        auth_key: None,
    },
    DcOption {
        id: 5,
        ipv4: SocketAddrV4::new(Ipv4Addr::new(91, 108, 56, 104), PRODUCTION_PORT),
        ipv6: SocketAddrV6::new(
            Ipv4Addr::new(91, 108, 56, 104).to_ipv6_compatible(),
            PRODUCTION_PORT,
            0,
            0,
        ),
        auth_key: None,
    },
];
