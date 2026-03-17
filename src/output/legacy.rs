//! Legacy pretty-print output using `comfy-table`.
//!
//! Renders the same logical sections as the new dashboard format (`super`) but
//! produces the classic multi-table layout that was the original output style.
//! The section data is built with the same helpers in the parent module and
//! structured using the shared [`super::table::Section`] type.
//!
//! Currently, this module is excluded from the project!

use alloy::{
    dyn_abi::TypedData,
    primitives::{B256, hex},
};
use colored::Colorize;
use comfy_table::{Cell, Color, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};

use super::table::{Accent, Section};

// ── Internal helpers ─────────────────────────────────────────────────────────

fn new_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);
    table
}

fn print_section_header(title: &str) {
    println!("{}", format!("── {title} ──").bold().cyan());
}

fn accent_to_key_color(accent: Accent) -> Color {
    match accent {
        Accent::Blue => Color::Blue,
        Accent::Cyan => Color::Cyan,
        Accent::Yellow => Color::Yellow,
        Accent::Magenta => Color::Magenta,
    }
}

/// Render a slice of [`Section`]s as separate comfy-table blocks, each
/// preceded by a section header line.
fn render_sections(sections: &[Section<'_>]) {
    for section in sections {
        print_section_header(section.title);
        let key_color = accent_to_key_color(section.accent);

        let mut table = new_table();
        table.set_header(vec![
            Cell::new("Field").fg(Color::Cyan),
            Cell::new("Value").fg(Color::Cyan),
        ]);
        for (key, value) in section.rows {
            table.add_row(vec![
                Cell::new(*key).fg(key_color),
                Cell::new(*value).fg(Color::White),
            ]);
        }
        println!("{table}");
        println!();
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Print the pretty hash output (legacy multi-table format).
pub fn print_pretty_hash_output(json: &TypedData, signing_hash: &B256) -> eyre::Result<()> {
    let dm = super::domain_meta_rows(json);
    let dp = super::domain_param_rows(json);
    let pt = super::primary_type_rows(json)?;
    let mf = super::message_rows(json);
    let primary_type = &json.primary_type;
    let pt_title = format!("Primary Type ({primary_type})");
    let mf_title = format!("{primary_type} Message Fields");
    let hr = vec![(
        "Signing Hash".to_string(),
        format!("0x{}", hex::encode(signing_hash)),
    )];

    let dm_r = super::to_refs(&dm);
    let dp_r = super::to_refs(&dp);
    let pt_r = super::to_refs(&pt);
    let mf_r = super::to_refs(&mf);
    let hr_r = super::to_refs(&hr);

    super::print_title("EIP-712 Hash Result");

    let mut sections: Vec<Section<'_>> = vec![Section {
        title: "Domain (EIP712Domain)",
        accent: Accent::Cyan,
        rows: &dm_r,
    }];
    if !dp_r.is_empty() {
        sections.push(Section {
            title: "Domain Parameters",
            accent: Accent::Cyan,
            rows: &dp_r,
        });
    }
    sections.push(Section {
        title: &pt_title,
        accent: Accent::Magenta,
        rows: &pt_r,
    });
    if !mf_r.is_empty() {
        sections.push(Section {
            title: &mf_title,
            accent: Accent::Yellow,
            rows: &mf_r,
        });
    }
    sections.push(Section {
        title: "Signing Hash",
        accent: Accent::Blue,
        rows: &hr_r,
    });

    render_sections(&sections);
    Ok(())
}

/// Print the pretty sign output (legacy multi-table format).
pub fn print_pretty_sign_output(
    json: &TypedData,
    signing_hash: &B256,
    signature: &alloy::signers::Signature,
) -> eyre::Result<()> {
    let dm = super::domain_meta_rows(json);
    let dp = super::domain_param_rows(json);
    let pt = super::primary_type_rows(json)?;
    let mf = super::message_rows(json);
    let primary_type = &json.primary_type;
    let pt_title = format!("Primary Type ({primary_type})");
    let mf_title = format!("{primary_type} Message Fields");
    let type_hash = json.type_hash()?;
    let sr = vec![
        ("Signature".to_string(), format!("{signature}")),
        (
            "Signing Hash".to_string(),
            format!("0x{}", hex::encode(signing_hash)),
        ),
        (
            "Type Hash".to_string(),
            format!("0x{}", hex::encode(type_hash)),
        ),
    ];

    let dm_r = super::to_refs(&dm);
    let dp_r = super::to_refs(&dp);
    let pt_r = super::to_refs(&pt);
    let mf_r = super::to_refs(&mf);
    let sr_r = super::to_refs(&sr);

    super::print_title("EIP-712 Signature Result");

    let mut sections: Vec<Section<'_>> = vec![Section {
        title: "Domain (EIP712Domain)",
        accent: Accent::Cyan,
        rows: &dm_r,
    }];
    if !dp_r.is_empty() {
        sections.push(Section {
            title: "Domain Parameters",
            accent: Accent::Cyan,
            rows: &dp_r,
        });
    }
    sections.push(Section {
        title: &pt_title,
        accent: Accent::Magenta,
        rows: &pt_r,
    });
    if !mf_r.is_empty() {
        sections.push(Section {
            title: &mf_title,
            accent: Accent::Yellow,
            rows: &mf_r,
        });
    }
    sections.push(Section {
        title: "Signature",
        accent: Accent::Blue,
        rows: &sr_r,
    });

    render_sections(&sections);
    Ok(())
}

/// Print the pretty verify output (legacy multi-table format).
pub fn print_pretty_verify_output(
    json: &TypedData,
    address: &alloy::primitives::Address,
    signing_hash: &B256,
) -> eyre::Result<()> {
    let dm = super::domain_meta_rows(json);
    let dp = super::domain_param_rows(json);
    let pt = super::primary_type_rows(json)?;
    let mf = super::message_rows(json);
    let primary_type = &json.primary_type;
    let pt_title = format!("Primary Type ({primary_type})");
    let mf_title = format!("{primary_type} Message Fields");
    let vr = vec![
        ("Status".to_string(), "Verified ✓".to_string()),
        ("Recovered Address".to_string(), format!("{address}")),
        (
            "Signing Hash".to_string(),
            format!("0x{}", hex::encode(signing_hash)),
        ),
    ];

    let dm_r = super::to_refs(&dm);
    let dp_r = super::to_refs(&dp);
    let pt_r = super::to_refs(&pt);
    let mf_r = super::to_refs(&mf);
    let vr_r = super::to_refs(&vr);

    super::print_title("EIP-712 Verification Result");

    let mut sections: Vec<Section<'_>> = vec![Section {
        title: "Domain (EIP712Domain)",
        accent: Accent::Cyan,
        rows: &dm_r,
    }];
    if !dp_r.is_empty() {
        sections.push(Section {
            title: "Domain Parameters",
            accent: Accent::Cyan,
            rows: &dp_r,
        });
    }
    sections.push(Section {
        title: &pt_title,
        accent: Accent::Magenta,
        rows: &pt_r,
    });
    if !mf_r.is_empty() {
        sections.push(Section {
            title: &mf_title,
            accent: Accent::Yellow,
            rows: &mf_r,
        });
    }
    sections.push(Section {
        title: "Verification",
        accent: Accent::Blue,
        rows: &vr_r,
    });

    render_sections(&sections);
    Ok(())
}
