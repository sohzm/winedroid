use std::fmt;
use std::path::{Path, PathBuf};

use crate::abi::GuestAbi;
use crate::apk::{ApkArchive, ApkEntry};
use crate::dex::{self, DexSummary};
use crate::elf::{self, ElfSummary};
use crate::error::{Result, WinedroidError};
use crate::manifest::{self, AndroidManifest};
use crate::native_deps::NativeDependencyPlan;
use crate::native_runtime::NativeRuntimePlan;
use crate::resource_table::{self, ResourceFile, ResourceTableSummary};
use crate::trace;

#[derive(Debug, Clone)]
pub struct InspectionReport {
    path: PathBuf,
    manifest: AndroidManifest,
    dex_files: Vec<DexFileSummary>,
    native_libraries: Vec<NativeLibrary>,
    resource_table: Option<ResourceTableInspection>,
    signing_certificates: Vec<Vec<u8>>,
    entry_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DexFileSummary {
    name: String,
    compressed_size: u64,
    uncompressed_size: u64,
    compression_method: String,
    dex: DexSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DexFileMetadata {
    pub name: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub compression_method: String,
    pub version: String,
    pub checksum: u32,
    pub declared_file_size: u32,
    pub string_count: u32,
    pub type_count: u32,
    pub method_count: u32,
    pub class_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceTableInspection {
    summary: ResourceTableSummary,
    files: Vec<ResourceFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeLibrary {
    abi: String,
    name: String,
    compressed_size: u64,
    uncompressed_size: u64,
    elf: ElfSummary,
}

impl NativeLibrary {
    pub fn abi(&self) -> &str {
        &self.abi
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn apk_path(&self) -> String {
        format!("lib/{}/{}", self.abi, self.name)
    }

    pub fn needed_libraries(&self) -> &[String] {
        &self.elf.needed_libraries
    }

    #[cfg(test)]
    pub(crate) fn for_test(abi: &str, name: &str, elf: ElfSummary) -> Self {
        Self {
            abi: abi.to_owned(),
            name: name.to_owned(),
            compressed_size: 0,
            uncompressed_size: 0,
            elf,
        }
    }
}

pub fn inspect_apk(path: &Path) -> Result<InspectionReport> {
    let mut apk = {
        let _span = trace::span("inspect", "open APK archive");
        ApkArchive::open(path)?
    };
    let entries = {
        let _span = trace::span("inspect", "read APK entries");
        apk.entries()?
    };
    trace::event("inspect", format!("APK entries: {}", entries.len()));
    let manifest_data = {
        let _span = trace::span("inspect", "read AndroidManifest.xml");
        apk.read_required_file("AndroidManifest.xml")?
    };
    let manifest = {
        let _span = trace::span("inspect", "parse AndroidManifest.xml");
        manifest::parse_manifest(&manifest_data)?
    };
    let resource_table = {
        let _span = trace::span("inspect", "inspect resources.arsc");
        read_resource_table(&mut apk)?
    };
    let dex_files = {
        let _span = trace::span("inspect", "inspect DEX files");
        read_dex_files(&mut apk, &entries)?
    };
    let native_libraries = {
        let _span = trace::span("inspect", "inspect native libraries");
        read_native_libraries(&mut apk, &entries)?
    };
    let signing_certificates = {
        let _span = trace::span("inspect", "inspect APK signatures");
        crate::apk_signature::extract_signing_certificates(path)?
    };

    Ok(InspectionReport {
        path: path.to_owned(),
        manifest,
        dex_files,
        native_libraries,
        resource_table,
        signing_certificates,
        entry_count: entries.len(),
    })
}

fn read_resource_table(apk: &mut ApkArchive) -> Result<Option<ResourceTableInspection>> {
    let Some(data) = apk.read_optional_file("resources.arsc")? else {
        return Ok(None);
    };

    let summary = resource_table::summarize_resource_table(&data)?;
    let files = resource_table::resource_file_index(&data)?;

    Ok(Some(ResourceTableInspection { summary, files }))
}

fn read_dex_files(apk: &mut ApkArchive, entries: &[ApkEntry]) -> Result<Vec<DexFileSummary>> {
    let mut files = Vec::new();

    for entry in entries {
        if !dex::is_dex_file(&entry.name) {
            continue;
        }

        let data = {
            let _span = trace::stat_span("perf", "inspect read dex entry");
            apk.read_required_file(&entry.name)?
        };
        let dex = {
            let _span = trace::stat_span("perf", "inspect parse dex summary");
            dex::parse_dex(&data).map_err(|error| {
                WinedroidError::with_source(format!("failed to parse DEX `{}`", entry.name), error)
            })?
        };

        files.push(dex_file_summary(entry, dex));
    }

    files.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(files)
}

fn read_native_libraries(apk: &mut ApkArchive, entries: &[ApkEntry]) -> Result<Vec<NativeLibrary>> {
    let mut libraries = Vec::new();

    for entry in entries {
        let Some(native_library_path) = native_library_path(entry) else {
            continue;
        };

        let data = {
            let _span = trace::stat_span("perf", "inspect read native library");
            apk.read_required_file(&entry.name)?
        };
        let elf = {
            let _span = trace::stat_span("perf", "inspect parse native ELF");
            elf::parse_elf(&data).map_err(|error| {
                WinedroidError::with_source(
                    format!("failed to parse native ELF library `{}`", entry.name),
                    error,
                )
            })?
        };

        libraries.push(native_library(entry, native_library_path, elf));
    }

    libraries.sort_by(|left, right| {
        left.abi
            .cmp(&right.abi)
            .then_with(|| left.name.cmp(&right.name))
    });

    Ok(libraries)
}

fn dex_file_summary(entry: &ApkEntry, dex: DexSummary) -> DexFileSummary {
    DexFileSummary {
        name: entry.name.clone(),
        compressed_size: entry.compressed_size,
        uncompressed_size: entry.uncompressed_size,
        compression_method: entry.compression_method.clone(),
        dex,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeLibraryPath {
    abi: String,
    name: String,
}

fn native_library_path(entry: &ApkEntry) -> Option<NativeLibraryPath> {
    let mut parts = entry.name.split('/');

    let directory = parts.next()?;
    let abi = parts.next()?;
    let name = parts.next()?;

    if directory != "lib" || parts.next().is_some() || !name.ends_with(".so") {
        return None;
    }

    Some(NativeLibraryPath {
        abi: abi.to_owned(),
        name: name.to_owned(),
    })
}

fn native_library(entry: &ApkEntry, path: NativeLibraryPath, elf: ElfSummary) -> NativeLibrary {
    NativeLibrary {
        abi: path.abi,
        name: path.name,
        compressed_size: entry.compressed_size,
        uncompressed_size: entry.uncompressed_size,
        elf,
    }
}

impl fmt::Display for InspectionReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let launchable_activities = self.manifest.launchable_activities();
        let effective_min_sdk_version = self
            .manifest
            .effective_min_sdk_version()
            .map_err(|_| fmt::Error)?;
        let effective_target_sdk_version = self
            .manifest
            .effective_target_sdk_version()
            .map_err(|_| fmt::Error)?;
        let requests_legacy_external_storage = self
            .manifest
            .requests_legacy_external_storage()
            .map_err(|_| fmt::Error)?;
        let declared_legacy_external_storage = self
            .manifest
            .request_legacy_external_storage
            .map(|value| value.to_string());
        let declared_default_to_device_protected_storage = self
            .manifest
            .default_to_device_protected_storage
            .map(|value| value.to_string());
        let declared_direct_boot_aware = self
            .manifest
            .direct_boot_aware
            .map(|value| value.to_string());

        writeln!(formatter, "APK: {}", self.path.display())?;
        writeln!(formatter, "Entries: {}", self.entry_count)?;
        writeln!(formatter)?;

        writeln!(formatter, "Package")?;
        write_optional(formatter, "name", self.manifest.package_name.as_deref())?;
        write_optional(
            formatter,
            "application",
            self.manifest.application_name.as_deref(),
        )?;
        write_optional(
            formatter,
            "versionCode",
            self.manifest.version_code.as_deref(),
        )?;
        write_optional(
            formatter,
            "versionName",
            self.manifest.version_name.as_deref(),
        )?;
        write_optional(
            formatter,
            "compileSdkVersion",
            self.manifest.compile_sdk_version.as_deref(),
        )?;
        write_optional(
            formatter,
            "minSdkVersion",
            self.manifest.min_sdk_version.as_deref(),
        )?;
        writeln!(
            formatter,
            "  effectiveMinSdkVersion: {effective_min_sdk_version}"
        )?;
        write_optional(
            formatter,
            "targetSdkVersion",
            self.manifest.target_sdk_version.as_deref(),
        )?;
        writeln!(
            formatter,
            "  effectiveTargetSdkVersion: {effective_target_sdk_version}"
        )?;
        write_optional(
            formatter,
            "requestLegacyExternalStorage",
            declared_legacy_external_storage.as_deref(),
        )?;
        writeln!(
            formatter,
            "  effectiveRequestLegacyExternalStorage: {requests_legacy_external_storage}"
        )?;
        write_optional(
            formatter,
            "defaultToDeviceProtectedStorage",
            declared_default_to_device_protected_storage.as_deref(),
        )?;
        writeln!(
            formatter,
            "  effectiveDefaultToDeviceProtectedStorage: {}",
            self.manifest.defaults_to_device_protected_storage(false)
        )?;
        write_optional(
            formatter,
            "directBootAware",
            declared_direct_boot_aware.as_deref(),
        )?;
        writeln!(
            formatter,
            "  effectiveDirectBootAware: {}",
            self.manifest.is_direct_boot_aware(false)
        )?;
        write_optional(
            formatter,
            "sharedUserId",
            self.manifest.shared_user_id.as_deref(),
        )?;
        write_optional(formatter, "split", self.manifest.split_name.as_deref())?;
        writeln!(formatter)?;

        writeln!(formatter, "Signing certificates")?;
        if self.signing_certificates.is_empty() {
            writeln!(formatter, "  none (unsigned)")?;
        } else {
            for cert in &self.signing_certificates {
                writeln!(
                    formatter,
                    "  SHA-256: {}",
                    crate::apk_signature::sha256_fingerprint_colon_hex(cert)
                )?;
            }
        }
        writeln!(formatter)?;

        writeln!(formatter, "Launchable activities")?;
        write_string_list(formatter, &launchable_activities)?;
        writeln!(formatter)?;

        let launch_classes: Vec<String> = launchable_activities
            .iter()
            .map(|component| {
                self.resolve_launch_component(component)
                    .unwrap_or_else(|_| component.clone())
            })
            .collect();

        writeln!(formatter, "Launch activity classes")?;
        write_launch_activity_classes(formatter, &launch_classes, &self.dex_files)?;
        writeln!(formatter)?;

        writeln!(
            formatter,
            "Components: {} services, {} receivers, {} providers, {} uses-library",
            self.manifest.services.len(),
            self.manifest.receivers.len(),
            self.manifest.providers.len(),
            self.manifest.uses_libraries.len()
        )?;
        writeln!(formatter)?;

        writeln!(formatter, "Permissions")?;
        let permission_declarations = self
            .manifest
            .permissions
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        write_string_list(formatter, &permission_declarations)?;
        writeln!(formatter)?;

        writeln!(formatter, "DEX files")?;
        write_dex_file_summaries(formatter, &self.dex_files)?;
        writeln!(formatter)?;

        writeln!(formatter, "ABI support")?;
        let native_runtime_plan = self.native_runtime_plan();
        write_native_runtime_plan(formatter, &native_runtime_plan)?;
        writeln!(formatter)?;

        writeln!(formatter, "Native dependency plan")?;
        write_native_dependency_plan(
            formatter,
            &self.native_dependency_plan(&native_runtime_plan),
        )?;
        writeln!(formatter)?;

        writeln!(formatter, "Native libraries")?;
        write_native_libraries(formatter, &self.native_libraries)?;
        writeln!(formatter)?;

        writeln!(formatter, "Resources")?;
        match &self.resource_table {
            Some(table) => {
                writeln!(
                    formatter,
                    "  resources.arsc: {} bytes, {} package(s)",
                    table.summary.byte_size, table.summary.package_count
                )?;
                writeln!(formatter, "  file resources: {}", table.files.len())?;
            }
            None => {
                writeln!(formatter, "  none")?;
            }
        }

        Ok(())
    }
}

impl InspectionReport {
    pub(crate) fn version_name(&self) -> Option<&str> {
        self.manifest.version_name.as_deref()
    }

    pub(crate) fn version_code(&self) -> Option<&str> {
        self.manifest.version_code.as_deref()
    }

    pub fn package_name(&self) -> Option<&str> {
        self.manifest.package_name.as_deref()
    }

    pub fn manifest(&self) -> &AndroidManifest {
        &self.manifest
    }

    pub fn signing_certificates(&self) -> &[Vec<u8>] {
        &self.signing_certificates
    }

    pub fn launchable_activities(&self) -> Vec<String> {
        self.manifest.launchable_activities()
    }

    pub fn resolved_launch_activities(&self) -> Result<Vec<String>> {
        self.launchable_activities()
            .into_iter()
            .map(|component| self.resolve_launch_component(&component))
            .collect()
    }

    pub fn primary_launch_activity(&self) -> Result<Option<String>> {
        let activities = self.launchable_activities();
        if activities.len() == 1 {
            self.resolve_launch_component(&activities[0]).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn single_launch_activity(&self) -> Result<String> {
        let activities = self.launchable_activities();

        if activities.is_empty() {
            return Err(WinedroidError::new(
                "manifest does not declare a launchable activity",
            ));
        }

        if activities.len() > 1 {
            return Err(WinedroidError::new(format!(
                "manifest declares multiple launchable activities: {}",
                activities.join(", ")
            )));
        }

        self.resolve_launch_component(&activities[0])
    }

    fn resolve_launch_component(&self, component: &str) -> Result<String> {
        let Some(activity) = self.manifest.activity(component) else {
            return Ok(component.to_owned());
        };

        if !activity.is_alias {
            return Ok(component.to_owned());
        }

        // cite: frameworks-base PackageParser.java#parseActivityAlias
        let Some(target) = activity.target_activity.as_deref() else {
            return Err(WinedroidError::new(format!(
                "<activity-alias> {component} does not specify android:targetActivity"
            )));
        };

        if self.manifest.activity(target).is_none() {
            return Err(WinedroidError::new(format!(
                "<activity-alias> target activity {target} not found in manifest"
            )));
        }

        // cite: frameworks-base ActivityThread.java#performLaunchActivity
        Ok(target.to_owned())
    }

    pub fn entry_count(&self) -> usize {
        self.entry_count
    }

    pub fn dex_file_names(&self) -> Vec<String> {
        self.dex_files
            .iter()
            .map(|file| file.name.clone())
            .collect()
    }

    pub fn dex_metadata(&self) -> Vec<DexFileMetadata> {
        self.dex_files
            .iter()
            .map(|file| DexFileMetadata {
                name: file.name.clone(),
                compressed_size: file.compressed_size,
                uncompressed_size: file.uncompressed_size,
                compression_method: file.compression_method.clone(),
                version: file.dex.version.clone(),
                checksum: file.dex.checksum,
                declared_file_size: file.dex.declared_file_size,
                string_count: file.dex.string_count,
                type_count: file.dex.type_count,
                method_count: file.dex.method_count,
                class_count: file.dex.class_count,
            })
            .collect()
    }

    pub fn resource_table_summary(&self) -> Option<ResourceTableSummary> {
        self.resource_table
            .as_ref()
            .map(|table| table.summary.clone())
    }

    pub fn resource_files(&self) -> Vec<ResourceFile> {
        self.resource_table
            .as_ref()
            .map(|table| table.files.clone())
            .unwrap_or_default()
    }

    pub fn native_runtime_plan(&self) -> NativeRuntimePlan {
        NativeRuntimePlan::detect(&self.native_abi_names())
    }

    pub fn native_dependency_plan(
        &self,
        native_runtime_plan: &NativeRuntimePlan,
    ) -> NativeDependencyPlan {
        let selected_libraries = match native_runtime_plan.selected_guest_abi {
            Some(abi) => self.native_libraries_for_abi(abi),
            None => Vec::new(),
        };

        NativeDependencyPlan::build(native_runtime_plan.selected_guest_abi, &selected_libraries)
    }

    pub fn native_libraries_for_abi(&self, abi: GuestAbi) -> Vec<NativeLibrary> {
        self.native_libraries
            .iter()
            .filter(|library| library.abi == abi.as_str())
            .cloned()
            .collect()
    }

    pub fn native_abi_names(&self) -> Vec<String> {
        let mut names = self
            .native_libraries
            .iter()
            .map(|library| library.abi.clone())
            .collect::<Vec<_>>();

        names.sort();
        names.dedup();
        names
    }

    #[cfg(test)]
    pub(crate) fn for_test(manifest: AndroidManifest) -> Self {
        Self {
            path: PathBuf::from("test.apk"),
            manifest,
            dex_files: Vec::new(),
            native_libraries: Vec::new(),
            resource_table: None,
            signing_certificates: Vec::new(),
            entry_count: 0,
        }
    }
}

fn write_optional(
    formatter: &mut fmt::Formatter<'_>,
    label: &str,
    value: Option<&str>,
) -> fmt::Result {
    match value {
        Some(value) if !value.is_empty() => writeln!(formatter, "  {label}: {value}"),
        _ => writeln!(formatter, "  {label}: not declared"),
    }
}

fn write_string_list(formatter: &mut fmt::Formatter<'_>, values: &[String]) -> fmt::Result {
    if values.is_empty() {
        return writeln!(formatter, "  none");
    }

    for value in values {
        writeln!(formatter, "  {value}")?;
    }

    Ok(())
}

fn write_launch_activity_classes(
    formatter: &mut fmt::Formatter<'_>,
    activities: &[String],
    dex_files: &[DexFileSummary],
) -> fmt::Result {
    if activities.is_empty() {
        return writeln!(formatter, "  none");
    }

    for activity in activities {
        let descriptor = activity_descriptor(activity);
        let locations = dex_files_containing_descriptor(dex_files, &descriptor);

        if locations.is_empty() {
            writeln!(formatter, "  {activity}: not found")?;
        } else {
            writeln!(formatter, "  {activity}: {}", locations.join(", "))?;
        }
    }

    Ok(())
}

fn dex_files_containing_descriptor(dex_files: &[DexFileSummary], descriptor: &str) -> Vec<String> {
    dex_files
        .iter()
        .filter(|file| {
            file.dex
                .class_descriptors
                .iter()
                .any(|class_descriptor| class_descriptor == descriptor)
        })
        .map(|file| file.name.clone())
        .collect()
}

fn activity_descriptor(activity: &str) -> String {
    format!("L{};", activity.replace('.', "/"))
}

fn write_dex_file_summaries(
    formatter: &mut fmt::Formatter<'_>,
    files: &[DexFileSummary],
) -> fmt::Result {
    if files.is_empty() {
        return writeln!(formatter, "  none");
    }

    for file in files {
        writeln!(
            formatter,
            "  {}: dex {}, checksum 0x{:08x}, {} classes, {} strings, {} types, {} methods, declared {} bytes, {} bytes compressed, {} bytes uncompressed, {}",
            file.name,
            file.dex.version,
            file.dex.checksum,
            file.dex.class_count,
            file.dex.string_count,
            file.dex.type_count,
            file.dex.method_count,
            file.dex.declared_file_size,
            file.compressed_size,
            file.uncompressed_size,
            file.compression_method
        )?;
    }

    Ok(())
}

fn write_native_runtime_plan(
    formatter: &mut fmt::Formatter<'_>,
    plan: &NativeRuntimePlan,
) -> fmt::Result {
    writeln!(formatter, "  host: {}", plan.host)?;
    writeln!(
        formatter,
        "  APK native ABIs: {}",
        plan.inventory.known_display()
    )?;
    writeln!(
        formatter,
        "  unknown ABI dirs: {}",
        plan.inventory.unknown_display()
    )?;
    writeln!(
        formatter,
        "  selected guest ABI: {}",
        plan.selected_guest_abi_display()
    )?;
    writeln!(formatter, "  native backend branch: {}", plan.branch)?;
    writeln!(
        formatter,
        "  native backend status: {}",
        native_backend_status(plan)
    )?;

    Ok(())
}

fn native_backend_status(plan: &NativeRuntimePlan) -> String {
    plan.execution_strategy().status()
}

fn write_native_dependency_plan(
    formatter: &mut fmt::Formatter<'_>,
    plan: &NativeDependencyPlan,
) -> fmt::Result {
    writeln!(
        formatter,
        "  selected guest ABI: {}",
        plan.selected_guest_abi_display()
    )?;
    writeln!(
        formatter,
        "  app libraries: {}",
        plan.app_libraries_display()
    )?;
    writeln!(
        formatter,
        "  Android system libraries: {}",
        plan.android_system_libraries_display()
    )?;
    writeln!(
        formatter,
        "  unresolved libraries: {}",
        plan.missing_libraries_display()
    )?;

    Ok(())
}

fn write_native_libraries(
    formatter: &mut fmt::Formatter<'_>,
    libraries: &[NativeLibrary],
) -> fmt::Result {
    if libraries.is_empty() {
        return writeln!(formatter, "  none");
    }

    for library in libraries {
        let needed = needed_libraries_display(&library.elf.needed_libraries);

        writeln!(
            formatter,
            "  {}/{}: {}, {}, {}, needs [{}], {} bytes compressed, {} bytes uncompressed",
            library.abi,
            library.name,
            library.elf.class,
            library.elf.endian,
            library.elf.machine,
            needed,
            library.compressed_size,
            library.uncompressed_size
        )?;
    }

    Ok(())
}

fn needed_libraries_display(libraries: &[String]) -> String {
    if libraries.is_empty() {
        return "none".to_owned();
    }

    libraries.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_primary_and_secondary_dex_files() {
        assert!(dex::is_dex_file("classes.dex"));
        assert!(dex::is_dex_file("classes2.dex"));
        assert!(dex::is_dex_file("classes12.dex"));
        assert!(!dex::is_dex_file("classes.dex.backup"));
        assert!(!dex::is_dex_file("assets/classes.dex"));
        assert!(!dex::is_dex_file("classesx.dex"));
    }

    #[test]
    fn reads_native_library_path() {
        let entry = ApkEntry {
            name: "lib/arm64-v8a/libmain.so".to_owned(),
            compressed_size: 10,
            uncompressed_size: 20,
            compression_method: "Deflated".to_owned(),
        };

        let path = native_library_path(&entry).expect("native library path should parse");
        let library = native_library(&entry, path, test_elf_summary());

        assert_eq!(library.abi, "arm64-v8a");
        assert_eq!(library.name, "libmain.so");
        assert_eq!(library.compressed_size, 10);
        assert_eq!(library.uncompressed_size, 20);
        assert_eq!(library.apk_path(), "lib/arm64-v8a/libmain.so");
    }

    #[test]
    fn collects_unique_native_abi_names() {
        let mut report = InspectionReport::for_test(AndroidManifest::default());
        report.native_libraries = vec![
            NativeLibrary {
                abi: "x86_64".to_owned(),
                name: "libone.so".to_owned(),
                compressed_size: 1,
                uncompressed_size: 1,
                elf: test_elf_summary(),
            },
            NativeLibrary {
                abi: "arm64-v8a".to_owned(),
                name: "libtwo.so".to_owned(),
                compressed_size: 1,
                uncompressed_size: 1,
                elf: test_elf_summary(),
            },
            NativeLibrary {
                abi: "x86_64".to_owned(),
                name: "libthree.so".to_owned(),
                compressed_size: 1,
                uncompressed_size: 1,
                elf: test_elf_summary(),
            },
        ];

        assert_eq!(
            report.native_abi_names(),
            vec!["arm64-v8a".to_owned(), "x86_64".to_owned()]
        );
    }

    #[test]
    fn builds_activity_descriptor() {
        assert_eq!(
            activity_descriptor("com.example.MainActivity"),
            "Lcom/example/MainActivity;"
        );
    }

    #[test]
    fn filters_native_libraries_by_guest_abi() {
        let mut report = InspectionReport::for_test(AndroidManifest::default());
        report.native_libraries = vec![
            NativeLibrary {
                abi: "arm64-v8a".to_owned(),
                name: "libarm.so".to_owned(),
                compressed_size: 1,
                uncompressed_size: 1,
                elf: test_elf_summary(),
            },
            NativeLibrary {
                abi: "x86_64".to_owned(),
                name: "libx86.so".to_owned(),
                compressed_size: 1,
                uncompressed_size: 1,
                elf: test_elf_summary(),
            },
        ];

        let libraries = report.native_libraries_for_abi(GuestAbi::Arm64V8a);

        assert_eq!(libraries.len(), 1);
        assert_eq!(libraries[0].name(), "libarm.so");
    }

    #[test]
    fn formats_effective_sdk_legacy_storage_and_permission_constraints() {
        // cite: frameworks-base PackageParser.java#DEFAULT_MIN_SDK_VERSION, DEFAULT_TARGET_SDK_VERSION, parseBaseApkCommon, parseBaseApplication, parseUsesPermission
        // cite: frameworks-base ApplicationInfo.java#PRIVATE_FLAG_DEFAULT_TO_DEVICE_PROTECTED_STORAGE, PRIVATE_FLAG_DIRECT_BOOT_AWARE
        let report = InspectionReport::for_test(AndroidManifest {
            min_sdk_version: Some("28".to_owned()),
            uses_sdk_declared: true,
            permissions: vec![crate::manifest::UsesPermission {
                name: "android.permission.READ_EXTERNAL_STORAGE".to_owned(),
                max_sdk_version: Some(32),
                required_feature: None,
                required_not_feature: None,
            }],
            ..AndroidManifest::default()
        });

        let output = report.to_string();

        assert!(output.contains("effectiveTargetSdkVersion: 28"));
        assert!(output.contains("effectiveMinSdkVersion: 28"));
        assert!(output.contains("requestLegacyExternalStorage: not declared"));
        assert!(output.contains("effectiveRequestLegacyExternalStorage: true"));
        assert!(output.contains("defaultToDeviceProtectedStorage: not declared"));
        assert!(output.contains("effectiveDefaultToDeviceProtectedStorage: false"));
        assert!(output.contains("directBootAware: not declared"));
        assert!(output.contains("effectiveDirectBootAware: false"));
        assert!(output.contains("android.permission.READ_EXTERNAL_STORAGE (maxSdkVersion=32)"));
    }

    fn test_elf_summary() -> ElfSummary {
        ElfSummary {
            class: crate::elf::ElfClass::Elf64,
            endian: crate::elf::ElfEndian::Little,
            machine: crate::elf::ElfMachine::Aarch64,
            needed_libraries: Vec::new(),
        }
    }
}
