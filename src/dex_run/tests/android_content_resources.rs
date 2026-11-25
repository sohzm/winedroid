use super::super::*;
use super::helpers::*;

#[test]
// cite: frameworks-base Resources.java#getIdentifier
// cite: frameworks-base public-final.xml#content, home
fn builtin_get_identifier_resolves_android_framework_ids() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "getIdentifier".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ILLL".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec![
                "Ljava/lang/String;".to_owned(),
                "Ljava/lang/String;".to_owned(),
                "Ljava/lang/String;".to_owned(),
            ],
        },
    };
    let identifier = |name: &str, def_type: &str, def_package: &str| {
        program.invoke_builtin_method(
            &method,
            Some(&test_invocation(vec![
                DexValue::Reference(std::rc::Rc::from(
                    "Landroid/content/res/Resources;.bootstrap",
                )),
                DexValue::String(std::rc::Rc::from(name.to_owned())),
                DexValue::String(std::rc::Rc::from(def_type.to_owned())),
                DexValue::String(std::rc::Rc::from(def_package.to_owned())),
            ])),
        )
    };

    assert_eq!(
        identifier("action_bar", "id", "android"),
        Some(DexExecutionResult::Int(ANDROID_ACTION_BAR_VIEW_ID))
    );
    assert_eq!(
        identifier("content", "id", "android"),
        Some(DexExecutionResult::Int(0x0102_0002))
    );
    assert_eq!(
        identifier("android:id/home", "id", ""),
        Some(DexExecutionResult::Int(0x0102_002c))
    );
    assert_eq!(
        identifier("does_not_exist", "id", "android"),
        Some(DexExecutionResult::Int(0))
    );
}

#[test]
fn builtin_application_get_process_name_returns_bootstrap_process() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/app/Application;".to_owned(),
        name: "getProcessName".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: Vec::new(),
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&method, None),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from(BOOTSTRAP_PROCESS_NAME.to_owned())
        )))
    );
}

#[test]
fn builtin_application_get_process_name_returns_loaded_package() {
    let program = DexProgram::with_package_name(
        Vec::new(),
        Some("com.darkempire78.opencalculator".to_owned()),
    );
    let method = DexMethodId {
        class_descriptor: "Landroid/app/Application;".to_owned(),
        name: "getProcessName".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: Vec::new(),
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&method, None),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from("com.darkempire78.opencalculator")
        )))
    );
}

#[test]
fn builtin_context_get_package_name_returns_loaded_package() {
    let program = DexProgram::with_package_name(
        Vec::new(),
        Some("com.darkempire78.opencalculator".to_owned()),
    );
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getPackageName".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let invocation = test_invocation(vec![DexValue::Reference(std::rc::Rc::from("context"))]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from("com.darkempire78.opencalculator")
        )))
    );
}

#[test]
fn builtin_context_check_permission_returns_granted() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "checkPermission".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ILII".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec![
                "Ljava/lang/String;".to_owned(),
                "I".to_owned(),
                "I".to_owned(),
            ],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("context")),
        DexValue::String(std::rc::Rc::from("android.permission.ACCESS_NETWORK_STATE")),
        DexValue::Int(1000),
        DexValue::Int(10000),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Int(ANDROID_PERMISSION_GRANTED))
    );
}

#[test]
fn builtin_context_check_permission_uses_external_storage_runtime_grants() {
    // cite: frameworks-base ContextImpl.java#checkPermission, checkSelfPermission
    let mut program = DexProgram::new(Vec::new());
    program.external_storage_policy = ExternalStoragePolicy::new(
        34,
        34,
        vec![crate::external_storage::READ_MEDIA_IMAGES.to_owned()],
        crate::external_storage::ExternalStorageRuntimeAccess::new(
            vec![crate::external_storage::READ_MEDIA_IMAGES.to_owned()],
            false,
            false,
        ),
    );
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "checkPermission".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ILII".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec![
                "Ljava/lang/String;".to_owned(),
                "I".to_owned(),
                "I".to_owned(),
            ],
        },
    };

    let result = |permission: &str| {
        let invocation = test_invocation(vec![
            DexValue::Reference(std::rc::Rc::from("context")),
            DexValue::String(std::rc::Rc::from(permission)),
            DexValue::Int(1000),
            DexValue::Int(10000),
        ]);
        program.invoke_builtin_method(&method, Some(&invocation))
    };

    assert_eq!(
        result(crate::external_storage::READ_MEDIA_IMAGES),
        Some(DexExecutionResult::Int(ANDROID_PERMISSION_GRANTED))
    );
    assert_eq!(
        result(crate::external_storage::READ_MEDIA_VIDEO),
        Some(DexExecutionResult::Int(ANDROID_PERMISSION_DENIED))
    );
    assert_eq!(
        result(crate::external_storage::MANAGE_EXTERNAL_STORAGE),
        Some(DexExecutionResult::Int(ANDROID_PERMISSION_DENIED))
    );
    assert_eq!(
        result(crate::external_storage::MANAGE_MEDIA),
        Some(DexExecutionResult::Int(ANDROID_PERMISSION_DENIED))
    );
    assert_eq!(
        result("android.permission.ACCESS_NETWORK_STATE"),
        Some(DexExecutionResult::Int(ANDROID_PERMISSION_GRANTED))
    );
    let foreign_process = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("context")),
        DexValue::String(std::rc::Rc::from(
            crate::external_storage::READ_MEDIA_IMAGES,
        )),
        DexValue::Int(BOOTSTRAP_PROCESS_ID + 1),
        DexValue::Int(BOOTSTRAP_PROCESS_UID),
    ]);
    assert_eq!(
        program.invoke_builtin_method(&method, Some(&foreign_process)),
        None
    );
}

#[test]
fn parsed_manifest_context_drives_application_target_and_storage_permission_checks() {
    // cite: frameworks-base PackageParser.java#parseBaseApkCommon, parseBaseApplication, parseUsesPermission
    // cite: frameworks-base ApplicationInfo.java#targetSdkVersion
    // cite: frameworks-base ContextImpl.java#checkPermission
    // cite: frameworks-base StorageManager.java#checkExternalStoragePermissionAndAppOp
    let manifest = crate::manifest::AndroidManifest {
        package_name: Some("com.example.storage".to_owned()),
        min_sdk_version: Some("22".to_owned()),
        target_sdk_version: Some("22".to_owned()),
        uses_sdk_declared: true,
        permissions: vec![
            crate::manifest::UsesPermission {
                name: crate::external_storage::READ_EXTERNAL_STORAGE.to_owned(),
                max_sdk_version: None,
                required_feature: None,
                required_not_feature: None,
            },
            crate::manifest::UsesPermission {
                name: crate::external_storage::WRITE_EXTERNAL_STORAGE.to_owned(),
                max_sdk_version: Some(34),
                required_feature: None,
                required_not_feature: None,
            },
            crate::manifest::UsesPermission {
                name: crate::external_storage::READ_MEDIA_IMAGES.to_owned(),
                max_sdk_version: None,
                required_feature: Some("android.hardware.camera".to_owned()),
                required_not_feature: None,
            },
        ],
        ..crate::manifest::AndroidManifest::default()
    };
    let context = ApkManifestContext::from_manifest(manifest)
        .expect("manifest runtime context should resolve");
    let mut program = DexProgram::new(Vec::new());
    program.apply_apk_manifest_context(context);
    let target_field = DexFieldId {
        class_descriptor: "Landroid/content/pm/ApplicationInfo;".to_owned(),
        type_descriptor: "I".to_owned(),
        name: "targetSdkVersion".to_owned(),
    };
    let check_permission = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "checkPermission".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ILII".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec![
                "Ljava/lang/String;".to_owned(),
                "I".to_owned(),
                "I".to_owned(),
            ],
        },
    };
    let check = |permission: &str| {
        program.invoke_builtin_method(
            &check_permission,
            Some(&test_invocation(vec![
                DexValue::Reference(std::rc::Rc::from("context")),
                DexValue::String(std::rc::Rc::from(permission)),
                DexValue::Int(BOOTSTRAP_PROCESS_ID),
                DexValue::Int(BOOTSTRAP_PROCESS_UID),
            ])),
        )
    };

    assert_eq!(program.target_sdk_version, 22);
    assert_eq!(
        program.read_bootstrap_instance_field(
            &DexValue::Reference(std::rc::Rc::from(
                "Landroid/content/pm/ApplicationInfo;.bootstrap",
            )),
            &target_field,
        ),
        Some(DexValue::Int(22))
    );
    assert_eq!(
        check(crate::external_storage::READ_EXTERNAL_STORAGE),
        Some(DexExecutionResult::Int(ANDROID_PERMISSION_GRANTED))
    );
    assert_eq!(
        check(crate::external_storage::WRITE_EXTERNAL_STORAGE),
        Some(DexExecutionResult::Int(ANDROID_PERMISSION_DENIED))
    );
    assert_eq!(
        check(crate::external_storage::READ_MEDIA_IMAGES),
        Some(DexExecutionResult::Int(ANDROID_PERMISSION_DENIED))
    );
    assert!(!program
        .external_storage_policy
        .allows_public_path(crate::external_storage::ExternalStorageAccess::Read));
}

#[test]
fn builtin_color_state_list_value_of_stores_default_color() {
    let program = DexProgram::new(Vec::new());
    let value_of = DexMethodId {
        class_descriptor: "Landroid/content/res/ColorStateList;".to_owned(),
        name: "valueOf".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Landroid/content/res/ColorStateList;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let default_color = DexMethodId {
        class_descriptor: "Landroid/content/res/ColorStateList;".to_owned(),
        name: "getDefaultColor".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "I".to_owned(),
            return_type: "I".to_owned(),
            parameters: Vec::new(),
        },
    };
    let Some(DexExecutionResult::Value(color_state_list)) =
        program.invoke_builtin_method(&value_of, Some(&test_invocation(vec![DexValue::Int(-32)])))
    else {
        panic!("ColorStateList.valueOf should allocate a color state list");
    };

    assert_eq!(
        program.invoke_builtin_method(
            &default_color,
            Some(&test_invocation(vec![color_state_list]))
        ),
        Some(DexExecutionResult::Int(-32))
    );
}

#[test]
fn builtin_context_get_content_resolver_returns_bootstrap_resolver() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getContentResolver".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/content/ContentResolver;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let invocation = test_invocation(vec![DexValue::Reference(std::rc::Rc::from("context"))]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/content/ContentResolver;.bootstrap")
        )))
    );
}

#[test]
fn builtin_content_resolver_register_content_observer_is_noop() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/ContentResolver;".to_owned(),
        name: "registerContentObserver".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VLZL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec![
                "Landroid/net/Uri;".to_owned(),
                "Z".to_owned(),
                "Landroid/database/ContentObserver;".to_owned(),
            ],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from(
            "Landroid/content/ContentResolver;.bootstrap",
        )),
        DexValue::Object(DexObjectRef {
            id: 12,
            class_descriptor: std::rc::Rc::from("Landroid/net/Uri;"),
        }),
        DexValue::Int(0),
        DexValue::Object(DexObjectRef {
            id: 13,
            class_descriptor: std::rc::Rc::from("Landroid/database/ContentObserver;"),
        }),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Void)
    );
}

#[test]
fn builtin_settings_global_get_uri_for_returns_settings_uri() {
    let program = DexProgram::new(Vec::new());
    let get_uri_for = DexMethodId {
        class_descriptor: "Landroid/provider/Settings$Global;".to_owned(),
        name: "getUriFor".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Landroid/net/Uri;".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let to_string = DexMethodId {
        class_descriptor: "Landroid/net/Uri;".to_owned(),
        name: "toString".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: Vec::new(),
        },
    };

    let Some(DexExecutionResult::Value(uri)) = program.invoke_builtin_method(
        &get_uri_for,
        Some(&test_invocation(vec![DexValue::String(std::rc::Rc::from(
            "animator_duration_scale",
        ))])),
    ) else {
        panic!("Settings.Global.getUriFor should return a Uri");
    };

    assert_eq!(
        program.invoke_builtin_method(&to_string, Some(&test_invocation(vec![uri]))),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from("content://settings/global/animator_duration_scale")
        )))
    );
}

#[test]
fn builtin_settings_global_get_float_returns_default_value() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/provider/Settings$Global;".to_owned(),
        name: "getFloat".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "FLLF".to_owned(),
            return_type: "F".to_owned(),
            parameters: vec![
                "Landroid/content/ContentResolver;".to_owned(),
                "Ljava/lang/String;".to_owned(),
                "F".to_owned(),
            ],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from(
            "Landroid/content/ContentResolver;.bootstrap",
        )),
        DexValue::String(std::rc::Rc::from("animator_duration_scale")),
        DexValue::Int(1.0_f32.to_bits() as i32),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::Float(1.0)))
    );
}

