use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::abi::GuestAbi;
use crate::apk::ApkArchive;
use crate::capability_fs::{Directory, MetadataKind};
use crate::dex;
use crate::error::{Result, WinedroidError};
use crate::external_storage;
use crate::inspect::{DexFileMetadata, InspectionReport};
use crate::native_deps::NativeDependencyPlan;
use crate::native_runtime::NativeRuntimePlan;
use crate::prefix::WinedroidPrefix;
use crate::resource_table::ResourceTableSummary;
use crate::trace;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledApp {
    pub package_name: String,
    pub launch_activity: Option<String>,
    pub app_dir: PathBuf,
    pub source_apk: PathBuf,
    pub package_artifact: PathBuf,
    pub data_dir: PathBuf,
    pub device_data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub files_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub native_dir: PathBuf,
    pub native_libraries: Vec<InstalledNativeLibrary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledNativeLibrary {
    pub abi: String,
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledPackageArtifact {
    pub version_name: Option<String>,
    pub version_code: Option<String>,
    pub package_name: String,
    pub launch_activity: Option<String>,
    pub launchable_activities: Vec<String>,
    pub split_apks: Vec<String>,
    pub shared_user_id: Option<String>,
    pub signing_certificates: Vec<String>,
    pub declared_permissions: Vec<String>,
    pub services_json: Vec<String>,
    pub receivers_json: Vec<String>,
    pub providers_json: Vec<String>,
    pub uses_libraries_json: Vec<String>,
    pub application_meta_data_json: Vec<String>,
    pub source_apk: PathBuf,
    pub source_size: u64,
    pub source_modified_unix_millis: u128,
    pub entry_count: usize,
    pub dex_files: Vec<String>,
    pub dex_metadata: Vec<DexFileMetadata>,
    pub resource_table_summary: Option<ResourceTableSummary>,
    pub resource_files: Vec<String>,
    pub native_abi_names: Vec<String>,
    pub selected_guest_abi: Option<String>,
    pub native_libraries: Vec<String>,
    pub android_system_libraries: Vec<String>,
    pub missing_native_libraries: Vec<String>,
}

pub fn install_apk(
    prefix: &WinedroidPrefix,
    apk_path: &Path,
    report: &InspectionReport,
) -> Result<InstalledApp> {
    install_apk_set(prefix, apk_path, &[], report)
}

pub fn install_apk_set(
    prefix: &WinedroidPrefix,
    apk_path: &Path,
    split_apk_paths: &[PathBuf],
    report: &InspectionReport,
) -> Result<InstalledApp> {
    let package_name = report
        .package_name()
        .ok_or_else(|| WinedroidError::new("manifest does not declare a package name"))?;
    let layout = AppLayout::new(prefix, package_name)?;
    {
        let _span = trace::span("install", "create app layout dirs");
        create_layout_dirs(&layout)?;
    }
    {
        let _span = trace::span("install", "provision external storage");
        external_storage::provision_shared_storage(prefix)?;
        external_storage::provision_app_external_storage(prefix, package_name)?;
    }
    {
        let _span = trace::span("install", "copy source APK");
        copy_source_apk(apk_path, &layout.source_apk)?;
    }
    let installed_splits = {
        let _span = trace::span("install", "install split APKs");
        install_split_apks(
            split_apk_paths,
            &layout,
            package_name,
            report.version_code(),
        )?
    };
    {
        let _span = trace::span("install", "extract runtime artifacts");
        install_runtime_artifacts(apk_path, &layout, report)?;
    }
    let mut native_libraries = {
        let _span = trace::span("install", "extract native libraries");
        install_native_libraries(apk_path, &layout, report)?
    };
    for split_path in split_apk_paths {
        let split_report = crate::inspect::inspect_apk(split_path)?;
        let mut split_native = install_native_libraries(split_path, &layout, &split_report)?;
        native_libraries.append(&mut split_native);
    }
    {
        let _span = trace::span("install", "write manifest summary");
        write_manifest_summary(&layout.manifest_summary, report)?;
    }
    let artifact = {
        let _span = trace::span("install", "write package artifact");
        let mut artifact =
            InstalledPackageArtifact::from_install(&layout.source_apk, report, &native_libraries)?;
        artifact.split_apks = installed_splits;
        write_package_artifact(&layout.package_artifact, &artifact)?;
        read_package_artifact(&layout.package_artifact)?
    };

    Ok(InstalledApp {
        package_name: package_name.to_owned(),
        launch_activity: artifact.launch_activity,
        app_dir: layout.app_dir,
        source_apk: layout.source_apk,
        package_artifact: layout.package_artifact,
        data_dir: layout.data_dir,
        device_data_dir: layout.device_data_dir,
        cache_dir: layout.cache_dir,
        files_dir: layout.files_dir,
        runtime_dir: layout.runtime_dir,
        native_dir: layout.native_dir,
        native_libraries,
    })
}

impl InstalledApp {
    pub fn load(prefix: &WinedroidPrefix, package_name: &str) -> Result<Self> {
        let layout = AppLayout::new(prefix, package_name)?;
        require_installed_regular_file(&layout.package_artifact, package_name)?;
        require_installed_regular_file(&layout.source_apk, package_name)?;
        prepare_persistent_dirs(&layout)?;
        let artifact = read_package_artifact(&layout.package_artifact)?;
        let native_libraries = installed_native_libraries(&layout.native_dir, &artifact);

        Ok(Self {
            package_name: artifact.package_name,
            launch_activity: artifact.launch_activity,
            app_dir: layout.app_dir,
            source_apk: layout.source_apk,
            package_artifact: layout.package_artifact,
            data_dir: layout.data_dir,
            device_data_dir: layout.device_data_dir,
            cache_dir: layout.cache_dir,
            files_dir: layout.files_dir,
            runtime_dir: layout.runtime_dir,
            native_dir: layout.native_dir,
            native_libraries,
        })
    }

    pub fn artifact(&self) -> Result<InstalledPackageArtifact> {
        read_package_artifact(&self.package_artifact)
    }
}

fn installed_native_libraries(
    native_dir: &Path,
    artifact: &InstalledPackageArtifact,
) -> Vec<InstalledNativeLibrary> {
    artifact
        .native_libraries
        .iter()
        .filter_map(|record| record.split_once('/'))
        .map(|(abi, name)| InstalledNativeLibrary {
            abi: abi.to_owned(),
            name: name.to_owned(),
            path: native_dir.join(abi).join(name),
        })
        .collect()
}

pub fn list_installed(prefix: &WinedroidPrefix) -> Result<Vec<InstalledPackageArtifact>> {
    let apps_root = prefix.apps_root();
    if !apps_root.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&apps_root).map_err(|error| {
        WinedroidError::with_source(
            format!("failed to read apps directory `{}`", apps_root.display()),
            error,
        )
    })?;

    let mut artifacts = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            WinedroidError::with_source(
                format!("failed to read apps directory `{}`", apps_root.display()),
                error,
            )
        })?;
        if !entry.path().is_dir() {
            continue;
        }
        let artifact_path = entry.path().join("package.artifact");
        if !artifact_path.exists() {
            continue;
        }
        artifacts.push(read_package_artifact(&artifact_path)?);
    }

    artifacts.sort_by(|left, right| left.package_name.cmp(&right.package_name));
    Ok(artifacts)
}

