mod table;

pub use table::{Accent, Section, render_dashboard};

use alloy::{
    dyn_abi::TypedData,
    primitives::{B256, hex},
};
use colored::Colorize;

type OwnedRows = Vec<(String, String)>;

fn to_refs(rows: &OwnedRows) -> Vec<(&str, &str)> {
    rows.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect()
}

fn print_title(title: &str) {
    println!();
    println!("{}", format!(" {title} ").bold().white().on_bright_blue());
    println!();
}

fn domain_meta_rows(json: &TypedData) -> OwnedRows {
    let domain = &json.domain;
    vec![
        ("encodeType".to_string(), domain.encode_type()),
        (
            "separator".to_string(),
            format!("0x{}", hex::encode(domain.separator())),
        ),
        (
            "typeHash".to_string(),
            format!("0x{}", hex::encode(domain.type_hash())),
        ),
    ]
}

fn domain_param_rows(json: &TypedData) -> OwnedRows {
    let domain = &json.domain;
    let mut rows = OwnedRows::new();
    if let Some(name) = &domain.name {
        rows.push(("name".to_string(), name.as_ref().to_string()));
    }
    if let Some(version) = &domain.version {
        rows.push(("version".to_string(), version.as_ref().to_string()));
    }
    if let Some(chain_id) = &domain.chain_id {
        rows.push(("chainId".to_string(), format!("{chain_id}")));
    }
    if let Some(verifying_contract) = &domain.verifying_contract {
        rows.push((
            "verifyingContract".to_string(),
            format!("{verifying_contract}"),
        ));
    }
    if let Some(salt) = &domain.salt {
        rows.push(("salt".to_string(), format!("0x{}", hex::encode(salt))));
    }
    rows
}

fn primary_type_rows(json: &TypedData) -> eyre::Result<OwnedRows> {
    let primary_type = &json.primary_type;
    let encode_type = json.resolver.encode_type(primary_type)?;
    let type_hash = json.type_hash()?;
    Ok(vec![
        ("encodeType".to_string(), encode_type),
        (
            "typeHash".to_string(),
            format!("0x{}", hex::encode(type_hash)),
        ),
    ])
}

fn message_rows(json: &TypedData) -> OwnedRows {
    let mut rows = OwnedRows::new();
    if let serde_json::Value::Object(map) = &json.message {
        for (key, value) in map {
            let display = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            rows.push((key.clone(), display));
        }
    }
    rows
}

/// Print the pretty hash output using the new dashboard table format.
pub fn print_pretty_hash_output(json: &TypedData, signing_hash: &B256) -> eyre::Result<()> {
    let dm = domain_meta_rows(json);
    let dp = domain_param_rows(json);
    let pt = primary_type_rows(json)?;
    let mf = message_rows(json);
    let primary_type = &json.primary_type;
    let pt_title = format!("Primary Type ({primary_type})");
    let mf_title = format!("{primary_type} Message Fields");
    let hr: OwnedRows = vec![(
        "Signing Hash".to_string(),
        format!("0x{}", hex::encode(signing_hash)),
    )];

    let dm_r = to_refs(&dm);
    let dp_r = to_refs(&dp);
    let pt_r = to_refs(&pt);
    let mf_r = to_refs(&mf);
    let hr_r = to_refs(&hr);

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

    print_title("EIP-712 Hash Result");
    println!("{}", render_dashboard(&sections, 100, 18));
    println!();

    Ok(())
}

/// Print the pretty sign output using the new dashboard table format.
pub fn print_pretty_sign_output(
    json: &TypedData,
    signing_hash: &B256,
    signature: &alloy::signers::Signature,
) -> eyre::Result<()> {
    let dm = domain_meta_rows(json);
    let dp = domain_param_rows(json);
    let pt = primary_type_rows(json)?;
    let mf = message_rows(json);
    let primary_type = &json.primary_type;
    let pt_title = format!("Primary Type ({primary_type})");
    let mf_title = format!("{primary_type} Message Fields");
    let type_hash = json.type_hash()?;
    let sr: OwnedRows = vec![
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

    let dm_r = to_refs(&dm);
    let dp_r = to_refs(&dp);
    let pt_r = to_refs(&pt);
    let mf_r = to_refs(&mf);
    let sr_r = to_refs(&sr);

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

    print_title("EIP-712 Signature Result");
    println!("{}", render_dashboard(&sections, 160, 18));
    println!();

    Ok(())
}

/// Print the pretty verify output using the new dashboard table format.
pub fn print_pretty_verify_output(
    json: &TypedData,
    address: &alloy::primitives::Address,
    signing_hash: &B256,
) -> eyre::Result<()> {
    let dm = domain_meta_rows(json);
    let dp = domain_param_rows(json);
    let pt = primary_type_rows(json)?;
    let mf = message_rows(json);
    let primary_type = &json.primary_type;
    let pt_title = format!("Primary Type ({primary_type})");
    let mf_title = format!("{primary_type} Message Fields");
    let vr: OwnedRows = vec![
        ("Status".to_string(), "Verified ✓".to_string()),
        ("Recovered Address".to_string(), format!("{address}")),
        (
            "Signing Hash".to_string(),
            format!("0x{}", hex::encode(signing_hash)),
        ),
    ];

    let dm_r = to_refs(&dm);
    let dp_r = to_refs(&dp);
    let pt_r = to_refs(&pt);
    let mf_r = to_refs(&mf);
    let vr_r = to_refs(&vr);

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

    print_title("EIP-712 Verification Result");
    println!("{}", render_dashboard(&sections, 100, 18));
    println!();

    Ok(())
}