#[test]
fn builtin_context_obtain_styled_attributes_returns_typed_array() {
    let program = DexProgram::new(Vec::new());
    let obtain = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "obtainStyledAttributes".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Landroid/content/res/TypedArray;".to_owned(),
            parameters: vec!["[I".to_owned()],
        },
    };
    let obtain_attribute_set = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "obtainStyledAttributes".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LLL".to_owned(),
            return_type: "Landroid/content/res/TypedArray;".to_owned(),
            parameters: vec!["Landroid/util/AttributeSet;".to_owned(), "[I".to_owned()],
        },
    };
    let obtain_attribute_set_with_style = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "obtainStyledAttributes".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LLLII".to_owned(),
            return_type: "Landroid/content/res/TypedArray;".to_owned(),
            parameters: vec![
                "Landroid/util/AttributeSet;".to_owned(),
                "[I".to_owned(),
                "I".to_owned(),
                "I".to_owned(),
            ],
        },
    };
    let theme_obtain = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources$Theme;".to_owned(),
        name: "obtainStyledAttributes".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Landroid/content/res/TypedArray;".to_owned(),
            parameters: vec!["[I".to_owned()],
        },
    };
    let theme_obtain_attribute_set_with_style = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources$Theme;".to_owned(),
        name: "obtainStyledAttributes".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LLLII".to_owned(),
            return_type: "Landroid/content/res/TypedArray;".to_owned(),
            parameters: vec![
                "Landroid/util/AttributeSet;".to_owned(),
                "[I".to_owned(),
                "I".to_owned(),
                "I".to_owned(),
            ],
        },
    };
    let has_value = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "hasValue".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZI".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_boolean = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getBoolean".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZIZ".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec!["I".to_owned(), "Z".to_owned()],
        },
    };
    let get_index_count = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getIndexCount".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "I".to_owned(),
            return_type: "I".to_owned(),
            parameters: Vec::new(),
        },
    };
    let get_index = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getIndex".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "II".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_changing_configurations = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getChangingConfigurations".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "I".to_owned(),
            return_type: "I".to_owned(),
            parameters: Vec::new(),
        },
    };
    let get_resources = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getResources".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/content/res/Resources;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let get_value = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getValue".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZIL".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec!["I".to_owned(), "Landroid/util/TypedValue;".to_owned()],
        },
    };
    let get_resource_id = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getResourceId".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "III".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned(), "I".to_owned()],
        },
    };
    let get_int = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getInt".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "III".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned(), "I".to_owned()],
        },
    };
    let get_dimension_pixel_offset = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getDimensionPixelOffset".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "III".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned(), "I".to_owned()],
        },
    };
    let get_dimension_pixel_size = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getDimensionPixelSize".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "III".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned(), "I".to_owned()],
        },
    };
    let get_layout_dimension = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getLayoutDimension".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "III".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned(), "I".to_owned()],
        },
    };
    let get_text = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getText".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Ljava/lang/CharSequence;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_string = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getString".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_text_array = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getTextArray".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "[Ljava/lang/CharSequence;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_color_state_list = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getColorStateList".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Landroid/content/res/ColorStateList;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_drawable = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getDrawable".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Landroid/graphics/drawable/Drawable;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let recycle = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "recycle".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("context")),
        DexValue::Reference(std::rc::Rc::from("[I:appcompat-window")),
    ]);
    let typed_array = DexValue::Reference(std::rc::Rc::from(
        "Landroid/content/res/TypedArray;:reference:[I:appcompat-window",
    ));
    let theme_typed_array = DexValue::Reference(std::rc::Rc::from(
        "Landroid/content/res/TypedArray;:reference:[I:theme-window",
    ));
    let theme_styled_attribute_set_typed_array = DexValue::Reference(std::rc::Rc::from(
        "Landroid/content/res/TypedArray;:reference:attrs:reference:[I:theme-menu-item:int 7:int 0"
            .to_owned(),
    ));
    let attribute_set_typed_array = DexValue::Reference(std::rc::Rc::from(
        "Landroid/content/res/TypedArray;:reference:attrs:reference:[I:menu-item-styleable"
            .to_owned(),
    ));
    let styled_attribute_set_typed_array = DexValue::Reference(std::rc::Rc::from(
        "Landroid/content/res/TypedArray;:int:0:reference:[I:list-popup-styleable:int 7:int 0"
            .to_owned(),
    ));
    let attribute_set_invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("context")),
        DexValue::Reference(std::rc::Rc::from("attrs")),
        DexValue::Reference(std::rc::Rc::from("[I:menu-item-styleable")),
    ]);
    let styled_attribute_set_invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("context")),
        DexValue::Int(0),
        DexValue::Reference(std::rc::Rc::from("[I:list-popup-styleable")),
        DexValue::Int(7),
        DexValue::Int(0),
    ]);
    let theme_invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from(
            "Landroid/content/res/Resources$Theme;.bootstrap",
        )),
        DexValue::Reference(std::rc::Rc::from("[I:theme-window")),
    ]);
    let theme_styled_attribute_set_invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from(
            "Landroid/content/res/Resources$Theme;.bootstrap",
        )),
        DexValue::Reference(std::rc::Rc::from("attrs")),
        DexValue::Reference(std::rc::Rc::from("[I:theme-menu-item")),
        DexValue::Int(7),
        DexValue::Int(0),
    ]);
    let has_value_invocation = test_invocation(vec![typed_array.clone(), DexValue::Int(117)]);
    let get_action_bar_invocation = test_invocation(vec![
        typed_array.clone(),
        DexValue::Int(117),
        DexValue::Int(0),
    ]);
    let get_boolean_invocation = test_invocation(vec![
        typed_array.clone(),
        DexValue::Int(118),
        DexValue::Int(0),
    ]);
    let get_index_invocation = test_invocation(vec![typed_array.clone(), DexValue::Int(4)]);
    let get_value_invocation = test_invocation(vec![
        typed_array.clone(),
        DexValue::Int(3),
        DexValue::Reference(std::rc::Rc::from("Landroid/util/TypedValue;:out")),
    ]);
    let get_resource_id_invocation = test_invocation(vec![
        typed_array.clone(),
        DexValue::Int(0),
        DexValue::Int(2131296591),
    ]);
    let get_int_invocation = test_invocation(vec![
        typed_array.clone(),
        DexValue::Int(2),
        DexValue::Int(5),
    ]);
    let get_dimension_invocation = test_invocation(vec![
        typed_array.clone(),
        DexValue::Int(4),
        DexValue::Int(12),
    ]);
    let get_text_invocation = test_invocation(vec![typed_array.clone(), DexValue::Int(3)]);
    let get_string_invocation = test_invocation(vec![typed_array.clone(), DexValue::Int(3)]);
    let get_text_array_invocation = test_invocation(vec![typed_array.clone(), DexValue::Int(3)]);
    let get_color_state_list_invocation =
        test_invocation(vec![typed_array.clone(), DexValue::Int(3)]);
    let get_drawable_invocation = test_invocation(vec![typed_array.clone(), DexValue::Int(0)]);

    assert_eq!(
        program.invoke_builtin_method(&obtain, Some(&invocation)),
        Some(DexExecutionResult::Value(typed_array.clone()))
    );
    assert_eq!(
        program.invoke_builtin_method(&obtain_attribute_set, Some(&attribute_set_invocation)),
        Some(DexExecutionResult::Value(attribute_set_typed_array))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &obtain_attribute_set_with_style,
            Some(&styled_attribute_set_invocation)
        ),
        Some(DexExecutionResult::Value(styled_attribute_set_typed_array))
    );
    assert_eq!(
        program.invoke_builtin_method(&theme_obtain, Some(&theme_invocation)),
        Some(DexExecutionResult::Value(theme_typed_array))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &theme_obtain_attribute_set_with_style,
            Some(&theme_styled_attribute_set_invocation)
        ),
        Some(DexExecutionResult::Value(
            theme_styled_attribute_set_typed_array
        ))
    );
    // cite: frameworks-base TypedArray.java#hasValue
    assert_eq!(
        program.invoke_builtin_method(&has_value, Some(&has_value_invocation)),
        Some(DexExecutionResult::Int(0))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_boolean, Some(&get_action_bar_invocation)),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_boolean, Some(&get_boolean_invocation)),
        Some(DexExecutionResult::Int(0))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &get_index_count,
            Some(&test_invocation(vec![typed_array.clone()]))
        ),
        Some(DexExecutionResult::Int(0))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_index, Some(&get_index_invocation)),
        Some(DexExecutionResult::Int(4))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &get_changing_configurations,
            Some(&test_invocation(vec![typed_array.clone()]))
        ),
        Some(DexExecutionResult::Int(0))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &get_resources,
            Some(&test_invocation(vec![typed_array.clone()]))
        ),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/content/res/Resources;.bootstrap")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_value, Some(&get_value_invocation)),
        Some(DexExecutionResult::Int(0))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_resource_id, Some(&get_resource_id_invocation)),
        Some(DexExecutionResult::Int(2131296591))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_int, Some(&get_int_invocation)),
        Some(DexExecutionResult::Int(5))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_dimension_pixel_offset, Some(&get_dimension_invocation)),
        Some(DexExecutionResult::Int(12))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_dimension_pixel_size, Some(&get_dimension_invocation)),
        Some(DexExecutionResult::Int(12))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_layout_dimension, Some(&get_dimension_invocation)),
        Some(DexExecutionResult::Int(12))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_text, Some(&get_text_invocation)),
        Some(DexExecutionResult::Value(DexValue::Null))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_string, Some(&get_string_invocation)),
        Some(DexExecutionResult::Value(DexValue::Null))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_text_array, Some(&get_text_array_invocation)),
        Some(DexExecutionResult::Value(DexValue::Null))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &get_color_state_list,
            Some(&get_color_state_list_invocation)
        ),
        Some(DexExecutionResult::Value(DexValue::Null))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_drawable, Some(&get_drawable_invocation)),
        Some(DexExecutionResult::Value(DexValue::Null))
    );
    assert_eq!(
        program.invoke_builtin_method(&recycle, Some(&test_invocation(vec![typed_array]))),
        Some(DexExecutionResult::Void)
    );
}

#[test]
fn typed_array_get_string_reads_current_xml_attribute_by_styleable_name() {
    let program = DexProgram::new(Vec::new());
    let get_string = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getString".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let parser = xml_resource_parser_reference(2132017153);
    let parser_key = map_identity_key(&parser);
    let styleable = program.allocate_filled_array(
        "[I".to_owned(),
        vec![DexValue::String(std::rc::Rc::from("key"))],
    );
    let styleable_key = map_identity_key(&styleable);
    let typed_array = DexValue::Reference(std::rc::Rc::from(format!(
        "Landroid/content/res/TypedArray;:{parser_key}:{styleable_key}:int:0:int:0"
    )));

    program.runtime.borrow_mut().xml_parsers.insert(
        parser_key,
        XmlParserState {
            resource_id: 2132017153,
            events: vec![XmlEvent::Start(binary_xml::XmlStartElement {
                name: "ListPreference".to_owned(),
                namespace: None,
                attributes: vec![binary_xml::XmlAttribute {
                    name: "key".to_owned(),
                    namespace: None,
                    value: binary_xml::XmlValue::String(
                        "darkempire78.opencalculator.APP_LANGUAGE".to_owned(),
                    ),
                }],
            })],
            position: XmlParserPosition::Event(0),
            closed: false,
        },
    );

    assert_eq!(
        program.invoke_builtin_method(
            &get_string,
            Some(&test_invocation(vec![typed_array, DexValue::Int(0)]))
        ),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from("darkempire78.opencalculator.APP_LANGUAGE")
        )))
    );
}

#[test]
fn typed_array_get_drawable_reads_current_xml_attribute_by_styleable_name() {
    let program = DexProgram::new(Vec::new());
    let get_drawable = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getDrawable".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Landroid/graphics/drawable/Drawable;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let parser = xml_resource_parser_reference(2132017153);
    let parser_key = map_identity_key(&parser);
    let styleable = program.allocate_filled_array(
        "[I".to_owned(),
        vec![DexValue::String(std::rc::Rc::from("icon"))],
    );
    let styleable_key = map_identity_key(&styleable);
    let typed_array = DexValue::Reference(std::rc::Rc::from(format!(
        "Landroid/content/res/TypedArray;:{parser_key}:{styleable_key}:int:0:int:0"
    )));

    program.runtime.borrow_mut().xml_parsers.insert(
        parser_key,
        XmlParserState {
            resource_id: 2132017153,
            events: vec![XmlEvent::Start(binary_xml::XmlStartElement {
                name: "Preference".to_owned(),
                namespace: None,
                attributes: vec![binary_xml::XmlAttribute {
                    name: "icon".to_owned(),
                    namespace: None,
                    value: binary_xml::XmlValue::Reference(2131230888),
                }],
            })],
            position: XmlParserPosition::Event(0),
            closed: false,
        },
    );

    assert_eq!(
        program.invoke_builtin_method(
            &get_drawable,
            Some(&test_invocation(vec![typed_array, DexValue::Int(0)]))
        ),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/graphics/drawable/Drawable;:resource:2131230888")
        )))
    );
}

#[test]
fn typed_array_get_resource_id_reads_current_xml_attribute_by_styleable_name() {
    let program = DexProgram::new(Vec::new());
    let get_resource_id = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getResourceId".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "III".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned(), "I".to_owned()],
        },
    };
    let parser = xml_resource_parser_reference(2132017153);
    let parser_key = map_identity_key(&parser);
    let mut styleable_entries = vec![DexValue::String(std::rc::Rc::from("unused")); 24];
    styleable_entries[23] = DexValue::String(std::rc::Rc::from("icon"));
    let styleable = program.allocate_filled_array("[I".to_owned(), styleable_entries);
    let styleable_key = map_identity_key(&styleable);
    let typed_array = DexValue::Reference(std::rc::Rc::from(format!(
        "Landroid/content/res/TypedArray;:{parser_key}:{styleable_key}:int:0:int:0"
    )));

    program.runtime.borrow_mut().xml_parsers.insert(
        parser_key,
        XmlParserState {
            resource_id: 2132017153,
            events: vec![XmlEvent::Start(binary_xml::XmlStartElement {
                name: "Preference".to_owned(),
                namespace: None,
                attributes: vec![binary_xml::XmlAttribute {
                    name: "icon".to_owned(),
                    namespace: None,
                    value: binary_xml::XmlValue::Reference(2131230888),
                }],
            })],
            position: XmlParserPosition::Event(0),
            closed: false,
        },
    );

    assert_eq!(
        program.invoke_builtin_method(
            &get_resource_id,
            Some(&test_invocation(vec![
                typed_array,
                DexValue::Int(23),
                DexValue::Int(7)
            ]))
        ),
        Some(DexExecutionResult::Int(2131230888))
    );
}