pub fn uninstall(prefix: &WinedroidPrefix, package_name: &str) -> Result<()> {
    let app_dir = prefix.app_dir(package_name)?;
    if !app_dir.exists() {
        return Err(WinedroidError::new(format!(
            "package `{package_name}` is not installed"
        )));
    }

    fs::remove_dir_all(&app_dir).map_err(|error| {
        WinedroidError::with_source(
            format!("failed to remove installed package `{}`", app_dir.display()),
            error,
        )
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AppLayout {
    app_dir: PathBuf,
    source_apk: PathBuf,
    manifest_summary: PathBuf,
    package_artifact: PathBuf,
    data_dir: PathBuf,
    device_data_dir: PathBuf,
    cache_dir: PathBuf,
    files_dir: PathBuf,
    runtime_dir: PathBuf,
    dex_dir: PathBuf,
    manifest_xml: PathBuf,
    resources_arsc: PathBuf,
    native_dir: PathBuf,
}

impl AppLayout {
    fn new(prefix: &WinedroidPrefix, package_name: &str) -> Result<Self> {
        let app_dir = prefix.app_dir(package_name)?;
        let data_dir = app_dir.join("data");
        let device_data_dir = app_dir.join("data_de");

        Ok(Self {
            source_apk: app_dir.join("source.apk"),
            manifest_summary: app_dir.join("manifest.txt"),
            package_artifact: app_dir.join("package.artifact"),
            cache_dir: data_dir.join("cache"),
            files_dir: data_dir.join("files"),
            data_dir,
            device_data_dir,
            runtime_dir: app_dir.join("runtime"),
            dex_dir: app_dir.join("runtime").join("dex"),
            manifest_xml: app_dir.join("runtime").join("AndroidManifest.xml"),
            resources_arsc: app_dir.join("runtime").join("resources.arsc"),
            native_dir: app_dir.join("native"),
            app_dir,
        })
    }
}

impl InstalledPackageArtifact {
    fn from_install(
        source_apk: &Path,
        report: &InspectionReport,
        native_libraries: &[InstalledNativeLibrary],
    ) -> Result<Self> {
        let metadata = fs::metadata(source_apk).map_err(|error| {
            WinedroidError::with_source(
                format!(
                    "failed to read installed APK metadata `{}`",
                    source_apk.display()
                ),
                error,
            )
        })?;
        let modified = metadata.modified().map_err(|error| {
            WinedroidError::with_source(
                format!(
                    "failed to read installed APK modification time `{}`",
                    source_apk.display()
                ),
                error,
            )
        })?;
        let source_modified_unix_millis = unix_millis(modified)?;
        let native_plan = report.native_runtime_plan();
        let native_dependency_plan = report.native_dependency_plan(&native_plan);

        let manifest = report.manifest();
        let launch_activity = report.primary_launch_activity()?;
        let launchable_activities = report.resolved_launch_activities()?;
        let signing_certificates = report
            .signing_certificates()
            .iter()
            .map(|der| crate::apk_signature::hex_encode(der))
            .collect();
        let declared_permissions = manifest
            .permissions
            .iter()
            .map(|permission| permission.name.clone())
            .collect();

        Ok(Self {
            version_name: report.version_name().map(str::to_owned),
            version_code: report.version_code().map(str::to_owned),
            package_name: report
                .package_name()
                .ok_or_else(|| WinedroidError::new("manifest does not declare a package name"))?
                .to_owned(),
            launch_activity,
            launchable_activities,
            split_apks: Vec::new(),
            shared_user_id: manifest.shared_user_id.clone(),
            signing_certificates,
            declared_permissions,
            services_json: manifest
                .services
                .iter()
                .map(serialize_service_json)
                .collect(),
            receivers_json: manifest
                .receivers
                .iter()
                .map(serialize_receiver_json)
                .collect(),
            providers_json: manifest
                .providers
                .iter()
                .map(serialize_provider_json)
                .collect(),
            uses_libraries_json: manifest
                .uses_libraries
                .iter()
                .map(serialize_uses_library_json)
                .collect(),
            application_meta_data_json: manifest
                .application_meta_data
                .iter()
                .map(serialize_meta_data_json)
                .collect(),
            source_apk: source_apk.to_path_buf(),
            source_size: metadata.len(),
            source_modified_unix_millis,
            entry_count: report.entry_count(),
            dex_files: report.dex_file_names(),
            dex_metadata: report.dex_metadata(),
            resource_table_summary: report.resource_table_summary(),
            resource_files: report
                .resource_files()
                .into_iter()
                .map(|resource| {
                    format!(
                        "0x{:08x}:{}/{}={}",
                        resource.resource_id, resource.type_name, resource.key_name, resource.path
                    )
                })
                .collect(),
            native_abi_names: report.native_abi_names(),
            selected_guest_abi: native_plan
                .selected_guest_abi
                .map(|abi| abi.as_str().to_owned()),
            native_libraries: native_libraries
                .iter()
                .map(|library| format!("{}/{}", library.abi, library.name))
                .collect(),
            android_system_libraries: native_dependency_plan.android_system_libraries,
            missing_native_libraries: native_dependency_plan.missing_libraries,
        })
    }

    pub fn native_runtime_plan(&self) -> NativeRuntimePlan {
        NativeRuntimePlan::detect(&self.native_abi_names)
    }

    pub fn native_dependency_plan(
        &self,
        native_runtime_plan: &NativeRuntimePlan,
    ) -> NativeDependencyPlan {
        let selected_guest_abi = native_runtime_plan.selected_guest_abi;
        let app_libraries = selected_guest_abi
            .map(|abi| self.native_library_names_for_abi(abi))
            .unwrap_or_default();

        NativeDependencyPlan {
            selected_guest_abi,
            app_libraries,
            android_system_libraries: self.android_system_libraries.clone(),
            missing_libraries: self.missing_native_libraries.clone(),
        }
    }

    fn native_library_names_for_abi(&self, abi: GuestAbi) -> Vec<String> {
        let mut names = self
            .native_libraries
            .iter()
            .filter_map(|record| record.split_once('/'))
            .filter(|(record_abi, _)| *record_abi == abi.as_str())
            .map(|(_, name)| name.to_owned())
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();
        names
    }
}

fn unix_millis(time: SystemTime) -> Result<u128> {
    let duration = time.duration_since(UNIX_EPOCH).map_err(|error| {
        WinedroidError::with_source("installed APK timestamp predates Unix epoch", error)
    })?;

    Ok(duration.as_millis())
}

fn create_layout_dirs(layout: &AppLayout) -> Result<()> {
    prepare_persistent_dirs(layout)?;
    validate_install_output_file(&layout.source_apk)?;
    validate_install_output_file(&layout.manifest_summary)?;
    validate_install_output_file(&layout.package_artifact)?;
    reset_generated_dir(&layout.runtime_dir)?;
    create_dir(&layout.dex_dir)?;
    reset_generated_dir(&layout.native_dir)?;

    Ok(())
}

fn validate_install_output_file(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(WinedroidError::new(format!(
                "install output path `{}` is not a real file",
                path.display()
            )))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(WinedroidError::with_source(
            format!("failed to inspect install output path `{}`", path.display()),
            error,
        )),
    }
}

fn require_installed_regular_file(path: &Path, package_name: &str) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(WinedroidError::new(format!(
                "installed package `{package_name}` path `{}` is not a real file",
                path.display()
            )))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(WinedroidError::new(
            format!("package `{package_name}` is not installed"),
        )),
        Err(error) => Err(WinedroidError::with_source(
            format!(
                "failed to inspect installed package `{package_name}` path `{}`",
                path.display()
            ),
            error,
        )),
    }
}

