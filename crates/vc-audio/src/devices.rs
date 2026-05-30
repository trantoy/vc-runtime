use cpal::traits::{DeviceTrait, HostTrait};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceReport {
    pub inputs: Vec<DeviceSummary>,
    pub outputs: Vec<DeviceSummary>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceSummary {
    /// Process-local index for the current device listing.
    pub index: usize,
    pub name: String,
}

pub fn list_devices() -> DeviceReport {
    let host = cpal::default_host();
    let mut warnings = backend_probe_warnings(host.id().name());

    let inputs = match host.input_devices() {
        Ok(devices) => collect_device_summaries(DeviceDirection::Input, devices, &mut warnings),
        Err(error) => {
            warnings.push(format!("failed to list input devices: {error}"));
            Vec::new()
        }
    };

    let outputs = match host.output_devices() {
        Ok(devices) => collect_device_summaries(DeviceDirection::Output, devices, &mut warnings),
        Err(error) => {
            warnings.push(format!("failed to list output devices: {error}"));
            Vec::new()
        }
    };

    DeviceReport {
        inputs,
        outputs,
        warnings,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeviceDirection {
    Input,
    Output,
}

impl DeviceDirection {
    fn label(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
        }
    }
}

fn backend_probe_warnings(host_name: &str) -> Vec<String> {
    if host_name == "ALSA" {
        vec![
            "ALSA may print low-level probe diagnostics to stderr for unavailable built-in devices; CPAL does not expose those diagnostics as structured errors".to_owned(),
        ]
    } else {
        Vec::new()
    }
}

fn collect_device_summaries(
    direction: DeviceDirection,
    devices: impl Iterator<Item = cpal::Device>,
    warnings: &mut Vec<String>,
) -> Vec<DeviceSummary> {
    collect_named_device_summaries(direction, devices.map(|device| device.name()), warnings)
}

fn collect_named_device_summaries<E>(
    direction: DeviceDirection,
    names: impl IntoIterator<Item = Result<String, E>>,
    warnings: &mut Vec<String>,
) -> Vec<DeviceSummary>
where
    E: fmt::Display,
{
    names
        .into_iter()
        .enumerate()
        .map(|(index, name)| match name {
            Ok(name) => DeviceSummary { index, name },
            Err(error) => {
                warnings.push(format!(
                    "failed to read {} device name at index {index}: {error}",
                    direction.label()
                ));
                DeviceSummary {
                    index,
                    name: format!("<unknown {} device>", direction.label()),
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_device_name_failures_as_warnings() {
        let mut warnings = Vec::new();

        let devices = collect_named_device_summaries(
            DeviceDirection::Input,
            [Ok("Mic".to_owned()), Err("name unavailable")],
            &mut warnings,
        );

        assert_eq!(
            devices,
            vec![
                DeviceSummary {
                    index: 0,
                    name: "Mic".to_owned(),
                },
                DeviceSummary {
                    index: 1,
                    name: "<unknown input device>".to_owned(),
                },
            ]
        );
        assert_eq!(
            warnings,
            vec!["failed to read input device name at index 1: name unavailable"]
        );
    }

    #[test]
    fn warns_about_alsa_probe_diagnostics() {
        let warnings = backend_probe_warnings("ALSA");

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("ALSA"));
        assert!(warnings[0].contains("stderr"));
    }
}
