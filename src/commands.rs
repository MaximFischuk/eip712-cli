use std::path::PathBuf;

use alloy::{
    dyn_abi::TypedData,
    hex,
    primitives::{Address, B256, Signature},
    signers::{
        Signer,
        k256::ecdsa::VerifyingKey,
        ledger::HDPath,
        local::{MnemonicBuilder, PrivateKeySigner, coins_bip39::English},
        utils::public_key_to_address,
    },
};
use clap::ArgMatches;

use crate::output;
use crate::schema;
use crate::signer::LedgerSigner;

/// Load typed data from a file argument or stdin, and extract the pretty flag.
fn load_input(args: &ArgMatches) -> eyre::Result<(TypedData, bool)> {
    let pretty = args.get_flag("pretty");
    let json = match args.get_one::<String>("input").map(PathBuf::from) {
        Some(file_path) => schema::load_and_validate(file_path)?,
        None => schema::load_and_validate_stdin()?,
    };
    Ok((json, pretty))
}

/// Parse the HD derivation path from CLI args.
/// Uses `--hd-path` if provided, otherwise `HDPath::LedgerLive(--index)`.
fn hd_path_from_args(args: &ArgMatches) -> HDPath {
    if let Some(path) = args.get_one::<String>("hd-path") {
        HDPath::Other(path.clone())
    } else {
        let index = args.get_one::<u32>("index").copied().unwrap_or(0);
        HDPath::LedgerLive(index as usize)
    }
}

fn print_sign_output(
    json: &TypedData,
    signing_hash: &B256,
    signature: &Signature,
    pretty: bool,
) -> eyre::Result<()> {
    if pretty {
        output::print_pretty_sign_output(json, signing_hash, signature)
    } else {
        println!("{signature}");
        Ok(())
    }
}

/// Run the `hash` subcommand.
pub fn run_hash(args: &ArgMatches) -> eyre::Result<()> {
    let (json, pretty) = load_input(args)?;
    let signing_hash = json.eip712_signing_hash()?;

    if pretty {
        output::print_pretty_hash_output(&json, &signing_hash)?;
    } else {
        println!("0x{}", hex::encode(signing_hash));
    }

    Ok(())
}

/// Run the `sign` subcommand.
pub async fn run_sign(args: &ArgMatches) -> eyre::Result<()> {
    let (json, pretty) = load_input(args)?;

    if args.get_flag("ledger") {
        let hd_path = hd_path_from_args(args);
        let signature = if args.get_flag("insecure") {
            eprintln!(
                "Warning: using Ledger in insecure mode. Make sure you trust the data you are signing."
            );
            let ledget = alloy::signers::ledger::LedgerSigner::new(hd_path, None).await?;
            ledget.sign_dynamic_typed_data(&json).await?
        } else {
            let ledger = LedgerSigner::new(hd_path).await?;
            ledger.sign_typed_data(&json).await?
        };
        let signing_hash = json.eip712_signing_hash()?;
        return print_sign_output(&json, &signing_hash, &signature, pretty);
    }

    let credential = if let Some(signer) = args.get_one::<PrivateKeySigner>("private-key") {
        signer.clone()
    } else if let Some(mnemonic) = args.get_one::<String>("mnemonic") {
        let mut builder = MnemonicBuilder::<English>::default().phrase(mnemonic);
        builder = if let Some(path) = args.get_one::<String>("hd-path") {
            builder.derivation_path(path.clone())?
        } else {
            builder.index(args.get_one::<u32>("index").copied().unwrap_or(0))?
        };
        builder.build()?
    } else {
        return Err(eyre::eyre!(
            "Either a private key or a mnemonic must be provided"
        ));
    };
    let signing_hash = json.eip712_signing_hash()?;
    let signature = credential.sign_hash(&signing_hash).await?;
    print_sign_output(&json, &signing_hash, &signature, pretty)
}

/// Run the `verify` subcommand.
pub fn run_verify(args: &ArgMatches) -> eyre::Result<()> {
    let (json, pretty) = load_input(args)?;
    let signing_hash = json.eip712_signing_hash()?;

    let signature = *args.get_one::<Signature>("signature").unwrap();

    let recovered_address = signature
        .recover_address_from_prehash(&signing_hash)
        .map_err(|e| eyre::eyre!("Failed to recover address from signature: {e}"))?;

    let expected_address = if let Some(addr) = args.get_one::<Address>("address") {
        *addr
    } else {
        let pk = args.get_one::<VerifyingKey>("public-key").unwrap();
        public_key_to_address(pk)
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
        println!("Verified");
    }

    Ok(())
}
