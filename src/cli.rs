use alloy::{
    primitives::{Address, Signature},
    signers::{k256::ecdsa::VerifyingKey, local::PrivateKeySigner},
};
use clap::{ArgGroup, Command, arg, builder::FalseyValueParser};

const PRIVATE_KEY_HELP_HEADING: &str = "Private Key Signer";
const MNEMONIC_HELP_HEADING: &str = "Mnemonic Signer";
const LEDGER_HELP_HEADING: &str = "Ledger Signer";
const TURNKEY_HELP_HEADING: &str = "Turnkey Signer";

const EIP712_INPUT_ENV_NAME: &str = "EIP712_INPUT";
const EIP712_PRETTY_ENV_NAME: &str = "EIP712_PRETTY";
const EIP712_PRIVATE_KEY_ENV_NAME: &str = "EIP712_PRIVATE_KEY";
const EIP712_MNEMONIC_ENV_NAME: &str = "EIP712_MNEMONIC";
const EIP712_HD_PATH_ENV_NAME: &str = "EIP712_HD_PATH";
const EIP712_HD_PATH_INDEX_ENV_NAME: &str = "EIP712_HD_PATH_INDEX";
const EIP712_LEDGER_ENV_NAME: &str = "EIP712_LEDGER";
const EIP712_INSECURE_ENV_NAME: &str = "EIP712_INSECURE";
const EIP712_PUBLIC_KEY_ENV_NAME: &str = "EIP712_PUBLIC_KEY";
const EIP712_ADDRESS_ENV_NAME: &str = "EIP712_ADDRESS";
const EIP712_SIGNATURE_ENV_NAME: &str = "EIP712_SIGNATURE";
const EIP712_TURNKEY_ENV_NAME: &str = "EIP712_TURNKEY";
const EIP712_TURNKEY_API_PRIVATE_KEY_ENV_NAME: &str = "EIP712_TURNKEY_API_PRIVATE_KEY";
const EIP712_TURNKEY_ORGANIZATION_ID_ENV_NAME: &str = "EIP712_TURNKEY_ORGANIZATION_ID";
const EIP712_TURNKEY_ADDRESS_ENV_NAME: &str = "EIP712_TURNKEY_ADDRESS";
const EIP712_TURNKEY_BASE_URL_ENV_NAME: &str = "EIP712_TURNKEY_BASE_URL";

