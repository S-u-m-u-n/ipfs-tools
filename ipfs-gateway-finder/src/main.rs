#[macro_use]
extern crate log;

use clap::{App, Arg};
use failure::ResultExt;
use ipfs_api::{IpfsClient, TryFromUri};
use ipfs_resolver_common::wantlist::JSONMessage;
use ipfs_resolver_common::{logging, Result};
use rand::{Rng, SeedableRng};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::io::Cursor;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use wantlist_client_lib::net::{APIClient, EventType};
use ipfs_api::IpfsApi;

#[tokio::main]
async fn main() -> Result<()> {
    logging::set_up_logging(false)?;

    do_probing().await?;

    Ok(())
}

/// The state for one gateway probe.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct ProbingState {
    data: Option<Vec<u8>>,
    cid_v1: Option<String>,
    cid_v0: Option<String>,

    http_request_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    http_requests_sent: Option<u32>,
    http_request_remote: Option<SocketAddr>,
    http_success_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    http_error_message: Option<String>,

    wantlist_message: Option<JSONMessage>,
}

/// Used for JSON printing
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct ProbingStateWithGatewayURL {
    gateway: String,
    gateway_url: String,

    #[serde(flatten)]
    state: ProbingState,
}

/// Does the actual probing.
async fn do_probing() -> Result<()> {
    let matches = App::new("IPFS Gateway Finder")
        .version(clap::crate_version!())
        .author("Leo Balduf <leobalduf@gmail.com>")
        .about("Finds overlay addresses of public IPFS gateways through probing their HTTP side with crafted content.\n\
        Prints results to STDOUT in JSON or CSV format, logs to STDERR.")
        .arg(
            Arg::with_name("bitswap_server_address")
                .long("monitor-logging-addr")
                .value_name("ADDRESS")
                .help("The address of the bitswap monitor to connect to")
                .default_value("localhost:4321")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("monitor_api_address")
                .long("monitor-api-addr")
                .value_name("ADDRESS")
                .help("The address of the HTTP IPFS API of the monitor")
                .default_value("localhost:5003")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("http_num_tries")
                .long("http-tries")
                .value_name("NUMBER OF TRIES")
                .help("The number of times the HTTP request to a gateway should be tried")
                .default_value("10")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("http_timeout_secs")
                .long("http-timeout")
                .value_name("SECONDS")
                .help("The request timeout in seconds for HTTP requests to a gateway")
                .default_value("60")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("gateway_list_url")
                .long("gateway-list")
                .value_name("URL")
                .help("The URL of the JSON gateway list to use")
                .default_value("https://raw.githubusercontent.com/ipfs/public-gateway-checker/master/gateways.json")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output_csv")
                .long("csv")
                .help("Whether to produce CSV output (instead of the default JSON output)")
        )
        .get_matches();

    // These all have defaults, so we can call unwrap safely.
    let bitswap_monitor_address = matches.value_of("bitswap_server_address").unwrap();
    info!(
        "using bitswap monitor logging address {}",
        bitswap_monitor_address
    );

    let monitor_api_address = matches.value_of("monitor_api_address").unwrap();
    info!("using monitoring node API at {}", monitor_api_address);
    let u = http::uri::Builder::new()
        .scheme(http::uri::Scheme::HTTP)
        .authority(monitor_api_address)
        .path_and_query("/")
        .build()
        .context("invalid monitor_api_address")?;
    let ipfs_client = ipfs_api::IpfsClient::from_str(u.to_string().as_str())?;

    let num_http_tries = matches
        .value_of("http_num_tries")
        .unwrap()
        .parse::<u32>()
        .context("invalid http_num_tries")?;
    info!("will try {} times via HTTP", num_http_tries);

    let http_timeout_secs = matches
        .value_of("http_timeout_secs")
        .unwrap()
        .parse::<u32>()
        .context("invalid http_timeout_secs")?;
    info!("will use {} seconds as HTTP timeout", http_timeout_secs);

    let produce_csv = matches.is_present("output_csv");
    if produce_csv {
        info!("will produce CSV output");
    } else {
        info!("will produce JSON output");
    }

    let gateway_list_url = Url::parse(matches.value_of("gateway_list_url").unwrap())
        .context("invalid gateway_list_url")?;

    // Get list of gateways.
    info!(
        "getting list of gateways from {}...",
        gateway_list_url.as_str()
    );
    let client = reqwest::blocking::Client::builder()
        .timeout(Some(Duration::from_secs(30)))
        .build()?;

    let gateway_list_resp = client.get(gateway_list_url).send()?;
    let gateway_list: Vec<String> = gateway_list_resp.json()?;
    info!("got {} gateways: {:?}", gateway_list.len(), gateway_list);

    // Initialize our states.
    let gateway_states: Arc<HashMap<String, Mutex<ProbingState>>> = Arc::new(
        gateway_list
            .into_iter()
            .map(|e| (e, Default::default()))
            .collect(),
    );

    // Generate random data to add to IPFS.
    info!("generating random data...");
    for (_, state) in gateway_states.iter() {
        let rng = rand::rngs::StdRng::from_entropy();
        let bytes: Vec<u8> = rng
            .sample_iter(rand::distributions::Standard)
            .take(1024)
            .collect();

        let mut state = state.lock().await;
        state.data = Some(bytes);
    }

    // Add to IPFS.
    info!("adding data to IPFS...");
    add_data_to_ipfs(&ipfs_client, gateway_states.clone())
        .await
        .context(
            "unable to add (all?) data to monitoring IPFS node. This might need manual cleanup",
        )?;

    // Wait for DHT propagation...
    info!("waiting some time for DHT propagation..");
    tokio::time::sleep(Duration::from_secs(60)).await;

    // Collect a list of all CIDs for easier searching.
    let mut cids = HashSet::new();
    for (_, state) in gateway_states.iter() {
        let state = state.lock().await;
        cids.insert(state.cid_v1.as_ref().unwrap().clone());
    }

    // Start listening for bitswap messages
    info!(
        "connecting to bitswap monitoring node at {}...",
        bitswap_monitor_address
    );
    let conn = TcpStream::connect(bitswap_monitor_address).await?;
    info!("connected.");

    let (monitoring_ready_tx, monitoring_ready_rx) = tokio::sync::oneshot::channel();
    monitor_bitswap(gateway_states.clone(), cids, conn, monitoring_ready_tx)
        .await
        .context("unable to start bitswap monitoring")?;

    info!("waiting for bitswap monitoring to be ready...");
    monitoring_ready_rx.await.unwrap();

    // Send one CID to each gateway
    info!("probing gateways...");
    // We have each of them send a value down this channel when they're done.
    // That way, we can wait for all of them to be finished (because we know how many we started).
    let (tx, mut done_rx) = tokio::sync::mpsc::channel(gateway_states.len());
    probe_http_gateways(
        num_http_tries,
        http_timeout_secs,
        gateway_states.clone(),
        tx,
    )
    .await;

    // Wait...
    info!("waiting for HTTP probing to finish...");
    for _i in 0..gateway_states.len() {
        done_rx.recv().await.unwrap()
    }

    info!("all HTTP workers are done or timed out, waiting some more time for bitswap messages...");
    tokio::time::sleep(Duration::from_secs(120)).await;

    // Remove data from IPFS.
    info!("removing data from monitoring IPFS node...");
    cleanup_ipfs(&ipfs_client, gateway_states.clone())
        .await
        .context(
            "unable to remove data from monitoring IPFS node. Probably needs manual cleanup",
        )?;

    // Print results
    info!("printing results..");
    if produce_csv {
        print_csv(gateway_states).await?;
    } else {
        print_json(gateway_states).await?;
    }

    Ok(())
}

