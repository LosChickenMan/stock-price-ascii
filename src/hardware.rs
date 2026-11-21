use std::process::Command;
use sysinfo::{Components, Networks, System};

#[derive(Debug, Clone, Default)]
pub struct HardwareMetrics {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub memory_used_gb: f64,
    pub memory_total_gb: f64,
    pub network_rx_kbps: f64,
    pub network_tx_kbps: f64,
    pub cpu_temperature_c: Option<f64>,
    pub gpu_percent: Option<f64>,
    pub gpu_temperature_c: Option<f64>,
    pub hostname: String,
    pub uptime_seconds: u64,
}

pub struct HardwareCollector {
    system: System,
    networks: Networks,
    components: Components,
    previous_rx: u64,
    previous_tx: u64,
}

impl HardwareCollector {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh();
        let components = Components::new_with_refreshed_list();

        let (rx, tx) = network_totals(&networks);

        Self {
            system,
            networks,
            components,
            previous_rx: rx,
            previous_tx: tx,
        }
    }

    pub fn sample(&mut self, elapsed_seconds: f64) -> HardwareMetrics {
        self.system.refresh_cpu();
        self.system.refresh_memory();
        self.networks.refresh();
        for component in self.components.iter_mut() {
            component.refresh();
        }

        let (rx, tx) = network_totals(&self.networks);
        let seconds = elapsed_seconds.max(0.1);
        let rx_rate = rx.saturating_sub(self.previous_rx) as f64 / seconds / 1024.0;
        let tx_rate = tx.saturating_sub(self.previous_tx) as f64 / seconds / 1024.0;
        self.previous_rx = rx;
        self.previous_tx = tx;

        let total_memory = self.system.total_memory() as f64;
        let used_memory = self.system.used_memory() as f64;
        let memory_percent = if total_memory > 0.0 {
            used_memory / total_memory * 100.0
        } else {
            0.0
        };

        let (cpu_temperature_c, sensor_gpu_temperature) =
            read_component_temperatures(&self.components);
        let (gpu_percent, nvidia_gpu_temperature) = read_nvidia_gpu();
        let gpu_temperature_c = nvidia_gpu_temperature.or(sensor_gpu_temperature);

        HardwareMetrics {
            cpu_percent: self.system.global_cpu_info().cpu_usage() as f64,
            memory_percent,
            memory_used_gb: used_memory / 1_073_741_824.0,
            memory_total_gb: total_memory / 1_073_741_824.0,
            network_rx_kbps: rx_rate,
            network_tx_kbps: tx_rate,
            cpu_temperature_c,
            gpu_percent,
            gpu_temperature_c,
            hostname: System::host_name().unwrap_or_else(|| "unknown-host".to_string()),
            uptime_seconds: System::uptime(),
        }
    }
}

fn read_component_temperatures(components: &Components) -> (Option<f64>, Option<f64>) {
    let mut cpu_values = Vec::new();
    let mut gpu_values = Vec::new();

    for component in components {
        let label = component.label().to_ascii_lowercase();
        let temp = component.temperature() as f64;
        if !temp.is_finite() || !(0.0..=150.0).contains(&temp) {
            continue;
        }

        if label.contains("gpu")
            || label.contains("nvidia")
            || label.contains("radeon")
            || label.contains("amdgpu")
        {
            gpu_values.push(temp);
        } else if label.contains("cpu")
            || label.contains("package")
            || label.contains("core")
            || label.contains("tctl")
            || label.contains("tdie")
            || label.contains("k10temp")
            || label.contains("zenpower")
        {
            cpu_values.push(temp);
        }
    }

    (average(cpu_values), average(gpu_values))
}

fn average(values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

// NVIDIA support is optional: on machines without nvidia-smi, the values stay N/A.
// The first detected NVIDIA GPU is displayed.
fn read_nvidia_gpu() -> (Option<f64>, Option<f64>) {
    let output = match Command::new("nvidia-smi")
        .args([
            "--query-gpu=utilization.gpu,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return (None, None),
    };

    let line = String::from_utf8_lossy(&output.stdout);
    let mut fields = line.lines().next().unwrap_or("").split(',').map(str::trim);

    let usage = fields.next().and_then(|value| value.parse::<f64>().ok());
    let temperature = fields.next().and_then(|value| value.parse::<f64>().ok());

    (usage, temperature)
}

fn network_totals(networks: &Networks) -> (u64, u64) {
    networks.iter().fold((0, 0), |(rx, tx), (_, data)| {
        (rx + data.total_received(), tx + data.total_transmitted())
    })
}
