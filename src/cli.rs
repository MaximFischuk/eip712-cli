use alloy::{
    primitives::{Address, Signature},
    signers::{k256::ecdsa::VerifyingKey, local::PrivateKeySigner},
};
use clap::{ArgGroup, Command, arg, builder::FalseyValueParser};

const PRIVATE_KEY_HELP_HEADING: &str = "Private Key Signer";
const MNEMONIC_HELP_HEADING: &str = "Mnemonic Signer";
const LEDGER_HELP_HEADING: &str = "Ledger Signer";

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
                .arg(arg!([input] "Path to the JSON file (reads from stdin if omitted)").index(1).env("EIP712_INPUT"))
                .arg(arg!(--pretty "Print output as a pretty colored table").env("EIP712_PRETTY")),
        )
        .subcommand(
            Command::new("sign")
                .about("Sign EIP-712 typed data")
                .arg(arg!(--"private-key" <PRIVATE_KEY> "The private key to sign the data with").value_parser(clap::value_parser!(PrivateKeySigner)).env("EIP712_PRIVATE_KEY").help_heading(PRIVATE_KEY_HELP_HEADING))
                .args([
                    arg!(--mnemonic <MNEMONIC> "The mnemonic to derive the private key from").env("EIP712_MNEMONIC").help_heading(MNEMONIC_HELP_HEADING),
                    arg!(--"hd-path" <HD_PATH> "HD derivation path (e.g. m/44'/60'/0'/0/0)")
                        .conflicts_with("private-key")
                        .env("EIP712_HD_PATH")
                        .help_heading(MNEMONIC_HELP_HEADING),
                    arg!(--index <INDEX> "HD derivation path index")
                        .conflicts_with("private-key")
                        .value_parser(clap::value_parser!(u32))
                        .env("EIP712_HD_PATH_INDEX")
                        .help_heading(MNEMONIC_HELP_HEADING),
                ])
                .args([
                    arg!(--ledger "Use a Ledger hardware wallet for signing")
                        .value_parser(FalseyValueParser::new())
                        .env("EIP712_LEDGER")
                        .help_heading(LEDGER_HELP_HEADING),
                    arg!(--"hd-path" <HD_PATH> "HD derivation path (e.g. m/44'/60'/0'/0/0)")
                        .conflicts_with("private-key")
                        .env("EIP712_HD_PATH")
                        .help_heading(LEDGER_HELP_HEADING),
                    arg!(--index <INDEX> "HD derivation path index")
                        .conflicts_with("private-key")
                        .value_parser(clap::value_parser!(u32))
                        .env("EIP712_HD_PATH_INDEX")
                        .help_heading(LEDGER_HELP_HEADING),
                    arg!(--insecure "Allow signing raw hashes blindly without showing the user the structured data. This allows using old Ledger Ethereum apps or Ledger Nano S")
                            .conflicts_with_all(["private-key", "mnemonic"])
                            .env("EIP712_INSECURE")
                            .help_heading(LEDGER_HELP_HEADING),
                ])
                .arg(
                    arg!(<input> "Path to the JSON file containing the EIP-712 typed data")
                        .required(true)
                        .index(1)
                        .env("EIP712_INPUT"),
                )
                .arg(arg!(--pretty "Print output as a pretty colored table").env("EIP712_PRETTY"))
                .group(
                    ArgGroup::new("secret")
                        .args(["private-key", "mnemonic", "ledger"])
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
                        .env("EIP712_INPUT"),
                )
                .arg(arg!(--"public-key" <PUBLIC_KEY> "The uncompressed public key to verify against (hex, 64 or 65 bytes)").value_parser(clap::value_parser!(VerifyingKey)).env("EIP712_PUBLIC_KEY"))
                .arg(arg!(--address <ADDRESS> "The Ethereum address to verify against").value_parser(clap::value_parser!(Address)).env("EIP712_ADDRESS"))
                .arg(
                    arg!(--signature <SIGNATURE> "The 65-byte signature to verify (hex encoded)")
                        .required(true)
                        .value_parser(clap::value_parser!(Signature))
                        .env("EIP712_SIGNATURE"),
                )
                .arg(arg!(--pretty "Print output as a pretty colored table").env("EIP712_PRETTY"))
                .group(
                    ArgGroup::new("verifier")
                        .args(["public-key", "address"])
                        .required(true),
                ),
        )
}