#[test]
// cite: frameworks-base TypedArray.java#getInt
// cite: frameworks-base public-final.xml#gravity
fn typed_array_get_int_reads_current_xml_integer_by_framework_styleable_name() {
    let program = DexProgram::new(Vec::new());
    let get_int = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getInt".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "III".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned(), "I".to_owned()],
        },
    };
    let parser = xml_resource_parser_reference(2131492893);
    let parser_key = map_identity_key(&parser);
    let styleable = program.allocate_filled_array("[I".to_owned(), vec![DexValue::Int(16842927)]);
    let styleable_key = map_identity_key(&styleable);
    let typed_array = DexValue::Reference(std::rc::Rc::from(format!(
        "Landroid/content/res/TypedArray;:{parser_key}:{styleable_key}:int:0:int:0"
    )));

    program.runtime.borrow_mut().xml_parsers.insert(
        parser_key,
        XmlParserState {
            resource_id: 2131492893,
            events: vec![XmlEvent::Start(binary_xml::XmlStartElement {
                name: "com.example.CustomView".to_owned(),
                namespace: None,
                attributes: vec![
                    binary_xml::XmlAttribute {
                        name: "gravity".to_owned(),
                        namespace: Some(
                            "http://schemas.android.com/apk/res/com.example".to_owned(),
                        ),
                        value: binary_xml::XmlValue::Hex(80),
                    },
                    binary_xml::XmlAttribute {
                        name: "gravity".to_owned(),
                        namespace: Some("http://schemas.android.com/apk/res/android".to_owned()),
                        value: binary_xml::XmlValue::Hex(48),
                    },
                ],
            })],
            position: XmlParserPosition::Event(0),
            closed: false,
        },
    );

    assert_eq!(
        program.invoke_builtin_method(
            &get_int,
            Some(&test_invocation(vec![
                typed_array.clone(),
                DexValue::Int(0),
                DexValue::Int(7)
            ]))
        ),
        Some(DexExecutionResult::Int(48))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &get_int,
            Some(&test_invocation(vec![
                typed_array,
                DexValue::Int(1),
                DexValue::Int(7)
            ]))
        ),
        Some(DexExecutionResult::Int(7))
    );
}

fn single_xml_attribute_typed_array(
    program: &DexProgram,
    attribute_name: &str,
    value: binary_xml::XmlValue,
) -> DexValue {
    let parser = xml_resource_parser_reference(2131492893);
    let parser_key = map_identity_key(&parser);
    let styleable = program.allocate_filled_array(
        "[I".to_owned(),
        vec![DexValue::String(std::rc::Rc::from(attribute_name))],
    );
    let styleable_key = map_identity_key(&styleable);
    let typed_array = DexValue::Reference(std::rc::Rc::from(format!(
        "Landroid/content/res/TypedArray;:{parser_key}:{styleable_key}:int:0:int:0"
    )));

    program.runtime.borrow_mut().xml_parsers.insert(
        parser_key,
        XmlParserState {
            resource_id: 2131492893,
            events: vec![XmlEvent::Start(binary_xml::XmlStartElement {
                name: "com.example.CustomView".to_owned(),
                namespace: None,
                attributes: vec![binary_xml::XmlAttribute {
                    name: attribute_name.to_owned(),
                    namespace: None,
                    value,
                }],
            })],
            position: XmlParserPosition::Event(0),
            closed: false,
        },
    );

    typed_array
}

#[test]
// cite: frameworks-base AttributeResolution.cpp#RetrieveAttributes
// cite: frameworks-base TypedArray.java#getInt, getColor, getDimensionPixelOffset, getDimensionPixelSize, getResourceId, getText, getTextArray, getValue, hasValue
fn typed_array_null_and_at_null_xml_values_follow_type_null_contracts() {
    for null_value in [
        binary_xml::XmlValue::Null,
        binary_xml::XmlValue::Reference(0),
    ] {
        let program = DexProgram::new(Vec::new());
        let typed_array = single_xml_attribute_typed_array(&program, "nullableValue", null_value);

        for method_name in [
            "getInt",
            "getColor",
            "getDimensionPixelOffset",
            "getDimensionPixelSize",
            "getResourceId",
        ] {
            let method = DexMethodId {
                class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
                name: method_name.to_owned(),
                proto: crate::dex::DexProto {
                    shorty: "III".to_owned(),
                    return_type: "I".to_owned(),
                    parameters: vec!["I".to_owned(), "I".to_owned()],
                },
            };

            assert_eq!(
                program.invoke_builtin_method(
                    &method,
                    Some(&test_invocation(vec![
                        typed_array.clone(),
                        DexValue::Int(0),
                        DexValue::Int(-37),
                    ]))
                ),
                Some(DexExecutionResult::Int(-37)),
                "{method_name} should return its default for typed null"
            );
        }

        for (method_name, return_type) in [
            ("getText", "Ljava/lang/CharSequence;"),
            ("getTextArray", "[Ljava/lang/CharSequence;"),
        ] {
            let method = DexMethodId {
                class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
                name: method_name.to_owned(),
                proto: crate::dex::DexProto {
                    shorty: "LI".to_owned(),
                    return_type: return_type.to_owned(),
                    parameters: vec!["I".to_owned()],
                },
            };

            assert_eq!(
                program.invoke_builtin_method(
                    &method,
                    Some(&test_invocation(vec![
                        typed_array.clone(),
                        DexValue::Int(0)
                    ]))
                ),
                Some(DexExecutionResult::Value(DexValue::Null))
            );
        }

        let has_value = DexMethodId {
            class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
            name: "hasValue".to_owned(),
            proto: crate::dex::DexProto {
                shorty: "ZI".to_owned(),
                return_type: "Z".to_owned(),
                parameters: vec!["I".to_owned()],
            },
        };
        assert_eq!(
            program.invoke_builtin_method(
                &has_value,
                Some(&test_invocation(vec![
                    typed_array.clone(),
                    DexValue::Int(0)
                ]))
            ),
            Some(DexExecutionResult::Int(0))
        );

        let get_value = DexMethodId {
            class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
            name: "getValue".to_owned(),
            proto: crate::dex::DexProto {
                shorty: "ZIL".to_owned(),
                return_type: "Z".to_owned(),
                parameters: vec!["I".to_owned(), "Landroid/util/TypedValue;".to_owned()],
            },
        };
        let out_value = program.allocate_object("Landroid/util/TypedValue;".to_owned());
        assert_eq!(
            program.invoke_builtin_method(
                &get_value,
                Some(&test_invocation(vec![
                    typed_array,
                    DexValue::Int(0),
                    out_value,
                ]))
            ),
            Some(DexExecutionResult::Int(0))
        );
    }
}

#[test]
// cite: frameworks-base TypedArray.java#getInt
// cite: frameworks-base XmlUtils.java#convertValueToInt
fn typed_array_get_int_empty_xml_string_returns_caller_default() {
    let program = DexProgram::new(Vec::new());
    let typed_array = single_xml_attribute_typed_array(
        &program,
        "emptyValue",
        binary_xml::XmlValue::String(String::new()),
    );
    let method = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getInt".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "III".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned(), "I".to_owned()],
        },
    };

    assert_eq!(
        program.invoke_builtin_method(
            &method,
            Some(&test_invocation(vec![
                typed_array,
                DexValue::Int(0),
                DexValue::Int(43),
            ]))
        ),
        Some(DexExecutionResult::Int(43))
    );
}

#[test]
// cite: frameworks-base TypedArray.java#getInt
// cite: frameworks-base XmlUtils.java#convertValueToInt
fn typed_array_get_int_nonempty_malformed_xml_string_does_not_return_caller_default() {
    let program = DexProgram::new(Vec::new());
    let typed_array = single_xml_attribute_typed_array(
        &program,
        "malformedValue",
        binary_xml::XmlValue::String("not-an-int".to_owned()),
    );
    let method = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getInt".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "III".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned(), "I".to_owned()],
        },
    };

    assert_eq!(
        program.invoke_builtin_method(
            &method,
            Some(&test_invocation(vec![
                typed_array,
                DexValue::Int(0),
                DexValue::Int(43),
            ]))
        ),
        None
    );
}

#[test]
// cite: frameworks-base TypedArray.java#getColor
fn typed_array_get_color_reads_current_xml_color_by_styleable_name() {
    let program = DexProgram::new(Vec::new());
    let get_color = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getColor".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "III".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned(), "I".to_owned()],
        },
    };
    let parser = xml_resource_parser_reference(2131492893);
    let parser_key = map_identity_key(&parser);
    let styleable = program.allocate_filled_array(
        "[I".to_owned(),
        vec![DexValue::String(std::rc::Rc::from("fadeColor"))],
    );
    let styleable_key = map_identity_key(&styleable);
    let typed_array = DexValue::Reference(std::rc::Rc::from(format!(
        "Landroid/content/res/TypedArray;:{parser_key}:{styleable_key}:int:0:int:0"
    )));

    program.runtime.borrow_mut().xml_parsers.insert(
        parser_key,
        XmlParserState {
            resource_id: 2131492893,
            events: vec![XmlEvent::Start(binary_xml::XmlStartElement {
                name: "com.example.CustomView".to_owned(),
                namespace: None,
                attributes: vec![binary_xml::XmlAttribute {
                    name: "fadeColor".to_owned(),
                    namespace: Some("http://schemas.android.com/apk/res/com.example".to_owned()),
                    value: binary_xml::XmlValue::Color(0x9911_2233),
                }],
            })],
            position: XmlParserPosition::Event(0),
            closed: false,
        },
    );

    assert_eq!(
        program.invoke_builtin_method(
            &get_color,
            Some(&test_invocation(vec![
                typed_array.clone(),
                DexValue::Int(0),
                DexValue::Int(7)
            ]))
        ),
        Some(DexExecutionResult::Int(0x9911_2233u32 as i32))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &get_color,
            Some(&test_invocation(vec![
                typed_array,
                DexValue::Int(1),
                DexValue::Int(7)
            ]))
        ),
        Some(DexExecutionResult::Int(7))
    );
}

#[test]
fn typed_array_get_float_reads_current_xml_attribute_by_framework_styleable_name() {
    let program = DexProgram::new(Vec::new());
    let get_float = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getFloat".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "FIF".to_owned(),
            return_type: "F".to_owned(),
            parameters: vec!["I".to_owned(), "F".to_owned()],
        },
    };
    let parser = xml_resource_parser_reference(2131165433);
    let parser_key = map_identity_key(&parser);
    let styleable = program.allocate_filled_array("[I".to_owned(), vec![DexValue::Int(16843778)]);
    let styleable_key = map_identity_key(&styleable);
    let typed_array = DexValue::Reference(std::rc::Rc::from(format!(
        "Landroid/content/res/TypedArray;:{parser_key}:{styleable_key}:int:0:int:0"
    )));

    program.runtime.borrow_mut().xml_parsers.insert(
        parser_key,
        XmlParserState {
            resource_id: 2131165433,
            events: vec![XmlEvent::Start(binary_xml::XmlStartElement {
                name: "vector".to_owned(),
                namespace: None,
                attributes: vec![binary_xml::XmlAttribute {
                    name: "viewportWidth".to_owned(),
                    namespace: Some("http://schemas.android.com/apk/res/android".to_owned()),
                    value: binary_xml::XmlValue::Float(24.0f32.to_bits()),
                }],
            })],
            position: XmlParserPosition::Event(0),
            closed: false,
        },
    );

    assert_eq!(
        program.invoke_builtin_method(
            &get_float,
            Some(&test_invocation(vec![
                typed_array,
                DexValue::Int(0),
                DexValue::Float(0.0)
            ]))
        ),
        Some(DexExecutionResult::Value(DexValue::Float(24.0)))
    );
}

