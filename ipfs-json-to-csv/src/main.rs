#[macro_use]
extern crate log;

mod config;
mod conntrack;

// use crate::conntrack::ConnectionDurationTracker;
use clap::{App, Arg};
use csv::Writer;
use failure::{err_msg, ResultExt};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use ipfs_resolver_common::{logging, wantlist, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, BufWriter};

fn main() -> Result<()> {
    logging::set_up_logging()?;

    let matches = App::new("IPFS wantlist JSON to CSV converter")
        .about("converts JSON files to CSV files and some other stuff")
        .help(
            "This will transform a JSON file into a CSV file.\n\
             Additionally, one line of CSV-formatted data will be written to Stdout,\
             containing timestamps of the first and last entry of the file.",
        )
        .arg(
            Arg::with_name("cfg")
                .long("config")
                .value_name("CONFIG")
                .default_value("config.yaml")
                .help("the config file to load"),
        )
        .get_matches();

    if !matches.is_present("cfg") {
        println!("{}", matches.usage());
        return Err(err_msg("missing config"));
    }
    let config = matches.value_of("cfg").unwrap();

    info!("attempting to load config from file '{}'", config);
    let config = config::Config::open(config).context("unable to load config")?;

    info!(
        "output file for wantlist entries is {}",
        config.wantlist_output_file_pattern
    );
    info!(
        "output file for connection events is {}",
        config.connection_events_output_file
    );
    info!(
        "output file for connection durations is {}",
        config.connection_duration_output_file
    );
    info!(
        "output file for ledger counts is {}",
        config.ledger_count_output_file
    );
    debug!("simulation config is {:?}", config.simulation_config);

    do_transform(config).context("unable to do transformation")?;

    Ok(())
}

struct SingleFileTransformResult {
    timestamps: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
    num_missing_ledgers: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct CSVLedgerCount {
    ts_secs: i64,
    missing_ledgers: usize,
    total_ledgers: usize,
}

fn do_transform_single_file(
    mut infile: BufReader<GzDecoder<File>>,
    wl_writer: &mut csv::Writer<GzEncoder<BufWriter<File>>>,
    engine: &mut wantlist::EngineSimulation,
    current_message_id: &mut i64,
) -> Result<SingleFileTransformResult> {
    let mut buf = String::new();
    let mut first_message_ts = None;
    let mut last_message_ts = None;
    let mut missing_ledgers = 0;
    while let Ok(n) = infile.read_line(&mut buf) {
        if n == 0 {
            break;
        }
        *current_message_id += 1;

        let message: wantlist::JSONMessage = serde_json::from_str(&buf)?;
        debug!("decoded message {:?}", message);
        if first_message_ts.is_none() {
            first_message_ts = Some(message.timestamp);
        }
        last_message_ts.replace(message.timestamp);

        // Update simulated wantlists.
        let ingest_result = engine.ingest(&message, *current_message_id)?;
        debug!("ingest result: {:?}", ingest_result);

        if ingest_result.missing_ledger {
            missing_ledgers += 1;
        }
        if let Some(entries) = ingest_result.wantlist_entries.as_ref() {
            entries
                .iter()
                .try_for_each(|e| wl_writer.serialize(e))
                .context("unable to serialize wantlist entries")?;
        }

        buf.clear();
    }

    Ok(SingleFileTransformResult {
        timestamps: if let Some(first) = first_message_ts {
            Some((first, last_message_ts.unwrap()))
        } else {
            None
        },
        num_missing_ledgers: missing_ledgers,
    })
}

fn do_transform(cfg: config::Config) -> Result<()> {
    let mut engine = wantlist::EngineSimulation::new(cfg.simulation_config.clone())
        .context("unable to set up engine simulation")?;
    let mut current_message_id: i64 = 0;
    let mut final_ts = None;

    let mut ledger_count_output_writer = csv::Writer::from_writer(GzEncoder::new(
        io::BufWriter::new(
            std::fs::File::create(cfg.ledger_count_output_file.clone())
                .context("unable to open missing ledgers output file for writing")?,
        ),
        Compression::default(),
    ));

    let input_files = cfg.glob_results().context("unable to glob")?;
    debug!("paths: {:?}", input_files);

    for path in input_files {
        info!("now working on {}", path.display());
        let input_file = BufReader::new(GzDecoder::new(
            std::fs::File::open(&path).context("unable to open input file for reading")?,
        ));

        let mut wl_output_writer =
            create_wl_output_writer(cfg.wantlist_output_file_pattern.clone(), current_message_id)
                .context("unable to create output file")?;

        let id_before = current_message_id;
        let before = std::time::Instant::now();
        let transform_result = do_transform_single_file(
            input_file,
            &mut wl_output_writer,
            &mut engine,
            &mut current_message_id,
        )
        .context(format!("unable to process file {}", path.display()))?;
        let num_messages = current_message_id - id_before;
        let time_diff = before.elapsed();

        info!(
            "processed {} messages in {:.1}s => {:.1}msg/s",
            num_messages,
            time_diff.as_secs_f32(),
            (num_messages as f64) / time_diff.as_secs_f64()
        );

        match transform_result.timestamps {
            Some((first, last)) => {
                info!("first ts: {}, last ts: {}", first, last);

                ledger_count_output_writer
                    .serialize(CSVLedgerCount {
                        ts_secs: last.timestamp(),
                        missing_ledgers: transform_result.num_missing_ledgers,
                        total_ledgers: engine.num_ledgers(),
                    })
                    .context("unable to serialize missing ledgers record")?;

                final_ts.replace(last);
            }
            None => info!("empty file?"),
        }
        info!(
            "{} missing ledgers, {} ledgers total",
            transform_result.num_missing_ledgers,
            engine.num_ledgers()
        );
    }

    info!("finalizing engine simulation...");
    if let Some(ts) = final_ts {
        let end_of_simulation_cancels =
            engine.generate_end_of_simulation_entries(ts, current_message_id + 1);

        let mut wl_output_writer =
            create_wl_output_writer(cfg.wantlist_output_file_pattern.clone(), current_message_id)
                .context("unable to create output file")?;

        end_of_simulation_cancels
            .iter()
            .try_for_each(|e| wl_output_writer.serialize(e))
            .context("unable to serialize end-of-simulation synthetic cancels")?;
    } else {
        warn!("missing final timestamp, unable to finalize")
    }

    Ok(())
}

fn create_wl_output_writer(
    pattern: String,
    current_message_id: i64,
) -> Result<Writer<GzEncoder<BufWriter<std::fs::File>>>> {
    Ok(csv::Writer::from_writer(GzEncoder::new(
        io::BufWriter::new(
            std::fs::File::create(pattern.replace("$id$", &format!("{:09}", current_message_id)))
                .context("unable to open wantlist output file for writing")?,
        ),
        Compression::default(),
    )))
}
