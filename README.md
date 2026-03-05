# EIP-712 CLI

A command-line tool for working with [EIP-712](https://eips.ethereum.org/EIPS/eip-712) typed structured data — hash, sign, and verify messages offline.

## Features

- **Hash** — compute the EIP-712 signing hash from a JSON file or stdin
- **Sign** — sign typed data with a private key or mnemonic phrase
- **Verify** — verify a 65-byte signature against an Ethereum address or public key
- Pretty-printed table output with `--pretty`
- JSON schema validation of input data

## Installation

### Homebrew (macOS / Linux)

```sh
brew tap MaximFischuk/eip712-cli
brew install eip712-cli
```

### Pre-built binaries

Download the latest release for your platform from the [Releases](https://github.com/MaximFischuk/eip712-cli/releases) page.

Available targets:

- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-apple-darwin` (macOS Intel)
- `x86_64-unknown-linux-gnu` (Linux x86_64)
- `aarch64-unknown-linux-gnu` (Linux ARM64)

### Build from source

Requires [Rust](https://rustup.rs/) (edition 2024).

```sh
git clone https://github.com/MaximFischuk/eip712-cli.git
cd eip712-cli
cargo build --release
# binary at target/release/eip712
```

## Usage

### Input format

The tool accepts [EIP-712](https://eips.ethereum.org/EIPS/eip-712) typed data as a JSON file following the standard schema:

```json
{
  "types": {
    "EIP712Domain": [
      { "name": "name", "type": "string" },
      { "name": "version", "type": "string" },
      { "name": "chainId", "type": "uint256" },
      { "name": "verifyingContract", "type": "address" }
    ],
    "Person": [
      { "name": "wallet", "type": "address" },
      { "name": "age", "type": "uint8" }
    ]
  },
  "primaryType": "Person",
  "domain": {
    "name": "My DApp",
    "version": "1",
    "chainId": 1,
    "verifyingContract": "0x0000000000000000000000000000000000000000"
  },
  "message": {
    "wallet": "0x0000000000000000000000000000000000000000",
    "age": 42
  }
}
```

### Hash

Compute the EIP-712 signing hash:

```sh
# from a file
eip712 hash message.json

# from stdin
cat message.json | eip712 hash

# pretty output
eip712 hash --pretty message.json
```

### Sign

Sign typed data with a private key:

```sh
eip712 sign --private-key 0x... message.json
```

Sign with a mnemonic phrase:

```sh
eip712 sign --mnemonic "word1 word2 ... word12" message.json

# use a specific derivation index (default: 0)
# derivation path: m/44'/60'/0'/0/{index}
eip712 sign --mnemonic "word1 word2 ... word12" --index 2 message.json
```

Pretty output:

```sh
eip712 sign --private-key 0x... --pretty message.json
```

### Verify

Verify a signature against an Ethereum address:

```sh
eip712 verify --address 0x... --signature 0x... message.json
```

Verify against an uncompressed public key (64 or 65 bytes, hex):

```sh
eip712 verify --public-key 04abcd... --signature 0x... message.json
```

The command exits with code 0 on success and non-zero on failure with a descriptive error message.

Pretty output:

```sh
eip712 verify --address 0x... --signature 0x... --pretty message.json
```

## License

[MIT](LICENSE)