fn prepare_persistent_dirs(layout: &AppLayout) -> Result<()> {
    let apps_dir = layout.app_dir.parent().ok_or_else(|| {
        WinedroidError::new(format!(
            "app directory `{}` has no apps parent",
            layout.app_dir.display()
        ))
    })?;
    create_real_directory(apps_dir)?;
    let apps = Directory::open_ambient(apps_dir).map_err(|error| {
        WinedroidError::with_source(
            format!(
                "failed to acquire apps directory capability `{}`",
                apps_dir.display()
            ),
            error,
        )
    })?;
    let package = layout
        .app_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            WinedroidError::new(format!(
                "app directory `{}` has no UTF-8 package name",
                layout.app_dir.display()
            ))
        })?;
    let package = CString::new(package)
        .map_err(|error| WinedroidError::with_source("package name contains NUL", error))?;
    let data = CString::new("data").expect("literal has no NUL");
    let device_data = CString::new("data_de").expect("literal has no NUL");
    let cache = CString::new("cache").expect("literal has no NUL");
    let files = CString::new("files").expect("literal has no NUL");
    ensure_persistent_capability_directory(&apps, std::slice::from_ref(&package), &layout.app_dir)?;
    ensure_persistent_capability_directory(
        &apps,
        &[package.clone(), data.clone()],
        &layout.data_dir,
    )?;
    ensure_persistent_capability_directory(
        &apps,
        &[package.clone(), device_data],
        &layout.device_data_dir,
    )?;
    migrate_legacy_directory_capability(&apps, &package, &data, &cache)?;
    migrate_legacy_directory_capability(&apps, &package, &data, &files)?;
    ensure_persistent_capability_directory(
        &apps,
        &[package.clone(), data.clone(), cache],
        &layout.cache_dir,
    )?;
    ensure_persistent_capability_directory(&apps, &[package, data, files], &layout.files_dir)?;

    Ok(())
}

fn ensure_persistent_capability_directory(
    apps: &Directory,
    components: &[CString],
    display_path: &Path,
) -> Result<()> {
    match apps.metadata(components) {
        Ok(metadata) if metadata.kind == MetadataKind::Directory => return Ok(()),
        Ok(_) => {
            return Err(WinedroidError::new(format!(
                "persistent app path `{}` is not a real directory",
                display_path.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(WinedroidError::with_source(
                format!(
                    "failed to inspect persistent app path `{}`",
                    display_path.display()
                ),
                error,
            ));
        }
    }
    apps.mkdir_all(components).map_err(|error| {
        WinedroidError::with_source(
            format!(
                "failed to create persistent app path `{}`",
                display_path.display()
            ),
            error,
        )
    })?;
    Ok(())
}

fn migrate_legacy_directory_capability(
    apps: &Directory,
    package: &CString,
    data: &CString,
    name: &CString,
) -> Result<()> {
    let legacy = [package.clone(), name.clone()];
    match apps.metadata(&legacy) {
        Ok(metadata) if metadata.kind == MetadataKind::Directory => {}
        Ok(_) => {
            return Err(WinedroidError::new(format!(
                "legacy app path `{}` is not a real directory",
                name.to_string_lossy()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(WinedroidError::with_source(
                format!(
                    "failed to inspect legacy directory `{}`",
                    name.to_string_lossy()
                ),
                error,
            ));
        }
    }
    let destination = [package.clone(), data.clone(), name.clone()];
    match apps.rename_noreplace(&legacy, apps, &destination) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(WinedroidError::new(format!(
                "legacy app directory `{}` conflicts with its data directory",
                name.to_string_lossy()
            )))
        }
        Err(error) => Err(WinedroidError::with_source(
            format!(
                "failed to migrate legacy app directory `{}`",
                name.to_string_lossy()
            ),
            error,
        )),
    }
}

fn create_real_directory(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(WinedroidError::new(format!(
                "persistent app path `{}` is not a real directory",
                path.display()
            )));
        }
        Ok(_) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(WinedroidError::with_source(
                format!("failed to inspect persistent app path `{}`", path.display()),
                error,
            ));
        }
    }

    create_dir(path)?;
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        WinedroidError::with_source(
            format!("failed to inspect persistent app path `{}`", path.display()),
            error,
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(WinedroidError::new(format!(
            "persistent app path `{}` is not a real directory",
            path.display()
        )));
    }

    Ok(())
}

fn create_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|error| {
        WinedroidError::with_source(
            format!("failed to create directory `{}`", path.display()),
            error,
        )
    })
}

fn reset_generated_dir(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| {
            WinedroidError::with_source(
                format!("failed to reset generated directory `{}`", path.display()),
                error,
            )
        })?;
    }

    create_dir(path)
}

fn copy_source_apk(source: &Path, destination: &Path) -> Result<()> {
    fs::copy(source, destination).map_err(|error| {
        WinedroidError::with_source(
            format!(
                "failed to copy APK from `{}` to `{}`",
                source.display(),
                destination.display()
            ),
            error,
        )
    })?;

    Ok(())
}

