use std::sync::Arc;

use alloy::{
    dyn_abi::{DynSolType, DynSolValue, Eip712Domain, PropertyDef, TypedData},
    hex,
    primitives::{Address, Signature, SignatureError, U256, normalize_v},
    signers::ledger::{
        HDPath, LedgerError,
        coins_ledger::{APDUCommand, Ledger, common::APDUData, transports::LedgerAsync},
    },
};
use eyre::eyre;
use tokio::sync::Mutex;

use crate::signer::ledger::types::{
    EIP712Type, INS, P1, P2, P2Definition, P2Implementation, TYPE_ARRAY_BIT, TYPE_SIZE_BIT,
};

use std::future::Future;
use std::pin::Pin;

/// Ledger signer implementation for EIP-712 signing.
/// This module provides secured functionality to sign EIP-712 messages using a Ledger hardware wallet.
///
/// ```mermaid
/// sequenceDiagram
///     participant HW as Hardware Wallet
///     participant SW as Software Wallet
///
///     loop N times for N structs
///         SW ->> HW: struct name
///         HW ->> SW: OK / KO
///         loop N times for N fields
///             SW ->> HW: struct definition
///             note over HW: store definition in RAM
///             HW ->> SW: OK / KO
///         end
///     end
///
///     opt if filtering is used
///         SW ->> HW: activate filtering
///         HW ->> SW: OK / KO
///     end
///
///     SW ->> HW: set root struct to "EIP712Domain"
///     HW ->> SW: OK / KO
///     loop N times for N fields
///         SW ->> HW: struct implementation
///         HW ->> SW: OK / KO
///         note over HW: field is displayed
///     end
///
///     opt if filtering is used
///         SW ->> HW: filtering message info
///         HW ->> SW: OK / KO
///     end
///
///     SW ->> HW: set root struct to primary type
///     HW ->> SW: OK / KO
///     loop N times for N fields
///         opt if filtering is used
///             SW ->> HW: filter
///             HW ->> SW: OK / KO
///         end
///
///         SW ->> HW: struct implementation
///         HW ->> SW: OK / KO
///         note over HW: field is displayed
///     end
///
///     SW ->> HW: SIGN
///     HW ->> SW: OK (with r,s,v) / KO
/// ```
mod types;

pub struct LedgerSigner {
    transport: Arc<Mutex<Ledger>>,
    derivation: HDPath, // aka DerivationType
}

impl LedgerSigner {
    /// Instantiate the application by acquiring a lock on the ledger device.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// use alloy_signer_ledger::{HDPath, LedgerSigner};
    ///
    /// let ledger = LedgerSigner::new(HDPath::LedgerLive(0)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(derivation: HDPath) -> Result<Self, LedgerError> {
        let transport = Ledger::init().await?;
        let _ = Self::get_address_with_path_transport(&transport, &derivation).await?;

