#[cfg(test)]
mod tests {
    use glossa::semantic::GlossaType;
    // We can't easily test `glossa_type_to_json_schema` directly without making it public or using an integration test,
    // but the `codecov/patch` failure means that the coverage dropped significantly because the new file `sybil.rs` has untested lines.
    // The previous test run showed `sybil` wasn't even included in coverage probably because `sybil.rs` is behind `#[cfg(feature = "nova")]`
    // and we need an integration test or similar to test it. Wait, there *is* a test module inside `sybil.rs` that tests the matching logic.
    // Is that enough? We might need an integration test to cover the `run_sybil` function, which is 90% of the lines.
}