fn install_split_apks(
    split_apk_paths: &[PathBuf],
    layout: &AppLayout,
    base_package_name: &str,
    base_version_code: Option<&str>,
) -> Result<Vec<String>> {
    // cite: frameworks-base PackageParser.java#parsePackageLite
    if split_apk_paths.is_empty() {
        return Ok(Vec::new());
    }
    let splits_dir = layout.app_dir.join("splits");
    create_dir(&splits_dir)?;
    let mut installed = Vec::with_capacity(split_apk_paths.len());
    for (index, split_path) in split_apk_paths.iter().enumerate() {
        let split_report = crate::inspect::inspect_apk(split_path)?;
        let split_pkg = split_report.package_name().ok_or_else(|| {
            WinedroidError::new(format!(
                "split APK `{}` does not declare a package name",
                split_path.display()
            ))
        })?;
        if split_pkg != base_package_name {
            return Err(WinedroidError::new(format!(
                "split APK `{}` package `{split_pkg}` does not match base package `{base_package_name}`",
                split_path.display()
            )));
        }
        if let (Some(base_vc), Some(split_vc)) = (base_version_code, split_report.version_code()) {
            if base_vc != split_vc {
                return Err(WinedroidError::new(format!(
                    "split APK `{}` versionCode `{split_vc}` does not match base versionCode `{base_vc}`",
                    split_path.display()
                )));
            }
        }
        let split_name = split_report
            .manifest()
            .split_name
            .clone()
            .unwrap_or_else(|| format!("split_{index}"));
        let dest_name = format!("{split_name}.apk");
        let dest = splits_dir.join(&dest_name);
        copy_source_apk(split_path, &dest)?;
        installed.push(format!("{split_name}\t{}", dest.display()));
    }
    Ok(installed)
}

fn serialize_meta_data_json(meta: &crate::manifest::MetaData) -> String {
    serde_json::json!({
        "name": meta.name,
        "value": meta.value,
        "resource": meta.resource,
    })
    .to_string()
}

fn serialize_intent_filter_json(filter: &crate::manifest::IntentFilter) -> serde_json::Value {
    serde_json::json!({
        "actions": filter.actions,
        "categories": filter.categories,
        "schemes": filter.schemes,
        "authorities": filter.authorities,
        "mime_types": filter.mime_types,
    })
}

fn serialize_service_json(service: &crate::manifest::ServiceComponent) -> String {
    serde_json::json!({
        "name": service.name,
        "exported": service.exported,
        "enabled": service.enabled,
        "permission": service.permission,
        "process": service.process,
        "intent_filters": service.intent_filters.iter().map(serialize_intent_filter_json).collect::<Vec<_>>(),
        "meta_data": service.meta_data.iter().map(|m| serde_json::json!({"name": m.name, "value": m.value, "resource": m.resource})).collect::<Vec<_>>(),
    })
    .to_string()
}

fn serialize_receiver_json(receiver: &crate::manifest::ReceiverComponent) -> String {
    serde_json::json!({
        "name": receiver.name,
        "exported": receiver.exported,
        "enabled": receiver.enabled,
        "permission": receiver.permission,
        "intent_filters": receiver.intent_filters.iter().map(serialize_intent_filter_json).collect::<Vec<_>>(),
        "meta_data": receiver.meta_data.iter().map(|m| serde_json::json!({"name": m.name, "value": m.value, "resource": m.resource})).collect::<Vec<_>>(),
    })
    .to_string()
}

fn serialize_provider_json(provider: &crate::manifest::ProviderComponent) -> String {
    serde_json::json!({
        "name": provider.name,
        "authorities": provider.authorities,
        "exported": provider.exported,
        "enabled": provider.enabled,
        "read_permission": provider.read_permission,
        "write_permission": provider.write_permission,
        "grant_uri_permissions": provider.grant_uri_permissions,
        "meta_data": provider.meta_data.iter().map(|m| serde_json::json!({"name": m.name, "value": m.value, "resource": m.resource})).collect::<Vec<_>>(),
    })
    .to_string()
}

fn serialize_uses_library_json(lib: &crate::manifest::UsesLibrary) -> String {
    serde_json::json!({
        "name": lib.name,
        "required": lib.required,
    })
    .to_string()
}

fn install_runtime_artifacts(
    apk_path: &Path,
    layout: &AppLayout,
    report: &InspectionReport,
) -> Result<()> {
    let mut apk = ApkArchive::open(apk_path)?;
    let manifest = apk.read_required_file("AndroidManifest.xml")?;
    fs::write(&layout.manifest_xml, manifest).map_err(|error| {
        WinedroidError::with_source(
            format!(
                "failed to extract AndroidManifest.xml to `{}`",
                layout.manifest_xml.display()
            ),
            error,
        )
    })?;

    for dex_file in report.dex_file_names() {
        let destination = runtime_dex_path(layout, &dex_file)?;
        let data = apk.read_required_file(&dex_file)?;
        fs::write(&destination, &data).map_err(|error| {
            WinedroidError::with_source(
                format!(
                    "failed to extract DEX file `{dex_file}` to `{}`",
                    destination.display()
                ),
                error,
            )
        })?;
        let parsed = dex::parse_dex_file(&dex_file, data)?;
        let cache = dex::encode_dex_metadata_cache(&parsed)?;
        let cache_destination = runtime_dex_cache_path(layout, &dex_file)?;
        fs::write(&cache_destination, cache).map_err(|error| {
            WinedroidError::with_source(
                format!(
                    "failed to write DEX metadata cache for `{dex_file}` to `{}`",
                    cache_destination.display()
                ),
                error,
            )
        })?;
    }

    if let Some(resources) = apk.read_optional_file("resources.arsc")? {
        fs::write(&layout.resources_arsc, resources).map_err(|error| {
            WinedroidError::with_source(
                format!(
                    "failed to extract resources.arsc to `{}`",
                    layout.resources_arsc.display()
                ),
                error,
            )
        })?;
    }

    Ok(())
}

fn runtime_dex_path(layout: &AppLayout, dex_file: &str) -> Result<PathBuf> {
    let file_name = Path::new(dex_file)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| WinedroidError::new(format!("invalid DEX file name `{dex_file}`")))?;
    if file_name != dex_file {
        return Err(WinedroidError::new(format!(
            "installed DEX file `{dex_file}` is not a root APK entry"
        )));
    }

    Ok(layout.dex_dir.join(file_name))
}

fn runtime_dex_cache_path(layout: &AppLayout, dex_file: &str) -> Result<PathBuf> {
    Ok(layout.dex_dir.join(format!(
        "{}.wdidx",
        runtime_dex_path(layout, dex_file)?
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| WinedroidError::new(format!("invalid DEX file name `{dex_file}`")))?
    )))
}

