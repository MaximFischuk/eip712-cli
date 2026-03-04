use std::{path::PathBuf, str::FromStr};

use alloy::{
    dyn_abi::TypedData,
    primitives::hex,
    signers::Signer,
    signers::local::{MnemonicBuilder, PrivateKeySigner, coins_bip39::English},
};
use clap::ArgMatches;

use crate::output;
use crate::schema;

/// Load typed data from a file argument or stdin, and extract the pretty flag.
fn load_input(args: &ArgMatches) -> eyre::Result<(TypedData, bool)> {
    let pretty = args.get_flag("pretty");
    let json = match args.get_one::<String>("input").map(PathBuf::from) {
        Some(file_path) => schema::load_and_validate(file_path)?,
        None => schema::load_and_validate_stdin()?,
    };
    Ok((json, pretty))
}

/// Run the `hash` subcommand.
pub fn run_hash(args: &ArgMatches) -> eyre::Result<()> {
    let (json, pretty) = load_input(args)?;
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
    let (json, pretty) = load_input(args)?;

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

    let signature = credential.sign_hash(&json.eip712_signing_hash()?).await?;

    if pretty {
        output::print_pretty_sign_output(&json, &signature)?;
    } else {
        print!("{signature}");
    }

    Ok(())
}
