use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use crate::art_probe;
use crate::dex_run;
use crate::error::{Result, WinedroidError};
use crate::inspect;
use crate::install;
use crate::launch;
use crate::prefix::WinedroidPrefix;
use crate::runtime_check;
use crate::trace;

#[derive(Debug, Default)]
struct GlobalOptions {
    verbose: bool,
    trace_categories: Option<String>,
}

impl GlobalOptions {
    fn trace_config(&self) -> trace::TraceConfig {
        trace::TraceConfig::from_options(self.verbose, self.trace_categories.as_deref())
    }
}

pub fn run() -> Result<()> {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_else(|| OsString::from("winedroid"));
    let (options, args) = parse_global_options(&program, args.collect())?;
    trace::configure(options.trace_config());

    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        return Err(WinedroidError::new(usage(&program)));
    };

    if command == OsStr::new("inspect") {
        return run_inspect(&program, args.collect());
    }

    if command == OsStr::new("runtime-check") {
        return run_runtime_check(&program, args.collect());
    }

    if command == OsStr::new("art-probe") {
        return run_art_probe(&program, args.collect());
    }

    if command == OsStr::new("dex-run") {
        return run_dex_run(&program, args.collect());
    }

    if command == OsStr::new("install") {
        return run_install(&program, args.collect());
    }

    if command == OsStr::new("list") {
        return run_list(&program, args.collect());
    }

    if command == OsStr::new("uninstall") {
        return run_uninstall(&program, args.collect());
    }

    if command == OsStr::new("launch") {
        return run_launch_installed(&program, args.collect());
    }

    run_launch(&program, command, args.collect())
}

fn parse_global_options(
    program: &OsStr,
    args: Vec<OsString>,
) -> Result<(GlobalOptions, Vec<OsString>)> {
    let mut options = GlobalOptions::default();
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];

        if arg == OsStr::new("--") {
            return Ok((options, args[index + 1..].to_vec()));
        }

        if arg == OsStr::new("--verbose") || arg == OsStr::new("-v") {
            options.verbose = true;
            index += 1;
            continue;
        }

        if arg == OsStr::new("--trace") {
            index += 1;
            let Some(value) = args.get(index) else {
                return Err(WinedroidError::new(format!(
                    "--trace expects a comma-separated category list\n\n{}",
                    usage(program)
                )));
            };
            options.trace_categories = Some(trace_argument_value(value)?);
            index += 1;
            continue;
        }

        if let Some(value) = trace_equals_value(arg)? {
            options.trace_categories = Some(value);
            index += 1;
            continue;
        }

        return Ok((options, args[index..].to_vec()));
    }

    Ok((options, Vec::new()))
}

fn trace_equals_value(arg: &OsStr) -> Result<Option<String>> {
    let Some(value) = arg.to_str() else {
        return Ok(None);
    };

    let Some(value) = value.strip_prefix("--trace=") else {
        return Ok(None);
    };

    Ok(Some(value.to_owned()))
}

fn trace_argument_value(value: &OsStr) -> Result<String> {
    let Some(value) = value.to_str() else {
        return Err(WinedroidError::new("--trace value is not valid UTF-8"));
    };

    Ok(value.to_owned())
}

fn run_inspect(program: &OsStr, args: Vec<OsString>) -> Result<()> {
    if args.len() != 1 {
        return Err(WinedroidError::new(format!(
            "inspect expects exactly one APK path\n\n{}",
            usage(program)
        )));
    }

    let apk_path = PathBuf::from(&args[0]);
    let report = inspect::inspect_apk(&apk_path)?;
    println!("{report}");

    Ok(())
}

fn run_runtime_check(program: &OsStr, args: Vec<OsString>) -> Result<()> {
    if !args.is_empty() {
        return Err(WinedroidError::new(format!(
            "runtime-check expects no arguments\n\n{}",
            usage(program)
        )));
    }

    let report = runtime_check::RuntimeCheckReport::detect();
    println!("{report}");

    Ok(())
}

fn run_art_probe(program: &OsStr, args: Vec<OsString>) -> Result<()> {
    if !args.is_empty() {
        return Err(WinedroidError::new(format!(
            "art-probe expects no arguments\n\n{}",
            usage(program)
        )));
    }

    let report = art_probe::ArtProbeReport::detect();
    println!("{report}");
    art_probe::ensure_probe_can_run(&report)
}