        Ok(Self {
            transport: Mutex::new(transport).into(),
            derivation,
        })
    }

    /// Copied from `alloy::signers::ledger::LedgerSigner::get_address_with_path_transport`
    async fn get_address_with_path_transport(
        transport: &Ledger,
        derivation: &HDPath,
    ) -> Result<Address, LedgerError> {
        let data = APDUData::new(&Self::path_to_bytes(derivation));

        let command = APDUCommand {
            cla: 0xe0,
            ins: INS::GET_PUBLIC_KEY as u8,
            p1: P1::NON_CONFIRM as u8,
            p2: P2::NO_CHAINCODE as u8,
            data,
            response_len: None,
        };

        let answer = transport.exchange(&command).await?;
        let result = answer.data().ok_or(LedgerError::UnexpectedNullResponse)?;

        let address = {
            // extract the address from the response
            let offset = 1 + result[0] as usize;
            let address_str = &result[offset + 1..offset + 1 + result[offset] as usize];
            let mut address = [0; 20];
            address.copy_from_slice(&hex::decode(address_str)?);
            address.into()
        };
        Ok(address)
    }

    // helper which converts a derivation path to bytes
    // Copied from `alloy::signers::ledger::LedgerSigner::path_to_bytes`
    fn path_to_bytes(derivation: &HDPath) -> Vec<u8> {
        let derivation = derivation.to_string();
        let elements = derivation.split('/').skip(1).collect::<Vec<_>>();
        let depth = elements.len();

        let mut bytes = vec![depth as u8];
        for derivation_index in elements {
            let hardened = derivation_index.contains('\'');
            let mut index = derivation_index.replace('\'', "").parse::<u32>().unwrap();
            if hardened {
                index |= 0x80000000;
            }

            bytes.extend(index.to_be_bytes());
        }

        bytes
    }

    pub async fn sign_typed_data(&self, data: &TypedData) -> eyre::Result<Signature> {
        const EIP712_MIN_VERSION: &str = ">=1.9.19";
        let req = semver::VersionReq::parse(EIP712_MIN_VERSION)?;
        let version = self.version().await?;

        // Enforce app version is greater than EIP712_MIN_VERSION
        if !req.matches(&version) {
            return Err(eyre!(
                "Ledger app version {version} does not satisfy {EIP712_MIN_VERSION}"
            ));
        }

        let transport = self.transport.lock().await;

        // Phase 1: Send all struct definitions
        self.send_struct_definitions(&transport, data).await?;

        // Phase 2: Send domain implementation
        self.send_domain_implementation(&transport, &data.domain)
            .await?;

        // Phase 3: Send message implementation
        self.send_message_implementation(&transport, data).await?;

        // Phase 4: Sign
        self.sign_eip712(&transport).await
    }

    // ── Phase 1: Struct Definitions ──────────────────────────────

    /// Send all struct type definitions to the Ledger device.
    /// This includes EIP712Domain and all types referenced by the primary type.
    async fn send_struct_definitions(
        &self,
        transport: &Ledger,
        data: &TypedData,
    ) -> eyre::Result<()> {
        async fn send_definition<T: TypeDefExt>(
            signer: &LedgerSigner,
            transport: &Ledger,
            key_name: &str,
            property: T,
        ) -> eyre::Result<()> {
            let field_data = encode_field_definition(key_name, property)?;
            signer
                .send_apdu(
                    transport,
                    INS::SEND_EIP712_STRUCT_DEFINITION,
                    P1::NON_CONFIRM as u8,
                    P2Definition::FIELD as u8,
                    &field_data,
                )
                .await
        }

        // Encode Domain struct definition
        let domain_def = data.domain();
        let mut domain_fields = Vec::new();
        if domain_def.name.is_some() {
            domain_fields.push(("name", DynSolType::String));
        }
        if domain_def.version.is_some() {
            domain_fields.push(("version", DynSolType::String));
        }
        if domain_def.chain_id.is_some() {
            domain_fields.push(("chainId", DynSolType::Uint(256)));
        }
        if domain_def.verifying_contract.is_some() {
            domain_fields.push(("verifyingContract", DynSolType::Address));
        }
        if domain_def.salt.is_some() {
            domain_fields.push(("salt", DynSolType::FixedBytes(32)));
        }

        self.send_apdu(
            transport,
            INS::SEND_EIP712_STRUCT_DEFINITION,
            P1::NON_CONFIRM as u8,
            P2Definition::NAME as u8,
            Eip712Domain::NAME.as_bytes(),
        )
        .await?;

        for (key, type_def) in domain_fields {
            send_definition(self, transport, key, type_def).await?;
        }

        // Encode message struct definitions (primary type and all referenced types)
        let types = data.resolver.linearize(&data.primary_type)?;

        for type_def in types {
            // Send struct name
            self.send_apdu(
                transport,
                INS::SEND_EIP712_STRUCT_DEFINITION,
                P1::NON_CONFIRM as u8,
                P2Definition::NAME as u8,
                type_def.type_name().as_bytes(),
            )
            .await?;

            // Send each field definition
            for field in type_def.props() {
                send_definition(self, transport, field.name(), field.clone()).await?;
            }
        }

        Ok(())
    }

    // ── Phase 2: Domain Implementation ───────────────────────────

    /// Send the EIP712Domain field values to the Ledger device.
    async fn send_domain_implementation(
        &self,
        transport: &Ledger,
        domain: &Eip712Domain,
    ) -> eyre::Result<()> {
        // Set root struct to "EIP712Domain"
        self.send_apdu(
            transport,
            INS::SEND_EIP712_STRUCT_IMPLEMENTATION,
            P1::NON_CONFIRM as u8,
            P2Implementation::ROOT as u8,
            Eip712Domain::NAME.as_bytes(),
        )
        .await?;

        if let Some(name) = &domain.name {
            self.send_field_value(transport, name.as_bytes()).await?;
        }

        if let Some(version) = &domain.version {
            self.send_field_value(transport, version.as_bytes()).await?;
        }

        if let Some(chain_id) = domain.chain_id {
            self.send_field_value(transport, &encode_uint256(&chain_id))
                .await?;
        }

        if let Some(verifying_contract) = &domain.verifying_contract {
            self.send_field_value(transport, verifying_contract.as_slice())
                .await?;
        }

        if let Some(salt) = &domain.salt {
            self.send_field_value(transport, salt.as_slice()).await?;
        }

        Ok(())
    }

    // ── Phase 3: Message Implementation ──────────────────────────

    /// Send the primary type message field values to the Ledger device.
    async fn send_message_implementation(
        &self,
        transport: &Ledger,
        data: &TypedData,
    ) -> eyre::Result<()> {
        // Set root struct to the primary type
        self.send_apdu(
            transport,
            INS::SEND_EIP712_STRUCT_IMPLEMENTATION,
            P1::NON_CONFIRM as u8,
            P2Implementation::ROOT as u8,
            data.primary_type.as_bytes(),
        )
        .await?;

        // Coerce message JSON to DynSolValue tree
        let coerced = data.coerce()?;

        // Send the fields of the message struct
        if let DynSolValue::CustomStruct { tuple, .. } = &coerced {
            self.send_struct_fields(transport, tuple, data).await?;
        } else {
            return Err(eyre!("Expected CustomStruct for primary type"));
        }

        Ok(())
    }

    /// Recursively send struct field values. For nested structs, sets a new root
    /// and recurses. For arrays, sends the array header then each element.
    async fn send_struct_fields(
        &self,
        transport: &Ledger,
        fields: &[DynSolValue],
        data: &TypedData,
    ) -> eyre::Result<()> {
        for field in fields.iter() {
            self.send_value(transport, field, data).await?;
        }

        Ok(())
    }

    /// Send a single value to the device, handling nested structs and arrays.
    fn send_value<'a>(
        &'a self,
        transport: &'a Ledger,
        value: &'a DynSolValue,
        data: &'a TypedData,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            match value {
                DynSolValue::CustomStruct { tuple, .. } => {
                    self.send_struct_fields(transport, tuple, data).await?;
                }
                DynSolValue::Array(items) | DynSolValue::FixedArray(items) => {
                    // Send array size
                    let size = items.len() as u8;
                    self.send_apdu(
                        transport,
                        INS::SEND_EIP712_STRUCT_IMPLEMENTATION,
                        P1::NON_CONFIRM as u8,
                        P2Implementation::ARRAY as u8,
                        &[size],
                    )
                    .await?;

                    for item in items {
                        self.send_value(transport, item, data).await?;
                    }
                }
                _ => {
                    let raw = encode_sol_value(value)?;
                    self.send_field_value(transport, &raw).await?;
                }
            }
            Ok(())
        })
    }

    /// Send a raw field value with the 2-byte BE length prefix.
    async fn send_field_value(&self, transport: &Ledger, value: &[u8]) -> eyre::Result<()> {
        let mut payload = Vec::with_capacity(2 + value.len());
        payload.extend_from_slice(&(value.len() as u16).to_be_bytes());
        payload.extend_from_slice(value);

        // For values larger than 255 bytes, send in chunks
        if payload.len() <= 255 {
            self.send_apdu(
                transport,
                INS::SEND_EIP712_STRUCT_IMPLEMENTATION,
                P1::NON_CONFIRM as u8,
                P2Implementation::FIELD as u8,
                &payload,
            )
            .await?;
        } else {
            let chunks: Vec<&[u8]> = payload.chunks(255).collect();
            let last_idx = chunks.len() - 1;
            for (i, chunk) in chunks.iter().enumerate() {
                let p1 = if i == last_idx {
                    P1::NON_CONFIRM as u8
                } else {
                    P1::PARTIAL as u8
                };
                self.send_apdu(
                    transport,
                    INS::SEND_EIP712_STRUCT_IMPLEMENTATION,
                    p1,
                    P2Implementation::FIELD as u8,
                    chunk,
                )
                .await?;
            }
        }

        Ok(())
    }

    // ── Phase 4: Sign ────────────────────────────────────────────

    /// Send the final SIGN command with P2=0x01 (full implementation).
    async fn sign_eip712(&self, transport: &Ledger) -> eyre::Result<Signature> {
        let data = Self::path_to_bytes(&self.derivation);

        let command = APDUCommand {
            cla: 0xe0,
            ins: INS::SIGN_ETH_EIP_712 as u8,
            p1: P1::NON_CONFIRM as u8,
            p2: P2::IMPL_FULL as u8,
            data: APDUData::new(&data),
            response_len: None,
        };

        let answer = transport.exchange(&command).await?;
        let result = answer.data().ok_or(LedgerError::UnexpectedNullResponse)?;

        if result.len() != 65 {
            return Err(LedgerError::ShortResponse {
                got: result.len(),
                expected: 65,
            }
            .into());
        }

        let parity = normalize_v(result[0] as u64).ok_or(LedgerError::SignatureError(
            SignatureError::InvalidParity(result[0] as u64),
        ))?;
        Ok(Signature::from_bytes_and_parity(&result[1..], parity))
    }

    // ── Helpers ──────────────────────────────────────────────────

    /// Send an APDU command and check for success.
    async fn send_apdu(
        &self,
        transport: &Ledger,
        ins: INS,
        p1: u8,
        p2: u8,
        data: &[u8],
    ) -> eyre::Result<()> {
        let command = APDUCommand {
            cla: 0xe0,
            ins: ins as u8,
            p1,
            p2,
            data: APDUData::new(data),
            response_len: None,
        };

        transport.exchange(&command).await?;
        Ok(())
    }

    /// Returns the semver of the Ethereum ledger app
    /// Copied from `alloy::signers::ledger::LedgerSigner::version`
    pub async fn version(&self) -> Result<semver::Version, LedgerError> {
        let transport = self.transport.lock().await;

        let command = APDUCommand {
            cla: 0xe0,
            ins: INS::GET_APP_CONFIGURATION as u8,
            p1: P1::NON_CONFIRM as u8,
            p2: P2::NO_CHAINCODE as u8,
            data: APDUData::new(&[]),
            response_len: None,
        };

        let answer = transport.exchange(&command).await?;
        let data = answer.data().ok_or(LedgerError::UnexpectedNullResponse)?;
        let &[_flags, major, minor, patch] = data else {
            return Err(LedgerError::ShortResponse {
                got: data.len(),
                expected: 4,
            });
        };
        let version = semver::Version::new(major as u64, minor as u64, patch as u64);
        Ok(version)
    }
}