#[test]
// cite: frameworks-base TypedArray.java#getDimension, getDimensionPixelOffset, getDimensionPixelSize, getLayoutDimension
// cite: frameworks-base TypedValue.java#complexToDimension, complexToDimensionPixelOffset, complexToDimensionPixelSize
fn typed_array_get_dimension_reads_current_xml_attribute_by_framework_styleable_name() {
    let program = DexProgram::new(Vec::new());
    let get_dimension = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getDimension".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "FIF".to_owned(),
            return_type: "F".to_owned(),
            parameters: vec!["I".to_owned(), "F".to_owned()],
        },
    };
    let get_dimension_pixel_offset = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getDimensionPixelOffset".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "III".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned(), "I".to_owned()],
        },
    };
    let get_dimension_pixel_size = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getDimensionPixelSize".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "III".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned(), "I".to_owned()],
        },
    };
    let get_layout_dimension = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getLayoutDimension".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "III".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned(), "I".to_owned()],
        },
    };
    let parser = xml_resource_parser_reference(2131165433);
    let parser_key = map_identity_key(&parser);
    let styleable = program.allocate_filled_array(
        "[I".to_owned(),
        vec![
            DexValue::Int(16843097),
            DexValue::String(std::rc::Rc::from("integerWidth")),
            DexValue::String(std::rc::Rc::from("stringWidth")),
        ],
    );
    let styleable_key = map_identity_key(&styleable);
    let typed_array = DexValue::Reference(std::rc::Rc::from(format!(
        "Landroid/content/res/TypedArray;:{parser_key}:{styleable_key}:int:0:int:0"
    )));

    program.runtime.borrow_mut().xml_parsers.insert(
        parser_key,
        XmlParserState {
            resource_id: 2131165433,
            events: vec![XmlEvent::Start(binary_xml::XmlStartElement {
                name: "vector".to_owned(),
                namespace: None,
                attributes: vec![
                    binary_xml::XmlAttribute {
                        name: "width".to_owned(),
                        namespace: Some("http://schemas.android.com/apk/res/android".to_owned()),
                        value: binary_xml::XmlValue::Dimension(0x0000_1001),
                    },
                    binary_xml::XmlAttribute {
                        name: "integerWidth".to_owned(),
                        namespace: None,
                        value: binary_xml::XmlValue::Integer(80),
                    },
                    binary_xml::XmlAttribute {
                        name: "stringWidth".to_owned(),
                        namespace: None,
                        value: binary_xml::XmlValue::String("16".to_owned()),
                    },
                ],
            })],
            position: XmlParserPosition::Event(0),
            closed: false,
        },
    );

    assert_eq!(
        program.invoke_builtin_method(
            &get_dimension,
            Some(&test_invocation(vec![
                typed_array.clone(),
                DexValue::Int(0),
                DexValue::Float(0.0)
            ]))
        ),
        Some(DexExecutionResult::Value(DexValue::Float(16.0)))
    );
    for method in [
        &get_dimension_pixel_offset,
        &get_dimension_pixel_size,
        &get_layout_dimension,
    ] {
        assert_eq!(
            program.invoke_builtin_method(
                method,
                Some(&test_invocation(vec![
                    typed_array.clone(),
                    DexValue::Int(0),
                    DexValue::Int(7)
                ]))
            ),
            Some(DexExecutionResult::Int(16))
        );
    }
    assert_eq!(
        program.invoke_builtin_method(
            &get_layout_dimension,
            Some(&test_invocation(vec![
                typed_array.clone(),
                DexValue::Int(1),
                DexValue::Int(7)
            ]))
        ),
        Some(DexExecutionResult::Int(80))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &get_layout_dimension,
            Some(&test_invocation(vec![
                typed_array.clone(),
                DexValue::Int(2),
                DexValue::Int(7)
            ]))
        ),
        Some(DexExecutionResult::Int(7))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &get_dimension_pixel_size,
            Some(&test_invocation(vec![
                typed_array,
                DexValue::Int(3),
                DexValue::Int(7)
            ]))
        ),
        Some(DexExecutionResult::Int(7))
    );
}

// cite: frameworks-base AttributeResolution.cpp#RetrieveAttributes
// cite: frameworks-base AssetManager2.cpp#AssetManager2::ResolveReference
// cite: frameworks-base TypedArray.java#getColor, getColorStateList, getString, getText, getTextArray, getValue, getValueAt
// cite: frameworks-base Resources.java#getTextArray, NotFoundException
// cite: frameworks-base ResourcesImpl.java#loadColorStateList, loadComplexColorFromName
#[test]
fn typed_array_missing_xml_reference_preserves_value_and_getter_contracts() {
    let program = DexProgram::new(Vec::new());
    let get_value = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getValue".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZIL".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec!["I".to_owned(), "Landroid/util/TypedValue;".to_owned()],
        },
    };
    let parser = xml_resource_parser_reference(2131165433);
    let parser_key = map_identity_key(&parser);
    let styleable = program.allocate_filled_array("[I".to_owned(), vec![DexValue::Int(16843041)]);
    let styleable_key = map_identity_key(&styleable);
    let typed_array = DexValue::Reference(std::rc::Rc::from(format!(
        "Landroid/content/res/TypedArray;:{parser_key}:{styleable_key}:int:0:int:0"
    )));
    let out_value = DexValue::Object(DexObjectRef {
        id: 42,
        class_descriptor: std::rc::Rc::from("Landroid/util/TypedValue;"),
    });

    program.runtime.borrow_mut().xml_parsers.insert(
        parser_key.clone(),
        XmlParserState {
            resource_id: 2131165433,
            events: vec![XmlEvent::Start(binary_xml::XmlStartElement {
                name: "vector".to_owned(),
                namespace: None,
                attributes: vec![binary_xml::XmlAttribute {
                    name: "tint".to_owned(),
                    namespace: Some("http://schemas.android.com/apk/res/android".to_owned()),
                    value: binary_xml::XmlValue::Reference(2131099744),
                }],
            })],
            position: XmlParserPosition::Event(0),
            closed: false,
        },
    );

    assert_eq!(
        program.invoke_builtin_method(
            &get_value,
            Some(&test_invocation(vec![
                typed_array.clone(),
                DexValue::Int(0),
                out_value.clone(),
            ])),
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("type")),
        Some(DexValue::Int(0x01))
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("data")),
        Some(DexValue::Int(2131099744))
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("resourceId")),
        Some(DexValue::Int(2131099744))
    );

    for (method_name, return_type) in [
        ("getText", "Ljava/lang/CharSequence;"),
        ("getString", "Ljava/lang/String;"),
    ] {
        let method = DexMethodId {
            class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
            name: method_name.to_owned(),
            proto: crate::dex::DexProto {
                shorty: "LI".to_owned(),
                return_type: return_type.to_owned(),
                parameters: vec!["I".to_owned()],
            },
        };
        assert_eq!(
            program.invoke_builtin_method(
                &method,
                Some(&test_invocation(vec![
                    typed_array.clone(),
                    DexValue::Int(0)
                ]))
            ),
            Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from("@2131099744")
            )))
        );
    }

    let get_color = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getColor".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "III".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned(), "I".to_owned()],
        },
    };
    let Some(DexExecutionResult::Throw(exception)) = program.invoke_builtin_method(
        &get_color,
        Some(&test_invocation(vec![
            typed_array.clone(),
            DexValue::Int(0),
            DexValue::Int(17),
        ])),
    ) else {
        panic!("TypedArray.getColor should reject an unresolved reference");
    };
    assert!(program.value_matches_type(&exception, "Ljava/lang/UnsupportedOperationException;"));

    let get_text_array = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getTextArray".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "[Ljava/lang/CharSequence;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    assert_resource_not_found(
        &program,
        program.invoke_builtin_method(
            &get_text_array,
            Some(&test_invocation(vec![
                typed_array.clone(),
                DexValue::Int(0),
            ])),
        ),
        "0x7f060060",
    );

    {
        let mut runtime = program.runtime.borrow_mut();
        let parser = runtime
            .xml_parsers
            .get_mut(&parser_key)
            .expect("parser should exist");
        let XmlEvent::Start(element) = &mut parser.events[0] else {
            panic!("parser should contain a start element");
        };
        element.attributes[0].value = binary_xml::XmlValue::Integer(42);
    }
    assert_resource_not_found(
        &program,
        program.invoke_builtin_method(
            &get_text_array,
            Some(&test_invocation(vec![
                typed_array.clone(),
                DexValue::Int(0),
            ])),
        ),
        "0x00000000",
    );

    {
        let mut runtime = program.runtime.borrow_mut();
        let parser = runtime
            .xml_parsers
            .get_mut(&parser_key)
            .expect("parser should exist");
        let XmlEvent::Start(element) = &mut parser.events[0] else {
            panic!("parser should contain a start element");
        };
        element.attributes[0].value = binary_xml::XmlValue::Boolean(true);
    }
    let get_color_state_list = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getColorStateList".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Landroid/content/res/ColorStateList;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let Some(DexExecutionResult::Throw(exception)) = program.invoke_builtin_method(
        &get_color_state_list,
        Some(&test_invocation(vec![typed_array, DexValue::Int(0)])),
    ) else {
        panic!("TypedArray.getColorStateList should reject a boolean");
    };
    assert!(program.value_matches_type(&exception, "Ljava/lang/UnsupportedOperationException;"));
}

#[test]
fn typed_array_get_value_resolves_android_framework_color_reference() {
    let program = DexProgram::new(Vec::new());
    let get_value = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getValue".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZIL".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec!["I".to_owned(), "Landroid/util/TypedValue;".to_owned()],
        },
    };
    let parser = xml_resource_parser_reference(2131165433);
    let parser_key = map_identity_key(&parser);
    let styleable = program.allocate_filled_array("[I".to_owned(), vec![DexValue::Int(16843780)]);
    let styleable_key = map_identity_key(&styleable);
    let typed_array = DexValue::Reference(std::rc::Rc::from(format!(
        "Landroid/content/res/TypedArray;:{parser_key}:{styleable_key}:int:0:int:0"
    )));
    let out_value = DexValue::Object(DexObjectRef {
        id: 43,
        class_descriptor: std::rc::Rc::from("Landroid/util/TypedValue;"),
    });

    program.runtime.borrow_mut().xml_parsers.insert(
        parser_key,
        XmlParserState {
            resource_id: 2131165433,
            events: vec![XmlEvent::Start(binary_xml::XmlStartElement {
                name: "path".to_owned(),
                namespace: None,
                attributes: vec![binary_xml::XmlAttribute {
                    name: "fillColor".to_owned(),
                    namespace: Some("http://schemas.android.com/apk/res/android".to_owned()),
                    value: binary_xml::XmlValue::Reference(0x0106_000b),
                }],
            })],
            position: XmlParserPosition::Event(0),
            closed: false,
        },
    );

    assert_eq!(
        program.invoke_builtin_method(
            &get_value,
            Some(&test_invocation(vec![
                typed_array,
                DexValue::Int(0),
                out_value.clone()
            ]))
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("type")),
        Some(DexValue::Int(0x1c))
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("data")),
        Some(DexValue::Int(-1))
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("resourceId")),
        Some(DexValue::Int(0x0106_000b))
    );
}

#[test]
fn builtin_typed_value_constructor_returns_void() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/util/TypedValue;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&method, None),
        Some(DexExecutionResult::Void)
    );
}

#[test]
fn builtin_xml_as_attribute_set_returns_parser_reference() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/util/Xml;".to_owned(),
        name: "asAttributeSet".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Landroid/util/AttributeSet;".to_owned(),
            parameters: vec!["Lorg/xmlpull/v1/XmlPullParser;".to_owned()],
        },
    };
    let parser = DexValue::Reference(std::rc::Rc::from(
        "Landroid/content/res/XmlResourceParser;:layout/2131624225",
    ));
    let invocation = test_invocation(vec![parser.clone()]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Value(parser))
    );
}

#[test]
fn builtin_xml_pull_parser_iterates_events() {
    let program = DexProgram::new(Vec::new());
    let parser = xml_resource_parser_reference(2131624225);
    let parser_key = map_identity_key(&parser);
    program.runtime.borrow_mut().xml_parsers.insert(
        parser_key,
        XmlParserState {
            resource_id: 2131624225,
            events: vec![
                XmlEvent::Start(binary_xml::XmlStartElement {
                    name: "menu".to_owned(),
                    namespace: None,
                    attributes: Vec::new(),
                }),
                XmlEvent::Start(binary_xml::XmlStartElement {
                    name: "item".to_owned(),
                    namespace: None,
                    attributes: vec![binary_xml::XmlAttribute {
                        name: "autoMirrored".to_owned(),
                        namespace: Some("http://schemas.android.com/apk/res/android".to_owned()),
                        value: binary_xml::XmlValue::Boolean(true),
                    }],
                }),
                XmlEvent::End(binary_xml::XmlEndElement {
                    name: "item".to_owned(),
                    namespace: None,
                }),
                XmlEvent::End(binary_xml::XmlEndElement {
                    name: "menu".to_owned(),
                    namespace: None,
                }),
            ],
            position: XmlParserPosition::StartDocument,
            closed: false,
        },
    );
    let get_event_type = DexMethodId {
        class_descriptor: "Lorg/xmlpull/v1/XmlPullParser;".to_owned(),
        name: "getEventType".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "I".to_owned(),
            return_type: "I".to_owned(),
            parameters: Vec::new(),
        },
    };
    let next = DexMethodId {
        class_descriptor: "Lorg/xmlpull/v1/XmlPullParser;".to_owned(),
        name: "next".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "I".to_owned(),
            return_type: "I".to_owned(),
            parameters: Vec::new(),
        },
    };
    let get_name = DexMethodId {
        class_descriptor: "Lorg/xmlpull/v1/XmlPullParser;".to_owned(),
        name: "getName".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let get_depth = DexMethodId {
        class_descriptor: "Lorg/xmlpull/v1/XmlPullParser;".to_owned(),
        name: "getDepth".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "I".to_owned(),
            return_type: "I".to_owned(),
            parameters: Vec::new(),
        },
    };
    let get_attribute_value = DexMethodId {
        class_descriptor: "Lorg/xmlpull/v1/XmlPullParser;".to_owned(),
        name: "getAttributeValue".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LLL".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: vec![
                "Ljava/lang/String;".to_owned(),
                "Ljava/lang/String;".to_owned(),
            ],
        },
    };
    let close = DexMethodId {
        class_descriptor: "Landroid/content/res/XmlResourceParser;".to_owned(),
        name: "close".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let invocation = test_invocation(vec![parser.clone()]);
    let get_attribute_value_invocation = test_invocation(vec![
        parser,
        DexValue::String(std::rc::Rc::from(
            "http://schemas.android.com/apk/res/android",
        )),
        DexValue::String(std::rc::Rc::from("autoMirrored")),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&get_event_type, Some(&invocation)),
        Some(DexExecutionResult::Int(XML_PULL_START_DOCUMENT))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_depth, Some(&invocation)),
        Some(DexExecutionResult::Int(0))
    );
    assert_eq!(
        program.invoke_builtin_method(&next, Some(&invocation)),
        Some(DexExecutionResult::Int(XML_PULL_START_TAG))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_depth, Some(&invocation)),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_name, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from("menu")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(&next, Some(&invocation)),
        Some(DexExecutionResult::Int(XML_PULL_START_TAG))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_depth, Some(&invocation)),
        Some(DexExecutionResult::Int(2))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_name, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from("item")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_attribute_value, Some(&get_attribute_value_invocation)),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from("true")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(&next, Some(&invocation)),
        Some(DexExecutionResult::Int(XML_PULL_END_TAG))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_depth, Some(&invocation)),
        Some(DexExecutionResult::Int(2))
    );
    assert_eq!(
        program.invoke_builtin_method(&next, Some(&invocation)),
        Some(DexExecutionResult::Int(XML_PULL_END_TAG))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_depth, Some(&invocation)),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.invoke_builtin_method(&next, Some(&invocation)),
        Some(DexExecutionResult::Int(XML_PULL_END_DOCUMENT))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_depth, Some(&invocation)),
        Some(DexExecutionResult::Int(0))
    );
    assert_eq!(
        program.invoke_builtin_method(&close, Some(&invocation)),
        Some(DexExecutionResult::Void)
    );
}

