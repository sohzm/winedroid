use std::collections::BTreeMap;

use serde_json::Value;

use crate::apk_signature::hex_decode;
use crate::config::WinedroidConfig;
use crate::error::Result;
use crate::install::{self, InstalledPackageArtifact};
use crate::prefix::WinedroidPrefix;

// cite: frameworks-base PackageManager.java#GET_SIGNATURES, GET_META_DATA, GET_SIGNING_CERTIFICATES
pub const GET_SIGNATURES: i32 = 0x0000_0040;
pub const GET_META_DATA: i32 = 0x0000_0080;
pub const GET_SIGNING_CERTIFICATES: i32 = 0x0800_0000;

const FAKE_PACKAGE_SIGNATURE_PERMISSION: &str = "android.permission.FAKE_PACKAGE_SIGNATURE";
const FAKE_SIGNATURE_META_DATA: &str = "fake-signature";

#[derive(Debug, Clone)]
pub struct PackageRegistry {
    packages: BTreeMap<String, InstalledPackageArtifact>,
    signature_spoofing: bool,
}

impl PackageRegistry {
    pub(crate) fn load(prefix: &WinedroidPrefix, config: &WinedroidConfig) -> Result<Self> {
        let installed = install::list_installed(prefix)?;
        let mut packages = BTreeMap::new();
        for artifact in installed {
            packages.insert(artifact.package_name.clone(), artifact);
        }
        Ok(Self {
            packages,
            signature_spoofing: config.signature_spoofing(),
        })
    }

    pub fn from_artifacts(
        artifacts: Vec<InstalledPackageArtifact>,
        signature_spoofing: bool,
    ) -> Self {
        let mut packages = BTreeMap::new();
        for artifact in artifacts {
            packages.insert(artifact.package_name.clone(), artifact);
        }
        Self {
            packages,
            signature_spoofing,
        }
    }

    pub fn get_package(&self, package_name: &str) -> Option<&InstalledPackageArtifact> {
        self.packages.get(package_name)
    }

    pub fn all_packages(&self) -> Vec<&InstalledPackageArtifact> {
        self.packages.values().collect()
    }

    pub fn signing_certificates(&self, package_name: &str) -> Option<Vec<Vec<u8>>> {
        let artifact = self.packages.get(package_name)?;
        if self.signature_spoofing
            && artifact
                .declared_permissions
                .iter()
                .any(|permission| permission == FAKE_PACKAGE_SIGNATURE_PERMISSION)
        {
            if let Some(spoofed_hex) = self.meta_data_value(artifact, FAKE_SIGNATURE_META_DATA) {
                if let Ok(der) = hex_decode(&spoofed_hex) {
                    return Some(vec![der]);
                }
            }
        }

        let mut certs = Vec::with_capacity(artifact.signing_certificates.len());
        for hex_cert in &artifact.signing_certificates {
            if let Ok(der) = hex_decode(hex_cert) {
                certs.push(der);
            }
        }
        Some(certs)
    }

