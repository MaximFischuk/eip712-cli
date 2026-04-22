# Ledger Signing

Use Ledger to sign EIP-712 typed data with a hardware wallet.

## Requirements

Before signing, make sure you have:

- a connected Ledger device
- the Ethereum app installed on the device
- the device unlocked and ready to confirm requests

## Basic usage

Sign with the default Ledger Live derivation path at index `0`:

```sh
eip712 sign --ledger message.json
```

Use a different Ledger Live account index:

```sh
eip712 sign --ledger --index 2 message.json
```

Use an explicit derivation path instead of an index:

```sh
eip712 sign --ledger --hd-path "m/44'/60'/0'/0/5" message.json
```

Pretty-print the output:

```sh
eip712 sign --ledger --pretty message.json
```

## Options

### `--ledger`

Enables Ledger signing.

### `--index <INDEX>`

Uses the Ledger Live derivation path format:

```text
m/44'/60'/0'/0/{index}
```

If omitted, the default index is `0`.

### `--hd-path <HD_PATH>`

Overrides the derivation path with an explicit value, for example:

```text
m/44'/60'/0'/0/0
```

Use this when you want full control over the selected account path.

### `--insecure`

Allows signing the raw EIP-712 hash blindly, without showing the structured data on the Ledger device.

Example:

```sh
eip712 sign --ledger --insecure message.json
```

This mode exists for compatibility with older Ledger Ethereum apps or devices such as Ledger Nano S.

Only use `--insecure` if you trust the input you are signing.

## Derivation path behavior

For Ledger signing, the CLI chooses the derivation path like this:

- if `--hd-path` is provided, that exact path is used
- otherwise, the CLI uses `--index`
- if neither is provided, it defaults to Ledger Live index `0`

You cannot use `--hd-path` and `--index` together.

## Environment variables

All Ledger options can also be provided through environment variables:

- `EIP712_LEDGER`
- `EIP712_HD_PATH`
- `EIP712_HD_PATH_INDEX`
- `EIP712_INSECURE`
- `EIP712_INPUT`
- `EIP712_PRETTY`

Example:

```sh
EIP712_LEDGER=true \
EIP712_HD_PATH_INDEX=1 \
eip712 sign message.json
```

## Troubleshooting

If signing fails, check the following:

- the Ledger device is unlocked
- the Ethereum app is open
- the selected derivation path matches the address you expect
- the device is connected and ready to approve the request

## Notes

- Ledger support is available only for the `sign` command.
- The signature returned by the CLI is a standard 65-byte Ethereum signature.