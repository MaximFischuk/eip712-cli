# Turnkey Signing

Use Turnkey to sign EIP-712 typed data with a remotely managed signer, without storing a local Ethereum private key in this CLI.

The CLI uses Alloy's Turnkey signer integration and sends signing requests through the Turnkey API.

## Required options

Turnkey signing is enabled with `--turnkey` and requires these values:

- `--turnkey-api-private-key <TURNKEY_API_PRIVATE_KEY>`
- `--turnkey-organization-id <TURNKEY_ORGANIZATION_ID>`
- `--turnkey-address <TURNKEY_ADDRESS>`

Optional:

- `--turnkey-base-url <TURNKEY_BASE_URL>` — override the default Turnkey API base URL

## Important note about the API key

`--turnkey-api-private-key` is the **Turnkey API private key** used to authenticate API requests. It is not the Ethereum private key that produces the signature.

The actual Ethereum signing key remains managed by Turnkey and is selected with `--turnkey-address`.

## Basic usage

Sign a message with Turnkey:

```sh
eip712 sign \
  --turnkey \
  --turnkey-api-private-key 0123abcd... \
  --turnkey-organization-id your-org-id \
  --turnkey-address 0x1234567890abcdef1234567890abcdef12345678 \
  message.json
```

Pretty output:

```sh
eip712 sign \
  --turnkey \
  --turnkey-api-private-key 0123abcd... \
  --turnkey-organization-id your-org-id \
  --turnkey-address 0x1234567890abcdef1234567890abcdef12345678 \
  --pretty \
  message.json
```

Use a custom Turnkey API base URL:

```sh
eip712 sign \
  --turnkey \
  --turnkey-api-private-key 0123abcd... \
  --turnkey-organization-id your-org-id \
  --turnkey-address 0x1234567890abcdef1234567890abcdef12345678 \
  --turnkey-base-url https://api.turnkey.com \
  message.json
```

## Environment variables

Turnkey-specific options can be provided via environment variables:

- `EIP712_TURNKEY`
- `EIP712_TURNKEY_API_PRIVATE_KEY`
- `EIP712_TURNKEY_ORGANIZATION_ID`
- `EIP712_TURNKEY_ADDRESS`
- `EIP712_TURNKEY_BASE_URL`

General `sign` command options also still apply, including:

- `EIP712_INPUT`
- `EIP712_PRETTY`

Example:

```sh
export EIP712_TURNKEY=true
export EIP712_TURNKEY_API_PRIVATE_KEY=0123abcd...
export EIP712_TURNKEY_ORGANIZATION_ID=your-org-id
export EIP712_TURNKEY_ADDRESS=0x1234567890abcdef1234567890abcdef12345678
export EIP712_INPUT=message.json
export EIP712_PRETTY=true

eip712 sign
```

## How signing works

When Turnkey mode is enabled, the CLI:

1. Loads and validates the EIP-712 JSON input
2. Computes the EIP-712 signing hash locally
3. Sends the hash to Turnkey for signing
4. Prints the resulting Ethereum signature

This means the typed data is still validated locally before the signing request is made.

## Verifying the signature

You can verify a Turnkey-produced signature with the same `verify` command used for any other signer:

```sh
eip712 verify \
  --address 0x1234567890abcdef1234567890abcdef12345678 \
  --signature 0x... \
  message.json
```

## Troubleshooting

### "Turnkey signer requires --turnkey-..."

One or more required Turnkey parameters were not provided. Make sure `--turnkey` is set and all required Turnkey options are present, either as CLI flags or environment variables.

### "invalid address"

`--turnkey-address` must be a valid Ethereum address for the Turnkey-managed key you want to use.

### Authentication or API errors

Double-check:

- your Turnkey API private key
- your organization ID
- the signing address
- the base URL if you are overriding it

## Security notes

- Do not commit `EIP712_TURNKEY_API_PRIVATE_KEY` to version control.
- Prefer environment variables or a local `.env` file for secrets.
- Treat the Turnkey API private key as a sensitive credential.