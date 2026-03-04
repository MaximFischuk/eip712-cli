mod cli;
mod commands;
mod output;
mod schema;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cmd = cli::build_cli().get_matches();

    match cmd.subcommand() {
        Some(("hash", args)) => {
            commands::run_hash(args)?;
        }
        Some(("sign", args)) => {
            commands::run_sign(args).await?;
        }
        _ => {
            eprintln!("No valid subcommand was provided. Use --help for more information.");
        }
    }
    Ok(())
}