#[test]
fn runtime_menu_items_parse_visible_enabled_titles_and_ids() {
    let events = vec![
        XmlEvent::Start(binary_xml::XmlStartElement {
            name: "menu".to_owned(),
            namespace: None,
            attributes: Vec::new(),
        }),
        XmlEvent::Start(binary_xml::XmlStartElement {
            name: "item".to_owned(),
            namespace: None,
            attributes: vec![
                binary_xml::XmlAttribute {
                    name: "id".to_owned(),
                    namespace: None,
                    value: binary_xml::XmlValue::Reference(0x7f09_0010),
                },
                binary_xml::XmlAttribute {
                    name: "title".to_owned(),
                    namespace: None,
                    value: binary_xml::XmlValue::String("Settings".to_owned()),
                },
                binary_xml::XmlAttribute {
                    name: "enabled".to_owned(),
                    namespace: None,
                    value: binary_xml::XmlValue::Boolean(true),
                },
                binary_xml::XmlAttribute {
                    name: "visible".to_owned(),
                    namespace: None,
                    value: binary_xml::XmlValue::Boolean(true),
                },
                binary_xml::XmlAttribute {
                    name: "onClick".to_owned(),
                    namespace: Some("http://schemas.android.com/apk/res/android".to_owned()),
                    value: binary_xml::XmlValue::String("openSettings".to_owned()),
                },
            ],
        }),
        XmlEvent::End(binary_xml::XmlEndElement {
            name: "item".to_owned(),
            namespace: None,
        }),
    ];
    let items =
        runtime_menu_items_from_events(0x7f0e_0000, &events, &[]).expect("menu items should parse");

    assert_eq!(
        items,
        vec![RuntimeMenuItem {
            item_id: 0x7f09_0010,
            title: "Settings".to_owned(),
            enabled: true,
            visible: true,
            on_click: Some("openSettings".to_owned()),
            icon_resource: None,
            show_as_action: 0,
        }]
    );
}

#[test]
fn menu_item_bootstrap_fields_attach_xml_on_click_listener() {
    let program = DexProgram::new(Vec::new());
    let item = DexValue::Object(DexObjectRef {
        id: 1,
        class_descriptor: std::rc::Rc::from("Lm/o;"),
    });
    let parsed_item = RuntimeMenuItem {
        item_id: 0x7f09_0010,
        title: "Settings".to_owned(),
        enabled: true,
        visible: true,
        on_click: Some("openSettings".to_owned()),
        icon_resource: None,
        show_as_action: 0,
    };
    let on_click_field = DexFieldId {
        class_descriptor: "Lm/o;".to_owned(),
        type_descriptor: "Landroid/view/MenuItem$OnMenuItemClickListener;".to_owned(),
        name: "p".to_owned(),
    };

    program.write_menu_item_bootstrap_fields(&item, &parsed_item);

    assert_eq!(
        program.read_stored_instance_field(&item, &on_click_field),
        Some(synthetic_menu_item_on_click_listener("openSettings"))
    );
}

#[test]
// cite: androidx SupportMenuInflater.java#setItem
fn popup_window_show_records_menu_overlay_from_adapter_menu() {
    let program = DexProgram::new(Vec::new());
    let popup = DexValue::Reference(std::rc::Rc::from("Landroid/widget/PopupWindow;:popup"));
    let list_view = DexValue::Reference(std::rc::Rc::from("Landroid/widget/ListView;:popup"));
    let adapter = DexValue::Reference(std::rc::Rc::from("Lm/j;:adapter"));
    let menu = DexValue::Reference(std::rc::Rc::from("Lm/m;:menu"));
    let anchor = view_reference(2131296591);
    let adapter_menu_field = DexFieldId {
        class_descriptor: "Lm/j;".to_owned(),
        type_descriptor: "Lm/m;".to_owned(),
        name: "a".to_owned(),
    };

    program.runtime.borrow_mut().menus.insert(
        map_identity_key(&menu),
        RuntimeMenu {
            resource_id: 0x7f0e_0000,
            items: vec![RuntimeMenuItem {
                item_id: 0x7f09_0010,
                title: "Settings".to_owned(),
                enabled: true,
                visible: true,
                on_click: Some("openSettings".to_owned()),
                icon_resource: None,
                show_as_action: 0,
            }],
        },
    );
    assert!(program.write_stored_instance_field(
        &popup,
        &popup_window_content_view_field(),
        list_view.clone()
    ));
    assert!(program.write_stored_instance_field(
        &list_view,
        &adapter_view_adapter_field(),
        adapter.clone()
    ));
    assert!(program.write_stored_instance_field(&adapter, &adapter_menu_field, menu));

    assert_eq!(
        program.show_popup_window(&test_invocation(vec![popup.clone(), anchor])),
        Some(())
    );
    assert_eq!(
        program.activity_overlays(),
        vec![DexActivityOverlay {
            anchor_view_id: Some(2131296591),
            items: vec![DexActivityOverlayItem {
                item_id: 0x7f09_0010,
                text: "Settings".to_owned(),
                enabled: true,
            }],
        }]
    );
    assert_eq!(
        program.runtime.borrow().active_menu_overlays[0].items[0].on_click,
        Some("openSettings".to_owned())
    );
    assert_eq!(
        program.popup_window_is_showing(&test_invocation(vec![popup.clone()])),
        Some(1)
    );
    assert_eq!(
        program.dismiss_popup_window(&test_invocation(vec![popup.clone()])),
        Some(())
    );
    assert!(program.activity_overlays().is_empty());
    assert_eq!(
        program.popup_window_is_showing(&test_invocation(vec![popup])),
        Some(0)
    );
}

#[test]
fn builtin_context_get_theme_returns_context_scoped_theme() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getTheme".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/content/res/Resources$Theme;".to_owned(),
            parameters: Vec::new(),
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&method, None),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/content/res/Resources$Theme;:reference:Landroid/content/Context;.bootstrap"
                .to_owned())
        )))
    );
}

#[test]
fn builtin_context_theme_wrapper_stores_base_context() {
    let program = DexProgram::new(Vec::new());
    let constructor = DexMethodId {
        class_descriptor: "Landroid/view/ContextThemeWrapper;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VLI".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["Landroid/content/Context;".to_owned(), "I".to_owned()],
        },
    };
    let get_base_context = DexMethodId {
        class_descriptor: "Landroid/content/ContextWrapper;".to_owned(),
        name: "getBaseContext".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/content/Context;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let wrapper = DexValue::Object(DexObjectRef {
        id: 1,
        class_descriptor: std::rc::Rc::from("LH0/g;"),
    });
    let base_context =
        DexValue::Reference(std::rc::Rc::from("Landroid/content/Context;.bootstrap"));
    let constructor_invocation = test_invocation(vec![
        wrapper.clone(),
        base_context.clone(),
        DexValue::Int(2131886439),
    ]);
    let getter_invocation = test_invocation(vec![wrapper]);

    assert_eq!(
        program.invoke_builtin_method(&constructor, Some(&constructor_invocation)),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&get_base_context, Some(&getter_invocation)),
        Some(DexExecutionResult::Value(base_context))
    );
}

#[test]
fn builtin_context_theme_wrapper_theme_inherits_base_theme() {
    let base_attr = 0x7f02_0000;
    let wrapper_attr = 0x7f02_0001;
    let program = DexProgram::with_resource_context(
        Vec::new(),
        None,
        None,
        None,
        None,
        Some(style_resource_table(&[
            TestStyleResource {
                name: "BaseTheme",
                parent: None,
                entries: vec![TestStyleEntry {
                    name: base_attr,
                    value_type: 0x1c,
                    data: 0xff11_1111,
                }],
            },
            TestStyleResource {
                name: "WrapperTheme",
                parent: None,
                entries: vec![TestStyleEntry {
                    name: wrapper_attr,
                    value_type: 0x1c,
                    data: 0xff22_2222,
                }],
            },
        ])),
    );
    let set_theme = DexMethodId {
        class_descriptor: "Landroid/app/Activity;".to_owned(),
        name: "setTheme".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VI".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let constructor = DexMethodId {
        class_descriptor: "Landroid/view/ContextThemeWrapper;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VLI".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["Landroid/content/Context;".to_owned(), "I".to_owned()],
        },
    };
    let get_theme = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getTheme".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/content/res/Resources$Theme;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let resolve_attribute = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources$Theme;".to_owned(),
        name: "resolveAttribute".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZILZ".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec![
                "I".to_owned(),
                "Landroid/util/TypedValue;".to_owned(),
                "Z".to_owned(),
            ],
        },
    };
    let base_context = DexValue::Object(DexObjectRef {
        id: 21,
        class_descriptor: std::rc::Rc::from("Lcom/example/MainActivity;"),
    });
    let wrapper = DexValue::Object(DexObjectRef {
        id: 22,
        class_descriptor: std::rc::Rc::from("LH0/g;"),
    });
    let base_out = DexValue::Object(DexObjectRef {
        id: 23,
        class_descriptor: std::rc::Rc::from("Landroid/util/TypedValue;"),
    });
    let wrapper_out = DexValue::Object(DexObjectRef {
        id: 24,
        class_descriptor: std::rc::Rc::from("Landroid/util/TypedValue;"),
    });

    assert_eq!(
        program.invoke_builtin_method(
            &set_theme,
            Some(&test_invocation(vec![
                base_context.clone(),
                DexValue::Int(0x7f01_0000),
            ]))
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &constructor,
            Some(&test_invocation(vec![
                wrapper.clone(),
                base_context,
                DexValue::Int(0x7f01_0001),
            ]))
        ),
        Some(DexExecutionResult::Void)
    );
    let Some(DexExecutionResult::Value(theme)) =
        program.invoke_builtin_method(&get_theme, Some(&test_invocation(vec![wrapper])))
    else {
        panic!("wrapper getTheme should return a theme");
    };

    assert_eq!(
        program.invoke_builtin_method(
            &resolve_attribute,
            Some(&test_invocation(vec![
                theme.clone(),
                DexValue::Int(base_attr as i32),
                base_out.clone(),
                DexValue::Int(1),
            ]))
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.read_stored_instance_field(&base_out, &typed_value_test_int_field("data")),
        Some(DexValue::Int(0xff11_1111u32 as i32))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &resolve_attribute,
            Some(&test_invocation(vec![
                theme,
                DexValue::Int(wrapper_attr as i32),
                wrapper_out.clone(),
                DexValue::Int(1),
            ]))
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.read_stored_instance_field(&wrapper_out, &typed_value_test_int_field("data")),
        Some(DexValue::Int(0xff22_2222u32 as i32))
    );
}

#[test]
fn builtin_theme_resolve_attribute_returns_false_for_missing_attribute() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources$Theme;".to_owned(),
        name: "resolveAttribute".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZILZ".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec![
                "I".to_owned(),
                "Landroid/util/TypedValue;".to_owned(),
                "Z".to_owned(),
            ],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from(
            "Landroid/content/res/Resources$Theme;.bootstrap",
        )),
        DexValue::Int(2130968624),
        DexValue::Object(DexObjectRef {
            id: 1,
            class_descriptor: std::rc::Rc::from("Landroid/util/TypedValue;"),
        }),
        DexValue::Int(1),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Int(0))
    );
}

#[test]
fn builtin_theme_apply_style_returns_void() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources$Theme;".to_owned(),
        name: "applyStyle".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VIZ".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["I".to_owned(), "Z".to_owned()],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from(
            "Landroid/content/res/Resources$Theme;.bootstrap",
        )),
        DexValue::Int(2131886440),
        DexValue::Int(0),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Void)
    );
}