fn encode_field_definition<T: TypeDefExt>(key_name: &str, property: T) -> eyre::Result<Vec<u8>> {
    fn __encode_field_definition(key_name: &str, type_def: &DynSolType) -> eyre::Result<Vec<u8>> {
        let mut data = Vec::new();

        let array_levels = get_array_levels(type_def);

        let has_array = !array_levels.is_empty();

        // Determine the base EIP-712 type and optional size
        let (eip712_type, type_size) = get_base_type(type_def)?;

        let has_size = type_size.is_some();

        // Build TypeDesc byte
        let mut type_desc: u8 = eip712_type as u8;
        if has_array {
            type_desc |= TYPE_ARRAY_BIT;
        }
        if has_size {
            type_desc |= TYPE_SIZE_BIT;
        }
        data.push(type_desc);

        // TypeName (only for custom types — use the base name, not the full
        // type string which may include array suffixes)
        if eip712_type == EIP712Type::Custom {
            let name = get_base_sol_type_name(type_def);
            data.push(name.len() as u8);
            data.extend_from_slice(name.as_bytes());
        }

        // TypeSize (only if TypeSize bit is set)
        if let Some(size) = type_size {
            data.push(size);
        }

        // Array info
        if has_array {
            data.push(array_levels.len() as u8);
            for level in &array_levels {
                match level {
                    ArrayLevel::Dynamic => data.push(0),
                    ArrayLevel::Fixed(size) => {
                        data.push(1);
                        data.push(*size);
                    }
                }
            }
        }

        // Key name (always present)
        let key_bytes = key_name.as_bytes();
        data.push(key_bytes.len() as u8);
        data.extend_from_slice(key_bytes);

        Ok(data)
    }

    let type_def = property.into_type_def()?;

    __encode_field_definition(key_name, &type_def)
}