fn install_native_libraries(
    apk_path: &Path,
    layout: &AppLayout,
    report: &InspectionReport,
) -> Result<Vec<InstalledNativeLibrary>> {
    let plan = report.native_runtime_plan();
    let Some(selected_abi) = plan.selected_guest_abi else {
        return Ok(Vec::new());
    };

    let libraries = report.native_libraries_for_abi(selected_abi);
    if libraries.is_empty() {
        return Ok(Vec::new());
    }

    let abi_dir = layout.native_dir.join(selected_abi.as_str());
    create_dir(&abi_dir)?;

    let mut apk = ApkArchive::open(apk_path)?;
    let mut installed_libraries = Vec::with_capacity(libraries.len());

    for library in libraries {
        let source_path = library.apk_path();
        let data = apk.read_required_file(&source_path)?;
        let destination = abi_dir.join(library.name());

        fs::write(&destination, data).map_err(|error| {
            WinedroidError::with_source(
                format!(
                    "failed to extract native library `{}` to `{}`",
                    source_path,
                    destination.display()
                ),
                error,
            )
        })?;

        installed_libraries.push(InstalledNativeLibrary {
            abi: library.abi().to_owned(),
            name: library.name().to_owned(),
            path: destination,
        });
    }

    installed_libraries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(installed_libraries)
}

fn write_manifest_summary(path: &Path, report: &InspectionReport) -> Result<()> {
    fs::write(path, format!("{report}")).map_err(|error| {
        WinedroidError::with_source(
            format!("failed to write manifest summary `{}`", path.display()),
            error,
        )
    })
}

fn write_package_artifact(path: &Path, artifact: &InstalledPackageArtifact) -> Result<()> {
    let mut lines = Vec::new();

    push_artifact_line(&mut lines, "version", "2")?;
    push_artifact_line(&mut lines, "package", &artifact.package_name)?;
    if let Some(version) = &artifact.version_name {
        push_artifact_line(
            &mut lines,
            "app_version_name_json",
            &serde_json::Value::String(version.clone()).to_string(),
        )?;
    }
    if let Some(version) = &artifact.version_code {
        push_artifact_line(
            &mut lines,
            "app_version_code_json",
            &serde_json::Value::String(version.clone()).to_string(),
        )?;
    }
    if let Some(launch_activity) = &artifact.launch_activity {
        push_artifact_line(&mut lines, "launch_activity", launch_activity)?;
    }
    for activity in &artifact.launchable_activities {
        push_artifact_line(&mut lines, "launchable_activity", activity)?;
    }
    for split in &artifact.split_apks {
        push_artifact_line(&mut lines, "split_apk", split)?;
    }
    if let Some(shared_user_id) = &artifact.shared_user_id {
        push_artifact_line(&mut lines, "shared_user_id", shared_user_id)?;
    }
    for cert in &artifact.signing_certificates {
        push_artifact_line(&mut lines, "signing_cert_hex", cert)?;
    }
    for perm in &artifact.declared_permissions {
        push_artifact_line(&mut lines, "declared_permission", perm)?;
    }
    for item in &artifact.services_json {
        push_artifact_line(&mut lines, "service_json", item)?;
    }
    for item in &artifact.receivers_json {
        push_artifact_line(&mut lines, "receiver_json", item)?;
    }
    for item in &artifact.providers_json {
        push_artifact_line(&mut lines, "provider_json", item)?;
    }
    for item in &artifact.uses_libraries_json {
        push_artifact_line(&mut lines, "uses_library_json", item)?;
    }
    for item in &artifact.application_meta_data_json {
        push_artifact_line(&mut lines, "app_meta_data_json", item)?;
    }
    push_artifact_line(
        &mut lines,
        "source_apk",
        &artifact.source_apk.display().to_string(),
    )?;
    push_artifact_line(&mut lines, "source_size", &artifact.source_size.to_string())?;
    push_artifact_line(
        &mut lines,
        "source_modified_unix_millis",
        &artifact.source_modified_unix_millis.to_string(),
    )?;
    push_artifact_line(&mut lines, "entry_count", &artifact.entry_count.to_string())?;
    for dex_file in &artifact.dex_files {
        push_artifact_line(&mut lines, "dex_file", dex_file)?;
    }
    for dex_metadata in &artifact.dex_metadata {
        push_artifact_line(
            &mut lines,
            "dex_metadata",
            &format_dex_metadata(dex_metadata)?,
        )?;
    }
    if let Some(summary) = &artifact.resource_table_summary {
        push_artifact_line(
            &mut lines,
            "resource_table_byte_size",
            &summary.byte_size.to_string(),
        )?;
        push_artifact_line(
            &mut lines,
            "resource_table_package_count",
            &summary.package_count.to_string(),
        )?;
    }
    for resource_file in &artifact.resource_files {
        push_artifact_line(&mut lines, "resource_file", resource_file)?;
    }
    for abi in &artifact.native_abi_names {
        push_artifact_line(&mut lines, "native_abi", abi)?;
    }
    if let Some(abi) = &artifact.selected_guest_abi {
        push_artifact_line(&mut lines, "selected_guest_abi", abi)?;
    }
    for library in &artifact.native_libraries {
        push_artifact_line(&mut lines, "native_library", library)?;
    }
    for library in &artifact.android_system_libraries {
        push_artifact_line(&mut lines, "android_system_library", library)?;
    }
    for library in &artifact.missing_native_libraries {
        push_artifact_line(&mut lines, "missing_native_library", library)?;
    }

    fs::write(path, lines.join("\n") + "\n").map_err(|error| {
        WinedroidError::with_source(
            format!("failed to write package artifact `{}`", path.display()),
            error,
        )
    })
}

fn read_package_artifact(path: &Path) -> Result<InstalledPackageArtifact> {
    let content = fs::read_to_string(path).map_err(|error| {
        WinedroidError::with_source(
            format!("failed to read package artifact `{}`", path.display()),
            error,
        )
    })?;
    let entries = parse_package_artifact_entries(path, &content)?;
    let version = required_artifact_field(path, &entries, "version")?;
    if version != "2" {
        return Err(WinedroidError::new(format!(
            "package artifact `{}` has unsupported version `{version}`",
            path.display()
        )));
    }
    let resource_table_summary = parse_resource_table_summary(path, &entries)?;
    let selected_guest_abi = optional_artifact_field(path, &entries, "selected_guest_abi")?;
    let native_libraries = artifact_fields(&entries, "native_library");
    let mut native_abi_names = artifact_fields(&entries, "native_abi");
    if native_abi_names.is_empty() {
        native_abi_names =
            fallback_native_abi_names(selected_guest_abi.as_deref(), &native_libraries);
    }

    Ok(InstalledPackageArtifact {
        version_name: optional_artifact_json_string(path, &entries, "app_version_name_json")?,
        version_code: optional_artifact_json_string(path, &entries, "app_version_code_json")?,
        package_name: required_artifact_field(path, &entries, "package")?,
        launch_activity: optional_artifact_field(path, &entries, "launch_activity")?,
        launchable_activities: artifact_fields(&entries, "launchable_activity"),
        split_apks: artifact_fields(&entries, "split_apk"),
        shared_user_id: optional_artifact_field(path, &entries, "shared_user_id")?,
        signing_certificates: artifact_fields(&entries, "signing_cert_hex"),
        declared_permissions: artifact_fields(&entries, "declared_permission"),
        services_json: artifact_fields(&entries, "service_json"),
        receivers_json: artifact_fields(&entries, "receiver_json"),
        providers_json: artifact_fields(&entries, "provider_json"),
        uses_libraries_json: artifact_fields(&entries, "uses_library_json"),
        application_meta_data_json: artifact_fields(&entries, "app_meta_data_json"),
        source_apk: PathBuf::from(required_artifact_field(path, &entries, "source_apk")?),
        source_size: parse_artifact_u64(
            path,
            "source_size",
            &required_artifact_field(path, &entries, "source_size")?,
        )?,
        source_modified_unix_millis: parse_artifact_u128(
            path,
            "source_modified_unix_millis",
            &required_artifact_field(path, &entries, "source_modified_unix_millis")?,
        )?,
        entry_count: parse_artifact_usize(
            path,
            "entry_count",
            &required_artifact_field(path, &entries, "entry_count")?,
        )?,
        dex_files: artifact_fields(&entries, "dex_file"),
        dex_metadata: artifact_fields(&entries, "dex_metadata")
            .into_iter()
            .map(|value| parse_dex_metadata(path, &value))
            .collect::<Result<Vec<_>>>()?,
        resource_table_summary,
        resource_files: artifact_fields(&entries, "resource_file"),
        native_abi_names,
        selected_guest_abi,
        native_libraries,
        android_system_libraries: artifact_fields(&entries, "android_system_library"),
        missing_native_libraries: artifact_fields(&entries, "missing_native_library"),
    })
}

