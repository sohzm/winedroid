use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::error::{Result, WinedroidError};
use crate::prefix::WinedroidPrefix;

const DEFAULT_CONFIG_JSON: &str = "{\n  \"ui_mode\": \"system\",\n  \"frame_rate\": \"host\",\n  \"display\": {\n    \"width\": 420,\n    \"height\": 760\n  }\n}\n";

const UI_MODE_TYPE_NORMAL: i32 = 0x01;
const UI_MODE_NIGHT_NO: i32 = 0x10;
const UI_MODE_NIGHT_YES: i32 = 0x20;
const DEFAULT_DISPLAY_WIDTH: u32 = 420;
const DEFAULT_DISPLAY_HEIGHT: u32 = 760;
const MAX_DISPLAY_DIMENSION: u32 = 4096;
const MAX_DISPLAY_PIXELS: u64 = 2_097_152;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DisplaySize {
    width: u32,
    height: u32,
}

impl DisplaySize {
    pub(crate) fn new(width: u32, height: u32) -> Result<Self> {
        validate_display_dimension("width", width)?;
        validate_display_dimension("height", height)?;

        let pixels = u64::from(width) * u64::from(height);
        if pixels > MAX_DISPLAY_PIXELS {
            return Err(WinedroidError::new(format!(
                "display area must not exceed {MAX_DISPLAY_PIXELS} logical pixels, got {width}x{height} ({pixels} pixels)"
            )));
        }

        Ok(Self { width, height })
    }

    fn parse(value: &Value) -> Result<Self> {
        let Some(object) = value.as_object() else {
            return Err(WinedroidError::new("display must be an object"));
        };

        for key in object.keys() {
            if key != "width" && key != "height" {
                return Err(WinedroidError::new(format!(
                    "unsupported display key `{key}`"
                )));
            }
        }

        let width = parse_display_dimension(object.get("width"), "width")?;
        let height = parse_display_dimension(object.get("height"), "height")?;
        Self::new(width, height)
    }

    pub(crate) fn width(self) -> u32 {
        self.width
    }

    pub(crate) fn height(self) -> u32 {
        self.height
    }

    pub(crate) fn width_i32(self) -> i32 {
        self.width as i32
    }

    pub(crate) fn height_i32(self) -> i32 {
        self.height as i32
    }
}

