use alloy::{
    dyn_abi::TypedData,
    primitives::{B256, hex},
};
use colored::Colorize;
use comfy_table::{Cell, Color, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};

/// Print the common preamble (banner, domain info, primary type info) shared by hash and sign output.
fn print_common_preamble(title: &str, json: &TypedData) -> eyre::Result<()> {
    println!();
    println!("{}", format!(" {title} ").bold().white().on_bright_blue());
    println!();

    print_domain_info(json)?;
    print_primary_type_info(json)?;

    Ok(())
}

/// Print the pretty hash output (preamble + signing hash table).
pub fn print_pretty_hash_output(
    json: &TypedData,
    signing_hash: &alloy::primitives::B256,
) -> eyre::Result<()> {
    print_common_preamble("EIP-712 Hash Result", json)?;

    print_section_header("Signing Hash");
    let mut table = new_table();
    table.set_header(vec![
        Cell::new("Field").fg(Color::Cyan),
        Cell::new("Value").fg(Color::Cyan),
    ]);
    table.add_row(vec![
        Cell::new("Signing Hash").fg(Color::Yellow),
        Cell::new(format!("0x{}", hex::encode(signing_hash))).fg(Color::Green),
    ]);
    println!("{table}");
    println!();

    Ok(())
}

/// Print the pretty sign output (preamble + signature table).
pub fn print_pretty_sign_output(
    json: &TypedData,
    signing_hash: &B256,
    signature: &alloy::signers::Signature,
) -> eyre::Result<()> {
    print_common_preamble("EIP-712 Signature Result", json)?;

    print_section_header("Signature");
    let mut table = new_table();
    table.set_header(vec![
        Cell::new("Field").fg(Color::Cyan),
        Cell::new("Value").fg(Color::Cyan),
    ]);
    table.add_row(vec![
        Cell::new("Signature").fg(Color::Yellow),
        Cell::new(format!("{signature}")).fg(Color::Green),
    ]);

    table.add_row(vec![
        Cell::new("Signing Hash").fg(Color::Yellow),
        Cell::new(format!("0x{}", hex::encode(signing_hash))).fg(Color::White),
    ]);

    let type_hash = json.type_hash()?;
    table.add_row(vec![
        Cell::new("Type Hash").fg(Color::Yellow),
        Cell::new(format!("0x{}", hex::encode(type_hash))).fg(Color::White),
    ]);
    println!("{table}");
    println!();

    Ok(())
}

fn print_domain_info(json: &TypedData) -> eyre::Result<()> {
    let domain = &json.domain;

    print_section_header("Domain (EIP712Domain)");

    let mut domain_meta_table = new_table();
    domain_meta_table.set_header(vec![
        Cell::new("Domain Property").fg(Color::Cyan),
        Cell::new("Value").fg(Color::Cyan),
    ]);

    let domain_encode_type = domain.encode_type();
    domain_meta_table.add_row(vec![
        Cell::new("encodeType").fg(Color::Yellow),
        Cell::new(&domain_encode_type).fg(Color::Magenta),
    ]);

    let domain_separator = domain.separator();
    domain_meta_table.add_row(vec![
        Cell::new("separator").fg(Color::Yellow),
        Cell::new(format!("0x{}", hex::encode(domain_separator))).fg(Color::Green),
    ]);

    let domain_type_hash = domain.type_hash();
    domain_meta_table.add_row(vec![
        Cell::new("typeHash").fg(Color::Yellow),
        Cell::new(format!("0x{}", hex::encode(domain_type_hash))).fg(Color::Green),
    ]);

    println!("{domain_meta_table}");
    println!();

    let mut params_table = new_table();
    params_table.set_header(vec![
        Cell::new("Parameter").fg(Color::Cyan),
        Cell::new("Value").fg(Color::Cyan),
    ]);

    if let Some(name) = &domain.name {
        params_table.add_row(vec![
            Cell::new("name").fg(Color::Yellow),
            Cell::new(name.as_ref()).fg(Color::White),
        ]);
    }
    if let Some(version) = &domain.version {
        params_table.add_row(vec![
            Cell::new("version").fg(Color::Yellow),
            Cell::new(version.as_ref()).fg(Color::White),
        ]);
    }
    if let Some(chain_id) = &domain.chain_id {
        params_table.add_row(vec![
            Cell::new("chainId").fg(Color::Yellow),
            Cell::new(format!("{chain_id}")).fg(Color::White),
        ]);
    }
    if let Some(verifying_contract) = &domain.verifying_contract {
        params_table.add_row(vec![
            Cell::new("verifyingContract").fg(Color::Yellow),
            Cell::new(format!("{verifying_contract}")).fg(Color::White),
        ]);
    }
    if let Some(salt) = &domain.salt {
        params_table.add_row(vec![
            Cell::new("salt").fg(Color::Yellow),
            Cell::new(format!("0x{}", hex::encode(salt))).fg(Color::White),
        ]);
    }

    println!("{params_table}");
    println!();

    Ok(())
}

fn print_primary_type_info(json: &TypedData) -> eyre::Result<()> {
    let primary_type = &json.primary_type;

    print_section_header(&format!("Primary Type ({primary_type})"));

    let mut type_table = new_table();
    type_table.set_header(vec![
        Cell::new("Property").fg(Color::Cyan),
        Cell::new("Value").fg(Color::Cyan),
    ]);

    let encode_type = json.resolver.encode_type(primary_type)?;
    type_table.add_row(vec![
        Cell::new("encodeType").fg(Color::Yellow),
        Cell::new(&encode_type).fg(Color::Magenta),
    ]);

    let type_hash = json.type_hash()?;
    type_table.add_row(vec![
        Cell::new("typeHash").fg(Color::Yellow),
        Cell::new(format!("0x{}", hex::encode(type_hash))).fg(Color::Green),
    ]);

    println!("{type_table}");
    println!();

    if let serde_json::Value::Object(map) = &json.message {
        print_section_header(&format!("{primary_type} Message Fields"));

        let mut msg_table = new_table();
        msg_table.set_header(vec![
            Cell::new("Field").fg(Color::Cyan),
            Cell::new("Value").fg(Color::Cyan),
        ]);

        for (key, value) in map {
            let display_value = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            msg_table.add_row(vec![
                Cell::new(key).fg(Color::Yellow),
                Cell::new(display_value).fg(Color::White),
            ]);
        }

        println!("{msg_table}");
        println!();
    }

    Ok(())
}

/// Print the pretty verify output (preamble + verified address table).
pub fn print_pretty_verify_output(
    json: &TypedData,
    address: &alloy::primitives::Address,
    signing_hash: &alloy::primitives::B256,
) -> eyre::Result<()> {
    print_common_preamble("EIP-712 Verification Result", json)?;

    print_section_header("Verification");
    let mut table = new_table();
    table.set_header(vec![
        Cell::new("Field").fg(Color::Cyan),
        Cell::new("Value").fg(Color::Cyan),
    ]);
    table.add_row(vec![
        Cell::new("Status").fg(Color::Yellow),
        Cell::new("Verified ✓").fg(Color::Green),
    ]);
    table.add_row(vec![
        Cell::new("Recovered Address").fg(Color::Yellow),
        Cell::new(format!("{address}")).fg(Color::Green),
    ]);
    table.add_row(vec![
        Cell::new("Signing Hash").fg(Color::Yellow),
        Cell::new(format!("0x{}", hex::encode(signing_hash))).fg(Color::White),
    ]);
    println!("{table}");
    println!();

    Ok(())
}

fn print_section_header(title: &str) {
    println!("{}", format!("── {title} ──").bold().cyan());
}

fn new_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);
    table
}
