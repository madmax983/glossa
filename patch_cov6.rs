fn validate_cli(cli: &Cli) -> Result<()> {
    // Validate global file parameter if provided
    if let Some(ref file) = cli.file {
        validate_extension(file)?;
    }

    // Validate subcommand file inputs
    if let Some(input) = cli.command.as_ref().and_then(|cmd| cmd.input_path()) {
        validate_extension(input)?;
    }

    Ok(())
}
