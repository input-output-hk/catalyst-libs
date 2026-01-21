# Test reports

## Catalyst-Libs Test Reports

Test reports for catalyst-libs are generated as part of the CI/CD pipeline.

### Integration Test Reports

Integration tests for catalyst-libs are run in the CI pipeline. Test results are available in:

- CI pipeline logs and artifacts
- Local test output when running tests

### Running Tests Locally

To run tests for all crates:

```bash
cd rust
cargo test --workspace
```

To run tests for a specific crate:

```bash
cd rust/<crate-name>
cargo test
```

### Test Coverage

Test coverage reports are generated as part of the CI process. Coverage targets:

- **Unit Tests**: 80%+ coverage for critical paths
- **Integration Tests**: Coverage for cross-crate functionality
- **Documentation Tests**: All code examples in documentation are tested

### Note on Catalyst Voices Test Reports

The Catalyst Voices application (in the catalyst-voices repository) has its own test reports:

- [Catalyst Voices Main Branch Test Report](https://input-output-hk.github.io/catalyst-voices/allure-action/main/test-report/latest.html)
- [Catalyst Voices Nightly Test Report](https://input-output-hk.github.io/catalyst-voices/allure-action/main/nightly-test-report/latest.html)

These reports are for the Catalyst Voices application, which uses catalyst-libs as a dependency, but are separate from catalyst-libs test reports.