fn fallback_native_abi_names(
    selected_guest_abi: Option<&str>,
    native_libraries: &[String],
) -> Vec<String> {
    let mut names = native_libraries
        .iter()
        .filter_map(|record| record.split_once('/').map(|(abi, _)| abi.to_owned()))
        .collect::<Vec<_>>();
    if let Some(abi) = selected_guest_abi {
        names.push(abi.to_owned());
    }
    names.sort();
    names.dedup();
    names
}

fn format_dex_metadata(metadata: &DexFileMetadata) -> Result<String> {
    reject_artifact_record_part("dex_metadata.name", &metadata.name)?;
    reject_artifact_record_part(
        "dex_metadata.compression_method",
        &metadata.compression_method,
    )?;
    reject_artifact_record_part("dex_metadata.version", &metadata.version)?;

    Ok([
        metadata.name.clone(),
        metadata.compressed_size.to_string(),
        metadata.uncompressed_size.to_string(),
        metadata.compression_method.clone(),
        metadata.version.clone(),
        metadata.checksum.to_string(),
        metadata.declared_file_size.to_string(),
        metadata.string_count.to_string(),
        metadata.type_count.to_string(),
        metadata.method_count.to_string(),
        metadata.class_count.to_string(),
    ]
    .join("\t"))
}

fn parse_dex_metadata(path: &Path, value: &str) -> Result<DexFileMetadata> {
    let parts = value.split('\t').collect::<Vec<_>>();
    if parts.len() != 11 {
        return Err(WinedroidError::new(format!(
            "package artifact `{}` dex_metadata expected 11 tab-separated fields, found {}",
            path.display(),
            parts.len()
        )));
    }

    Ok(DexFileMetadata {
        name: parts[0].to_owned(),
        compressed_size: parse_artifact_u64(path, "dex_metadata.compressed_size", parts[1])?,
        uncompressed_size: parse_artifact_u64(path, "dex_metadata.uncompressed_size", parts[2])?,
        compression_method: parts[3].to_owned(),
        version: parts[4].to_owned(),
        checksum: parse_artifact_u32(path, "dex_metadata.checksum", parts[5])?,
        declared_file_size: parse_artifact_u32(path, "dex_metadata.declared_file_size", parts[6])?,
        string_count: parse_artifact_u32(path, "dex_metadata.string_count", parts[7])?,
        type_count: parse_artifact_u32(path, "dex_metadata.type_count", parts[8])?,
        method_count: parse_artifact_u32(path, "dex_metadata.method_count", parts[9])?,
        class_count: parse_artifact_u32(path, "dex_metadata.class_count", parts[10])?,
    })
}

fn parse_resource_table_summary(
    path: &Path,
    entries: &[(String, String)],
) -> Result<Option<ResourceTableSummary>> {
    let byte_size = optional_artifact_field(path, entries, "resource_table_byte_size")?;
    let package_count = optional_artifact_field(path, entries, "resource_table_package_count")?;

    match (byte_size, package_count) {
        (Some(byte_size), Some(package_count)) => Ok(Some(ResourceTableSummary {
            byte_size: parse_artifact_usize(path, "resource_table_byte_size", &byte_size)?,
            package_count: parse_artifact_u32(
                path,
                "resource_table_package_count",
                &package_count,
            )?,
        })),
        (None, None) => Ok(None),
        _ => Err(WinedroidError::new(format!(
            "package artifact `{}` has partial resource table summary",
            path.display()
        ))),
    }
}

fn parse_package_artifact_entries(path: &Path, content: &str) -> Result<Vec<(String, String)>> {
    let mut entries = Vec::new();

    for (index, line) in content.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(WinedroidError::new(format!(
                "package artifact `{}` line {} is not key=value",
                path.display(),
                index + 1
            )));
        };
        if key.is_empty() {
            return Err(WinedroidError::new(format!(
                "package artifact `{}` line {} has an empty key",
                path.display(),
                index + 1
            )));
        }

        entries.push((key.to_owned(), value.to_owned()));
    }

    Ok(entries)
}

fn required_artifact_field(path: &Path, entries: &[(String, String)], key: &str) -> Result<String> {
    let values = artifact_fields(entries, key);
    if values.len() != 1 {
        return Err(WinedroidError::new(format!(
            "package artifact `{}` expected exactly one `{key}` field, found {}",
            path.display(),
            values.len()
        )));
    }

    Ok(values[0].clone())
}

fn optional_artifact_json_string(
    path: &Path,
    entries: &[(String, String)],
    key: &str,
) -> Result<Option<String>> {
    let Some(value) = optional_artifact_field(path, entries, key)? else {
        return Ok(None);
    };
    serde_json::from_str::<String>(&value)
        .map(Some)
        .map_err(|error| {
            WinedroidError::with_source(
                format!("invalid {key} in package artifact `{}`", path.display()),
                error,
            )
        })
}

fn optional_artifact_field(
    path: &Path,
    entries: &[(String, String)],
    key: &str,
) -> Result<Option<String>> {
    let values = artifact_fields(entries, key);
    if values.len() > 1 {
        return Err(WinedroidError::new(format!(
            "package artifact `{}` expected at most one `{key}` field, found {}",
            path.display(),
            values.len()
        )));
    }

    Ok(values.into_iter().next())
}