fn run_dex_run(program: &OsStr, args: Vec<OsString>) -> Result<()> {
    if args.len() != 3 {
        return Err(WinedroidError::new(format!(
            "dex-run expects an APK path, class name, and method name\n\n{}",
            usage(program)
        )));
    }

    let apk_path = PathBuf::from(&args[0]);
    let Some(class_name) = args[1].to_str() else {
        return Err(WinedroidError::new("dex-run class name is not valid UTF-8"));
    };
    let Some(method_name) = args[2].to_str() else {
        return Err(WinedroidError::new(
            "dex-run method name is not valid UTF-8",
        ));
    };

    dex_run::run_dex_method(&apk_path, class_name, method_name)
}

fn run_install(program: &OsStr, args: Vec<OsString>) -> Result<()> {
    if args.is_empty() {
        return Err(WinedroidError::new(format!(
            "install expects a base APK path and optional split APK paths\n\n{}",
            usage(program)
        )));
    }

    let base_apk = PathBuf::from(&args[0]);
    let split_apks: Vec<PathBuf> = args[1..].iter().map(PathBuf::from).collect();
    launch::install_set(&base_apk, &split_apks)
}

fn run_list(program: &OsStr, args: Vec<OsString>) -> Result<()> {
    if !args.is_empty() {
        return Err(WinedroidError::new(format!(
            "list expects no arguments\n\n{}",
            usage(program)
        )));
    }

    let prefix = WinedroidPrefix::default()?;
    let installed = install::list_installed(&prefix)?;

    if installed.is_empty() {
        println!("no packages installed");
        return Ok(());
    }

    for artifact in &installed {
        println!("{}", artifact.package_name);
        println!(
            "  launch activity: {}",
            artifact
                .launch_activity
                .as_deref()
                .unwrap_or("none (headless/multi-launcher)")
        );
        println!("  source: {}", artifact.source_apk.display());
        println!("  size: {} bytes", artifact.source_size);
    }

    Ok(())
}

fn run_uninstall(program: &OsStr, args: Vec<OsString>) -> Result<()> {
    let package_name = single_package_argument(program, "uninstall", &args)?;
    let prefix = WinedroidPrefix::default()?;
    install::uninstall(&prefix, package_name)?;
    println!("uninstalled {package_name}");

    Ok(())
}

fn run_launch_installed(program: &OsStr, args: Vec<OsString>) -> Result<()> {
    if args.len() == 3 && args[0] == OsStr::new("--activity") {
        let activity = args[1]
            .to_str()
            .ok_or_else(|| WinedroidError::new("launch activity name is not valid UTF-8"))?;
        let package_name = args[2]
            .to_str()
            .ok_or_else(|| WinedroidError::new("launch package name is not valid UTF-8"))?;
        return launch::launch_installed_activity(package_name, Some(activity));
    }
    let package_name = single_package_argument(program, "launch", &args)?;
    launch::launch_installed(package_name)
}

fn single_package_argument<'a>(
    program: &OsStr,
    command: &str,
    args: &'a [OsString],
) -> Result<&'a str> {
    if args.len() != 1 {
        return Err(WinedroidError::new(format!(
            "{command} expects exactly one package name\n\n{}",
            usage(program)
        )));
    }

    args[0]
        .to_str()
        .ok_or_else(|| WinedroidError::new(format!("{command} package name is not valid UTF-8")))
}

fn run_launch(program: &OsStr, apk_path: OsString, args: Vec<OsString>) -> Result<()> {
    if !args.is_empty() {
        return Err(WinedroidError::new(format!(
            "launch expects exactly one APK path\n\n{}",
            usage(program)
        )));
    }

    launch::launch_apk(&PathBuf::from(apk_path))
}

fn usage(program: &OsStr) -> String {
    let program_name = Path::new(program)
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("winedroid");

    format!(
        "usage: {program_name} [--verbose] [--trace categories] inspect <apk>\n       {program_name} [--verbose] [--trace categories] runtime-check\n       {program_name} [--verbose] [--trace categories] art-probe\n       {program_name} [--verbose] [--trace categories] dex-run <apk> <class> <method>\n       {program_name} [--verbose] [--trace categories] install <apk>\n       {program_name} [--verbose] [--trace categories] list\n       {program_name} [--verbose] [--trace categories] launch <package>\n       {program_name} [--verbose] [--trace categories] uninstall <package>\n       {program_name} [--verbose] [--trace categories] <apk>"
    )
}
