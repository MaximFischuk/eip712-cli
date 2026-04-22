# EIP-712 CLI

A command-line tool for working with [EIP-712](https://eips.ethereum.org/EIPS/eip-712) typed structured data — hash, sign, and verify messages.

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

<details>
<summary>Example <code>--pretty</code> output</summary>

```
EIP-712 Hash Result

╭───────────────────────────────────── Domain (EIP712Domain) ──────────────────────────────────────╮
│ encodeType         │ EIP712Domain(string name,string version,uint256 chainId,address             │
│                    │ verifyingContract)                                                          │
│ separator          │ 0x6137beb405d9ff777172aa879e33edb34a1460e701802746c5ef96e741710e59          │
│ typeHash           │ 0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f          │
├─────────────────────────────────────── Domain Parameters ────────────────────────────────────────┤
│ name               │ Ether Mail                                                                  │
│ version            │ 1                                                                           │
│ chainId            │ 5                                                                           │
│ verifyingContract  │ 0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC                                  │
├────────────────────────────────────── Primary Type (Mail) ───────────────────────────────────────┤
│ encodeType         │ Mail(Person from,Person to,string contents)Person(string name,address[]     │
│                    │ wallets)                                                                    │
│ typeHash           │ 0x5b1f8a8eaa25a46aa443b0cc79f29a9c3b8cdefbf74027593e73d8d407340c12          │
├────────────────────────────────────── Mail Message Fields ───────────────────────────────────────┤
│ contents           │ Hello, Bob!                                                                 │
│ from               │ {"name":"Cow","wallets":["0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826","0xDe │
│                    │ aDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF"]}                                   │
│ to                 │ {"name":"Bob","wallets":["0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB","0xB0 │
│                    │ BdaBea57B0BDABeA57b0bdABEA57b0BDabEa57","0xB0B0b0b0b0b0B0000000000000000000 │
│                    │ 00000000"]}                                                                 │
├────────────────────────────────────────── Signing Hash ──────────────────────────────────────────┤
│ Signing Hash       │ 0xe1ba5e914d73ca971c6f36e8c6782929b2fec69b602442297f0a4bac35b387ce          │
╰──────────────────────────────────────────────────────────────────────────────────────────────────╯
```

</details>

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

Sign with a Ledger hardware wallet:

See [docs/ledger.md](docs/ledger.md) for setup, derivation path options, insecure mode, and environment variables.

Sign with Turnkey:

See [docs/turnkey.md](docs/turnkey.md) for required credentials, environment variables, and usage examples.

Pretty output:

```sh
eip712 sign --private-key 0x... --pretty message.json
```

<details>
<summary>Example <code>--pretty</code> output</summary>

```
EIP-712 Signature Result

╭───────────────────────────────────── Domain (EIP712Domain) ──────────────────────────────────────╮
│ encodeType         │ EIP712Domain(string name,string version,uint256 chainId,address             │
│                    │ verifyingContract)                                                          │
│ separator          │ 0x6137beb405d9ff777172aa879e33edb34a1460e701802746c5ef96e741710e59          │
│ typeHash           │ 0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f          │
├─────────────────────────────────────── Domain Parameters ────────────────────────────────────────┤
│ name               │ Ether Mail                                                                  │
│ version            │ 1                                                                           │
│ chainId            │ 5                                                                           │
│ verifyingContract  │ 0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC                                  │
├────────────────────────────────────── Primary Type (Mail) ───────────────────────────────────────┤
│ encodeType         │ Mail(Person from,Person to,string contents)Person(string name,address[]     │
│                    │ wallets)                                                                    │
│ typeHash           │ 0x5b1f8a8eaa25a46aa443b0cc79f29a9c3b8cdefbf74027593e73d8d407340c12          │
├────────────────────────────────────── Mail Message Fields ───────────────────────────────────────┤
│ contents           │ Hello, Bob!                                                                 │
│ from               │ {"name":"Cow","wallets":["0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826","0xDe │
│                    │ aDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF"]}                                   │
│ to                 │ {"name":"Bob","wallets":["0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB","0xB0 │
│                    │ BdaBea57B0BDABeA57b0bdABEA57b0BDabEa57","0xB0B0b0b0b0b0B0000000000000000000 │
│                    │ 00000000"]}                                                                 │
├────────────────────────────────────────── Signature ─────────────────────────────────────────────┤
│ Signature          │ 0x6e54a02f96876c3aea0300ac9bcb4a255af3c0db948451b79a1133128b89997a5f8785c07 │
│                    │ c55735bd0406cf8362558c9b5547921e7a95730e330527854e193281c                   │
│ Signing Hash       │ 0xe1ba5e914d73ca971c6f36e8c6782929b2fec69b602442297f0a4bac35b387ce          │
│ Type Hash          │ 0x5b1f8a8eaa25a46aa443b0cc79f29a9c3b8cdefbf74027593e73d8d407340c12          │
╰──────────────────────────────────────────────────────────────────────────────────────────────────╯
```

</details>

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

<details>
<summary>Example <code>--pretty</code> output</summary>

```
EIP-712 Verification Result

╭───────────────────────────────────── Domain (EIP712Domain) ──────────────────────────────────────╮
│ encodeType         │ EIP712Domain(string name,string version,uint256 chainId,address             │
│                    │ verifyingContract)                                                          │
│ separator          │ 0x6137beb405d9ff777172aa879e33edb34a1460e701802746c5ef96e741710e59          │
│ typeHash           │ 0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f          │
├─────────────────────────────────────── Domain Parameters ────────────────────────────────────────┤
│ name               │ Ether Mail                                                                  │
│ version            │ 1                                                                           │
│ chainId            │ 5                                                                           │
│ verifyingContract  │ 0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC                                  │
├────────────────────────────────────── Primary Type (Mail) ───────────────────────────────────────┤
│ encodeType         │ Mail(Person from,Person to,string contents)Person(string name,address[]     │
│                    │ wallets)                                                                    │
│ typeHash           │ 0x5b1f8a8eaa25a46aa443b0cc79f29a9c3b8cdefbf74027593e73d8d407340c12          │
├────────────────────────────────────── Mail Message Fields ───────────────────────────────────────┤
│ contents           │ Hello, Bob!                                                                 │
│ from               │ {"name":"Cow","wallets":["0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826","0xDe │
│                    │ aDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF"]}                                   │
│ to                 │ {"name":"Bob","wallets":["0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB","0xB0 │
│                    │ BdaBea57B0BDABeA57b0bdABEA57b0BDabEa57","0xB0B0b0b0b0b0B0000000000000000000 │
│                    │ 00000000"]}                                                                 │
├────────────────────────────────────────── Verification ──────────────────────────────────────────┤
│ Status             │ Verified ✓                                                                  │
│ Recovered Address  │ 0xFCAd0B19bB29D4674531d6f115237E16AfCE377c                                  │
│ Signing Hash       │ 0xe1ba5e914d73ca971c6f36e8c6782929b2fec69b602442297f0a4bac35b387ce          │
╰──────────────────────────────────────────────────────────────────────────────────────────────────╯
```

</details>

## License

[MIT](LICENSE)