/// Extract the base type name, stripping array wrappers.
/// e.g. Array(CustomStruct { name: "Person" }) → "Person"
fn get_base_sol_type_name(type_def: &DynSolType) -> std::borrow::Cow<'_, str> {
    match type_def {
        DynSolType::Array(inner) | DynSolType::FixedArray(inner, _) => {
            get_base_sol_type_name(inner)
        }
        DynSolType::CustomStruct { name, .. } => std::borrow::Cow::Borrowed(name),
        other => unreachable!(
            "Expected only CustomStruct or array types at this point, got {:?}",
            other
        ),
    }
}

/// Determine the EIP-712 type enum and optional byte size from a base type
/// string (no array suffixes).
fn get_base_type(type_def: &DynSolType) -> eyre::Result<(EIP712Type, Option<u8>)> {
    match type_def {
        DynSolType::Address => Ok((EIP712Type::Address, None)),
        DynSolType::Bool => Ok((EIP712Type::Bool, None)),
        DynSolType::String => Ok((EIP712Type::String, None)),
        DynSolType::Bytes => Ok((EIP712Type::DynamicBytes, None)),
        DynSolType::FixedBytes(size) => Ok((EIP712Type::FixedBytes, Some(*size as u8))),
        DynSolType::Uint(bits) => Ok((EIP712Type::Uint, Some((*bits / 8) as u8))),
        DynSolType::Int(bits) => Ok((EIP712Type::Int, Some((*bits / 8) as u8))),
        DynSolType::Array(inner) | DynSolType::FixedArray(inner, _) => get_base_type(inner),
        DynSolType::CustomStruct { .. } => Ok((EIP712Type::Custom, None)),

        DynSolType::Tuple(_) => Err(eyre::eyre!("Unsupported type: Tuple")),
        DynSolType::Function => Err(eyre::eyre!("Unsupported type: Function")),
    }
}

