use bitlocker_control::run_with_args;

#[test]
fn test_help_flag_returns_success() {
    let args = vec!["bitlocker-control", "--help"];
    let result = run_with_args(args);

    assert!(
        result.is_ok(),
        "Calling --help should return Ok(()) without executing logic."
    );
}

#[test]
fn test_config_flag_displays_json_safely() {
    let args = vec!["bitlocker-control", "--config"];
    let result = run_with_args(args);

    assert!(
        result.is_ok(),
        "Calling --config should succeed and display the configuration."
    );
}

#[test]
fn test_missing_subcommand_returns_error() {
    let args = vec!["bitlocker-control"];
    let result = run_with_args(args);

    assert!(
        result.is_err(),
        "Running without arguments should return a Command error."
    );

    if let Err(e) = result {
        // Assert updated to reflect the new "Missing CLI arguments." string
        assert!(e.to_string().contains("Missing CLI arguments"));
    }
}

#[test]
fn test_invalid_subcommand_returns_error() {
    let args = vec!["bitlocker-control", "hack-the-mainframe"];
    let result = run_with_args(args);

    assert!(
        result.is_err(),
        "Running an invalid subcommand should return an error."
    );

    if let Err(e) = result {
        // Here we ensure it properly identifies invalid vs missing
        assert!(e.to_string().contains("Invalid CLI arguments"));
    }
}
