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

/// Load an EIP-712 JSON file, validate it against the schema, and parse it into `TypedData`.
pub fn load_and_validate(file_path: PathBuf) -> eyre::Result<TypedData> {
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
