#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;

use failure::ResultExt;
use std::env;
use tokio::net::TcpStream;

use ipfs_resolver_common::{logging, wantlist, Result};
use wantlist_client_lib::net::Connection;

mod prom;

#[tokio::main]
async fn main() -> Result<()> {
    logging::set_up_logging(false)?;

    let num_cancels = prom::ENTRIES_RECEIVED
        .get_metric_with_label_values(&["cancel", "false"])
        .unwrap();
    let num_want_block = prom::ENTRIES_RECEIVED
        .get_metric_with_label_values(&["want_block", "false"])
        .unwrap();
    let num_want_block_send_dont_have = prom::ENTRIES_RECEIVED
        .get_metric_with_label_values(&["want_block", "true"])
        .unwrap();
    let num_want_have = prom::ENTRIES_RECEIVED
        .get_metric_with_label_values(&["want_have", "false"])
        .unwrap();
    let num_want_have_send_dont_have = prom::ENTRIES_RECEIVED
        .get_metric_with_label_values(&["want_have", "true"])
        .unwrap();
    let num_unknown = prom::ENTRIES_RECEIVED
        .get_metric_with_label_values(&["unknown", "false"])
        .unwrap();

    let num_connected_found = prom::CONNECTION_EVENTS
        .get_metric_with_label_values(&["true", "true"])
        .unwrap();
    let num_connected_not_found = prom::CONNECTION_EVENTS
        .get_metric_with_label_values(&["true", "false"])
        .unwrap();
    let num_disconnected_found = prom::CONNECTION_EVENTS
        .get_metric_with_label_values(&["false", "true"])
        .unwrap();
    let num_disconnected_not_found = prom::CONNECTION_EVENTS
        .get_metric_with_label_values(&["false", "false"])
        .unwrap();

    let num_messages = &*prom::MESSAGES_RECEIVED;

    info!("reading .env...");
    dotenv::dotenv().ok();

    let listen_addr = env::var("WANTLIST_CLIENT_PROMETHEUS_LISTEN_ADDR")
        .context("WANTLIST_CLIENT_PROMETHEUS_LISTEN_ADDR must be set")?
        .parse()
        .expect("invalid WANTLIST_CLIENT_PROMETHEUS_LISTEN_ADDR");

    info!("starting prometheus stuff..");
    prom::run_prometheus(listen_addr)?;

    let addr =
        env::var("WANTLIST_LOGGING_TCP_ADDRESS").expect("WANTLIST_LOGGING_TCP_ADDRESS must be set");

    info!("connecting to wantlist server at {}...", addr);
    let conn = TcpStream::connect(addr.as_str()).await?;

    let client = Connection::new(conn).await?;
    let _remote = client.remote;
    let mut messages_in = client.messages_in;

    while let Some(wl) = messages_in.recv().await {
        if wl.peer_connected.is_some() && wl.peer_connected.unwrap() {
            // Unwrap this because I hope that works...
            if wl.connect_event_peer_found.unwrap() {
                num_connected_found.inc();
            } else {
                num_connected_not_found.inc();
            }
            println!(
                "{} {} {:38} {:25}",
                wl.timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                wl.peer,
                match &wl.address {
                    Some(address) => address.to_string(),
                    None => "".to_string(),
                },
                if wl.connect_event_peer_found.unwrap() {
                    "CONNECTED; FOUND"
                } else {
                    "CONNECTED; NOT FOUND"
                }
            )
        } else if wl.peer_disconnected.is_some() && wl.peer_disconnected.unwrap() {
            if wl.connect_event_peer_found.unwrap() {
                num_disconnected_found.inc();
            } else {
                num_disconnected_not_found.inc();
            }
            println!(
                "{} {} {:38} {:25}",
                wl.timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                wl.peer,
                match &wl.address {
                    Some(address) => address.to_string(),
                    None => "".to_string(),
                },
                if wl.connect_event_peer_found.unwrap() {
                    "DISCONNECTED; FOUND"
                } else {
                    "DISCONNECTED; NOT FOUND"
                }
            )
        } else if wl.received_entries.is_some() {
            num_messages.inc();

            match wl.received_entries {
                Some(entries) => {
                    for entry in entries.iter() {
                        if entry.cancel {
                            num_cancels.inc();
                        } else if entry.want_type == wantlist::JSON_WANT_TYPE_BLOCK {
                            if entry.send_dont_have {
                                num_want_block_send_dont_have.inc();
                            } else {
                                num_want_block.inc();
                            }
                        } else if entry.want_type == wantlist::JSON_WANT_TYPE_HAVE {
                            if entry.send_dont_have {
                                num_want_have_send_dont_have.inc();
                            } else {
                                num_want_have.inc();
                            }
                        } else {
                            num_unknown.inc();
                        }

                        println!(
                            "{} {} {:38} {:4} {:25} ({:10}) {}",
                            wl.timestamp
                                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                            wl.peer,
                            match &wl.address {
                                Some(address) => address.to_string(),
                                None => "".to_string(),
                            },
                            match wl.full_want_list {
                                Some(f) =>
                                    if f {
                                        "FULL"
                                    } else {
                                        "INC"
                                    },
                                None => {
                                    "???"
                                }
                            },
                            if entry.cancel {
                                "CANCEL".to_string()
                            } else if entry.want_type == wantlist::JSON_WANT_TYPE_BLOCK {
                                if entry.send_dont_have {
                                    "WANT_BLOCK|SEND_DONT_HAVE".to_string()
                                } else {
                                    "WANT_BLOCK".to_string()
                                }
                            } else if entry.want_type == wantlist::JSON_WANT_TYPE_HAVE {
                                if entry.send_dont_have {
                                    "WANT_HAVE|SEND_DONT_HAVE".to_string()
                                } else {
                                    "WANT_HAVE".to_string()
                                }
                            } else {
                                format!("WANT_UNKNOWN_TYPE_{}", entry.want_type)
                            },
                            entry.priority,
                            entry.cid.path
                        )
                    }
                }
                None => println!("empty entries"),
            }
        } else {
            println!("no connect/disconnect event and no entries?")
        }
    }

    println!("shut down");

    Ok(())
}
