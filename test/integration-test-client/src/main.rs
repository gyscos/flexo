use crate::http_client::{GetRequestTest, http_get, http_get_with_header_chunked, ChunkPattern, ConnAddr, GetRequest};
use std::time::Duration;
use crate::http_client::ClientHeader::{AutoGenerated, Custom};

mod http_client;

fn main() {
    test_malformed_header();
    println!("test_malformed_header: [SUCCESS]");
    test_partial_header();
    println!("test_partial_header: [SUCCESS]");
    test_persistent_connections_c2s();
    println!("test_persistent_connections: [SUCCESS]");
}

fn test_partial_header() {
    // Sending the header in multiple TCP segments does not cause the server to crash
    let uri = GetRequestTest {
        conn_addr: ConnAddr {
            host: "flexo-server-slow-primary".to_owned(),
            port: 7878,
        },
        get_requests: vec![GetRequest {
            path: "/community/os/x86_64/lostfiles-4.03-1-any.pkg.tar.xz".to_owned(),
            client_header: AutoGenerated,
        }]
    };
    let pattern = ChunkPattern {
        chunk_size: 3,
        wait_interval: Duration::from_millis(300),
    };
    let results = http_get_with_header_chunked(uri, Some(pattern));
    assert_eq!(results.len(), 1);
    let result = results.get(0).unwrap();
    assert_eq!(result.header_result.status_code, 200);
}

fn test_malformed_header() {
    let malformed_header = "this is not a valid http header".to_owned();
    let uri1 = GetRequestTest {
        conn_addr: ConnAddr {
            host: "flexo-server".to_owned(),
            port: 7878,
        },
        get_requests: vec![GetRequest {
            path: "/".to_owned(),
            client_header: Custom(malformed_header),
        }],
    };
    let results = http_get(uri1);
    assert_eq!(results.len(), 1);
    let result = results.get(0).unwrap();
    println!("result: {:?}", &result);
    assert_eq!(result.header_result.status_code, 400);
    // Test if the server is still up, i.e., the previous request hasn't crashed it:
    let uri2 = GetRequestTest {
        conn_addr: ConnAddr {
            host: "flexo-server".to_owned(),
            port: 7878,
        },
        get_requests: vec![GetRequest {
            path: "/status".to_owned(),
            client_header: AutoGenerated,
        }],
    };
    let results = http_get(uri2);
    assert_eq!(results.len(), 1);
    let result = results.get(0).unwrap();
    println!("result: {:?}", &result);
    assert_eq!(result.header_result.status_code, 200);
}

fn test_persistent_connections_c2s() {
    let request_test = GetRequestTest {
        conn_addr: ConnAddr {
            host: "flexo-server-delay".to_owned(),
            port: 7878,
        },
        get_requests: vec![
            GetRequest {
                path: "/test_1".to_owned(),
                client_header: AutoGenerated
            },
            GetRequest {
                path: "/test_2".to_owned(),
                client_header: AutoGenerated
            },
            GetRequest {
                path: "/test_3".to_owned(),
                client_header: AutoGenerated
            },
        ]
    };
    let results = http_get(request_test);
    assert_eq!(results.len(), 3);
    let all_ok = results.iter().all(|r| r.header_result.status_code == 200);
    assert!(all_ok);
}

