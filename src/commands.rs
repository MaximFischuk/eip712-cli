use std::{path::PathBuf, str::FromStr};

use alloy::{
    primitives::hex,
    signers::Signer,
    signers::local::{MnemonicBuilder, PrivateKeySigner, coins_bip39::English},
};
use clap::ArgMatches;
use eyre::OptionExt;

use crate::output;
use crate::schema::load_and_validate;

/// Extract the common arguments (input file path, pretty flag) from the CLI args.
fn parse_common_args(args: &ArgMatches) -> eyre::Result<(PathBuf, bool)> {
    let file_path = args
        .get_one::<String>("input")
        .map(PathBuf::from)
        .ok_or_eyre("Input file path is required")?;
    let pretty = args.get_flag("pretty");
    Ok((file_path, pretty))
}

/// Run the `hash` subcommand.
pub fn run_hash(args: &ArgMatches) -> eyre::Result<()> {
    let (file_path, pretty) = parse_common_args(args)?;

    let json = load_and_validate(file_path)?;
    let signing_hash = json.eip712_signing_hash()?;

    if pretty {
        output::print_pretty_hash_output(&json, &signing_hash)?;
    } else {
        print!("0x{}", hex::encode(signing_hash));
    }

    Ok(())
}

/// Run the `sign` subcommand.
pub async fn run_sign(args: &ArgMatches) -> eyre::Result<()> {
    let (file_path, pretty) = parse_common_args(args)?;

    let credential = if let Some(private_key) = args.get_one::<String>("private-key") {
        PrivateKeySigner::from_str(private_key)?
    } else if let Some(mnemonic) = args.get_one::<String>("mnemonic") {
        MnemonicBuilder::<English>::default()
            .phrase(mnemonic)
            .index(*args.get_one::<u32>("index").unwrap_or(&0))?
            .build()?
    } else {
        return Err(eyre::eyre!(
            "Either a private key or a mnemonic must be provided"
        ));
    };

    let json = load_and_validate(file_path)?;
    let signature = credential.sign_hash(&json.eip712_signing_hash()?).await?;

    if pretty {
        output::print_pretty_sign_output(&json, &signature)?;
    } else {
        print!("{signature}");
    }

    Ok(())
}
