use crate::utils::serde_base64;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, MutexGuard};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use dialoguer::{BasicHistory, Input, Select};
use dialoguer::theme::ColorfulTheme;
use log::{error, info, warn};
use pcap::Device;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use ys_sniffer::{Config as SnifferConfig, GamePacket, PacketSource};
use crate::config::{save_config, Config};
use crate::{system, utils};

/// A struct wrapper that allows the device to be displayed.
struct CaptureDevice(Device);

impl CaptureDevice {
    /// Converts a list of devices into a list of capture devices.
    pub fn into(devices: &Vec<Device>) -> Vec<CaptureDevice> {
        devices
            .into_iter()
            .map(|d| CaptureDevice(d.clone()))
            .collect()
    }
}

impl Display for CaptureDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let description = match self.0.desc {
            Some(ref desc) => desc,
            None => "No description"
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
    received: u128
}

impl Packet {
    /// Creates a new packet from the given data.
    pub fn new(data: GamePacket, received: u128) -> Self {
        Self {
            id: data.id,
            header: data.header,
            data: data.data,
            source: data.source,
            received
        }
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
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
    let mut config = Config::get();

    // Resolve the seeds file.
    let seeds_file = match system::resolve_path(&config.sniffer.seeds_file) {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(_) => "known-seeds.txt".to_string()
    };

    // Prepare the sniffer configuration.
    let sniffer_config = SnifferConfig {
        device_name: Some(get_device(&mut config)),
        known_seeds: seeds_file,
        filter: Some(config.sniffer.filter.clone()),
        server_port: config.sniffer.server_ports.clone(),
    };

    // Create the sending/receiving channel.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<GamePacket>();

    // Run the sniffer.
    let Ok(shutdown_hook) = ys_sniffer::sniff_async(sniffer_config, tx) else {
        error!("Failed to run the sniffer.");
        std::process::exit(1);
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
                current_time.duration_since(start_time.unwrap()).as_millis()
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
    let mut history = BasicHistory::new()
        .max_entries(8)
        .no_duplicates(true);

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

                info!("Logging is now {}", if !enabled { "enabled" } else { "disabled" });
            }
            "help" => {
                info!("Commands:");
                info!("  stop - Stops the sniffer.");
                info!("  log  - Toggles logging of packets.");
                info!("  help - Shows this help message.");
            }
            _ => info!("Unknown command: '{command}'")
        }
    }

    // Dump the packets to the file system.
    let encoded = serde_json::to_string_pretty(
        &*packets.lock().await
    ).unwrap();

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
