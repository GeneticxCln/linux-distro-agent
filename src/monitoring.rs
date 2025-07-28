use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: u64,
    pub cpu_usage: f64,
    pub memory_usage: MemoryInfo,
    pub disk_usage: Vec<DiskInfo>,
    pub network_stats: NetworkStats,
    pub load_average: LoadAverage,
    pub uptime: Duration,
    pub processes: ProcessStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total: u64,      // bytes
    pub available: u64,  // bytes
    pub used: u64,       // bytes
    pub cached: u64,     // bytes
    pub buffers: u64,    // bytes
    pub swap_total: u64, // bytes
    pub swap_used: u64,  // bytes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub device: String,
    pub mount_point: String,
    pub filesystem: String,
    pub total: u64,     // bytes
    pub used: u64,      // bytes
    pub available: u64, // bytes
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub interfaces: HashMap<String, NetworkInterface>,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub errors: u64,
    pub drops: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadAverage {
    pub one_min: f64,
    pub five_min: f64,
    pub fifteen_min: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStats {
    pub total: u32,
    pub running: u32,
    pub sleeping: u32,
    pub zombie: u32,
    pub stopped: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub last_check: u64,
    pub details: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

pub struct SystemMonitor {
    metrics_history: Vec<SystemMetrics>,
    max_history_size: usize,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            metrics_history: Vec::new(),
            max_history_size: 100, // Keep last 100 metrics snapshots
        }
    }

    pub fn collect_metrics(&mut self) -> Result<SystemMetrics> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let metrics = SystemMetrics {
            timestamp,
            cpu_usage: self.get_cpu_usage()?,
            memory_usage: self.get_memory_info()?,
            disk_usage: self.get_disk_usage()?,
            network_stats: self.get_network_stats()?,
            load_average: self.get_load_average()?,
            uptime: self.get_uptime()?,
            processes: self.get_process_stats()?,
        };

        // Store in history
        self.metrics_history.push(metrics.clone());
        if self.metrics_history.len() > self.max_history_size {
            self.metrics_history.remove(0);
        }

        Ok(metrics)
    }

    fn get_cpu_usage(&self) -> Result<f64> {
        // Read /proc/stat to get CPU usage
        let stat = fs::read_to_string("/proc/stat")?;
        if let Some(cpu_line) = stat.lines().next() {
            let values: Vec<u64> = cpu_line
                .split_whitespace()
                .skip(1)
                .take(7)
                .map(|s| s.parse().unwrap_or(0))
                .collect();

            if values.len() >= 4 {
                let idle = values[3];
                let total: u64 = values.iter().sum();
                let usage = if total > 0 {
                    100.0 - (idle as f64 / total as f64 * 100.0)
                } else {
                    0.0
                };
                return Ok(usage);
            }
        }
        Ok(0.0)
    }

    fn get_memory_info(&self) -> Result<MemoryInfo> {
        let meminfo = fs::read_to_string("/proc/meminfo")?;
        let mut values = HashMap::new();

        for line in meminfo.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let value = value.split_whitespace().next().unwrap_or("0");
                if let Ok(val) = value.parse::<u64>() {
                    values.insert(key.trim(), val * 1024); // Convert from kB to bytes
                }
            }
        }

        Ok(MemoryInfo {
            total: values.get("MemTotal").copied().unwrap_or(0),
            available: values.get("MemAvailable").copied().unwrap_or(0),
            used: values.get("MemTotal").copied().unwrap_or(0) - values.get("MemAvailable").copied().unwrap_or(0),
            cached: values.get("Cached").copied().unwrap_or(0),
            buffers: values.get("Buffers").copied().unwrap_or(0),
            swap_total: values.get("SwapTotal").copied().unwrap_or(0),
            swap_used: values.get("SwapTotal").copied().unwrap_or(0) - values.get("SwapFree").copied().unwrap_or(0),
        })
    }

    fn get_disk_usage(&self) -> Result<Vec<DiskInfo>> {
        let output = Command::new("df")
            .args(["-B1", "--output=source,target,fstype,size,used,avail,pcent"])
            .output()?;

        let mut disks = Vec::new();
        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 7 {
                let usage_percent = fields[6].trim_end_matches('%').parse().unwrap_or(0.0);
                disks.push(DiskInfo {
                    device: fields[0].to_string(),
                    mount_point: fields[1].to_string(),
                    filesystem: fields[2].to_string(),
                    total: fields[3].parse().unwrap_or(0),
                    used: fields[4].parse().unwrap_or(0),
                    available: fields[5].parse().unwrap_or(0),
                    usage_percent,
                });
            }
        }

        Ok(disks)
    }

    fn get_network_stats(&self) -> Result<NetworkStats> {
        let net_dev = fs::read_to_string("/proc/net/dev")?;
        let mut interfaces = HashMap::new();
        let mut total_rx = 0;
        let mut total_tx = 0;

        for line in net_dev.lines().skip(2) {
            if let Some((interface, stats)) = line.split_once(':') {
                let interface = interface.trim();
                let stats: Vec<u64> = stats
                    .split_whitespace()
                    .take(16)
                    .map(|s| s.parse().unwrap_or(0))
                    .collect();

                if stats.len() >= 16 {
                    let rx_bytes = stats[0];
                    let tx_bytes = stats[8];
                    
                    total_rx += rx_bytes;
                    total_tx += tx_bytes;

                    interfaces.insert(interface.to_string(), NetworkInterface {
                        rx_bytes,
                        tx_bytes,
                        rx_packets: stats[1],
                        tx_packets: stats[9],
                        errors: stats[2] + stats[10],
                        drops: stats[3] + stats[11],
                    });
                }
            }
        }

        Ok(NetworkStats {
            interfaces,
            total_rx_bytes: total_rx,
            total_tx_bytes: total_tx,
        })
    }

    fn get_load_average(&self) -> Result<LoadAverage> {
        let loadavg = fs::read_to_string("/proc/loadavg")?;
        let values: Vec<&str> = loadavg.split_whitespace().collect();

        Ok(LoadAverage {
            one_min: values.first().unwrap_or(&"0").parse().unwrap_or(0.0),
            five_min: values.get(1).unwrap_or(&"0").parse().unwrap_or(0.0),
            fifteen_min: values.get(2).unwrap_or(&"0").parse().unwrap_or(0.0),
        })
    }

    fn get_uptime(&self) -> Result<Duration> {
        let uptime = fs::read_to_string("/proc/uptime")?;
        let uptime_seconds: f64 = uptime
            .split_whitespace()
            .next()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0.0);

        Ok(Duration::from_secs_f64(uptime_seconds))
    }

    fn get_process_stats(&self) -> Result<ProcessStats> {
        let stat = fs::read_to_string("/proc/stat")?;
        let mut total = 0;
        let mut running = 0;
        let mut sleeping = 0;
        let mut zombie = 0;
        let mut stopped = 0;

        for line in stat.lines() {
            if line.starts_with("processes") {
                total = line.split_whitespace()
                    .nth(1)
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
            }
        }

        // Get process states from /proc/*/stat
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() {
                if let Ok(stat_content) = fs::read_to_string(format!("/proc/{pid}/stat")) {
                        if let Some(state) = stat_content.split_whitespace().nth(2) {
                            match state {
                                "R" => running += 1,
                                "S" | "D" => sleeping += 1,
                                "Z" => zombie += 1,
                                "T" => stopped += 1,
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        Ok(ProcessStats {
            total,
            running,
            sleeping,
            zombie,
            stopped,
        })
    }

    pub fn run_health_checks(&self) -> Vec<HealthCheck> {
        let mut checks = Vec::new();

        // Check disk space
        if let Ok(disks) = self.get_disk_usage() {
            for disk in disks {
                let status = if disk.usage_percent > 90.0 {
                    HealthStatus::Critical
                } else if disk.usage_percent > 80.0 {
                    HealthStatus::Warning
                } else {
                    HealthStatus::Healthy
                };

                checks.push(HealthCheck {
                    name: format!("disk_usage_{}", disk.mount_point.replace('/', "_")),
                    status,
                    message: format!("Disk usage: {:.1}%", disk.usage_percent),
                    last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    details: Some(HashMap::from([
                        ("mount_point".to_string(), disk.mount_point.clone()),
                        ("device".to_string(), disk.device.clone()),
                        ("usage_percent".to_string(), disk.usage_percent.to_string()),
                    ])),
                });
            }
        }

        // Check memory usage
        if let Ok(memory) = self.get_memory_info() {
            let usage_percent = if memory.total > 0 {
                (memory.used as f64 / memory.total as f64) * 100.0
            } else {
                0.0
            };

            let status = if usage_percent > 90.0 {
                HealthStatus::Critical
            } else if usage_percent > 80.0 {
                HealthStatus::Warning
            } else {
                HealthStatus::Healthy
            };

            checks.push(HealthCheck {
                name: "memory_usage".to_string(),
                status,
                message: format!("Memory usage: {usage_percent:.1}%"),
                last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                details: Some(HashMap::from([
                    ("usage_percent".to_string(), usage_percent.to_string()),
                    ("total_gb".to_string(), (memory.total / 1024 / 1024 / 1024).to_string()),
                    ("used_gb".to_string(), (memory.used / 1024 / 1024 / 1024).to_string()),
                ])),
            });
        }

        // Check load average
        if let Ok(load) = self.get_load_average() {
            let cpu_count = num_cpus::get() as f64;
            let load_percent = (load.one_min / cpu_count) * 100.0;

            let status = if load_percent > 100.0 {
                HealthStatus::Critical
            } else if load_percent > 80.0 {
                HealthStatus::Warning
            } else {
                HealthStatus::Healthy
            };

            checks.push(HealthCheck {
                name: "load_average".to_string(),
                status,
                message: format!("Load average: {:.2} (1m)", load.one_min),
                last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                details: Some(HashMap::from([
                    ("load_1m".to_string(), load.one_min.to_string()),
                    ("load_5m".to_string(), load.five_min.to_string()),
                    ("load_15m".to_string(), load.fifteen_min.to_string()),
                    ("cpu_count".to_string(), cpu_count.to_string()),
                ])),
            });
        }

        checks
    }

    pub fn get_history(&self) -> &Vec<SystemMetrics> {
        &self.metrics_history
    }

    pub fn get_latest_metrics(&self) -> Option<&SystemMetrics> {
        self.metrics_history.last()
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}
