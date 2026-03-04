use std::io::Read;
use std::path::PathBuf;

use alloy::dyn_abi::TypedData;

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

/// Load EIP-712 JSON from a file path, validate, and parse into `TypedData`.
pub fn load_and_validate(file_path: PathBuf) -> eyre::Result<TypedData> {
    let reader = std::fs::File::open(file_path)?;
    read_and_validate(reader)
}

/// Load EIP-712 JSON from stdin, validate, and parse into `TypedData`.
pub fn load_and_validate_stdin() -> eyre::Result<TypedData> {
    let reader = std::io::stdin().lock();
    read_and_validate(reader)
}

/// Validate and parse EIP-712 JSON from any reader.
fn read_and_validate(reader: impl Read) -> eyre::Result<TypedData> {
    let instance: serde_json::Value = serde_json::from_reader(reader)?;

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
