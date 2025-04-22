use crate::config::{save_config, Config};
use crate::utils::serde_base64;
use crate::{system, utils};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{BasicHistory, Input, Select};
use log::{error, info, warn};
use pcap::Device;
use pcap_file::pcap::PcapReader;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, MutexGuard};
use std::time::Instant;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;
use ys_sniffer::{Config as SnifferConfig, GamePacket, PacketSource};

/// A struct wrapper that allows the device to be displayed.
struct CaptureDevice(Device);

impl CaptureDevice {
    /// Converts a list of devices into a list of capture devices.
    pub fn into(devices: &[Device]) -> Vec<CaptureDevice> {
        devices.iter().map(|d| CaptureDevice(d.clone())).collect()
    }
}

impl Display for CaptureDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let description = match self.0.desc {
            Some(ref desc) => desc,
            None => "No description",
        };

        write!(f, "{} - ({})", description, self.0.name)
    }
}

/// Fetches the device from the configuration.
///
/// If it's empty, it will prompt the user to select a device.
fn get_device(config: &mut MutexGuard<Config>) -> String {
    // Check if the device name exists in the config.
    let device_name = &config.sniffer.device_name;
    if !device_name.is_empty() {
        return device_name.clone();
    }

    // Otherwise, prompt the user to select a device.
    let Ok(device_list) = Device::list() else {
        error!("Failed to fetch device list.");
        std::process::exit(1);
    };

    // Print the device list.
    let device_names = CaptureDevice::into(&device_list);
    let device = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a network device to capture from")
        .default(0)
        .items(&device_names)
        .interact();

    // Read the selected device.
    let Ok(index) = device else {
        error!("No device selected.");
        std::process::exit(1)
    };
    let device = &device_list[index];

    // Write the name to the configuration.
    config.sniffer.device_name = device.name.clone();
    if save_config(config).is_err() {
        warn!("Failed to save the configuration; continuing ephemeral.");
    }

    device.name.clone()
}

/// Holds more data about a `GamePacket`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Packet {
    pub id: u16,

    #[serde(with = "serde_base64")]
    pub header: Vec<u8>,

    #[serde(with = "serde_base64")]
    pub data: Vec<u8>,

    /// The source of the packet.
    ///
    /// Can be either `client` or `server`.
    pub source: PacketSource,

    /// The offset from when the packet was received and when sniffing began.
    ///
    /// This number is in milliseconds, and is an unsigned long.
    received: u128,
}

impl Packet {
    /// Creates a new packet from the given data.
    pub fn new(data: GamePacket, received: u128) -> Self {
        Self {
            id: data.id,
            header: data.header,
            data: data.data,
            source: data.source,
            received,
        }
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}ms] [{} -> {}] {} of length {}",
            self.received,
            self.source,
            utils::opposite(self.source),
            self.id,
            self.data.len()
        )
    }
}

