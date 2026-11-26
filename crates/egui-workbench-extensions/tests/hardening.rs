//! Regression tests for manifest validation.

use egui_workbench_extensions::manifest::ApiRange;
use egui_workbench_extensions::{Capability, ExtensionManifest, ManifestError};

fn manifest(id: &str, api: ApiRange) -> ExtensionManifest {
    ExtensionManifest {
        id: id.to_owned(),
        version: "0.1.0".to_owned(),
        name: "x".to_owned(),
        api,
        entry: "x.wasm".to_owned(),
        checksum: None,
        signature: None,
        capabilities: Default::default(),
        author: None,
        description: None,
    }
}

#[test]
fn an_unparsable_api_bound_is_rejected() {
    let range = ApiRange {
        min: "abc".to_owned(),
        max: "9.9".to_owned(),
    };
    assert!(!range.allows("0.15"));
    assert!(ApiRange::line("0.15").allows("0.15"));
}

#[test]
fn ids_are_plain_reverse_dns_names() {
    let range = ApiRange::line(egui_workbench_extensions::HOST_API_VERSION);
    assert!(manifest("com.example.hello", range.clone()).validate().is_ok());
    for bad in ["", "..", ".hidden", "a b", "a/b", "a\nb", "é"] {
        assert!(
            matches!(manifest(bad, range.clone()).validate(), Err(ManifestError::Id(_))),
            "{bad:?}"
        );
    }
}

#[test]
fn an_empty_filesystem_prefix_is_not_a_capability() {
    assert!(Capability::parse("filesystem:read:").is_err());
    assert!(Capability::parse("filesystem:read:notes").is_ok());
}

#[test]
fn the_api_error_names_the_real_host_version() {
    let range = ApiRange {
        min: "9.0".to_owned(),
        max: "9.1".to_owned(),
    };
    let message = manifest("a.b", range).validate().unwrap_err().to_string();
    assert!(
        message.contains(egui_workbench_extensions::HOST_API_VERSION),
        "{message}"
    );
}
