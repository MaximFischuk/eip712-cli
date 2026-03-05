use clap::{ArgGroup, Command, arg};

/// Build the CLI command structure for the EIP-712 tool.
pub fn build_cli() -> Command {
    Command::new("Eip712 Cli")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("A command-line tool for working with EIP-712 typed data.")
        .help_expected(true)
        .propagate_version(true)
        .subcommand(
            Command::new("hash")
                .about("Hash EIP-712 typed data")
                .arg(arg!([input] "Path to the JSON file (reads from stdin if omitted)").index(1))
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
        .subcommand(
            Command::new("verify")
                .about("Verify an EIP-712 signature")
                .arg(
                    arg!(<input> "Path to the JSON file containing the EIP-712 typed data")
                        .required(true)
                        .index(1),
                )
                .arg(arg!(--"public-key" <PUBLIC_KEY> "The uncompressed public key to verify against (hex, 64 or 65 bytes)"))
                .arg(arg!(--address <ADDRESS> "The Ethereum address to verify against"))
                .arg(
                    arg!(--signature <SIGNATURE> "The 65-byte signature to verify (hex encoded)")
                        .required(true),
                )
                .arg(arg!(--pretty "Print output as a pretty colored table"))
                .group(
                    ArgGroup::new("verifier")
                        .args(["public-key", "address"])
                        .required(true),
                ),
        )
}
