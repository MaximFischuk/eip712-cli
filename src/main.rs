use std::{path::PathBuf, str::FromStr};

use alloy::{
    dyn_abi::TypedData,
    primitives::hex,
    signers::Signer,
    signers::local::{MnemonicBuilder, PrivateKeySigner, coins_bip39::English},
};
use clap::{ArgGroup, ArgMatches, Command, arg};
use colored::Colorize;
use comfy_table::{Cell, Color, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};
use eyre::OptionExt;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cmd = Command::new("Eip712 Cli")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("A command-line tool for working with EIP-712 typed data.")
        .help_expected(true)
        .propagate_version(true)
        .subcommand(
            Command::new("hash")
                .about("Hash EIP-712 typed data")
                .arg(
                    arg!(<input> "Path to the JSON file containing the EIP-712 typed data")
                        .required(true)
                        .index(1),
                )
                .arg(arg!(--pretty "Print output as a pretty colored table")),
        )
        .subcommand(
            Command::new("sign")
                .about("Sign EIP-712 typed data")
                .arg(arg!(--"private-key" <PRIVATE_KEY> "The private key to sign the data with"))
                .args([
                    arg!(--mnemonic <MNEMONIC> "The mnemonic to derive the private key from"),
                    arg!(--index <INDEX> "The index of the derived private key (default: 0)")
                        .default_value("0")
                        .conflicts_with("private-key"),
                ])
                .arg(
                    arg!(<input> "Path to the JSON file containing the EIP-712 typed data")
                        .required(true)
                        .index(1),
                )
                .arg(arg!(--pretty "Print output as a pretty colored table"))
                .group(
                    ArgGroup::new("secret")
                        .args(["private-key", "mnemonic"])
                        .required(true),
                )
                .group(
                    ArgGroup::new("mnemonic-args")
                        .args(["mnemonic", "index"])
                        .multiple(true),
                ),
        )
        .get_matches();

    match cmd.subcommand() {
        Some(("hash", args)) => {
            run_hash(args)?;
        }
        Some(("sign", args)) => {
            run_sign(args).await?;
        }
        _ => {
            eprintln!("No valid subcommand was provided. Use --help for more information.");
        }
    }
    Ok(())
}

fn load_and_validate(file_path: PathBuf) -> eyre::Result<TypedData> {
    let instance: serde_json::Value = serde_json::from_reader(std::fs::File::open(file_path)?)?;

    let schema = serde_json::from_str(EIP712_ABI_SCHEMA)?;
    let validator = jsonschema::validator_for(&schema)?;

    let evaluation = validator.evaluate(&instance);
    let errors: Vec<_> = evaluation.iter_errors().collect();
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("Validation error: {}", error.error);
        }
        return Err(eyre::eyre!("Input JSON failed EIP-712 schema validation"));
    }

    let json: TypedData = serde_json::from_value(instance)?;
    Ok(json)
}

fn run_hash(args: &ArgMatches) -> eyre::Result<()> {
    let file_path = args
        .get_one::<String>("input")
        .map(PathBuf::from)
        .ok_or_eyre("Input file path is required")?;
    let pretty = args.get_flag("pretty");

    let json = load_and_validate(file_path)?;
    let signing_hash = json.eip712_signing_hash()?;

    if pretty {
        print_pretty_hash_output(&json, &signing_hash)?;
    } else {
        print!("0x{}", hex::encode(signing_hash));
    }

    Ok(())
}

async fn run_sign(args: &ArgMatches) -> eyre::Result<()> {
    let file_path = args
        .get_one::<String>("input")
        .map(PathBuf::from)
        .ok_or_eyre("Input file path is required")?;
    let pretty = args.get_flag("pretty");

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
        print_pretty_sign_output(&json, &signature)?;
    } else {
        print!("{signature}");
    }

    Ok(())
}

fn print_pretty_hash_output(
    json: &TypedData,
    signing_hash: &alloy::primitives::B256,
) -> eyre::Result<()> {
    println!();
    println!(
        "{}",
        " EIP-712 Hash Result ".bold().white().on_bright_blue()
    );
    println!();

    // --- Domain ---
    print_domain_info(json)?;

    // --- Primary Type ---
    print_primary_type_info(json)?;

    // --- Signing Hash ---
    print_section_header("Signing Hash");
    let mut table = new_table();
    table.set_header(vec![
        Cell::new("Field").fg(Color::Cyan),
        Cell::new("Value").fg(Color::Cyan),
    ]);
    table.add_row(vec![
        Cell::new("Signing Hash").fg(Color::Yellow),
        Cell::new(format!("0x{}", hex::encode(signing_hash))).fg(Color::Green),
    ]);
    println!("{table}");
    println!();

    Ok(())
}

fn print_pretty_sign_output(
    json: &TypedData,
    signature: &alloy::signers::Signature,
) -> eyre::Result<()> {
    println!();
    println!(
        "{}",
        " EIP-712 Signature Result ".bold().white().on_bright_blue()
    );
    println!();

    // --- Domain ---
    print_domain_info(json)?;

    // --- Primary Type ---
    print_primary_type_info(json)?;

    // --- Signature ---
    print_section_header("Signature");
    let mut table = new_table();
    table.set_header(vec![
        Cell::new("Field").fg(Color::Cyan),
        Cell::new("Value").fg(Color::Cyan),
    ]);
    table.add_row(vec![
        Cell::new("Signature").fg(Color::Yellow),
        Cell::new(format!("{signature}")).fg(Color::Green),
    ]);

    let signing_hash = json.eip712_signing_hash()?;
    table.add_row(vec![
        Cell::new("Signing Hash").fg(Color::Yellow),
        Cell::new(format!("0x{}", hex::encode(signing_hash))).fg(Color::White),
    ]);

    let type_hash = json.type_hash()?;
    table.add_row(vec![
        Cell::new("Type Hash").fg(Color::Yellow),
        Cell::new(format!("0x{}", hex::encode(type_hash))).fg(Color::White),
    ]);
    println!("{table}");
    println!();

    Ok(())
}