/// Runs the sniffer for the CLI application.
pub async fn run_cli() {
    let (mut rx, shutdown_hook) = match run_sniffer().await {
        Ok((shutdown_hook, rx)) => (shutdown_hook, rx),
        Err(error) => {
            error!("Failed to run the sniffer: {:#?}", error);
            std::process::exit(1);
        }
    };

    // Create mutex for storing packets.
    let mut start_time: Option<Instant> = None;

    let log_enabled = Arc::new(AtomicBool::new(false));
    let packets = Arc::new(Mutex::new(Vec::new()));

    // Spawn a thread to read the packets.
    let do_log = log_enabled.clone();
    let packet_list = packets.clone();

    tokio::spawn(async move {
        while let Some(packet) = rx.recv().await {
            if start_time.is_none() {
                start_time = Some(Instant::now());
            }

            // Lock the list to push the packet.
            let mut list = packet_list.lock().await;

            // Create a new packet with the current time.
            let current_time = Instant::now();
            let packet = Packet::new(
                packet,
                current_time.duration_since(start_time.unwrap()).as_millis(),
            );

            // Write the packet to the console.
            if do_log.load(Ordering::Relaxed) {
                info!("{}", packet);
            }

            // Push the packet to the list.
            list.push(packet);

            // Drop the list to be used again later.
            drop(list);
        }

        info!("Sniffer has finished sniffing.");
    });

    // Prepare for user input.
    let mut history = BasicHistory::new().max_entries(8).no_duplicates(true);

    loop {
        // Read the console for user commands.
        let command = match Input::<String>::with_theme(&ColorfulTheme::default())
            .history_with(&mut history)
            .interact_text()
        {
            Ok(command) => command,
            Err(error) => {
                error!("Failed to read command: {:#?}", error);
                continue;
            }
        };

        // Parse the command and execute it.
        match command.as_str() {
            "stop" => break,
            "log" => {
                // Toggle the value of the log.
                let enabled = log_enabled.load(Ordering::Relaxed);
                log_enabled.store(!enabled, Ordering::Relaxed);

                info!(
                    "Logging is now {}",
                    if !enabled { "enabled" } else { "disabled" }
                );
            }
            "help" => {
                info!("Commands:");
                info!("  stop - Stops the sniffer.");
                info!("  log  - Toggles logging of packets.");
                info!("  help - Shows this help message.");
            }
            _ => info!("Unknown command: '{command}'"),
        }
    }

    // Dump the packets to the file system.
    let encoded = serde_json::to_string_pretty(&*packets.lock().await).unwrap();

    let Ok(app_data_dir) = utils::app_data_dir() else {
        error!("Failed to fetch the application data directory.");
        std::process::exit(1);
    };

    let path = app_data_dir
        .join("dumps")
        .join(format!("dump-{}.json", utils::unix_timestamp()));
    if let Err(error) = utils::write_file(&path, encoded) {
        error!("Failed to write the packet dump: {:#?}", error);
    }

    // If we hit here, we should stop the sniffer.
    shutdown_hook.send(()).unwrap();

    info!("Sniffer has been shut down.");
}

/// This is the result that `run_sniffer` returns.
///
/// It returns two things:
///
/// 1. The packet receiver.
/// 2. The sniffer's shutdown hook.
type SnifferRunResult = (UnboundedReceiver<GamePacket>, crossbeam_channel::Sender<()>);

/// Runs the actual sniffer.
///
/// Pulls the configuration for the sniffer from the global config.
pub async fn run_sniffer() -> Result<SnifferRunResult, &'static str> {
    let mut config = Config::get();

    // Resolve the seeds file.
    let seeds_file = match system::resolve_path(&config.sniffer.seeds_file) {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(_) => "known-seeds.txt".to_string(),
    };

    // Prepare the sniffer configuration.
    let sniffer_config = SnifferConfig {
        device_name: Some(get_device(&mut config)),
        known_seeds: seeds_file,
        filter: Some(config.sniffer.filter.clone()),
        server_port: config.sniffer.server_ports.clone(),
    };

    // Drop the lock so we don't carry it across await points.
    drop(config);

    // Create the sending/receiving channel.
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<GamePacket>();

    // Run the sniffer.
    let shutdown_hook = ys_sniffer::sniff_async(sniffer_config, tx)
        .map_err(|_| "Failed to run the sniffer.")?;

    Ok((rx, shutdown_hook))
}

/// A packet that is displayed on the frontend.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualPacket {
    /// The timestamp of the packet.
    ///
    /// This is offset from the connection was created.
    time: f32,

    /// The source of the packet.
    #[serde(with = "src_string")]
    source: PacketSource,

    /// The packet ID.
    packet_id: u16,

    /// The display name of the packet.
    ///
    /// If unknown, this should be an obfuscated name.
    ///
    /// If no obfuscated name is shown, the packet's ID in numerical form should be shown.
    packet_name: String,

    /// The length of the packet's data.
    length: u64,

    /// The packet's decoded data to be shown to the user.
    data: String,

    /// The raw binary packet data.
    ///
    /// This will be Base64-encoded
    #[serde(with = "serde_base64")]
    binary: Vec<u8>,

    /// The index of the packet.
    ///
    /// This represents the array index.
    index: u32,
}

