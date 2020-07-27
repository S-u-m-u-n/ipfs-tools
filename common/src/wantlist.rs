use crate::Result;
use failure::{err_msg, ResultExt};
use parity_multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem;

/// Constants for the `want_type` field of an `JSONWantlistEntry`.
pub const JSON_WANT_TYPE_BLOCK: i32 = 0;
pub const JSON_WANT_TYPE_HAVE: i32 = 1;

/// The JSON encoding of a CID.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct JsonCID {
    #[serde(rename = "/")]
    pub path: String,
}

/// A single entry of a wantlist message.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct JSONWantlistEntry {
    #[serde(alias = "Priority")]
    pub priority: i32,
    #[serde(alias = "Cancel")]
    pub cancel: bool,
    #[serde(alias = "SendDontHave")]
    pub send_dont_have: bool,
    #[serde(alias = "Cid")]
    pub cid: JsonCID,
    #[serde(alias = "WantType")]
    pub want_type: i32,
}

/// A wantlist message.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct JSONMessage {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub peer: String,
    pub address: Option<Multiaddr>,
    pub received_entries: Option<Vec<JSONWantlistEntry>>,
    pub full_want_list: Option<bool>,
    pub peer_connected: Option<bool>,
    pub peer_disconnected: Option<bool>,
    pub connect_event_peer_found: Option<bool>,
}

/// Message type constants for CSV files.
pub const CSV_MESSAGE_TYPE_INCREMENTAL: i32 = 1;
pub const CSV_MESSAGE_TYPE_FULL: i32 = 2;
pub const CSV_MESSAGE_TYPE_SYNTHETIC: i32 = 3;

/// Entry type constants for CSV files.
pub const CSV_ENTRY_TYPE_CANCEL: i32 = 1;
pub const CSV_ENTRY_TYPE_WANT_BLOCK: i32 = 2;
pub const CSV_ENTRY_TYPE_WANT_BLOCK_SEND_DONT_HAVE: i32 = 3;
pub const CSV_ENTRY_TYPE_WANT_HAVE: i32 = 4;
pub const CSV_ENTRY_TYPE_WANT_HAVE_SEND_DONT_HAVE: i32 = 5;
pub const CSV_ENTRY_TYPE_SYNTHETIC_CANCEL_FULL_WANTLIST: i32 = 6;
pub const CSV_ENTRY_TYPE_SYNTHETIC_CANCEL_DISCONNECT: i32 = 7;

/// Connection event type constants for CSV files.
pub const CSV_CONNECTION_EVENT_CONNECTED_FOUND: i32 = 1;
pub const CSV_CONNECTION_EVENT_CONNECTED_NOT_FOUND: i32 = 2;
pub const CSV_CONNECTION_EVENT_DISCONNECTED_FOUND: i32 = 3;
pub const CSV_CONNECTION_EVENT_DISCONNECTED_NOT_FOUND: i32 = 4;

/// Reasons for duplicate messages.
pub const CSV_DUPLICATE_STATUS_NO_DUP: u32 = 0;
pub const CSV_DUPLICATE_STATUS_DUP_FULL_WANTLIST: u32 = 1;
pub const CSV_DUPLICATE_STATUS_DUP_RECONNECT: u32 = 2;
pub const CSV_DUPLICATE_STATUS_DUP_SLIDING_WINDOW: u32 = 4;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CSVWantlistEntry {
    pub message_id: i64,
    pub message_type: i32,
    pub timestamp_seconds: i64,
    pub timestamp_subsec_milliseconds: u32,
    pub peer_id: String,
    pub address: String,
    pub priority: i32,
    pub entry_type: i32,
    pub cid: String,
    pub duplicate_status: u32,
    pub sliding_window_smallest_match: u32,
}

impl CSVWantlistEntry {
    pub fn from_wantlist_entries(
        entries: Vec<WantlistEntry>,
        message: &JSONMessage,
        id: i64,
        message_type: i32,
        entry_type: i32,
        duplicate_status: u32,
        sliding_window_smallest_match: u32,
    ) -> Vec<CSVWantlistEntry> {
        let message_timestamp_seconds = message.timestamp.timestamp();
        let message_timestamp_subsec_millis = message.timestamp.timestamp_subsec_millis();

        entries
            .into_iter()
            .map(|e| CSVWantlistEntry {
                message_id: id,
                message_type,
                timestamp_seconds: message_timestamp_seconds,
                timestamp_subsec_milliseconds: message_timestamp_subsec_millis,
                peer_id: message.peer.clone(),
                address: match &message.address {
                    Some(address) => address.to_string(),
                    None => "".to_string(),
                },
                priority: 0,
                entry_type,
                cid: e.cid,
                duplicate_status,
                sliding_window_smallest_match,
            })
            .collect()
    }