fn artifact_fields(entries: &[(String, String)], key: &str) -> Vec<String> {
    entries
        .iter()
        .filter(|(entry_key, _)| entry_key == key)
        .map(|(_, value)| value.clone())
        .collect()
}

fn parse_artifact_u64(path: &Path, key: &str, value: &str) -> Result<u64> {
    value.parse::<u64>().map_err(|error| {
        WinedroidError::with_source(
            format!(
                "package artifact `{}` field `{key}` is not a u64",
                path.display()
            ),
            error,
        )
    })
}

fn parse_artifact_u128(path: &Path, key: &str, value: &str) -> Result<u128> {
    value.parse::<u128>().map_err(|error| {
        WinedroidError::with_source(
            format!(
                "package artifact `{}` field `{key}` is not a u128",
                path.display()
            ),
            error,
        )
    })
}

fn parse_artifact_u32(path: &Path, key: &str, value: &str) -> Result<u32> {
    value.parse::<u32>().map_err(|error| {
        WinedroidError::with_source(
            format!(
                "package artifact `{}` field `{key}` is not a u32",
                path.display()
            ),
            error,
        )
    })
}

fn parse_artifact_usize(path: &Path, key: &str, value: &str) -> Result<usize> {
    value.parse::<usize>().map_err(|error| {
        WinedroidError::with_source(
            format!(
                "package artifact `{}` field `{key}` is not a usize",
                path.display()
            ),
            error,
        )
    })
}

fn reject_artifact_record_part(key: &str, value: &str) -> Result<()> {
    if value.contains('\t') || value.contains('\n') || value.contains('\r') {
        return Err(WinedroidError::new(format!(
            "package artifact value for `{key}` contains a record separator"
        )));
    }

    Ok(())
}