    pub fn application_meta_data(
        &self,
        package_name: &str,
    ) -> Vec<(String, Option<String>, Option<u32>)> {
        let Some(artifact) = self.packages.get(package_name) else {
            return Vec::new();
        };
        artifact
            .application_meta_data_json
            .iter()
            .filter_map(|raw| serde_json::from_str::<Value>(raw).ok())
            .filter_map(|val| {
                let name = val.get("name")?.as_str()?.to_owned();
                let value = val.get("value").and_then(|v| v.as_str()).map(str::to_owned);
                let resource = val
                    .get("resource")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32);
                Some((name, value, resource))
            })
            .collect()
    }

    pub fn resolve_service(
        &self,
        action: Option<&str>,
        package_name: Option<&str>,
        component_class: Option<&str>,
    ) -> Option<(String, String)> {
        // cite: frameworks-base ApplicationPackageManager.java#resolveService
        let candidates: Vec<&InstalledPackageArtifact> = match package_name {
            Some(pkg) => self.packages.get(pkg).into_iter().collect(),
            None => self.packages.values().collect(),
        };

        for artifact in candidates {
            for raw in &artifact.services_json {
                let Ok(service) = serde_json::from_str::<Value>(raw) else {
                    continue;
                };
                let Some(name) = service.get("name").and_then(|v| v.as_str()) else {
                    continue;
                };
                if let Some(target_class) = component_class {
                    if name == target_class {
                        return Some((artifact.package_name.clone(), name.to_owned()));
                    }
                    continue;
                }
                if let Some(target_action) = action {
                    let matches_action = service
                        .get("intent_filters")
                        .and_then(|v| v.as_array())
                        .is_some_and(|filters| {
                            filters.iter().any(|filter| {
                                filter
                                    .get("actions")
                                    .and_then(|v| v.as_array())
                                    .is_some_and(|actions| {
                                        actions.iter().any(|a| a.as_str() == Some(target_action))
                                    })
                            })
                        });
                    if matches_action {
                        return Some((artifact.package_name.clone(), name.to_owned()));
                    }
                }
            }
        }
        None
    }

    pub fn resolve_content_provider(&self, authority: &str) -> Option<(String, String)> {
        // cite: frameworks-base ApplicationPackageManager.java#resolveContentProvider
        for artifact in self.packages.values() {
            for raw in &artifact.providers_json {
                let Ok(provider) = serde_json::from_str::<Value>(raw) else {
                    continue;
                };
                let Some(name) = provider.get("name").and_then(|v| v.as_str()) else {
                    continue;
                };
                let matches = provider
                    .get("authorities")
                    .and_then(|v| v.as_array())
                    .is_some_and(|auths| auths.iter().any(|a| a.as_str() == Some(authority)));
                if matches {
                    return Some((artifact.package_name.clone(), name.to_owned()));
                }
            }
        }
        None
    }

    pub fn query_intent_activities(&self, package_name: Option<&str>) -> Vec<(String, String)> {
        // cite: frameworks-base ApplicationPackageManager.java#queryIntentActivities
        let candidates: Vec<&InstalledPackageArtifact> = match package_name {
            Some(pkg) => self.packages.get(pkg).into_iter().collect(),
            None => self.packages.values().collect(),
        };
        let mut results = Vec::new();
        for artifact in candidates {
            for activity in &artifact.launchable_activities {
                results.push((artifact.package_name.clone(), activity.clone()));
            }
        }
        results
    }

    fn meta_data_value(&self, artifact: &InstalledPackageArtifact, key: &str) -> Option<String> {
        artifact
            .application_meta_data_json
            .iter()
            .filter_map(|raw| serde_json::from_str::<Value>(raw).ok())
            .find(|val| val.get("name").and_then(|v| v.as_str()) == Some(key))
            .and_then(|val| val.get("value").and_then(|v| v.as_str()).map(str::to_owned))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_artifact(
        package_name: &str,
        cert_hex: &str,
        permissions: Vec<&str>,
        meta_data: Vec<(&str, &str)>,
    ) -> InstalledPackageArtifact {
        InstalledPackageArtifact {
            version_name: Some("1.0".to_owned()),
            version_code: Some("1".to_owned()),
            package_name: package_name.to_owned(),
            launch_activity: None,
            launchable_activities: Vec::new(),
            split_apks: Vec::new(),
            shared_user_id: None,
            signing_certificates: vec![cert_hex.to_owned()],
            declared_permissions: permissions.into_iter().map(str::to_owned).collect(),
            services_json: vec![serde_json::json!({
                "name": format!("{package_name}.GmsService"),
                "intent_filters": [{"actions": ["com.google.android.gms.BIND_SERVICE"]}]
            })
            .to_string()],
            receivers_json: Vec::new(),
            providers_json: vec![serde_json::json!({
                "name": format!("{package_name}.GservicesProvider"),
                "authorities": ["com.google.android.gsf.gservices"]
            })
            .to_string()],
            uses_libraries_json: Vec::new(),
            application_meta_data_json: meta_data
                .into_iter()
                .map(|(k, v)| serde_json::json!({"name": k, "value": v}).to_string())
                .collect(),
            source_apk: PathBuf::from("/tmp/base.apk"),
            source_size: 100,
            source_modified_unix_millis: 1,
            entry_count: 1,
            dex_files: Vec::new(),
            dex_metadata: Vec::new(),
            resource_table_summary: None,
            resource_files: Vec::new(),
            native_abi_names: Vec::new(),
            selected_guest_abi: None,
            native_libraries: Vec::new(),
            android_system_libraries: Vec::new(),
            missing_native_libraries: Vec::new(),
        }
    }

    #[test]
    fn signature_spoofing_requires_opt_in_and_declared_permission() {
        let real_hex = "aabbccdd";
        let spoof_hex = "11223344";
        let artifact = sample_artifact(
            "com.google.android.gms",
            real_hex,
            vec![FAKE_PACKAGE_SIGNATURE_PERMISSION],
            vec![(FAKE_SIGNATURE_META_DATA, spoof_hex)],
        );

        let strict_registry = PackageRegistry::from_artifacts(vec![artifact.clone()], false);
        assert_eq!(
            strict_registry.signing_certificates("com.google.android.gms"),
            Some(vec![vec![0xaa, 0xbb, 0xcc, 0xdd]])
        );

        let spoof_registry = PackageRegistry::from_artifacts(vec![artifact], true);
        assert_eq!(
            spoof_registry.signing_certificates("com.google.android.gms"),
            Some(vec![vec![0x11, 0x22, 0x33, 0x44]])
        );
    }

    #[test]
    fn resolves_cross_package_service_and_content_provider() {
        let artifact = sample_artifact("com.google.android.gms", "aa", Vec::new(), Vec::new());
        let registry = PackageRegistry::from_artifacts(vec![artifact], false);

        assert_eq!(
            registry.resolve_service(Some("com.google.android.gms.BIND_SERVICE"), None, None),
            Some((
                "com.google.android.gms".to_owned(),
                "com.google.android.gms.GmsService".to_owned()
            ))
        );
        assert_eq!(
            registry.resolve_content_provider("com.google.android.gsf.gservices"),
            Some((
                "com.google.android.gms".to_owned(),
                "com.google.android.gms.GservicesProvider".to_owned()
            ))
        );
        assert!(registry.get_package("com.missing.pkg").is_none());
    }
}