#[test]
fn builtin_activity_set_theme_resolves_style_bag_attribute() {
    let program = DexProgram::with_resource_context(
        Vec::new(),
        None,
        None,
        None,
        None,
        Some(minimal_style_resource_table(0x7f02_0000, 0xff12_3456)),
    );
    let set_theme = DexMethodId {
        class_descriptor: "Landroid/app/Activity;".to_owned(),
        name: "setTheme".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VI".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_theme = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getTheme".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/content/res/Resources$Theme;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let resolve_attribute = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources$Theme;".to_owned(),
        name: "resolveAttribute".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZILZ".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec![
                "I".to_owned(),
                "Landroid/util/TypedValue;".to_owned(),
                "Z".to_owned(),
            ],
        },
    };
    let activity = DexValue::Object(DexObjectRef {
        id: 7,
        class_descriptor: std::rc::Rc::from("Lcom/example/MainActivity;"),
    });
    let out_value = DexValue::Object(DexObjectRef {
        id: 8,
        class_descriptor: std::rc::Rc::from("Landroid/util/TypedValue;"),
    });

    assert_eq!(
        program.invoke_builtin_method(
            &set_theme,
            Some(&test_invocation(vec![
                activity.clone(),
                DexValue::Int(0x7f01_0000)
            ]))
        ),
        Some(DexExecutionResult::Void)
    );
    let Some(DexExecutionResult::Value(theme)) =
        program.invoke_builtin_method(&get_theme, Some(&test_invocation(vec![activity])))
    else {
        panic!("getTheme should return a theme");
    };

    assert_eq!(
        program.invoke_builtin_method(
            &resolve_attribute,
            Some(&test_invocation(vec![
                theme,
                DexValue::Int(0x7f02_0000),
                out_value.clone(),
                DexValue::Int(1),
            ]))
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("type")),
        Some(DexValue::Int(0x1c))
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("data")),
        Some(DexValue::Int(0xff12_3456u32 as i32))
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("resourceId")),
        Some(DexValue::Int(0x7f02_0000))
    );
}

#[test]
fn builtin_theme_apply_style_resolves_parent_force_and_attribute_references() {
    let base_attr = 0x7f02_0000;
    let alias_attr = 0x7f02_0001;
    let program = DexProgram::with_resource_context(
        Vec::new(),
        None,
        None,
        None,
        None,
        Some(style_resource_table(&[
            TestStyleResource {
                name: "BaseTheme",
                parent: None,
                entries: vec![TestStyleEntry {
                    name: base_attr,
                    value_type: 0x1c,
                    data: 0xff11_1111,
                }],
            },
            TestStyleResource {
                name: "ChildTheme",
                parent: Some(0x7f01_0000),
                entries: vec![TestStyleEntry {
                    name: alias_attr,
                    value_type: 0x02,
                    data: base_attr,
                }],
            },
            TestStyleResource {
                name: "OverlayTheme",
                parent: None,
                entries: vec![TestStyleEntry {
                    name: base_attr,
                    value_type: 0x1c,
                    data: 0xff33_3333,
                }],
            },
        ])),
    );
    let apply_style = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources$Theme;".to_owned(),
        name: "applyStyle".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VIZ".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["I".to_owned(), "Z".to_owned()],
        },
    };
    let resolve_attribute = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources$Theme;".to_owned(),
        name: "resolveAttribute".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZILZ".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec![
                "I".to_owned(),
                "Landroid/util/TypedValue;".to_owned(),
                "Z".to_owned(),
            ],
        },
    };
    let non_force_theme = DexValue::Reference(std::rc::Rc::from(
        "Landroid/content/res/Resources$Theme;:non-force",
    ));
    let forced_theme = DexValue::Reference(std::rc::Rc::from(
        "Landroid/content/res/Resources$Theme;:forced",
    ));
    let inherited_out = DexValue::Object(DexObjectRef {
        id: 31,
        class_descriptor: std::rc::Rc::from("Landroid/util/TypedValue;"),
    });
    let alias_out = DexValue::Object(DexObjectRef {
        id: 32,
        class_descriptor: std::rc::Rc::from("Landroid/util/TypedValue;"),
    });
    let raw_alias_out = DexValue::Object(DexObjectRef {
        id: 33,
        class_descriptor: std::rc::Rc::from("Landroid/util/TypedValue;"),
    });
    let forced_out = DexValue::Object(DexObjectRef {
        id: 34,
        class_descriptor: std::rc::Rc::from("Landroid/util/TypedValue;"),
    });

    assert_eq!(
        program.invoke_builtin_method(
            &apply_style,
            Some(&test_invocation(vec![
                non_force_theme.clone(),
                DexValue::Int(0x7f01_0001),
                DexValue::Int(1),
            ]))
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &apply_style,
            Some(&test_invocation(vec![
                non_force_theme.clone(),
                DexValue::Int(0x7f01_0002),
                DexValue::Int(0),
            ]))
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &resolve_attribute,
            Some(&test_invocation(vec![
                non_force_theme.clone(),
                DexValue::Int(base_attr as i32),
                inherited_out.clone(),
                DexValue::Int(1),
            ]))
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.read_stored_instance_field(&inherited_out, &typed_value_test_int_field("data")),
        Some(DexValue::Int(0xff11_1111u32 as i32))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &resolve_attribute,
            Some(&test_invocation(vec![
                non_force_theme.clone(),
                DexValue::Int(alias_attr as i32),
                alias_out.clone(),
                DexValue::Int(1),
            ]))
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.read_stored_instance_field(&alias_out, &typed_value_test_int_field("data")),
        Some(DexValue::Int(0xff11_1111u32 as i32))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &resolve_attribute,
            Some(&test_invocation(vec![
                non_force_theme,
                DexValue::Int(alias_attr as i32),
                raw_alias_out.clone(),
                DexValue::Int(0),
            ]))
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.read_stored_instance_field(&raw_alias_out, &typed_value_test_int_field("type")),
        Some(DexValue::Int(0x02))
    );
    assert_eq!(
        program.read_stored_instance_field(&raw_alias_out, &typed_value_test_int_field("data")),
        Some(DexValue::Int(base_attr as i32))
    );

    assert_eq!(
        program.invoke_builtin_method(
            &apply_style,
            Some(&test_invocation(vec![
                forced_theme.clone(),
                DexValue::Int(0x7f01_0001),
                DexValue::Int(1),
            ]))
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &apply_style,
            Some(&test_invocation(vec![
                forced_theme.clone(),
                DexValue::Int(0x7f01_0002),
                DexValue::Int(1),
            ]))
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &resolve_attribute,
            Some(&test_invocation(vec![
                forced_theme,
                DexValue::Int(base_attr as i32),
                forced_out.clone(),
                DexValue::Int(1),
            ]))
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.read_stored_instance_field(&forced_out, &typed_value_test_int_field("data")),
        Some(DexValue::Int(0xff33_3333u32 as i32))
    );
}

#[test]
fn builtin_theme_resolve_attribute_preserves_complex_style_references() {
    let style_attr = 0x7f02_0000;
    let style_resource = 0x7f01_0001;
    let program = DexProgram::with_resource_context(
        Vec::new(),
        None,
        None,
        None,
        None,
        Some(style_resource_table(&[
            TestStyleResource {
                name: "AppTheme",
                parent: None,
                entries: vec![TestStyleEntry {
                    name: style_attr,
                    value_type: 0x01,
                    data: style_resource,
                }],
            },
            TestStyleResource {
                name: "PreferenceScreenStyle",
                parent: None,
                entries: vec![TestStyleEntry {
                    name: 0x7f02_0001,
                    value_type: 0x1c,
                    data: 0xff12_3456,
                }],
            },
        ])),
    );
    let resolve_attribute = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources$Theme;".to_owned(),
        name: "resolveAttribute".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZILZ".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec![
                "I".to_owned(),
                "Landroid/util/TypedValue;".to_owned(),
                "Z".to_owned(),
            ],
        },
    };
    let theme = DexValue::Reference(std::rc::Rc::from(
        "Landroid/content/res/Resources$Theme;:settings",
    ));
    let out_value = DexValue::Object(DexObjectRef {
        id: 35,
        class_descriptor: std::rc::Rc::from("Landroid/util/TypedValue;"),
    });

    assert_eq!(
        program.invoke_builtin_method(
            &DexMethodId {
                class_descriptor: "Landroid/content/res/Resources$Theme;".to_owned(),
                name: "applyStyle".to_owned(),
                proto: crate::dex::DexProto {
                    shorty: "VIZ".to_owned(),
                    return_type: "V".to_owned(),
                    parameters: vec!["I".to_owned(), "Z".to_owned()],
                },
            },
            Some(&test_invocation(vec![
                theme.clone(),
                DexValue::Int(0x7f01_0000),
                DexValue::Int(1),
            ]))
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &resolve_attribute,
            Some(&test_invocation(vec![
                theme,
                DexValue::Int(style_attr as i32),
                out_value.clone(),
                DexValue::Int(1),
            ]))
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("type")),
        Some(DexValue::Int(0x01))
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("data")),
        Some(DexValue::Int(style_resource as i32))
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("resourceId")),
        Some(DexValue::Int(style_resource as i32))
    );
}

// cite: frameworks-base Context.java#getString, getText
// cite: frameworks-base Resources.java#getString, getText, NotFoundException
#[test]
fn builtin_context_missing_string_throws_catchable_not_found_with_resource_id() {
    let program = DexProgram::new(Vec::new());
    let get_string = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getString".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_text = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getText".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Ljava/lang/CharSequence;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("context")),
        DexValue::Int(2131951620),
    ]);

    assert_resource_not_found(
        &program,
        program.invoke_builtin_method(&get_string, Some(&invocation)),
        "0x7f130004",
    );
    assert_resource_not_found(
        &program,
        program.invoke_builtin_method(&get_text, Some(&invocation)),
        "0x7f130004",
    );
}

#[test]
fn builtin_resources_get_string_and_text_resolve_loaded_resource_table() {
    let program = DexProgram::with_resource_context(
        Vec::new(),
        None,
        None,
        None,
        None,
        Some(minimal_test_resource_table(
            "string",
            "popup_title",
            "Create timer",
        )),
    );
    let get_string = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "getString".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_text = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "getText".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Ljava/lang/CharSequence;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from(
            "Landroid/content/res/Resources;.bootstrap",
        )),
        DexValue::Int(0x7f01_0000),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&get_string, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from("Create timer")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_text, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from("Create timer")
        )))
    );
}

// cite: frameworks-base Context.java#getColor, getDrawable
// cite: frameworks-base Resources.java#getColor, NotFoundException
#[test]
fn builtin_context_get_drawable_returns_reference_and_missing_color_throws() {
    let program = DexProgram::new(Vec::new());
    let get_drawable = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getDrawable".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Landroid/graphics/drawable/Drawable;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_color = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getColor".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "II".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("context")),
        DexValue::Int(2131230999),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&get_drawable, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/graphics/drawable/Drawable;:resource:2131230999")
        )))
    );
    assert_resource_not_found(
        &program,
        program.invoke_builtin_method(&get_color, Some(&invocation)),
        "0x7f080117",
    );
}

#[test]
fn builtin_resources_get_drawable_theme_overload_returns_resource_reference() {
    let program = DexProgram::new(Vec::new());
    let get_drawable = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "getDrawable".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LIL".to_owned(),
            return_type: "Landroid/graphics/drawable/Drawable;".to_owned(),
            parameters: vec![
                "I".to_owned(),
                "Landroid/content/res/Resources$Theme;".to_owned(),
            ],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from(
            "Landroid/content/res/Resources;.bootstrap",
        )),
        DexValue::Int(2131230999),
        DexValue::Null,
    ]);

    // cite: frameworks-base Resources.java#getDrawable
    // cite: androidx ImageResources.android.kt#imageResource
    assert_eq!(
        program.invoke_builtin_method(&get_drawable, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/graphics/drawable/Drawable;:resource:2131230999")
        )))
    );
}

#[test]
fn builtin_context_get_resources_returns_bootstrap_resources() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getResources".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/content/res/Resources;".to_owned(),
            parameters: Vec::new(),
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&method, None),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/content/res/Resources;.bootstrap")
        )))
    );
}

#[test]
fn builtin_resources_get_system_returns_bootstrap_resources() {
    // cite: frameworks-base Resources.java#getSystem
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "getSystem".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/content/res/Resources;".to_owned(),
            parameters: Vec::new(),
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&method, None),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/content/res/Resources;.bootstrap")
        )))
    );
}

#[test]
fn builtin_context_get_application_info_returns_bootstrap_info() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getApplicationInfo".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/content/pm/ApplicationInfo;".to_owned(),
            parameters: Vec::new(),
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&method, None),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/content/pm/ApplicationInfo;.bootstrap")
        )))
    );
}

