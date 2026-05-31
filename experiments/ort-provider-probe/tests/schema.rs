use std::fs;

use ort_provider_probe::*;

#[test]
fn parse_sample_report_schema() {
    let contents = fs::read_to_string("fixtures/sample-report.json").unwrap();
    let report: ProbeReport = serde_json::from_str(&contents).unwrap();

    assert_eq!(report.requested_provider, "CUDAExecutionProvider");
    assert_eq!(report.errors.len(), 0);
    assert_eq!(report.observed_provider_assignment.len(), 1);
    assert_eq!(
        report.observed_provider_assignment[0].observed_provider,
        Some("CUDAExecutionProvider".to_string())
    );
}

#[test]
fn fixture_rejects_request_with_fallback() {
    let request = ProbeRequest {
        requested_provider: "TensorRTExecutionProvider".to_string(),
        fixture: None,
    };
    let dry_run_report = run_probe(&request);
    let report_with_fixture = run_probe(&ProbeRequest {
        requested_provider: request.requested_provider.clone(),
        fixture: Some("fixtures/host-provider-matrix.json".into()),
    });

    assert_eq!(
        dry_run_report.provider_probe_status,
        ProviderProbeStatus::NotProbedDryRun
    );
    assert_eq!(
        report_with_fixture.provider_probe_status,
        ProviderProbeStatus::RequestedProviderUnavailableWithFallback
    );
    assert_eq!(
        report_with_fixture.requested_provider,
        "TensorRTExecutionProvider"
    );
    assert_eq!(
        report_with_fixture
            .observed_provider_assignment
            .first()
            .expect("assignment")
            .observed_provider,
        Some("CPUExecutionProvider".to_string())
    );
    assert_eq!(report_with_fixture.errors.len(), 1);

    assert_eq!(
        report_with_fixture
            .observed_provider_assignment
            .first()
            .expect("assignment")
            .observed_with,
        ObservedWith::Fixture
    );
    assert!(
        !report_with_fixture
            .observed_provider_assignment
            .first()
            .expect("assignment")
            .evidence
            .is_empty()
    );

    let fixture_contents = fs::read_to_string("fixtures/host-provider-matrix.json").unwrap();
    let parsed_fixture = parse_fixture_str(&fixture_contents).unwrap();

    assert_eq!(dry_run_report.errors.len(), 2);
    assert_eq!(
        dry_run_report.errors[0],
        "No CUDA/ORT runtime probe executed in this prototype"
    );
    assert_eq!(parsed_fixture.fallback_chain[0], "CUDAExecutionProvider");
    assert_eq!(
        parsed_fixture.fallback_chain.len(),
        3,
        "sample fixture has 3 fallback entries"
    );
}