impl Default for DisplaySize {
    fn default() -> Self {
        Self {
            width: DEFAULT_DISPLAY_WIDTH,
            height: DEFAULT_DISPLAY_HEIGHT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WinedroidConfig {
    ui_mode: WinedroidUiMode,
    frame_rate: WinedroidFrameRate,
    display_size: DisplaySize,
    signature_spoofing: bool,
}

impl Default for WinedroidConfig {
    fn default() -> Self {
        Self {
            ui_mode: WinedroidUiMode::System,
            frame_rate: WinedroidFrameRate::Host,
            display_size: DisplaySize::default(),
            signature_spoofing: false,
        }
    }
}

impl WinedroidConfig {
    pub(crate) fn load_or_create(prefix: &WinedroidPrefix) -> Result<Self> {
        let path = prefix.config_path();

        if !path.exists() {
            create_default_config(prefix.root(), &path)?;
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path).map_err(|error| {
            WinedroidError::with_source(
                format!("failed to read Winedroid config `{}`", path.display()),
                error,
            )
        })?;

        Self::parse(&contents).map_err(|error| {
            WinedroidError::new(format!(
                "failed to parse Winedroid config `{}`\n{error}",
                path.display()
            ))
        })
    }

    #[cfg(test)]
    pub(crate) fn with_ui_mode(ui_mode: WinedroidUiMode) -> Self {
        Self {
            ui_mode,
            ..Self::default()
        }
    }

    #[cfg(test)]
    pub(crate) fn with_frame_rate(frame_rate: WinedroidFrameRate) -> Self {
        Self {
            frame_rate,
            ..Self::default()
        }
    }

    #[cfg(test)]
    pub(crate) fn with_display_size(width: u32, height: u32) -> Self {
        Self {
            display_size: DisplaySize::new(width, height)
                .expect("test display size should be valid"),
            ..Self::default()
        }
    }

    pub(crate) fn override_display_size(mut self, display_size: DisplaySize) -> Self {
        self.display_size = display_size;
        self
    }

    pub(crate) fn android_configuration_ui_mode(&self) -> i32 {
        self.ui_mode.android_configuration_ui_mode()
    }

    pub(crate) fn frame_rate(&self) -> WinedroidFrameRate {
        self.frame_rate
    }

    pub(crate) fn display_size(&self) -> DisplaySize {
        self.display_size
    }

    pub(crate) fn signature_spoofing(&self) -> bool {
        self.signature_spoofing
    }

    fn parse(contents: &str) -> Result<Self> {
        let value: Value = serde_json::from_str(contents)
            .map_err(|error| WinedroidError::with_source("config is not valid JSON", error))?;
        let Some(object) = value.as_object() else {
            return Err(WinedroidError::new("config root must be a JSON object"));
        };

        for key in object.keys() {
            if key != "ui_mode"
                && key != "frame_rate"
                && key != "display"
                && key != "signature_spoofing"
            {
                return Err(WinedroidError::new(format!(
                    "unsupported config key `{key}`"
                )));
            }
        }

        let ui_mode = match object.get("ui_mode") {
            Some(value) => WinedroidUiMode::parse(value)?,
            None => WinedroidUiMode::System,
        };

        let frame_rate = match object.get("frame_rate") {
            Some(value) => WinedroidFrameRate::parse(value)?,
            None => WinedroidFrameRate::Host,
        };

        let display_size = match object.get("display") {
            Some(value) => DisplaySize::parse(value)?,
            None => DisplaySize::default(),
        };

        let signature_spoofing = match object.get("signature_spoofing") {
            Some(Value::Bool(enabled)) => *enabled,
            Some(_) => {
                return Err(WinedroidError::new(
                    "signature_spoofing must be a JSON boolean",
                ));
            }
            None => false,
        };

        Ok(Self {
            ui_mode,
            frame_rate,
            display_size,
            signature_spoofing,
        })
    }
}

fn parse_display_dimension(value: Option<&Value>, name: &str) -> Result<u32> {
    let Some(value) = value else {
        return Err(WinedroidError::new(format!(
            "display.{name} is required when display is configured"
        )));
    };
    let Some(value) = value.as_u64() else {
        return Err(WinedroidError::new(format!(
            "display.{name} must be an integer from 1 through {MAX_DISPLAY_DIMENSION}"
        )));
    };
    if value == 0 || value > u64::from(MAX_DISPLAY_DIMENSION) {
        return Err(WinedroidError::new(format!(
            "display.{name} must be an integer from 1 through {MAX_DISPLAY_DIMENSION}, got {value}"
        )));
    }

    Ok(value as u32)
}

fn validate_display_dimension(name: &str, value: u32) -> Result<()> {
    if value == 0 || value > MAX_DISPLAY_DIMENSION {
        return Err(WinedroidError::new(format!(
            "display.{name} must be an integer from 1 through {MAX_DISPLAY_DIMENSION}, got {value}"
        )));
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WinedroidUiMode {
    System,
    Light,
    Dark,
}

impl WinedroidUiMode {
    fn parse(value: &Value) -> Result<Self> {
        let Some(value) = value.as_str() else {
            return Err(WinedroidError::new("ui_mode must be a string"));
        };

        match value {
            "system" => Ok(Self::System),
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            _ => Err(WinedroidError::new(format!(
                "ui_mode must be `system`, `light`, or `dark`, got `{value}`"
            ))),
        }
    }

    fn android_configuration_ui_mode(self) -> i32 {
        match self {
            // cite: frameworks-base UiModeManager.java#MODE_NIGHT_NO
            Self::System => UI_MODE_TYPE_NORMAL | UI_MODE_NIGHT_NO,
            Self::Light => UI_MODE_TYPE_NORMAL | UI_MODE_NIGHT_NO,
            Self::Dark => UI_MODE_TYPE_NORMAL | UI_MODE_NIGHT_YES,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WinedroidFrameRate {
    Host,
    Fixed { frames_per_second: u32 },
}

impl WinedroidFrameRate {
    fn parse(value: &Value) -> Result<Self> {
        if let Some(text) = value.as_str() {
            return match text {
                "host" => Ok(Self::Host),
                _ => Err(WinedroidError::new(format!(
                    "frame_rate string must be `host`, got `{text}`"
                ))),
            };
        }

        if let Some(number) = value.as_u64() {
            if number == 0 || number > u64::from(u32::MAX) {
                return Err(WinedroidError::new(format!(
                    "frame_rate must be a positive frames-per-second value, got `{number}`"
                )));
            }
            return Ok(Self::Fixed {
                frames_per_second: number as u32,
            });
        }

        Err(WinedroidError::new(
            "frame_rate must be the string `host` or a positive integer frames-per-second value",
        ))
    }
}

fn create_default_config(prefix_root: &Path, path: &Path) -> Result<()> {
    fs::create_dir_all(prefix_root).map_err(|error| {
        WinedroidError::with_source(
            format!(
                "failed to create Winedroid prefix `{}`",
                prefix_root.display()
            ),
            error,
        )
    })?;
    fs::write(path, DEFAULT_CONFIG_JSON).map_err(|error| {
        WinedroidError::with_source(
            format!(
                "failed to write default Winedroid config `{}`",
                path.display()
            ),
            error,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn missing_config_file_is_created_with_default_runtime_settings() {
        let root = unique_test_dir("missing");
        let prefix = WinedroidPrefix::from_root(root.clone());

        let config = WinedroidConfig::load_or_create(&prefix).expect("config should load");

        assert_eq!(config, WinedroidConfig::default());
        assert_eq!(
            config.android_configuration_ui_mode(),
            UI_MODE_TYPE_NORMAL | UI_MODE_NIGHT_NO
        );
        assert_eq!(config.display_size(), DisplaySize::default());
        assert_eq!(
            fs::read_to_string(prefix.config_path()).expect("default config should exist"),
            DEFAULT_CONFIG_JSON
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn parses_light_and_dark_ui_modes() {
        let light =
            WinedroidConfig::parse("{\"ui_mode\":\"light\"}").expect("light config should parse");
        let dark =
            WinedroidConfig::parse("{\"ui_mode\":\"dark\"}").expect("dark config should parse");

        assert_eq!(light.android_configuration_ui_mode(), 0x11);
        assert_eq!(dark.android_configuration_ui_mode(), 0x21);
    }

    #[test]
    fn rejects_unknown_config_keys() {
        let error = WinedroidConfig::parse("{\"ui_mode\":\"dark\",\"package\":\"x\"}")
            .expect_err("unknown key should fail");

        assert!(
            error.to_string().contains("unsupported config key"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn frame_rate_defaults_to_host_when_absent() {
        let config =
            WinedroidConfig::parse("{\"ui_mode\":\"system\"}").expect("config should parse");

        assert_eq!(config.frame_rate(), WinedroidFrameRate::Host);
    }

    #[test]
    fn display_size_defaults_when_absent_and_parses_atomic_custom_size() {
        let default =
            WinedroidConfig::parse("{\"ui_mode\":\"system\"}").expect("legacy config should parse");
        let custom = WinedroidConfig::parse("{\"display\":{\"width\":420,\"height\":910}}")
            .expect("custom display should parse");

        assert_eq!(default.display_size(), DisplaySize::default());
        assert_eq!(custom.display_size(), DisplaySize::new(420, 910).unwrap());
    }

    #[test]
    fn display_size_rejects_partial_wrong_type_zero_unknown_and_excessive_area() {
        for invalid in [
            "{\"display\":null}",
            "{\"display\":{\"width\":420}}",
            "{\"display\":{\"height\":910}}",
            "{\"display\":{\"width\":420,\"height\":\"910\"}}",
            "{\"display\":{\"width\":420.0,\"height\":910}}",
            "{\"display\":{\"width\":420.5,\"height\":910}}",
            "{\"display\":{\"width\":-420,\"height\":910}}",
            "{\"display\":{\"width\":true,\"height\":910}}",
            "{\"display\":{\"width\":0,\"height\":910}}",
            "{\"display\":{\"width\":4097,\"height\":1}}",
            "{\"display\":{\"width\":420,\"height\":910,\"density\":1}}",
            "{\"display\":{\"width\":2048,\"height\":2048}}",
        ] {
            let error = WinedroidConfig::parse(invalid)
                .expect_err(&format!("display config `{invalid}` should be rejected"));
            assert!(
                error.to_string().contains("display"),
                "unexpected error for `{invalid}`: {error}"
            );
        }
    }

    #[test]
    fn display_size_accepts_dimension_and_area_boundaries() {
        assert_eq!(
            DisplaySize::new(4096, 512).expect("maximum display area should be valid"),
            DisplaySize {
                width: 4096,
                height: 512,
            }
        );
        assert!(DisplaySize::new(4096, 513).is_err());
        assert!(DisplaySize::new(1, 4096).is_ok());
    }

    #[test]
    fn parses_host_and_fixed_frame_rates() {
        let host = WinedroidConfig::parse("{\"frame_rate\":\"host\"}")
            .expect("host frame_rate should parse");
        let fixed =
            WinedroidConfig::parse("{\"frame_rate\":120}").expect("fixed frame_rate should parse");

        assert_eq!(host.frame_rate(), WinedroidFrameRate::Host);
        assert_eq!(
            fixed.frame_rate(),
            WinedroidFrameRate::Fixed {
                frames_per_second: 120
            }
        );
    }

    #[test]
    fn rejects_invalid_frame_rates() {
        for invalid in [
            "{\"frame_rate\":0}",
            "{\"frame_rate\":-1}",
            "{\"frame_rate\":59.5}",
            "{\"frame_rate\":\"fast\"}",
            "{\"frame_rate\":true}",
        ] {
            let error = WinedroidConfig::parse(invalid)
                .expect_err(&format!("frame_rate `{invalid}` should be rejected"));
            assert!(
                error.to_string().contains("frame_rate"),
                "unexpected error for `{invalid}`: {error}"
            );
        }
    }

    #[test]
    fn parses_opt_in_signature_spoofing_and_defaults_false() {
        let default_cfg = WinedroidConfig::parse("{}").expect("empty config should parse");
        assert!(!default_cfg.signature_spoofing());

        let enabled_cfg = WinedroidConfig::parse("{\"signature_spoofing\":true}")
            .expect("signature_spoofing boolean should parse");
        assert!(enabled_cfg.signature_spoofing());

        let err = WinedroidConfig::parse("{\"signature_spoofing\":\"yes\"}")
            .expect_err("non-boolean signature_spoofing should be rejected");
        assert!(err.to_string().contains("signature_spoofing"));
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock should be after unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!(
            "winedroid-config-test-{name}-{}-{nanos}",
            std::process::id()
        ))
    }
}