    pub fn from_json_message(message: JSONMessage, id: i64) -> Result<Vec<CSVWantlistEntry>> {
        let entries = message
            .received_entries
            .ok_or_else(|| err_msg("no entries when converting from JSON message"))?;
        let peer = message.peer.clone();
        let timestamp = message.timestamp;
        let timestamp_seconds = timestamp.timestamp();
        let timestamp_subsec_millis = timestamp.timestamp_subsec_millis();
        let full_want_list = message.full_want_list;
        let address = match &message.address {
            Some(address) => address.to_string(),
            None => "".to_string(),
        };

        let csv_entries = entries
            .into_iter()
            .map(|entry| CSVWantlistEntry {
                message_id: id,
                message_type: match full_want_list {
                    Some(full) => {
                        if full {
                            CSV_MESSAGE_TYPE_FULL
                        } else {
                            CSV_MESSAGE_TYPE_INCREMENTAL
                        }
                    }
                    None => {
                        // TODO forbid this
                        CSV_MESSAGE_TYPE_INCREMENTAL
                    }
                },
                timestamp_seconds,
                timestamp_subsec_milliseconds: timestamp_subsec_millis,
                peer_id: peer.clone(),
                address: address.clone(),
                priority: entry.priority,
                entry_type: if entry.cancel {
                    CSV_ENTRY_TYPE_CANCEL
                } else {
                    match entry.want_type {
                        JSON_WANT_TYPE_BLOCK => {
                            if entry.send_dont_have {
                                CSV_ENTRY_TYPE_WANT_BLOCK_SEND_DONT_HAVE
                            } else {
                                CSV_ENTRY_TYPE_WANT_BLOCK
                            }
                        }
                        JSON_WANT_TYPE_HAVE => {
                            if entry.send_dont_have {
                                CSV_ENTRY_TYPE_WANT_HAVE_SEND_DONT_HAVE
                            } else {
                                CSV_ENTRY_TYPE_WANT_HAVE
                            }
                        }
                        _ => panic!(format!("unknown JSON want type {}", entry.want_type)),
                    }
                },
                cid: entry.cid.path,
                duplicate_status: CSV_DUPLICATE_STATUS_NO_DUP,
                sliding_window_smallest_match: 0,
            })
            .collect();

        Ok(csv_entries)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CSVConnectionEvent {
    pub message_id: i64,
    pub timestamp_seconds: i64,
    pub timestamp_subsec_millis: u32,
    pub peer_id: String,
    pub address: String,
    pub event_type: i32,
}

impl CSVConnectionEvent {
    pub fn from_json_message(message: JSONMessage, id: i64) -> Result<CSVConnectionEvent> {
        let found = message
            .connect_event_peer_found
            .ok_or_else(|| err_msg("connection event is missing found"))?;
        let disconnected = message
            .peer_disconnected
            .ok_or_else(|| err_msg("connection event is missing peer_disconnected"))?;
        let connected = message
            .peer_connected
            .ok_or_else(|| err_msg("connection event is missing peer_connected"))?;
        ensure!(
            disconnected || connected,
            "connection event needs either connected or disconnected to be set.."
        );
        ensure!(
            !(disconnected && connected),
            "connection event needs exactly one of connected or disconnected to be set"
        );

        let event_type = if disconnected {
            if found {
                CSV_CONNECTION_EVENT_DISCONNECTED_FOUND
            } else {
                CSV_CONNECTION_EVENT_DISCONNECTED_NOT_FOUND
            }
        } else {
            if found {
                CSV_CONNECTION_EVENT_CONNECTED_FOUND
            } else {
                CSV_CONNECTION_EVENT_CONNECTED_NOT_FOUND
            }
        };

        Ok(CSVConnectionEvent {
            message_id: id,
            timestamp_seconds: message.timestamp.timestamp(),
            timestamp_subsec_millis: message.timestamp.timestamp_subsec_millis(),
            peer_id: message.peer,
            address: match &message.address {
                Some(address) => address.to_string(),
                None => "".to_string(),
            },
            event_type,
        })
    }
}

/*
enum WantType {
    Block,
    Have,
}*/

#[derive(Clone, Debug)]
pub struct WantlistEntry {
    cid: String,
    //want_type: WantType,
    ts: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug)]
pub struct Ledger {
    connection_count: i32,
    wanted_entries: Vec<WantlistEntry>,
    wanted_entries_before_disconnect: Option<Vec<WantlistEntry>>,
    connected_ts: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EngineSimulationConfig {
    pub allow_empty_full_wantlist: bool,
    pub allow_empty_connection_event: bool,
    pub insert_full_wantlist_synth_cancels: bool,
    pub insert_disconnect_synth_cancels: bool,
    pub insert_full_wantlist_duplicate_markers: bool,
    pub insert_reconnect_duplicate_markers: bool,
    pub reconnect_duplicate_duration_secs: u32,
    pub insert_sliding_window_duplicate_markers: bool,
    pub sliding_window_lengths: Vec<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct EngineSimulation {
    peers: HashMap<String, Ledger>,
    cfg: EngineSimulationConfig,
}

#[derive(Clone, Debug, Default)]
pub struct IngestResult {
    pub missing_ledger: bool,
    pub wantlist_entries: Option<Vec<CSVWantlistEntry>>,
    pub connection_event: Option<CSVConnectionEvent>,
}

impl EngineSimulation {
    pub fn new(mut cfg: EngineSimulationConfig) -> Result<EngineSimulation> {
        cfg.sliding_window_lengths.sort();
        if cfg.sliding_window_lengths.len() > 0 {
            ensure!(
                cfg.sliding_window_lengths[0] > 0,
                "sliding windows must be >0"
            )
        }
        Ok(EngineSimulation {
            peers: HashMap::new(),
            cfg,
        })
    }

    fn ingest_wantlist_message(
        &mut self,
        msg: &JSONMessage,
        entries: &Vec<JSONWantlistEntry>,
        msg_id: i64,
    ) -> Result<IngestResult> {
        let mut missing_ledger = false;
        let ledger = self.peers.entry(msg.peer.clone()).or_insert_with(|| {
            debug!("received wantlist from {} ({:?}), but don't have a ledger for that peer. Starting empty one with one connection.", msg.peer, msg.address);
            missing_ledger = true;
            Ledger { connection_count: 1, wanted_entries: Default::default(), wanted_entries_before_disconnect: Default::default(), connected_ts: Some(msg.timestamp.clone()) }
        });
        if ledger.connection_count == 0 {
            warn!("got wantlist entries from peer {}, but we are still disconnected from that peer (was previously connected). This is either an error in how IPFS notifies about connection events, or in how we ingest them.", msg.peer);
            ledger.connection_count = 1;
            ledger.connected_ts = Some(msg.timestamp.clone());
        }
        assert!(ledger.connection_count > 0);

        let (mut full_wl_dups, mut full_wl_synth_cancels) = (None, None);
        let (new_wants, new_cancels) = Self::split_wants_cancels(&entries);

        let sliding_window_dups = Self::get_duplicate_entries_from_sliding_windows(
            &self.cfg.sliding_window_lengths,
            ledger,
            msg.timestamp.clone(),
            &new_wants,
        );

        match &msg.full_want_list {
            Some(full) => match full {
                true => {
                    let new_wants = new_wants
                        .iter()
                        .map(|c| WantlistEntry {
                            cid: c.cid.path.clone(),
                            ts: msg.timestamp.clone(),
                        })
                        .collect();
                    let old_wants = mem::replace(&mut ledger.wanted_entries, new_wants);
                    ledger
                        .wanted_entries
                        .sort_unstable_by(|e1, e2| e1.cid.cmp(&e2.cid));

                    let (full_wl_dups_t, full_wl_synth_cancels_t) =
                        Self::get_duplicate_entries_and_synth_cancels_from_full_wantlist(
                            old_wants,
                            &ledger.wanted_entries,
                        );
                    full_wl_dups = full_wl_dups_t;
                    full_wl_synth_cancels = full_wl_synth_cancels_t;
                }
                false => {
                    Self::apply_new_entries(
                        &mut ledger.wanted_entries,
                        new_wants,
                        new_cancels,
                        &msg.peer,
                        msg.timestamp.clone(),
                    )?;
                }
            },
            None => {
                if self.cfg.allow_empty_full_wantlist {
                    debug!("got empty full_want_list, assuming incremental.");
                    Self::apply_new_entries(
                        &mut ledger.wanted_entries,
                        new_wants,
                        new_cancels,
                        &msg.peer,
                        msg.timestamp.clone(),
                    )?;
                } else {
                    error!("got empty full_want_list, but is not allowed.");
                    return Err(err_msg("got empty full_want_list, should be set"));
                }
            }
        }

        let reconnect_dups = Self::get_duplicate_entries_from_reconnect(
            ledger,
            msg.timestamp.clone(),
            self.cfg.reconnect_duplicate_duration_secs,
        );

        let synth_cancels = full_wl_synth_cancels.map(|cs| {
            CSVWantlistEntry::from_wantlist_entries(
                cs,
                &msg,
                msg_id,
                CSV_MESSAGE_TYPE_SYNTHETIC,
                CSV_ENTRY_TYPE_SYNTHETIC_CANCEL_FULL_WANTLIST,
                CSV_DUPLICATE_STATUS_NO_DUP,
                0,
            )
        });
        let mut entries = CSVWantlistEntry::from_json_message(msg.clone(), msg_id)
            .context("unable to convert entries to JSON")?;

        // Mark duplicates
        for entry in entries.iter_mut() {
            if let Some(full_wl_dups) = full_wl_dups.as_ref() {
                if full_wl_dups.iter().find(|e| e.cid == entry.cid).is_some() {
                    // This is a dup
                    entry.duplicate_status += CSV_DUPLICATE_STATUS_DUP_FULL_WANTLIST
                }
            }
            if let Some(reconnect_dups) = reconnect_dups.as_ref() {
                if reconnect_dups.iter().find(|e| e.cid == entry.cid).is_some() {
                    // This is a dup too
                    entry.duplicate_status += CSV_DUPLICATE_STATUS_DUP_RECONNECT
                }
            }
            if let Some(sliding_window_dups) = sliding_window_dups.as_ref() {
                if let Some((_, win_size)) =
                    sliding_window_dups.iter().find(|e| e.0.cid == entry.cid)
                {
                    // This is a dup too!
                    entry.duplicate_status += CSV_DUPLICATE_STATUS_DUP_SLIDING_WINDOW;
                    entry.sliding_window_smallest_match = *win_size;
                }
            }
        }

        // Add synthetic cancels
        if let Some(cancels) = synth_cancels {
            entries.extend(cancels.into_iter())
        }

        Ok(IngestResult {
            missing_ledger,
            wantlist_entries: Some(entries),
            connection_event: None,
        })
    }

    fn get_duplicate_entries_from_sliding_windows(
        sliding_window_lengths: &Vec<u32>,
        ledger: &Ledger,
        msg_ts: chrono::DateTime<chrono::Utc>,
        new_entries: &Vec<&JSONWantlistEntry>,
    ) -> Option<Vec<(WantlistEntry, u32)>> {
        if sliding_window_lengths.is_empty() {
            return None;
        }

        let windows: Vec<_> = sliding_window_lengths
            .iter()
            .map(|e| (*e, msg_ts - chrono::Duration::seconds(*e as i64)))
            .collect();

        // cfg.sliding_window_lengths is ordered, so the biggest window is at the end.
        let (_, biggest_window_start) = windows.last().unwrap().clone();

        // Find elements in the ledger that fall in the largest window
        let dups: Vec<_> = ledger
            .wanted_entries
            .iter()
            .filter(|n| {
                n.ts > biggest_window_start
                    && new_entries.iter().find(|e| e.cid.path == n.cid).is_some()
            })
            .cloned()
            .collect();

        if dups.is_empty() {
            return None;
        }

        // Now figure out which window they actually matched
        let dups = dups
            .into_iter()
            .map(|e| {
                // We know it matches one of these because it matched the biggest one.
                for (win_size, win_start) in windows.iter() {
                    if &e.ts > win_start {
                        return (e, *win_size);
                    }
                }
                unreachable!("at least one window should match")
            })
            .collect();

        Some(dups)
    }

    fn get_duplicate_entries_and_synth_cancels_from_full_wantlist(
        old_entries: Vec<WantlistEntry>,
        new_entries: &Vec<WantlistEntry>,
    ) -> (Option<Vec<WantlistEntry>>, Option<Vec<WantlistEntry>>) {
        if old_entries.is_empty() {
            return (None, None);
        }
        let (dups, cancels): (Vec<WantlistEntry>, Vec<WantlistEntry>) = old_entries
            .into_iter()
            .partition(|e| new_entries.iter().find(|n| n.cid == e.cid).is_some());

        if dups.is_empty() {
            // Cancels must be something
            assert!(!cancels.is_empty());
            (None, Some(cancels))
        } else if cancels.is_empty() {
            // Vice versa
            assert!(!dups.is_empty());
            (Some(dups), None)
        } else {
            (Some(dups), Some(cancels))
        }
    }

    fn get_duplicate_entries_from_reconnect(
        ledger: &mut Ledger,
        msg_ts: chrono::DateTime<chrono::Utc>,
        reconnect_duplicate_duration_secs: u32,
    ) -> Option<Vec<WantlistEntry>> {
        let old_wants = ledger.wanted_entries_before_disconnect.take();
        let reconnect_ts = ledger.connected_ts.clone().unwrap();
        match old_wants {
            Some(old_wants) => {
                // Split into duplicates and non-duplicates.
                let (dups, no_dups): (Vec<WantlistEntry>, Vec<WantlistEntry>) =
                    old_wants.into_iter().partition(|e| {
                        ledger
                            .wanted_entries
                            .iter()
                            .find(|n| n.cid == e.cid)
                            .is_some()
                    });

                // If we're still within the time limit, write back non-dups and emit duplicates.
                let limit_ts = reconnect_ts
                    + chrono::Duration::seconds(reconnect_duplicate_duration_secs as i64);
                if limit_ts > msg_ts {
                    // we're within limits.
                    ledger.wanted_entries_before_disconnect = Some(no_dups);
                    Some(dups)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn ingest_connection_event(&mut self, msg: &JSONMessage, msg_id: i64) -> Result<IngestResult> {
        let mut missing_ledger = false;
        match &msg.peer_disconnected {
            Some(disconnected) => {
                let found = msg.connect_event_peer_found.ok_or_else(|| {
                    err_msg(
                        "message had peer_disconnected set but connect_event_peer_found missing",
                    )
                })?;

                if *disconnected {
                    // Disconnected, decrement connection counter.
                    let ledger = self.peers.entry(msg.peer.clone()).or_insert_with(|| {
                        if found {
                            debug!("disconnect event had connect_event_peer_found=true, but we don't have a ledger for peer {}", msg.peer);
                        }
                        // If not present, we return a fresh entry with one connection, because we decrement the counter right away.
                        missing_ledger = true;
                        Ledger { connection_count: 1, wanted_entries: Default::default(), wanted_entries_before_disconnect: Default::default(), connected_ts: Some(msg.timestamp.clone()) }
                    });

                    ledger.connection_count -= 1;
                    assert!(ledger.connection_count >= 0);

                    if ledger.connection_count == 0 {
                        if !ledger.wanted_entries.is_empty() {
                            ledger.wanted_entries_before_disconnect =
                                Some(mem::take(&mut ledger.wanted_entries));

                            return Ok(IngestResult {
                                missing_ledger,
                                wantlist_entries: if self.cfg.insert_disconnect_synth_cancels {
                                    // emit synthetic cancels for the disconnect
                                    Some(CSVWantlistEntry::from_wantlist_entries(
                                        ledger.wanted_entries_before_disconnect.clone().unwrap(),
                                        msg,
                                        msg_id,
                                        CSV_MESSAGE_TYPE_SYNTHETIC,
                                        CSV_ENTRY_TYPE_SYNTHETIC_CANCEL_DISCONNECT,
                                        CSV_DUPLICATE_STATUS_NO_DUP,
                                        0,
                                    ))
                                } else {
                                    None
                                },
                                connection_event: Some(
                                    CSVConnectionEvent::from_json_message(msg.clone(), msg_id)
                                        .unwrap(),
                                ),
                            });
                        }
                    }
                }
            }
            None => {
                if self.cfg.allow_empty_connection_event {
                    warn!(
                        "got empty message and no connection event from message {:?}",
                        msg
                    );
                } else {
                    error!(
                        "got empty message and no connection event from message {:?}",
                        msg
                    );
                    return Err(err_msg("got no wantlist entries and no connection event"));
                }
            }
        }

        match &msg.peer_connected {
            Some(connected) => {
                let found = msg.connect_event_peer_found.ok_or_else(|| {
                    err_msg("message had peer_connected set but connect_event_peer_found missing")
                })?;

                if *connected {
                    // Connected, increment connection counter :)
                    let ledger = self.peers.get_mut(&msg.peer);
                    match ledger {
                        Some(ledger) => {
                            if found && ledger.connection_count == 0 {
                                warn!("connect event had connect_event_peer_found=true, but our ledger has zero connections for peer {}", msg.peer);
                            } else if !found && ledger.connection_count > 0 {
                                warn!("connect event had connect_event_peer_found=false, but we have a ledger with at least one connection for peer {}", msg.peer);
                                // To be consistent with IPFS behaviour, we assume IPFS is right about found==false and didn't report an earlier disconnect.
                                // That means we need to clear out our ledger.
                                let entries = mem::take(&mut ledger.wanted_entries);
                                ledger.wanted_entries_before_disconnect = Some(entries);
                            }
                        }
                        None => {
                            if found {
                                warn!("connect event had connect_event_peer_found=true, but we don't have a ledger for peer {}", msg.peer);
                            }
                        }
                    }

                    let ledger = self
                        .peers
                        .entry(msg.peer.clone())
                        .or_insert_with(|| Ledger {
                            connection_count: 0,
                            wanted_entries: Default::default(),
                            wanted_entries_before_disconnect: Default::default(),
                            connected_ts: Some(msg.timestamp.clone()),
                        });

                    ledger.connection_count += 1;
                }
            }
            None => {
                if self.cfg.allow_empty_connection_event {
                    warn!(
                        "got empty message and no connection event from message {:?}",
                        msg
                    );
                } else {
                    error!(
                        "got empty message and no connection event from message {:?}",
                        msg
                    );
                    return Err(err_msg("got no wantlist entries and no connection event"));
                }
            }
        }
        Ok(IngestResult {
            missing_ledger,
            wantlist_entries: None,
            connection_event: Some(
                CSVConnectionEvent::from_json_message(msg.clone(), msg_id).unwrap(),
            ),
        })
    }

    pub fn ingest(&mut self, msg: &JSONMessage, msg_id: i64) -> Result<IngestResult> {
        match &msg.received_entries {
            Some(entries) => {
                // This is a wantlist message.
                self.ingest_wantlist_message(msg, entries, msg_id)
            }
            None => {
                // This is a connection event.
                self.ingest_connection_event(msg, msg_id)
            }
        }
    }

    fn split_wants_cancels(
        new_entries: &[JSONWantlistEntry],
    ) -> (Vec<&JSONWantlistEntry>, Vec<&JSONWantlistEntry>) {
        let (cancels, wants): (Vec<&JSONWantlistEntry>, Vec<&JSONWantlistEntry>) =
            new_entries.iter().partition(|e| e.cancel);
        (wants, cancels)
    }

    fn apply_new_entries(
        current_entries: &mut Vec<WantlistEntry>,
        wants: Vec<&JSONWantlistEntry>,
        cancels: Vec<&JSONWantlistEntry>,
        peer: &str,
        ts: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        for cancel in cancels {
            if let Ok(i) = current_entries.binary_search_by(|e| e.cid.cmp(&cancel.cid.path)) {
                current_entries.remove(i);
            } else {
                // Not found.
                debug!(
                    "got CANCEL for CID {} from peer {}, but don't have an entry for that",
                    cancel.cid.path, peer
                )
            }
        }

        for want in wants {
            match current_entries.binary_search_by(|e| e.cid.cmp(&want.cid.path)) {
                Ok(i) => {
                    // we already have the entry, we need to update its timestamp.
                    current_entries[i].ts = ts.clone();
                }
                Err(i) => current_entries.insert(
                    i,
                    WantlistEntry {
                        cid: want.cid.path.clone(),
                        ts: ts.clone(),
                    },
                ),
            }
        }

        Ok(())
    }
}
