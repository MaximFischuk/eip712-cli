use std::{path::PathBuf, str::FromStr};

use alloy::{
    dyn_abi::TypedData,
    primitives::{Address, Signature, hex},
    signers::{
        Signer,
        local::{MnemonicBuilder, PrivateKeySigner, coins_bip39::English},
        utils::raw_public_key_to_address,
    },
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

/// Derive an Ethereum address from an uncompressed public key (64 or 65 bytes).
fn address_from_public_key_bytes(bytes: &[u8]) -> eyre::Result<Address> {
    let raw = match bytes.len() {
        64 => bytes,
        65 if bytes[0] == 0x04 => &bytes[1..],
        _ => {
            return Err(eyre::eyre!(
                "Public key must be 64 or 65 bytes (uncompressed), got {} bytes",
                bytes.len()
            ));
        }
    };
    Ok(raw_public_key_to_address(raw))
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

/// Run the `verify` subcommand.
pub fn run_verify(args: &ArgMatches) -> eyre::Result<()> {
    let (json, pretty) = load_input(args)?;
    let signing_hash = json.eip712_signing_hash()?;

    let sig_str = args.get_one::<String>("signature").unwrap();
    let sig_hex = sig_str.strip_prefix("0x").unwrap_or(sig_str);
    let sig_bytes = hex::decode(sig_hex)?;
    eyre::ensure!(
        sig_bytes.len() == 65,
        "Signature must be exactly 65 bytes, got {}",
        sig_bytes.len()
    );

    let signature = Signature::try_from(sig_bytes.as_slice())
        .map_err(|e| eyre::eyre!("Invalid signature: {e}"))?;

    let recovered_address = signature
        .recover_address_from_prehash(&signing_hash)
        .map_err(|e| eyre::eyre!("Failed to recover address from signature: {e}"))?;

    let expected_address = if let Some(addr_str) = args.get_one::<String>("address") {
        Address::from_str(addr_str)?
    } else {
        let pk_str = args.get_one::<String>("public-key").unwrap();
        let pk_hex = pk_str.strip_prefix("0x").unwrap_or(pk_str);
        let pk_bytes = hex::decode(pk_hex)?;
        address_from_public_key_bytes(&pk_bytes)?
    };

    eyre::ensure!(
        recovered_address == expected_address,
        "Signature verification failed: recovered address {} does not match expected address {}",
        recovered_address,
        expected_address
    );

    if pretty {
        output::print_pretty_verify_output(&json, &recovered_address, &signing_hash)?;
    } else {
        print!("Verified");
    }

    Ok(())
}