/// Starts a task to listen on the specified bitswap monitor for any of the given CIDs.
async fn monitor_bitswap(
    gateway_states: Arc<HashMap<String, Mutex<ProbingState>>>,
    mut cids: HashSet<String>,
    conn: TcpStream,
    monitoring_ready_tx: tokio::sync::oneshot::Sender<()>,
) -> Result<()> {
    let (monitoring_client,mut event_chan) = APIClient::new(conn)
        .await
        .context("unable to create ipfs monitoring API client")?;
    monitoring_client.subscribe().await.context("unable to subscribe to events")?;

    tokio::task::spawn(async move {
        let mut sender = Some(monitoring_ready_tx);
        info!("receiving bitswap messages from...");
        while let Some(event) = event_chan.recv().await {
            if let Some(sender) = sender.take() {
                info!("got bitswap messages, connection is working");
                sender.send(()).unwrap();
            }

            match event.inner {
                EventType::BitswapMessage(msg) => {
                    for entry in &msg.wantlist_entries {
                        if cids.contains(&entry.cid.path) {
                            debug!("received interesting CID {}", entry.cid.path);
                            for (gw, state) in gateway_states.iter() {
                                let mut state = state.lock().await;
                                if state.wantlist_message.is_none() {
                                    if entry.cid.path.eq(state.cid_v1.as_ref().unwrap()) {
                                        info!(
                                        "got wantlist CID {} from peer {}, which is gateway {}",
                                        entry.cid.path, event.peer, gw
                                    );
                                        state.wantlist_message = Some(JSONMessage{
                                            timestamp: event.timestamp,
                                            peer: event.peer.clone(),
                                            address: None,
                                            received_entries: Some(msg.wantlist_entries.clone()),
                                            full_want_list: Some(msg.full_wantlist),
                                            peer_connected: None,
                                            peer_disconnected: None,
                                            connect_event_peer_found: None
                                        });
                                        break;
                                    }
                                }
                            }

                            // We remove this from our interesting CID list because we only need it once.
                            cids.remove(&entry.cid.path);

                            break;
                        }
                    }

                }
                EventType::ConnectionEvent(_) => {}
            }

        }
    });

    Ok(())
}