// cite: frameworks-base Resources.java#getDimension, getDimensionPixelSize, getBoolean, getColor, NotFoundException
#[test]
fn builtin_resources_missing_values_throw_and_framework_colors_resolve() {
    let program = DexProgram::new(Vec::new());
    let get_dimension = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "getDimension".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "FI".to_owned(),
            return_type: "F".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_dimension_pixel_size = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "getDimensionPixelSize".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "II".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_boolean = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "getBoolean".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZI".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_color = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "getColor".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "II".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_color_with_theme = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "getColor".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "IIL".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec![
                "I".to_owned(),
                "Landroid/content/res/Resources$Theme;".to_owned(),
            ],
        },
    };
    let get_resource_name = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "getResourceName".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_configuration = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "getConfiguration".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/content/res/Configuration;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let get_display_metrics = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "getDisplayMetrics".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/util/DisplayMetrics;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let update_configuration = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "updateConfiguration".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VLL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec![
                "Landroid/content/res/Configuration;".to_owned(),
                "Landroid/util/DisplayMetrics;".to_owned(),
            ],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("resources")),
        DexValue::Int(2131165300),
    ]);
    let themed_invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("resources")),
        DexValue::Int(2131165300),
        DexValue::Reference(std::rc::Rc::from("theme")),
    ]);
    let framework_white_invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("resources")),
        DexValue::Int(0x0106_000b),
    ]);
    let framework_black_invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("resources")),
        DexValue::Int(0x0106_000c),
    ]);
    let framework_transparent_invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("resources")),
        DexValue::Int(0x0106_000d),
    ]);
    let framework_primary_light_invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("resources")),
        DexValue::Int(0x0106_0060),
    ]);
    let framework_accent2_100_invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("resources")),
        DexValue::Int(0x0106_0047),
    ]);
    let update_invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("resources")),
        DexValue::Reference(std::rc::Rc::from("configuration")),
        DexValue::Reference(std::rc::Rc::from("display-metrics")),
    ]);

    assert_resource_not_found(
        &program,
        program.invoke_builtin_method(&get_dimension, Some(&invocation)),
        "0x7f070074",
    );
    assert_resource_not_found(
        &program,
        program.invoke_builtin_method(&get_dimension_pixel_size, Some(&invocation)),
        "0x7f070074",
    );
    assert_resource_not_found(
        &program,
        program.invoke_builtin_method(&get_boolean, Some(&invocation)),
        "0x7f070074",
    );
    assert_resource_not_found(
        &program,
        program.invoke_builtin_method(&get_color, Some(&invocation)),
        "0x7f070074",
    );
    assert_resource_not_found(
        &program,
        program.invoke_builtin_method(&get_color_with_theme, Some(&themed_invocation)),
        "0x7f070074",
    );
    assert_eq!(
        program.invoke_builtin_method(&get_color, Some(&framework_white_invocation)),
        Some(DexExecutionResult::Int(-1))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_color, Some(&framework_black_invocation)),
        Some(DexExecutionResult::Int(0xff00_0000u32 as i32))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_color, Some(&framework_transparent_invocation)),
        Some(DexExecutionResult::Int(0))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_color, Some(&framework_primary_light_invocation)),
        Some(DexExecutionResult::Int(0xff2a_4174u32 as i32))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_color, Some(&framework_accent2_100_invocation)),
        Some(DexExecutionResult::Int(0xffdc_e2f9u32 as i32))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_resource_name, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from("resource:2131165300")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_configuration, None),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/content/res/Configuration;.bootstrap")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_display_metrics, None),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/util/DisplayMetrics;.bootstrap")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(&update_configuration, Some(&update_invocation)),
        Some(DexExecutionResult::Void)
    );
}

#[test]
fn builtin_resources_get_value_writes_file_backed_typed_value() {
    let program = DexProgram::with_resource_context(
        Vec::new(),
        None,
        None,
        None,
        None,
        Some(minimal_test_resource_table(
            "drawable",
            "icon",
            "res/drawable/icon.xml",
        )),
    );
    let get_value = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "getValue".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VILZ".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec![
                "I".to_owned(),
                "Landroid/util/TypedValue;".to_owned(),
                "Z".to_owned(),
            ],
        },
    };
    let out_value = DexValue::Object(DexObjectRef {
        id: 7,
        class_descriptor: std::rc::Rc::from("Landroid/util/TypedValue;"),
    });
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("resources")),
        DexValue::Int(0x7f01_0000),
        out_value.clone(),
        DexValue::Int(1),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&get_value, Some(&invocation)),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("type")),
        Some(DexValue::Int(0x03))
    );
    assert_eq!(
        program.read_stored_instance_field(
            &out_value,
            &typed_value_test_char_sequence_field("string")
        ),
        Some(DexValue::String(std::rc::Rc::from("res/drawable/icon.xml")))
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("resourceId")),
        Some(DexValue::Int(0x7f01_0000))
    );
}

#[test]
fn android_complex_dimensions_decode_to_pixel_sizes() {
    assert_eq!(android_complex_dimension_to_pixel_size(0x0000_1001), 16);
    assert_eq!(android_complex_dimension_to_pixel_size(0x0000_0e02), 14);
    assert_eq!(android_dimension_float_to_pixel_size(0.2), 1);
    assert_eq!(android_dimension_float_to_pixel_size(-0.2), -1);
}

#[test]
fn builtin_configuration_constructor_returns_void() {
    let program = DexProgram::new(Vec::new());
    let constructor = DexMethodId {
        class_descriptor: "Landroid/content/res/Configuration;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let copy_constructor = DexMethodId {
        class_descriptor: "Landroid/content/res/Configuration;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["Landroid/content/res/Configuration;".to_owned()],
        },
    };
    let get_locales = DexMethodId {
        class_descriptor: "Landroid/content/res/Configuration;".to_owned(),
        name: "getLocales".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/os/LocaleList;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let get_layout_direction = DexMethodId {
        class_descriptor: "Landroid/content/res/Configuration;".to_owned(),
        name: "getLayoutDirection".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "I".to_owned(),
            return_type: "I".to_owned(),
            parameters: Vec::new(),
        },
    };
    let update_from = DexMethodId {
        class_descriptor: "Landroid/content/res/Configuration;".to_owned(),
        name: "updateFrom".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "IL".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["Landroid/content/res/Configuration;".to_owned()],
        },
    };
    let receiver = DexValue::Reference(std::rc::Rc::from("receiver-configuration"));
    let delta = DexValue::Reference(std::rc::Rc::from("delta-configuration"));
    let default_receiver = DexValue::Reference(std::rc::Rc::from("default-configuration"));
    let copy_receiver = DexValue::Reference(std::rc::Rc::from("copy-receiver-configuration"));
    let copy_source = DexValue::Reference(std::rc::Rc::from("copy-source-configuration"));
    let orientation_field = DexFieldId {
        class_descriptor: "Landroid/content/res/Configuration;".to_owned(),
        type_descriptor: "I".to_owned(),
        name: "orientation".to_owned(),
    };
    let screen_width_field = DexFieldId {
        class_descriptor: "Landroid/content/res/Configuration;".to_owned(),
        type_descriptor: "I".to_owned(),
        name: "screenWidthDp".to_owned(),
    };
    let font_scale_field = DexFieldId {
        class_descriptor: "Landroid/content/res/Configuration;".to_owned(),
        type_descriptor: "F".to_owned(),
        name: "fontScale".to_owned(),
    };
    let ui_mode_field = DexFieldId {
        class_descriptor: "Landroid/content/res/Configuration;".to_owned(),
        type_descriptor: "I".to_owned(),
        name: "uiMode".to_owned(),
    };
    let update_invocation = test_invocation(vec![receiver.clone(), delta.clone()]);
    let constructor_invocation = test_invocation(vec![default_receiver.clone()]);
    let copy_invocation = test_invocation(vec![copy_receiver.clone(), copy_source.clone()]);

    assert!(program.write_stored_instance_field(&delta, &orientation_field, DexValue::Int(1),));
    assert!(program.write_stored_instance_field(&delta, &screen_width_field, DexValue::Int(411),));
    assert!(program.write_stored_instance_field(&delta, &font_scale_field, DexValue::Float(1.0),));
    assert!(program.write_stored_instance_field(
        &copy_source,
        &font_scale_field,
        DexValue::Float(1.25),
    ));

    assert_eq!(
        program.invoke_builtin_method(&constructor, Some(&constructor_invocation)),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.read_stored_instance_field(&default_receiver, &font_scale_field),
        Some(DexValue::Float(BOOTSTRAP_FONT_SCALE))
    );
    assert_eq!(
        program.read_stored_instance_field(&default_receiver, &ui_mode_field),
        // cite: frameworks-base UiModeManager.java#MODE_NIGHT_NO
        Some(DexValue::Int(0x11))
    );
    assert_eq!(
        program.invoke_builtin_method(&copy_constructor, Some(&copy_invocation)),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.read_stored_instance_field(&copy_receiver, &font_scale_field),
        Some(DexValue::Float(1.25))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_locales, None),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/os/LocaleList;.bootstrap")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_layout_direction, None),
        Some(DexExecutionResult::Int(0))
    );
    assert_eq!(
        program.invoke_builtin_method(&update_from, Some(&update_invocation)),
        Some(DexExecutionResult::Int(0x4000_0480))
    );
    assert_eq!(
        program.read_stored_instance_field(&receiver, &orientation_field),
        Some(DexValue::Int(1))
    );
    assert_eq!(
        program.read_stored_instance_field(&receiver, &screen_width_field),
        Some(DexValue::Int(411))
    );
    assert_eq!(
        program.read_stored_instance_field(&receiver, &font_scale_field),
        Some(DexValue::Float(1.0))
    );
    assert_eq!(
        program.invoke_builtin_method(&update_from, Some(&update_invocation)),
        Some(DexExecutionResult::Int(0))
    );
}

#[test]
fn builtin_configuration_uses_winedroid_ui_mode_config() {
    let program = DexProgram::with_resource_context_and_config(
        Vec::new(),
        None,
        None,
        None,
        None,
        None,
        crate::config::WinedroidConfig::with_ui_mode(crate::config::WinedroidUiMode::Dark),
    );
    let constructor = DexMethodId {
        class_descriptor: "Landroid/content/res/Configuration;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let resources_get_configuration = DexMethodId {
        class_descriptor: "Landroid/content/res/Resources;".to_owned(),
        name: "getConfiguration".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/content/res/Configuration;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let constructed = DexValue::Reference(std::rc::Rc::from("constructed-configuration"));
    let ui_mode_field = DexFieldId {
        class_descriptor: "Landroid/content/res/Configuration;".to_owned(),
        type_descriptor: "I".to_owned(),
        name: "uiMode".to_owned(),
    };

    assert_eq!(
        program.invoke_builtin_method(
            &constructor,
            Some(&test_invocation(vec![constructed.clone()]))
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.read_stored_instance_field(&constructed, &ui_mode_field),
        Some(DexValue::Int(0x21))
    );
    assert_eq!(
        program.invoke_builtin_method(&resources_get_configuration, None),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/content/res/Configuration;.bootstrap")
        )))
    );
    assert_eq!(
        program.read_stored_instance_field(
            &DexValue::Reference(std::rc::Rc::from(
                "Landroid/content/res/Configuration;.bootstrap"
            )),
            &ui_mode_field
        ),
        Some(DexValue::Int(0x21))
    );
}

#[test]
fn builtin_context_obtain_styled_attributes_reads_theme_values() {
    let program = DexProgram::with_resource_context(
        Vec::new(),
        None,
        None,
        None,
        None,
        Some(minimal_style_resource_table(0x7f02_0000, 0xff12_3456)),
    );
    let set_theme = DexMethodId {
        class_descriptor: "Landroid/app/Activity;".to_owned(),
        name: "setTheme".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VI".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let obtain = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "obtainStyledAttributes".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Landroid/content/res/TypedArray;".to_owned(),
            parameters: vec!["[I".to_owned()],
        },
    };
    let has_value = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "hasValue".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZI".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_value = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getValue".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZIL".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec!["I".to_owned(), "Landroid/util/TypedValue;".to_owned()],
        },
    };
    let get_color_state_list = DexMethodId {
        class_descriptor: "Landroid/content/res/TypedArray;".to_owned(),
        name: "getColorStateList".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Landroid/content/res/ColorStateList;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_default_color = DexMethodId {
        class_descriptor: "Landroid/content/res/ColorStateList;".to_owned(),
        name: "getDefaultColor".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "I".to_owned(),
            return_type: "I".to_owned(),
            parameters: Vec::new(),
        },
    };
    let activity = DexValue::Object(DexObjectRef {
        id: 17,
        class_descriptor: std::rc::Rc::from("Lcom/example/MainActivity;"),
    });
    let out_value = DexValue::Object(DexObjectRef {
        id: 18,
        class_descriptor: std::rc::Rc::from("Landroid/util/TypedValue;"),
    });
    let styleable = program.allocate_filled_array(
        "[I".to_owned(),
        vec![DexValue::Int(0x7f02_0000), DexValue::Int(0x7f02_0001)],
    );

    assert_eq!(
        program.invoke_builtin_method(
            &set_theme,
            Some(&test_invocation(vec![
                activity.clone(),
                DexValue::Int(0x7f01_0000)
            ]))
        ),
        Some(DexExecutionResult::Void)
    );
    let Some(DexExecutionResult::Value(typed_array)) =
        program.invoke_builtin_method(&obtain, Some(&test_invocation(vec![activity, styleable])))
    else {
        panic!("obtainStyledAttributes should return a typed array");
    };

    assert_eq!(
        program.invoke_builtin_method(
            &has_value,
            Some(&test_invocation(vec![
                typed_array.clone(),
                DexValue::Int(0)
            ]))
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &has_value,
            Some(&test_invocation(vec![
                typed_array.clone(),
                DexValue::Int(1)
            ]))
        ),
        Some(DexExecutionResult::Int(0))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &get_value,
            Some(&test_invocation(vec![
                typed_array.clone(),
                DexValue::Int(0),
                out_value.clone(),
            ]))
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("type")),
        Some(DexValue::Int(0x1c))
    );
    assert_eq!(
        program.read_stored_instance_field(&out_value, &typed_value_test_int_field("data")),
        Some(DexValue::Int(0xff12_3456u32 as i32))
    );

    let Some(DexExecutionResult::Value(color_state_list)) = program.invoke_builtin_method(
        &get_color_state_list,
        Some(&test_invocation(vec![typed_array, DexValue::Int(0)])),
    ) else {
        panic!("getColorStateList should return a color state list");
    };
    assert_eq!(
        program.invoke_builtin_method(
            &get_default_color,
            Some(&test_invocation(vec![color_state_list]))
        ),
        Some(DexExecutionResult::Int(0xff12_3456u32 as i32))
    );
}