fn push_artifact_line(lines: &mut Vec<String>, key: &str, value: &str) -> Result<()> {
    if value.contains('\n') || value.contains('\r') {
        return Err(WinedroidError::new(format!(
            "package artifact value for `{key}` contains a newline"
        )));
    }

    lines.push(format!("{key}={value}"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;
    use crate::abi::{HostArch, HostOs, HostPlatform};

    #[test]
    fn persistent_layout_migrates_legacy_files_and_cache_without_data_loss() {
        let root = test_install_root("migrate");
        let prefix = WinedroidPrefix::from_root(root.clone());
        let layout = AppLayout::new(&prefix, "com.example").expect("layout should resolve");
        fs::create_dir_all(layout.app_dir.join("cache/nested")).expect("legacy cache should exist");
        fs::create_dir_all(layout.app_dir.join("files")).expect("legacy files should exist");
        fs::write(layout.app_dir.join("cache/nested/cache.bin"), b"cache")
            .expect("legacy cache data should exist");
        fs::write(layout.app_dir.join("files/state.bin"), b"state")
            .expect("legacy file data should exist");

        prepare_persistent_dirs(&layout).expect("legacy directories should migrate");

        assert_eq!(
            fs::read(layout.cache_dir.join("nested/cache.bin"))
                .expect("migrated cache should remain"),
            b"cache"
        );
        assert_eq!(
            fs::read(layout.files_dir.join("state.bin")).expect("migrated file should remain"),
            b"state"
        );
        assert!(!layout.app_dir.join("cache").exists());
        assert!(!layout.app_dir.join("files").exists());
        assert!(layout.data_dir.is_dir());
        assert!(layout.device_data_dir.is_dir());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn persistent_layout_rejects_ambiguous_legacy_directory_conflicts() {
        let root = test_install_root("conflict");
        let prefix = WinedroidPrefix::from_root(root.clone());
        let layout = AppLayout::new(&prefix, "com.example").expect("layout should resolve");
        fs::create_dir_all(layout.app_dir.join("cache")).expect("legacy cache should exist");
        fs::create_dir_all(&layout.cache_dir).expect("new cache should exist");
        fs::write(layout.app_dir.join("cache/legacy.bin"), b"legacy")
            .expect("legacy cache file should exist");
        fs::write(layout.cache_dir.join("current.bin"), b"current")
            .expect("current cache file should exist");

        let error = prepare_persistent_dirs(&layout).expect_err("conflict should fail loudly");

        assert!(error.to_string().contains("conflicts"));
        assert_eq!(
            fs::read(layout.app_dir.join("cache/legacy.bin")).expect("legacy data should remain"),
            b"legacy"
        );
        assert_eq!(
            fs::read(layout.cache_dir.join("current.bin")).expect("current data should remain"),
            b"current"
        );

        fs::remove_dir_all(root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn persistent_layout_rejects_a_symlinked_data_root_before_migration() {
        let root = test_install_root("data-symlink");
        let prefix = WinedroidPrefix::from_root(root.clone());
        let layout = AppLayout::new(&prefix, "com.example").expect("layout should resolve");
        let outside = root.join("outside");
        fs::create_dir_all(&layout.app_dir).expect("app directory should exist");
        fs::create_dir_all(&outside).expect("outside directory should exist");
        fs::create_dir_all(layout.app_dir.join("cache")).expect("legacy cache should exist");
        fs::write(layout.app_dir.join("cache/legacy.bin"), b"legacy")
            .expect("legacy cache data should exist");
        std::os::unix::fs::symlink(&outside, &layout.data_dir)
            .expect("test data symlink should exist");

        let error = prepare_persistent_dirs(&layout).expect_err("data symlink should be rejected");

        assert!(error.to_string().contains("not a real directory"));
        assert_eq!(
            fs::read(layout.app_dir.join("cache/legacy.bin")).expect("legacy data should remain"),
            b"legacy"
        );
        assert!(!outside.join("cache").exists());

        fs::remove_dir_all(root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn persistent_layout_rejects_a_symlinked_apps_root_before_writing() {
        let root = test_install_root("apps-symlink");
        let outside_apps = root.join("outside-apps");
        fs::create_dir_all(outside_apps.join("com.example"))
            .expect("outside app directory should exist");
        fs::create_dir_all(&root).expect("prefix root should exist");
        std::os::unix::fs::symlink(&outside_apps, root.join("apps"))
            .expect("test apps symlink should exist");
        let prefix = WinedroidPrefix::from_root(root.clone());
        let layout = AppLayout::new(&prefix, "com.example").expect("layout should resolve");

        let error = prepare_persistent_dirs(&layout).expect_err("apps symlink should be rejected");

        assert!(error.to_string().contains("not a real directory"));
        assert!(!outside_apps.join("com.example/data").exists());
        assert!(!outside_apps.join("com.example/runtime").exists());

        fs::remove_dir_all(root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn persistent_layout_rejects_a_symlinked_device_data_root() {
        let root = test_install_root("device-data-symlink");
        let prefix = WinedroidPrefix::from_root(root.clone());
        let layout = AppLayout::new(&prefix, "com.example").expect("layout should resolve");
        let outside = root.join("outside-device-data");
        fs::create_dir_all(&layout.app_dir).expect("app directory should exist");
        fs::create_dir_all(&outside).expect("outside directory should exist");
        std::os::unix::fs::symlink(&outside, &layout.device_data_dir)
            .expect("test device-data symlink should exist");

        let error =
            prepare_persistent_dirs(&layout).expect_err("device data symlink should be rejected");

        assert!(error.to_string().contains("not a real directory"));
        assert!(outside
            .read_dir()
            .expect("outside directory should remain readable")
            .next()
            .is_none());

        fs::remove_dir_all(root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn install_layout_rejects_a_symlinked_output_before_resetting_generated_data() {
        let root = test_install_root("output-symlink");
        let prefix = WinedroidPrefix::from_root(root.clone());
        fs::create_dir_all(root.join("apps/com.example")).expect("app directory should exist");
        let layout = AppLayout::new(&prefix, "com.example").expect("layout should resolve");
        let outside = root.join("outside.apk");
        fs::write(&outside, b"outside").expect("outside file should exist");
        std::os::unix::fs::symlink(&outside, &layout.source_apk)
            .expect("test output symlink should exist");

        let error = create_layout_dirs(&layout).expect_err("output symlink should be rejected");

        assert!(error.to_string().contains("not a real file"));
        assert_eq!(
            fs::read(&outside).expect("outside file should remain"),
            b"outside"
        );
        assert!(!layout.runtime_dir.exists());
        assert!(!layout.native_dir.exists());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn package_artifact_roundtrips_native_launch_metadata() {
        let path = std::env::temp_dir().join(format!(
            "winedroid-package-artifact-{}-{}.txt",
            std::process::id(),
            "native"
        ));
        let artifact = InstalledPackageArtifact {
            version_name: Some("1.2\nbeta".to_owned()),
            version_code: Some("12".to_owned()),
            package_name: "com.example".to_owned(),
            launch_activity: Some("com.example.MainActivity".to_owned()),
            launchable_activities: vec!["com.example.MainActivity".to_owned()],
            split_apks: Vec::new(),
            shared_user_id: None,
            signing_certificates: Vec::new(),
            declared_permissions: Vec::new(),
            services_json: Vec::new(),
            receivers_json: Vec::new(),
            providers_json: Vec::new(),
            uses_libraries_json: Vec::new(),
            application_meta_data_json: Vec::new(),
            source_apk: PathBuf::from("/tmp/example.apk"),
            source_size: 123,
            source_modified_unix_millis: 456,
            entry_count: 7,
            dex_files: vec!["classes.dex".to_owned()],
            dex_metadata: Vec::new(),
            resource_table_summary: None,
            resource_files: Vec::new(),
            native_abi_names: vec!["arm64-v8a".to_owned(), "x86".to_owned()],
            selected_guest_abi: Some("arm64-v8a".to_owned()),
            native_libraries: vec!["arm64-v8a/libmain.so".to_owned()],
            android_system_libraries: vec!["libc.so".to_owned(), "liblog.so".to_owned()],
            missing_native_libraries: vec!["libandroid.so".to_owned()],
        };

        write_package_artifact(&path, &artifact).expect("artifact should write");
        let loaded = read_package_artifact(&path).expect("artifact should read");
        let _ = std::fs::remove_file(&path);

        assert_eq!(loaded.version_name, artifact.version_name);
        assert_eq!(loaded.version_code, artifact.version_code);
        assert_eq!(loaded.native_abi_names, artifact.native_abi_names);
        assert_eq!(
            loaded.android_system_libraries,
            artifact.android_system_libraries
        );
        assert_eq!(
            loaded.missing_native_libraries,
            artifact.missing_native_libraries
        );

        let plan = NativeRuntimePlan::new(
            HostPlatform {
                os: HostOs::Macos,
                arch: HostArch::Arm64,
            },
            &loaded.native_abi_names,
        );
        let dependency_plan = loaded.native_dependency_plan(&plan);

        assert_eq!(dependency_plan.app_libraries, vec!["libmain.so"]);
        assert_eq!(
            dependency_plan.android_system_libraries,
            vec!["libc.so", "liblog.so"]
        );
        assert_eq!(dependency_plan.missing_libraries, vec!["libandroid.so"]);
    }

    #[test]
    fn package_artifact_infers_native_abis_for_version_two_records() {
        let names = fallback_native_abi_names(
            Some("arm64-v8a"),
            &[
                "arm64-v8a/libmain.so".to_owned(),
                "x86/libmain.so".to_owned(),
            ],
        );

        assert_eq!(names, vec!["arm64-v8a", "x86"]);
    }

    #[test]
    fn runtime_dex_path_accepts_root_dex_entries() {
        let layout = test_layout();

        assert_eq!(
            runtime_dex_path(&layout, "classes2.dex").expect("root DEX should resolve"),
            PathBuf::from("/tmp/winedroid-test/runtime/dex/classes2.dex")
        );
    }

    #[test]
    fn runtime_dex_path_rejects_nested_dex_entries() {
        let layout = test_layout();
        let error = runtime_dex_path(&layout, "assets/classes.dex")
            .expect_err("nested DEX entry should be rejected");

        assert!(error
            .to_string()
            .contains("installed DEX file `assets/classes.dex` is not a root APK entry"));
    }

    fn test_layout() -> AppLayout {
        AppLayout {
            app_dir: PathBuf::from("/tmp/winedroid-test"),
            source_apk: PathBuf::from("/tmp/winedroid-test/source.apk"),
            manifest_summary: PathBuf::from("/tmp/winedroid-test/manifest.txt"),
            package_artifact: PathBuf::from("/tmp/winedroid-test/package.artifact"),
            data_dir: PathBuf::from("/tmp/winedroid-test/data"),
            device_data_dir: PathBuf::from("/tmp/winedroid-test/data_de"),
            cache_dir: PathBuf::from("/tmp/winedroid-test/data/cache"),
            files_dir: PathBuf::from("/tmp/winedroid-test/data/files"),
            runtime_dir: PathBuf::from("/tmp/winedroid-test/runtime"),
            dex_dir: PathBuf::from("/tmp/winedroid-test/runtime/dex"),
            manifest_xml: PathBuf::from("/tmp/winedroid-test/runtime/AndroidManifest.xml"),
            resources_arsc: PathBuf::from("/tmp/winedroid-test/runtime/resources.arsc"),
            native_dir: PathBuf::from("/tmp/winedroid-test/native"),
        }
    }

    fn test_install_root(name: &str) -> PathBuf {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "winedroid-install-{name}-{}-{id}",
            std::process::id()
        ));
        fs::remove_dir_all(&root).ok();
        root
    }
}
