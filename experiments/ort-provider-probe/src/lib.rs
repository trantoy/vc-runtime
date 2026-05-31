//! ORT provider probe prototype.
//!
//! This module intentionally avoids requiring ONNX Runtime runtime libs at the
//! default execution path. It models a probe workflow so we can validate output
//! shape and assignment granularity rules before full runtime integration.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};
use std::{fs, io};

#[derive(Debug)]
pub struct ProbeError {
    pub message: String,
}

impl fmt::Display for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ProbeError {}

impl From<io::Error> for ProbeError {
    fn from(err: io::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ProbeRequest {
    /// Provider name requested by user/application config.
    pub requested_provider: String,
    /// Optional fixture with host/provider availability simulation.
    pub fixture: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct FixtureProbeState {
    #[serde(default)]
    pub available_providers: Vec<String>,
    #[serde(default)]
    pub fallback_chain: Vec<String>,
}

impl Default for FixtureProbeState {
    fn default() -> Self {
        Self {
            available_providers: Vec::new(),
            fallback_chain: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ProviderAssignment {
    /// Scope at which this assignment is observed.
    pub scope: AssignmentScope,
    /// Requested provider key from config/runtime input.
    pub requested_provider: String,
    /// Provider that can be used by execution at this scope.
    pub observed_provider: Option<String>,
    /// How confidence/observability was established.
    pub observed_with: ObservedWith,
    /// Evidence string suitable for diagnostics.
    pub evidence: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentScope {
    /// Request-level only; no session-level evidence yet.
    RequestedOnly,
    /// Assignment was observed through session builder/fallback model.
    Session,
    /// Op-level evidence (not available in this prototype).
    Op,
    /// Node-level evidence (not available in this prototype).
    Node,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObservedWith {
    /// No provider probe executed.
    DryRun,
    /// Provider info sourced from fixture simulation.
    Fixture,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderProbeStatus {
    /// Probe was not executed; JSON schema can still be produced.
    NotProbedDryRun,
    /// Requested provider is in available provider list.
    RequestedProviderAvailable,
    /// Requested provider unavailable; fallback to another provider observed.
    RequestedProviderUnavailableWithFallback,
    /// Requested provider unavailable; no fallback available in tested chain.
    RequestedProviderUnavailableNoFallback,
    /// Fixture parse error.
    FixtureInvalid,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAssignmentGranularity {
    /// We only validated intent, no real ORT session evidence.
    DryRun,
    /// Assignment is inferred at session-configuration granularity.
    Session,
    /// Assignment not probed.
    NotProbed,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProbeReport {
    pub requested_provider: String,
    pub provider_probe_status: ProviderProbeStatus,
    pub observed_provider_assignment: Vec<ProviderAssignment>,
    pub provider_assignment_granularity: ProviderAssignmentGranularity,
    pub errors: Vec<String>,
}

impl ProbeReport {
    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

fn canonical_provider(name: &str) -> String {
    let compacted = name
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect::<String>();

    match compacted.as_str() {
        "cpu" | "cpuep" | "cpuexecutionprovider" => "CPUExecutionProvider".to_string(),
        "cuda" | "cudaprovider" | "cudagpuexecutionprovider" | "cudagpuep" => {
            "CUDAExecutionProvider".to_string()
        }
        "coreml" | "coremlep" | "coremlexecutionprovider" => "CoreMLExecutionProvider".to_string(),
        "core" => "CoreMLExecutionProvider".to_string(),
        "openvino" | "openvinoexecutionprovider" => "OpenVINOExecutionProvider".to_string(),
        "tensorrt" | "tensorrtexecutionprovider" => "TensorRTExecutionProvider".to_string(),
        "dml" | "directml" | "directmlexecutionprovider" => "DmlExecutionProvider".to_string(),
        "rocm" | "rocmexecutionprovider" => "RocmExecutionProvider".to_string(),
        _ => name.trim().to_string(),
    }
}

fn normalized_set(values: &[String]) -> HashSet<String> {
    values
        .iter()
        .map(|value| canonical_provider(value))
        .collect()
}

fn build_dry_run_report(requested_provider: &str) -> ProbeReport {
    ProbeReport {
        requested_provider: canonical_provider(requested_provider),
        provider_probe_status: ProviderProbeStatus::NotProbedDryRun,
        observed_provider_assignment: vec![ProviderAssignment {
            scope: AssignmentScope::RequestedOnly,
            requested_provider: canonical_provider(requested_provider),
            observed_provider: None,
            observed_with: ObservedWith::DryRun,
            evidence: "ORT probe intentionally not executed in this minimal mode".to_string(),
        }],
        provider_assignment_granularity: ProviderAssignmentGranularity::DryRun,
        errors: vec![
            "No CUDA/ORT runtime probe executed in this prototype".to_string(),
            "Use --fixture to run deterministic host simulation".to_string(),
        ],
    }
}

fn build_fixture_report(request: &str, fixture: &FixtureProbeState) -> ProbeReport {
    let requested = canonical_provider(request);
    let available = normalized_set(&fixture.available_providers);

    if available.is_empty() {
        return ProbeReport {
            requested_provider: requested.clone(),
            provider_probe_status: ProviderProbeStatus::RequestedProviderUnavailableNoFallback,
            observed_provider_assignment: vec![ProviderAssignment {
                scope: AssignmentScope::Session,
                requested_provider: requested,
                observed_provider: None,
                observed_with: ObservedWith::Fixture,
                evidence: "Fixture has no available_providers".to_string(),
            }],
            provider_assignment_granularity: ProviderAssignmentGranularity::Session,
            errors: vec!["Fixture simulates zero available providers".to_string()],
        };
    }

    if available.contains(&requested) {
        return ProbeReport {
            requested_provider: requested.clone(),
            provider_probe_status: ProviderProbeStatus::RequestedProviderAvailable,
            observed_provider_assignment: vec![ProviderAssignment {
                scope: AssignmentScope::Session,
                requested_provider: requested.clone(),
                observed_provider: Some(requested.clone()),
                observed_with: ObservedWith::Fixture,
                evidence: "requested provider present in fixture availability list".to_string(),
            }],
            provider_assignment_granularity: ProviderAssignmentGranularity::Session,
            errors: Vec::new(),
        };
    }

    let fallback_chain = fixture
        .fallback_chain
        .iter()
        .map(String::as_str)
        .filter(|candidate| available.contains(&canonical_provider(candidate)))
        .next();

    match fallback_chain {
        Some(fallback) => {
            let fallback_provider = canonical_provider(fallback);
            ProbeReport {
                requested_provider: requested.clone(),
                provider_probe_status: ProviderProbeStatus::RequestedProviderUnavailableWithFallback,
                observed_provider_assignment: vec![ProviderAssignment {
                    scope: AssignmentScope::Session,
                    requested_provider: requested.clone(),
                    observed_provider: Some(fallback_provider),
                    observed_with: ObservedWith::Fixture,
                    evidence: "requested provider missing in fixture; selected first matching provider from fallback_chain".to_string(),
                }],
                provider_assignment_granularity: ProviderAssignmentGranularity::Session,
                errors: vec![format!(
                    "requested provider {:?} was not available in fixture",
                    request
                )],
            }
        }
        None => ProbeReport {
            requested_provider: requested.clone(),
            provider_probe_status: ProviderProbeStatus::RequestedProviderUnavailableNoFallback,
            observed_provider_assignment: vec![ProviderAssignment {
                scope: AssignmentScope::Session,
                requested_provider: requested.clone(),
                observed_provider: None,
                observed_with: ObservedWith::Fixture,
                evidence:
                    "requested provider missing and no matching provider in fixture fallback_chain"
                        .to_string(),
            }],
            provider_assignment_granularity: ProviderAssignmentGranularity::Session,
            errors: vec![format!(
                "requested provider {:?} not found in available providers",
                request
            )],
        },
    }
}

pub fn parse_fixture(path: impl AsRef<Path>) -> Result<FixtureProbeState, ProbeError> {
    let contents = fs::read_to_string(path)?;
    let fixture: FixtureProbeState = serde_json::from_str(&contents).map_err(|err| ProbeError {
        message: err.to_string(),
    })?;
    Ok(fixture)
}

pub fn parse_fixture_str(source: &str) -> Result<FixtureProbeState, ProbeError> {
    let fixture: FixtureProbeState = serde_json::from_str(source).map_err(|err| ProbeError {
        message: err.to_string(),
    })?;
    Ok(fixture)
}

pub fn run_probe(request: &ProbeRequest) -> ProbeReport {
    match &request.fixture {
        Some(path) => match parse_fixture(path) {
            Ok(fixture) => build_fixture_report(&request.requested_provider, &fixture),
            Err(err) => ProbeReport {
                requested_provider: canonical_provider(&request.requested_provider),
                provider_probe_status: ProviderProbeStatus::FixtureInvalid,
                observed_provider_assignment: vec![ProviderAssignment {
                    scope: AssignmentScope::RequestedOnly,
                    requested_provider: canonical_provider(&request.requested_provider),
                    observed_provider: None,
                    observed_with: ObservedWith::DryRun,
                    evidence: format!("failed to parse fixture: {err}"),
                }],
                provider_assignment_granularity: ProviderAssignmentGranularity::NotProbed,
                errors: vec![err.message],
            },
        },
        None => build_dry_run_report(&request.requested_provider),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_schema_serialization() {
        let report = ProbeReport {
            requested_provider: "CUDAExecutionProvider".to_string(),
            provider_probe_status: ProviderProbeStatus::NotProbedDryRun,
            observed_provider_assignment: vec![ProviderAssignment {
                scope: AssignmentScope::RequestedOnly,
                requested_provider: "CUDAExecutionProvider".to_string(),
                observed_provider: None,
                observed_with: ObservedWith::DryRun,
                evidence: "dry-run stub".to_string(),
            }],
            provider_assignment_granularity: ProviderAssignmentGranularity::DryRun,
            errors: vec!["No runtime probe executed".to_string()],
        };

        let json = serde_json::to_string_pretty(&report).unwrap();
        let decoded: ProbeReport = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.requested_provider, "CUDAExecutionProvider");
        assert_eq!(
            decoded.provider_probe_status,
            ProviderProbeStatus::NotProbedDryRun
        );
        assert_eq!(decoded.observed_provider_assignment.len(), 1);
    }

    #[test]
    fn fixture_probe_matches_expected_assignment() {
        let fixture = parse_fixture_str(
            r#"{
                "available_providers": ["OpenVINOExecutionProvider"],
                "fallback_chain": ["CUDAExecutionProvider", "CPUExecutionProvider", "OpenVINOExecutionProvider"]
            }"#,
        )
        .unwrap();

        let request = ProbeRequest {
            requested_provider: "CUDAExecutionProvider".to_string(),
            fixture: None,
        };
        let report = build_fixture_report(&request.requested_provider, &fixture);

        assert_eq!(
            report.provider_probe_status,
            ProviderProbeStatus::RequestedProviderUnavailableWithFallback
        );
        assert_eq!(
            report
                .observed_provider_assignment
                .first()
                .expect("assignment")
                .observed_provider,
            Some("OpenVINOExecutionProvider".to_string())
        );
    }
}