#[test]
fn builtin_locale_list_to_language_tags_returns_bootstrap_tag() {
    let program = DexProgram::new(Vec::new());
    let constructor = DexMethodId {
        class_descriptor: "Landroid/os/LocaleList;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["[Ljava/util/Locale;".to_owned()],
        },
    };
    let to_language_tags = DexMethodId {
        class_descriptor: "Landroid/os/LocaleList;".to_owned(),
        name: "toLanguageTags".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let get_default = DexMethodId {
        class_descriptor: "Landroid/os/LocaleList;".to_owned(),
        name: "getDefault".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/os/LocaleList;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let is_empty = DexMethodId {
        class_descriptor: "Landroid/os/LocaleList;".to_owned(),
        name: "isEmpty".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "Z".to_owned(),
            return_type: "Z".to_owned(),
            parameters: Vec::new(),
        },
    };
    let size = DexMethodId {
        class_descriptor: "Landroid/os/LocaleList;".to_owned(),
        name: "size".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "I".to_owned(),
            return_type: "I".to_owned(),
            parameters: Vec::new(),
        },
    };
    let get = DexMethodId {
        class_descriptor: "Landroid/os/LocaleList;".to_owned(),
        name: "get".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Ljava/util/Locale;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let invocation = test_invocation(vec![DexValue::Reference(std::rc::Rc::from(
        "Landroid/os/LocaleList;.bootstrap",
    ))]);
    let get_invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("Landroid/os/LocaleList;.bootstrap")),
        DexValue::Int(0),
    ]);
    let constructor_invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("Landroid/os/LocaleList;.bootstrap")),
        DexValue::Object(DexObjectRef {
            id: 1,
            class_descriptor: std::rc::Rc::from("[Ljava/util/Locale;"),
        }),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&constructor, Some(&constructor_invocation)),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&get_default, None),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/os/LocaleList;.bootstrap")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(&to_language_tags, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from("en-US")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(&is_empty, Some(&invocation)),
        Some(DexExecutionResult::Int(0))
    );
    assert_eq!(
        program.invoke_builtin_method(&size, Some(&invocation)),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.invoke_builtin_method(&get, Some(&get_invocation)),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Ljava/util/Locale;:default")
        )))
    );
}

fn minimal_test_resource_table(type_name: &str, key_name: &str, value: &str) -> Vec<u8> {
    const RES_STRING_POOL_TYPE: u16 = 0x0001;
    const RES_TABLE_TYPE: u16 = 0x0002;
    const RES_TABLE_PACKAGE_TYPE: u16 = 0x0200;
    const RES_TABLE_TYPE_TYPE: u16 = 0x0201;
    const UTF8_FLAG: u32 = 0x0000_0100;
    const VALUE_TYPE_STRING: u8 = 0x03;

    fn push_u16(data: &mut Vec<u8>, value: u16) {
        data.extend(value.to_le_bytes());
    }

    fn push_u32(data: &mut Vec<u8>, value: u32) {
        data.extend(value.to_le_bytes());
    }

    fn push_utf8_string(data: &mut Vec<u8>, value: &str) {
        let length = value.len();
        assert!(length < 0x80);
        data.push(length as u8);
        data.push(length as u8);
        data.extend(value.as_bytes());
        data.push(0);
    }

    fn string_pool(values: &[&str]) -> Vec<u8> {
        let mut string_data = Vec::new();
        let mut offsets = Vec::new();

        for value in values {
            offsets.push(string_data.len() as u32);
            push_utf8_string(&mut string_data, value);
        }

        let header_size = 28usize;
        let strings_start = header_size + values.len() * 4;
        let chunk_size = strings_start + string_data.len();
        let mut pool = Vec::new();

        push_u16(&mut pool, RES_STRING_POOL_TYPE);
        push_u16(&mut pool, header_size as u16);
        push_u32(&mut pool, chunk_size as u32);
        push_u32(&mut pool, values.len() as u32);
        push_u32(&mut pool, 0);
        push_u32(&mut pool, UTF8_FLAG);
        push_u32(&mut pool, strings_start as u32);
        push_u32(&mut pool, 0);

        for offset in offsets {
            push_u32(&mut pool, offset);
        }

        pool.extend(string_data);
        pool
    }

    let global_strings = string_pool(&[value]);
    let type_strings = string_pool(&[type_name]);
    let key_strings = string_pool(&[key_name]);
    let mut type_chunk = Vec::new();

    push_u16(&mut type_chunk, RES_TABLE_TYPE_TYPE);
    push_u16(&mut type_chunk, 20);
    push_u32(&mut type_chunk, 40);
    type_chunk.push(0x01);
    type_chunk.push(0);
    push_u16(&mut type_chunk, 0);
    push_u32(&mut type_chunk, 1);
    push_u32(&mut type_chunk, 24);
    push_u32(&mut type_chunk, 0);
    push_u16(&mut type_chunk, 8);
    push_u16(&mut type_chunk, 0);
    push_u32(&mut type_chunk, 0);
    push_u16(&mut type_chunk, 8);
    type_chunk.push(0);
    type_chunk.push(VALUE_TYPE_STRING);
    push_u32(&mut type_chunk, 0);

    let type_strings_offset = 288u32;
    let key_strings_offset = type_strings_offset + type_strings.len() as u32;
    let package_size = 288 + type_strings.len() + key_strings.len() + type_chunk.len();
    let mut package = Vec::new();

    push_u16(&mut package, RES_TABLE_PACKAGE_TYPE);
    push_u16(&mut package, 288);
    push_u32(&mut package, package_size as u32);
    push_u32(&mut package, 0x7f);
    package.extend(vec![0; 256]);
    push_u32(&mut package, type_strings_offset);
    push_u32(&mut package, 0);
    push_u32(&mut package, key_strings_offset);
    push_u32(&mut package, 0);
    push_u32(&mut package, 0);
    package.extend(type_strings);
    package.extend(key_strings);
    package.extend(type_chunk);

    let table_size = 12 + global_strings.len() + package.len();
    let mut table = Vec::new();
    push_u16(&mut table, RES_TABLE_TYPE);
    push_u16(&mut table, 12);
    push_u32(&mut table, table_size as u32);
    push_u32(&mut table, 1);
    table.extend(global_strings);
    table.extend(package);
    table
}

struct TestStyleResource<'a> {
    name: &'a str,
    parent: Option<u32>,
    entries: Vec<TestStyleEntry>,
}

struct TestStyleEntry {
    name: u32,
    value_type: u8,
    data: u32,
}

fn minimal_style_resource_table(attribute_id: u32, color: u32) -> Vec<u8> {
    style_resource_table(&[TestStyleResource {
        name: "AppTheme",
        parent: None,
        entries: vec![TestStyleEntry {
            name: attribute_id,
            value_type: 0x1c,
            data: color,
        }],
    }])
}

fn style_resource_table(styles: &[TestStyleResource<'_>]) -> Vec<u8> {
    const RES_STRING_POOL_TYPE: u16 = 0x0001;
    const RES_TABLE_TYPE: u16 = 0x0002;
    const RES_TABLE_PACKAGE_TYPE: u16 = 0x0200;
    const RES_TABLE_TYPE_TYPE: u16 = 0x0201;
    const UTF8_FLAG: u32 = 0x0000_0100;
    const ENTRY_FLAG_COMPLEX: u16 = 0x0001;

    fn push_u16(data: &mut Vec<u8>, value: u16) {
        data.extend(value.to_le_bytes());
    }

    fn push_u32(data: &mut Vec<u8>, value: u32) {
        data.extend(value.to_le_bytes());
    }

    fn push_utf8_string(data: &mut Vec<u8>, value: &str) {
        let length = value.len();
        assert!(length < 0x80);
        data.push(length as u8);
        data.push(length as u8);
        data.extend(value.as_bytes());
        data.push(0);
    }

    fn string_pool(values: &[&str]) -> Vec<u8> {
        let mut string_data = Vec::new();
        let mut offsets = Vec::new();

        for value in values {
            offsets.push(string_data.len() as u32);
            push_utf8_string(&mut string_data, value);
        }

        let header_size = 28usize;
        let strings_start = header_size + values.len() * 4;
        let chunk_size = strings_start + string_data.len();
        let mut pool = Vec::new();

        push_u16(&mut pool, RES_STRING_POOL_TYPE);
        push_u16(&mut pool, header_size as u16);
        push_u32(&mut pool, chunk_size as u32);
        push_u32(&mut pool, values.len() as u32);
        push_u32(&mut pool, 0);
        push_u32(&mut pool, UTF8_FLAG);
        push_u32(&mut pool, strings_start as u32);
        push_u32(&mut pool, 0);

        for offset in offsets {
            push_u32(&mut pool, offset);
        }

        pool.extend(string_data);
        pool
    }

    let global_strings = string_pool(&[]);
    let type_strings = string_pool(&["style"]);
    let key_names = styles.iter().map(|style| style.name).collect::<Vec<_>>();
    let key_strings = string_pool(&key_names);
    let mut entry_offsets = Vec::new();
    let mut entry_data = Vec::new();

    for (style_index, style) in styles.iter().enumerate() {
        entry_offsets.push(entry_data.len() as u32);
        push_u16(&mut entry_data, 16);
        push_u16(&mut entry_data, ENTRY_FLAG_COMPLEX);
        push_u32(&mut entry_data, style_index as u32);
        push_u32(&mut entry_data, style.parent.unwrap_or(0));
        push_u32(&mut entry_data, style.entries.len() as u32);

        for entry in &style.entries {
            push_u32(&mut entry_data, entry.name);
            push_u16(&mut entry_data, 8);
            entry_data.push(0);
            entry_data.push(entry.value_type);
            push_u32(&mut entry_data, entry.data);
        }
    }

    let entries_start = 20 + entry_offsets.len() * 4;
    let type_chunk_size = entries_start + entry_data.len();
    let mut type_chunk = Vec::new();

    push_u16(&mut type_chunk, RES_TABLE_TYPE_TYPE);
    push_u16(&mut type_chunk, 20);
    push_u32(&mut type_chunk, type_chunk_size as u32);
    type_chunk.push(0x01);
    type_chunk.push(0);
    push_u16(&mut type_chunk, 0);
    push_u32(&mut type_chunk, styles.len() as u32);
    push_u32(&mut type_chunk, entries_start as u32);
    for offset in entry_offsets {
        push_u32(&mut type_chunk, offset);
    }
    type_chunk.extend(entry_data);

    let type_strings_offset = 288u32;
    let key_strings_offset = type_strings_offset + type_strings.len() as u32;
    let package_size = 288 + type_strings.len() + key_strings.len() + type_chunk.len();
    let mut package = Vec::new();

    push_u16(&mut package, RES_TABLE_PACKAGE_TYPE);
    push_u16(&mut package, 288);
    push_u32(&mut package, package_size as u32);
    push_u32(&mut package, 0x7f);
    package.extend(vec![0; 256]);
    push_u32(&mut package, type_strings_offset);
    push_u32(&mut package, 0);
    push_u32(&mut package, key_strings_offset);
    push_u32(&mut package, 0);
    push_u32(&mut package, 0);
    package.extend(type_strings);
    package.extend(key_strings);
    package.extend(type_chunk);

    let table_size = 12 + global_strings.len() + package.len();
    let mut table = Vec::new();
    push_u16(&mut table, RES_TABLE_TYPE);
    push_u16(&mut table, 12);
    push_u32(&mut table, table_size as u32);
    push_u32(&mut table, 1);
    table.extend(global_strings);
    table.extend(package);
    table
}

// cite: frameworks-base Resources.java#NotFoundException
// cite: libcore Throwable.java#detailMessage
fn assert_resource_not_found(
    program: &DexProgram,
    result: Option<DexExecutionResult>,
    expected_message: &str,
) {
    let Some(DexExecutionResult::Throw(exception)) = result else {
        panic!("resource lookup should throw Resources.NotFoundException: {result:?}");
    };
    let detail_message = DexFieldId {
        class_descriptor: "Ljava/lang/Throwable;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "detailMessage".to_owned(),
    };

    assert!(program.value_matches_type(
        &exception,
        "Landroid/content/res/Resources$NotFoundException;"
    ));
    assert!(program.value_matches_type(&exception, "Ljava/lang/RuntimeException;"));
    let Some(DexValue::String(message)) =
        program.read_stored_instance_field(&exception, &detail_message)
    else {
        panic!("Resources.NotFoundException should carry a detail message");
    };
    assert!(
        message.contains(expected_message),
        "resource failure message {message:?} should contain {expected_message:?}"
    );
}

fn typed_value_test_int_field(name: &str) -> DexFieldId {
    DexFieldId {
        class_descriptor: "Landroid/util/TypedValue;".to_owned(),
        type_descriptor: "I".to_owned(),
        name: name.to_owned(),
    }
}

fn typed_value_test_char_sequence_field(name: &str) -> DexFieldId {
    DexFieldId {
        class_descriptor: "Landroid/util/TypedValue;".to_owned(),
        type_descriptor: "Ljava/lang/CharSequence;".to_owned(),
        name: name.to_owned(),
    }
}