fn print_domain_info(json: &TypedData) -> eyre::Result<()> {
    let domain = &json.domain;

    print_section_header("Domain (EIP712Domain)");

    // Domain encode_type, separator, and type_hash
    let mut domain_meta_table = new_table();
    domain_meta_table.set_header(vec![
        Cell::new("Domain Property").fg(Color::Cyan),
        Cell::new("Value").fg(Color::Cyan),
    ]);

    let domain_encode_type = domain.encode_type();
    domain_meta_table.add_row(vec![
        Cell::new("encodeType").fg(Color::Yellow),
        Cell::new(&domain_encode_type).fg(Color::Magenta),
    ]);

    let domain_separator = domain.separator();
    domain_meta_table.add_row(vec![
        Cell::new("separator").fg(Color::Yellow),
        Cell::new(format!("0x{}", hex::encode(domain_separator))).fg(Color::Green),
    ]);

    let domain_type_hash = domain.type_hash();
    domain_meta_table.add_row(vec![
        Cell::new("typeHash").fg(Color::Yellow),
        Cell::new(format!("0x{}", hex::encode(domain_type_hash))).fg(Color::Green),
    ]);

    println!("{domain_meta_table}");
    println!();

    // Domain parameters table
    let mut params_table = new_table();
    params_table.set_header(vec![
        Cell::new("Parameter").fg(Color::Cyan),
        Cell::new("Value").fg(Color::Cyan),
    ]);

    if let Some(name) = &domain.name {
        params_table.add_row(vec![
            Cell::new("name").fg(Color::Yellow),
            Cell::new(name.as_ref()).fg(Color::White),
        ]);
    }
    if let Some(version) = &domain.version {
        params_table.add_row(vec![
            Cell::new("version").fg(Color::Yellow),
            Cell::new(version.as_ref()).fg(Color::White),
        ]);
    }
    if let Some(chain_id) = &domain.chain_id {
        params_table.add_row(vec![
            Cell::new("chainId").fg(Color::Yellow),
            Cell::new(format!("{chain_id}")).fg(Color::White),
        ]);
    }
    if let Some(verifying_contract) = &domain.verifying_contract {
        params_table.add_row(vec![
            Cell::new("verifyingContract").fg(Color::Yellow),
            Cell::new(format!("{verifying_contract}")).fg(Color::White),
        ]);
    }
    if let Some(salt) = &domain.salt {
        params_table.add_row(vec![
            Cell::new("salt").fg(Color::Yellow),
            Cell::new(format!("0x{}", hex::encode(salt))).fg(Color::White),
        ]);
    }

    println!("{params_table}");
    println!();

    Ok(())
}

fn print_primary_type_info(json: &TypedData) -> eyre::Result<()> {
    let primary_type = &json.primary_type;

    print_section_header(&format!("Primary Type ({primary_type})"));

    // Primary type encoding info
    let mut type_table = new_table();
    type_table.set_header(vec![
        Cell::new("Property").fg(Color::Cyan),
        Cell::new("Value").fg(Color::Cyan),
    ]);

    let encode_type = json.resolver.encode_type(primary_type)?;
    type_table.add_row(vec![
        Cell::new("encodeType").fg(Color::Yellow),
        Cell::new(&encode_type).fg(Color::Magenta),
    ]);

    let type_hash = json.type_hash()?;
    type_table.add_row(vec![
        Cell::new("typeHash").fg(Color::Yellow),
        Cell::new(format!("0x{}", hex::encode(type_hash))).fg(Color::Green),
    ]);

    println!("{type_table}");
    println!();

    // Message fields
    if let serde_json::Value::Object(map) = &json.message {
        print_section_header(&format!("{primary_type} Message Fields"));

        let mut msg_table = new_table();
        msg_table.set_header(vec![
            Cell::new("Field").fg(Color::Cyan),
            Cell::new("Value").fg(Color::Cyan),
        ]);

        for (key, value) in map {
            let display_value = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            msg_table.add_row(vec![
                Cell::new(key).fg(Color::Yellow),
                Cell::new(display_value).fg(Color::White),
            ]);
        }

        println!("{msg_table}");
        println!();
    }

    Ok(())
}

fn print_section_header(title: &str) {
    println!("{}", format!("── {title} ──").bold().cyan());
}

fn new_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);
    table
}

const EIP712_ABI_SCHEMA: &str = r#"
{
    "type": "object",
    "properties": {
        "types": {
            "type": "object",
            "properties": {
                "EIP712Domain": { "type": "array" }
            },
            "additionalProperties": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "type": { "type": "string" }
                    },
                    "required": ["name", "type"]
                }
            },
            "required": ["EIP712Domain"]
        },
        "primaryType": { "type": "string" },
        "domain": { "type": "object" },
        "message": { "type": "object" }
    },
    "required": ["types", "primaryType", "domain"]
}
"#;