impl VisualPacket {
    /// Converts a `GamePacket` into a `VisualPacket`.
    pub fn into_game(
        packet: &GamePacket,
        start_time: Instant
    ) -> Self {
        // Decode the packet's data.
        let decoded = match protoshark::decode(&packet.data) {
            Ok(decoded) => serde_json::to_string(&decoded).unwrap(),
            Err(_) => Default::default()
        };

        Self {
            time: Instant::now().duration_since(start_time).as_secs_f32(),
            source: packet.source,
            packet_id: packet.id,
            packet_name: packet.id.to_string(),
            length: packet.data.len() as u64,
            data: decoded,
            binary: packet.data.clone(),
            index: 0
        }
    }
}

/// Reads and parses the selected file for packets.
///
/// If the file is in a JSON file, it will try to be parsed as a `Packet` or `VisualPacket`.
///
/// If the file is in `pcap` format, it will try to be parsed as a `Packet`.
#[tauri::command]
pub fn sniffer__load(file_path: String) -> Result<Vec<VisualPacket>, &'static str> {
    // Read the file.
    let file_path = PathBuf::from(file_path);
    let Ok(file) = File::open(&file_path) else {
        return Err("Failed to open the file.");
    };

    // Check if the data is a packet capture.
    let data = match utils::read_file(&file_path) {
        Ok(data) => data,
        Err(_) => return Err("Failed to read the file."),
    };
    if let Ok(reader) = PcapReader::new(&file) {
        return read_pcap(reader);
    }

    // Otherwise, try treating the data as plain-text JSON.
    let json_data = match serde_json::from_slice::<Vec<Value>>(&data) {
        Ok(data) => data,
        Err(_) => return Err("Invalid JSON data provided"),
    };

    // If the data is empty, return nothing now.
    if json_data.is_empty() {
        return Ok(vec![]);
    }

    // Check if the first element contains a 'binary' field.
    let first = &json_data[0];
    match first.get("binary") {
        Some(Value::String(_)) => Ok(json_data
            .iter()
            .map(|value| serde_json::from_value::<VisualPacket>(value.clone()).unwrap())
            .collect::<Vec<VisualPacket>>()),
        None | Some(Value::Null) => read_json(
            json_data
                .iter()
                .map(|value| serde_json::from_value::<Packet>(value.clone()).unwrap())
                .collect::<Vec<Packet>>(),
        ),
        _ => Err("Invalid JSON data provided"),
    }
}

/// Reads the packets from a pcap file.
fn read_pcap<R: Read>(_: PcapReader<R>) -> Result<Vec<VisualPacket>, &'static str> {
    Err("Not implemented")
}

/// Reads the JSON data as a list of packets.
///
/// This method exists to run `protoshark` on the data.
fn read_json(data: Vec<Packet>) -> Result<Vec<VisualPacket>, &'static str> {
    let mut packets = Vec::new();

    // Get the first packet.
    // This will serve as the base time for the packets.
    let Some(first) = data.first() else {
        // This means no packets exist.
        // We can just return an empty array.
        return Ok(packets);
    };
    let base_time = first.received;

    for packet in data {
        // Run `protoshark` to decode the packet.
        let Ok(decoded) = protoshark::decode(&packet.data) else {
            warn!("Failed to decode packet: {}", packet.id);
            continue;
        };

        packets.push(VisualPacket {
            time: (packet.received - base_time) as f32,
            source: packet.source,
            packet_id: packet.id,
            packet_name: packet.id.to_string(),
            length: packet.data.len() as u64,
            data: serde_json::to_string(&decoded).unwrap(),
            binary: packet.data.clone(),
            index: packets.len() as u32,
        });
    }

    Ok(packets)
}

mod src_string {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use ys_sniffer::PacketSource;

    pub fn serialize<S: Serializer>(source: &PacketSource, s: S) -> Result<S::Ok, S::Error> {
        String::serialize(&source.to_string().to_lowercase(), s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<PacketSource, D::Error> {
        let string = String::deserialize(d)?;
        match string.as_str() {
            "client" => Ok(PacketSource::Client),
            "server" => Ok(PacketSource::Server),
            _ => Err(serde::de::Error::custom("invalid packet source")),
        }
    }
}