/// Build the CLI command structure for the EIP-712 tool.
pub fn build_cli() -> Command {
    Command::new("Eip712 Cli")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("A command-line tool for working with EIP-712 typed data.")
        .help_expected(true)
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("hash")
                .about("Hash EIP-712 typed data")
                .arg(arg!([input] "Path to the JSON file (reads from stdin if omitted)").index(1).env(EIP712_INPUT_ENV_NAME))
                .arg(arg!(--pretty "Print output as a pretty colored table").env(EIP712_PRETTY_ENV_NAME)),
        )
        .subcommand(
            Command::new("sign")
                .about("Sign EIP-712 typed data")
                .arg(arg!(--"private-key" <PRIVATE_KEY> "The private key to sign the data with").value_parser(clap::value_parser!(PrivateKeySigner)).env(EIP712_PRIVATE_KEY_ENV_NAME).help_heading(PRIVATE_KEY_HELP_HEADING))
                .args([
                    arg!(--mnemonic <MNEMONIC> "The mnemonic to derive the private key from").env(EIP712_MNEMONIC_ENV_NAME).help_heading(MNEMONIC_HELP_HEADING),
                    arg!(--"hd-path" <HD_PATH> "HD derivation path (e.g. m/44'/60'/0'/0/0)")
                        .conflicts_with("private-key")
                        .env(EIP712_HD_PATH_ENV_NAME)
                        .help_heading(MNEMONIC_HELP_HEADING),
                    arg!(--index <INDEX> "HD derivation path index")
                        .conflicts_with("private-key")
                        .value_parser(clap::value_parser!(u32))
                        .env(EIP712_HD_PATH_INDEX_ENV_NAME)
                        .help_heading(MNEMONIC_HELP_HEADING),
                ])
                .args([
                    arg!(--ledger "Use a Ledger hardware wallet for signing")
                        .value_parser(FalseyValueParser::new())
                        .env(EIP712_LEDGER_ENV_NAME)
                        .help_heading(LEDGER_HELP_HEADING),
                    arg!(--"hd-path" <HD_PATH> "HD derivation path (e.g. m/44'/60'/0'/0/0)")
                        .conflicts_with("private-key")
                        .env(EIP712_HD_PATH_ENV_NAME)
                        .help_heading(LEDGER_HELP_HEADING),
                    arg!(--index <INDEX> "HD derivation path index")
                        .conflicts_with("private-key")
                        .value_parser(clap::value_parser!(u32))
                        .env(EIP712_HD_PATH_INDEX_ENV_NAME)
                        .help_heading(LEDGER_HELP_HEADING),
                    arg!(--insecure "Allow signing raw hashes blindly without showing the user the structured data. This allows using old Ledger Ethereum apps or Ledger Nano S")
                            .conflicts_with_all(["private-key", "mnemonic"])
                            .env(EIP712_INSECURE_ENV_NAME)
                            .help_heading(LEDGER_HELP_HEADING),
                ])
                .args([
                    arg!(--turnkey "Use Turnkey for signing")
                        .value_parser(FalseyValueParser::new())
                        .env(EIP712_TURNKEY_ENV_NAME)
                        .requires_all([
                            "turnkey-api-private-key",
                            "turnkey-organization-id",
                            "turnkey-address",
                        ])
                        .help_heading(TURNKEY_HELP_HEADING),
                    arg!(--"turnkey-api-private-key" <TURNKEY_API_PRIVATE_KEY> "The Turnkey API private key (P-256) used to authorize signing requests")
                        .env(EIP712_TURNKEY_API_PRIVATE_KEY_ENV_NAME)
                        .requires("turnkey")
                        .help_heading(TURNKEY_HELP_HEADING),
                    arg!(--"turnkey-organization-id" <TURNKEY_ORGANIZATION_ID> "The Turnkey organization ID that owns the signing key")
                        .env(EIP712_TURNKEY_ORGANIZATION_ID_ENV_NAME)
                        .requires("turnkey")
                        .help_heading(TURNKEY_HELP_HEADING),
                    arg!(--"turnkey-address" <TURNKEY_ADDRESS> "The Ethereum address of the Turnkey-managed signing key")
                        .value_parser(clap::value_parser!(Address))
                        .env(EIP712_TURNKEY_ADDRESS_ENV_NAME)
                        .requires("turnkey")
                        .help_heading(TURNKEY_HELP_HEADING),
                    arg!(--"turnkey-base-url" <TURNKEY_BASE_URL> "Optional Turnkey API base URL override")
                        .env(EIP712_TURNKEY_BASE_URL_ENV_NAME)
                        .requires("turnkey")
                        .help_heading(TURNKEY_HELP_HEADING),
                ])
                .arg(
                    arg!(<input> "Path to the JSON file containing the EIP-712 typed data")
                        .required(true)
                        .index(1)
                        .env(EIP712_INPUT_ENV_NAME),
                )
                .arg(arg!(--pretty "Print output as a pretty colored table").env(EIP712_PRETTY_ENV_NAME))
                .group(
                    ArgGroup::new("secret")
                        .args(["private-key", "mnemonic", "ledger", "turnkey"])
                        .required(true),
                )
                .group(
                    ArgGroup::new("mnemonic-args")
                        .args(["mnemonic", "index", "hd-path"])
                        .multiple(true),
                )
                .group(
                    ArgGroup::new("ledger-args")
                        .args(["ledger", "index", "hd-path"])
                        .multiple(true),
                )
                .group(
                    ArgGroup::new("derivation-args")
                        .args(["hd-path", "index"])
                        .multiple(false),
                ),
        )
        .subcommand(
            Command::new("verify")
                .about("Verify an EIP-712 signature")
                .arg(
                    arg!(<input> "Path to the JSON file containing the EIP-712 typed data")
                        .required(true)
                        .index(1)
                        .env(EIP712_INPUT_ENV_NAME),
                )
                .arg(arg!(--"public-key" <PUBLIC_KEY> "The uncompressed public key to verify against (hex, 64 or 65 bytes)").value_parser(clap::value_parser!(VerifyingKey)).env(EIP712_PUBLIC_KEY_ENV_NAME))
                .arg(arg!(--address <ADDRESS> "The Ethereum address to verify against").value_parser(clap::value_parser!(Address)).env(EIP712_ADDRESS_ENV_NAME))
                .arg(
                    arg!(--signature <SIGNATURE> "The 65-byte signature to verify (hex encoded)")
                        .required(true)
                        .value_parser(clap::value_parser!(Signature))
                        .env(EIP712_SIGNATURE_ENV_NAME),
                )
                .arg(arg!(--pretty "Print output as a pretty colored table").env(EIP712_PRETTY_ENV_NAME))
                .group(
                    ArgGroup::new("verifier")
                        .args(["public-key", "address"])
                        .required(true),
                ),
        )
}