enum ArrayLevel {
    Dynamic,
    Fixed(u8),
}

fn get_array_levels(type_def: &DynSolType) -> Vec<ArrayLevel> {
    let mut levels = Vec::new();
    let mut current = type_def;
    loop {
        match current {
            DynSolType::Array(next) => {
                levels.push(ArrayLevel::Dynamic);
                current = next.as_ref();
            }
            DynSolType::FixedArray(next, size) => {
                levels.push(ArrayLevel::Fixed(*size as u8));
                current = next.as_ref();
            }
            _ => break levels,
        }
    }
}

/// Encode a DynSolValue as raw bytes for sending to the Ledger device.
fn encode_sol_value(value: &DynSolValue) -> eyre::Result<Vec<u8>> {
    match value {
        DynSolValue::Bool(b) => Ok(vec![*b as u8]),
        DynSolValue::Int(i, bits) => {
            let bytes = (*bits / 8).max(1);
            let be = i.to_be_bytes::<32>();
            Ok(be[32 - bytes..].to_vec())
        }
        DynSolValue::Uint(u, bits) => {
            let bytes = (*bits / 8).max(1);
            let be = u.to_be_bytes::<32>();
            Ok(be[32 - bytes..].to_vec())
        }
        DynSolValue::FixedBytes(word, size) => Ok(word[..*size].to_vec()),
        DynSolValue::Address(addr) => Ok(addr.as_slice().to_vec()),
        DynSolValue::Bytes(b) => Ok(b.clone()),
        DynSolValue::String(s) => Ok(s.as_bytes().to_vec()),
        _ => Err(eyre!("Unexpected value type for field encoding")),
    }
}

/// Encode a U256 as minimal big-endian bytes (strip leading zeros).
fn encode_uint256(value: &U256) -> Vec<u8> {
    let be = value.to_be_bytes::<32>();
    // Find first non-zero byte
    let start = be.iter().position(|&b| b != 0).unwrap_or(31);
    be[start..].to_vec()
}

trait TypeDefExt {
    fn into_type_def(self) -> eyre::Result<DynSolType>;
}

impl TypeDefExt for DynSolType {
    fn into_type_def(self) -> eyre::Result<DynSolType> {
        Ok(self)
    }
}

impl TypeDefExt for PropertyDef {
    fn into_type_def(self) -> eyre::Result<DynSolType> {
        parse_type_def(&self)
    }
}

fn parse_type_def(property: &PropertyDef) -> eyre::Result<DynSolType> {
    match DynSolType::parse(property.type_name()) {
        Ok(r#type) => Ok(r#type),
        // Interpret parsing errors as custom struct, handling array suffixes
        // e.g. "Person[]" → Array(CustomStruct { name: "Person" })
        Err(alloy::dyn_abi::Error::TypeParser(_)) => {
            let type_name = property.type_name();
            let base_end = type_name.find('[').unwrap_or(type_name.len());
            let base_name = &type_name[..base_end];

            let mut result = DynSolType::CustomStruct {
                name: base_name.to_string(),
                tuple: Vec::new(),
                prop_names: Vec::new(),
            };

            // Parse and wrap array suffixes
            let mut remaining = &type_name[base_end..];
            while remaining.starts_with('[') {
                if let Some(close) = remaining.find(']') {
                    let inner = &remaining[1..close];
                    result = if inner.is_empty() {
                        DynSolType::Array(Box::new(result))
                    } else if let Ok(size) = inner.parse::<usize>() {
                        DynSolType::FixedArray(Box::new(result), size)
                    } else {
                        DynSolType::Array(Box::new(result))
                    };
                    remaining = &remaining[close + 1..];
                } else {
                    break;
                }
            }

            Ok(result)
        }
        Err(e) => Err(e.into()),
    }
}