/// Adds the data of the given gateway states to the monitoring IPFS node via its API.
async fn add_data_to_ipfs(
    ipfs_client: &IpfsClient,
    gateway_states: Arc<HashMap<String, Mutex<ProbingState>>>,
) -> Result<()> {
    for (_, state) in gateway_states.iter() {
        let mut state = state.lock().await;
        let data = state.data.as_ref().unwrap().clone();
        let d = Cursor::new(data);

        let add_resp = ipfs_client
            .add(d)
            .await
            .context("unable to add data to IPFS")?;

        let c = cid::Cid::from_str(&add_resp.hash)?;
        let cid_v1 = cid::Cid::new_v1(c.codec(), c.hash().to_owned());
        let cid_v1_encoded = multibase::encode(multibase::Base::Base32Lower, cid_v1.to_bytes());
        info!("added CID {} = {}", add_resp.hash, cid_v1_encoded);

        state.cid_v0 = Some(add_resp.hash.clone());
        state.cid_v1 = Some(cid_v1_encoded);
    }

    Ok(())
}

/// Removes the data added for gateway probing from the monitoring IPFS node via its API.
async fn cleanup_ipfs(
    client: &ipfs_api::IpfsClient,
    gateway_states: Arc<HashMap<String, Mutex<ProbingState>>>,
) -> Result<()> {
    for (_, state) in gateway_states.iter() {
        let state = state.lock().await;
        let cid = state.cid_v0.as_ref().unwrap();
        debug!("cleaning up CID {}", cid);

        client.pin_rm(cid.as_str(), true).await?;

        info!("cleaned up CID {}", cid);
    }

    Ok(())
}

/// Launches asynchronous tasks to request the generated data via the HTTP side of the given gateways.
async fn probe_http_gateways(
    num_http_tries: u32,
    http_timeout_secs: u32,
    gateway_states: Arc<HashMap<String, Mutex<ProbingState>>>,
    tx: Sender<()>,
) {
    for (gateway, state) in gateway_states.iter() {
        let task_state = gateway_states.clone();
        let task_gateway = gateway.clone();
        let task_cid = {
            let state = state.lock().await;
            state.cid_v1.as_ref().unwrap().clone()
        };
        let task_done = tx.clone();
        tokio::task::spawn(async move {
            let res = probe_gateway(
                task_state.clone(),
                &task_gateway,
                &task_cid,
                num_http_tries,
                Duration::from_secs(http_timeout_secs as u64),
            )
            .await;

            match res {
                Ok(()) => {}
                Err(err) => {
                    error!("HTTP failed for {}: {:?}", task_gateway, err);
                    let mut state = task_state.get(&task_gateway).unwrap().lock().await;
                    if state.http_error_message.is_none() {
                        state.http_error_message = Some(format!("{}", err))
                    } //what if not? how does that even happen...
                }
            }

            task_done.send(()).await.unwrap();
        });
    }
}

/// Probes a single gateway via HTTP.
/// This performs multiple requests and waits for the given timeout duration each time.
async fn probe_gateway(
    gateway_state: Arc<HashMap<String, Mutex<ProbingState>>>,
    gateway_url: &str,
    cid: &str,
    num_tries: u32,
    timeout: Duration,
) -> Result<()> {
    let mut url = Url::parse(gateway_url.replace(":hash", &cid).as_str())?;
    url.set_fragment(Some("x-ipfs-companion-no-redirect"));
    let mut last_err = None;

    {
        let mut state = gateway_state.get(gateway_url).unwrap().lock().await;
        state.http_request_timestamp = Some(chrono::Utc::now());
    }

    for i in 0..num_tries {
        debug!("requesting {}, try  {}...", url, i + 1);
        let resp = reqwest::Client::builder()
            .timeout(timeout.clone())
            .build()?
            .get(url.clone())
            .send()
            .await;
        {
            let mut state = gateway_state.get(gateway_url).unwrap().lock().await;
            state.http_requests_sent = Some(i + 1);
        }
        match resp {
            Ok(resp) => {
                let remote = resp.remote_addr().clone();
                let body = resp.bytes().await?;

                let mut state = gateway_state.get(gateway_url).unwrap().lock().await;
                state.http_request_remote = remote;

                if !body.eq(state.data.as_ref().unwrap()) {
                    debug!("data mismatch for gateway {}, got {} bytes, expected {} (and maybe different bytes)", gateway_url, body.len(), state.data.as_ref().unwrap().len());
                    last_err = Some(format!(
                        "data mismatch, expected {} bytes, got {} bytes (and maybe different ones)",
                        state.data.as_ref().unwrap().len(),
                        body.len()
                    ));
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    continue;
                }

                info!("got correct response from gateway {}", gateway_url);
                state.http_success_timestamp = Some(chrono::Utc::now());
                return Ok(());
            }
            Err(err) => {
                info!("error requesting {}, try {}: {:?}", gateway_url, i + 1, err);
                last_err = Some(format!("{}", err))
            }
        }
    }
    info!(
        "did not get a correct response from gateway {} after {} tries",
        gateway_url, num_tries
    );
    let mut state = gateway_state.get(gateway_url).unwrap().lock().await;
    if let Some(err_msg) = last_err {
        state.http_error_message = Some(format!(
            "did not get a correct response after {} tries, last error: {}",
            num_tries, err_msg
        ));
    }

    Ok(())
}

/// Prints the results as a stream of JSON objects.
async fn print_json(gateway_states: Arc<HashMap<String, Mutex<ProbingState>>>) -> Result<()> {
    for (gateway, state) in gateway_states.iter() {
        let state = state.lock().await;
        let gw = gateway.replace(":hash.", "");
        let gw_url = Url::parse(&gw)?;
        let augmented = ProbingStateWithGatewayURL {
            gateway: gw_url.host_str().unwrap().to_string(),
            gateway_url: gateway.clone(),
            state: state.clone(),
        };
        println!("{}", serde_json::to_string(&augmented).unwrap())
    }

    Ok(())
}

/// Prints the results as CSV.
async fn print_csv(gateway_states: Arc<HashMap<String, Mutex<ProbingState>>>) -> Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(vec![]);
    writer.write_field("gateway")?;
    writer.write_field("gateway_url")?;
    writer.write_field("cid")?;
    writer.write_field("http_request_ts")?;
    writer.write_field("http_request_ts_subsec_millis")?;
    writer.write_field("http_error_message")?;
    writer.write_field("http_success_ts")?;
    writer.write_field("http_success_ts_subsec_millis")?;
    writer.write_field("http_requests_sent")?;
    writer.write_field("http_remote")?;
    writer.write_field("first_wl_ts")?;
    writer.write_field("first_wl_ts_subsec_millis")?;
    writer.write_field("first_wl_peer")?;
    writer.write_field("first_wl_address")?;
    writer.write_record(None::<&[u8]>)?;

    for (gateway, state) in gateway_states.iter() {
        let state = state.lock().await;
        let gw = gateway.replace(":hash.", "");
        let gw_url = Url::parse(&gw)?;
        writer.write_field(gw_url.host_str().unwrap())?;
        writer.write_field(gateway)?;
        writer.write_field(
            state
                .cid_v1
                .as_ref()
                .map_or("".to_string(), |cid| format!("{}", cid)),
        )?;
        writer.write_field(
            state
                .http_request_timestamp
                .map_or("".to_string(), |ts| format!("{}", ts.timestamp())),
        )?;
        writer.write_field(state.http_request_timestamp.map_or("".to_string(), |ts| {
            format!("{}", ts.timestamp_subsec_millis())
        }))?;
        writer.write_field(
            state
                .http_error_message
                .as_ref()
                .or(Some(&"".to_string()))
                .unwrap(),
        )?;
        writer.write_field(
            state
                .http_success_timestamp
                .map_or("".to_string(), |ts| format!("{}", ts.timestamp())),
        )?;
        writer.write_field(state.http_success_timestamp.map_or("".to_string(), |ts| {
            format!("{}", ts.timestamp_subsec_millis())
        }))?;
        writer.write_field(
            state
                .http_requests_sent
                .map_or("".to_string(), |reqs| format!("{}", reqs)),
        )?;
        writer.write_field(
            state
                .http_request_remote
                .map_or("".to_string(), |remote| format!("{}", remote)),
        )?;
        writer.write_field(
            state
                .wantlist_message
                .as_ref()
                .map_or("".to_string(), |msg| {
                    format!("{}", msg.timestamp.clone().timestamp())
                }),
        )?;
        writer.write_field(
            state
                .wantlist_message
                .as_ref()
                .map_or("".to_string(), |msg| {
                    format!("{}", msg.timestamp.clone().timestamp_subsec_millis())
                }),
        )?;
        writer.write_field(
            state
                .wantlist_message
                .as_ref()
                .map_or("".to_string(), |msg| format!("{}", msg.peer.clone())),
        )?;
        writer.write_field(
            state
                .wantlist_message
                .as_ref()
                .map_or("".to_string(), |msg| {
                    format!(
                        "{}",
                        msg.address
                            .as_ref()
                            .map_or("".to_string(), |addr| format!("{}", addr))
                    )
                }),
        )?;
        writer.write_record(None::<&[u8]>)?;
    }

    let data = String::from_utf8(writer.into_inner()?)?;
    println!("{}", data);

    Ok(())
}