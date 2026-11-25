use super::super::*;
use super::helpers::*;

#[test]
fn builtin_context_get_system_service_returns_named_reference() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getSystemService".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Ljava/lang/Object;".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("context")),
        DexValue::String(std::rc::Rc::from("connectivity")),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/net/ConnectivityManager;:connectivity")
        )))
    );
}

// cite: frameworks-base SystemServiceRegistry.java#ACCESSIBILITY_SERVICE, AccessibilityManager
#[test]
fn frameworks_system_service_registry_returns_typed_accessibility_manager() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getSystemService".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Ljava/lang/Object;".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("context")),
        DexValue::String(std::rc::Rc::from("accessibility")),
    ]);
    let expected = DexValue::Reference(std::rc::Rc::from(
        "Landroid/view/accessibility/AccessibilityManager;:accessibility",
    ));

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Value(expected.clone()))
    );
    assert!(program.value_matches_type(
        &expected,
        "Landroid/view/accessibility/AccessibilityManager;"
    ));
}

// cite: frameworks-base SystemServiceRegistry.java#CLIPBOARD_SERVICE, ClipboardManager
// cite: frameworks-base ClipboardManager.java#ClipboardManager
#[test]
fn frameworks_system_service_registry_returns_clipboard_manager() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getSystemService".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Ljava/lang/Object;".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("context")),
        DexValue::String(std::rc::Rc::from("clipboard")),
    ]);
    let expected = DexValue::Reference(std::rc::Rc::from(
        "Landroid/content/ClipboardManager;:clipboard",
    ));

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Value(expected.clone()))
    );
    assert!(program.value_matches_type(&expected, "Landroid/content/ClipboardManager;"));
    assert!(program.value_matches_type(&expected, "Landroid/text/ClipboardManager;"));
}

// cite: frameworks-base SystemServiceRegistry.java#VIBRATOR_MANAGER_SERVICE, VibratorManager, SystemVibratorManager
// cite: frameworks-base SystemVibratorManager.java#SystemVibratorManager
#[test]
fn frameworks_system_service_registry_returns_system_vibrator_manager() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getSystemService".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Ljava/lang/Object;".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("context")),
        DexValue::String(std::rc::Rc::from("vibrator_manager")),
    ]);
    let expected = DexValue::Reference(std::rc::Rc::from(
        "Landroid/os/SystemVibratorManager;:vibrator_manager",
    ));

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Value(expected.clone()))
    );
    assert!(program.value_matches_type(&expected, "Landroid/os/VibratorManager;"));
}

// cite: frameworks-base SystemServiceRegistry.java#WINDOW_SERVICE, WindowManager
// cite: frameworks-base WindowManagerImpl.java#WindowManagerImpl
#[test]
fn frameworks_system_service_registry_returns_window_manager_impl() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getSystemService".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Ljava/lang/Object;".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("context")),
        DexValue::String(std::rc::Rc::from("window")),
    ]);
    let expected =
        DexValue::Reference(std::rc::Rc::from("Landroid/view/WindowManagerImpl;:window"));

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Value(expected.clone()))
    );
    assert!(program.value_matches_type(&expected, "Landroid/view/WindowManager;"));
}

#[test]
fn builtin_context_get_system_service_class_returns_typed_service_reference() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getSystemService".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Ljava/lang/Object;".to_owned(),
            parameters: vec!["Ljava/lang/Class;".to_owned()],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("context")),
        DexValue::Reference(std::rc::Rc::from(
            "Ljava/lang/Class;:Landroid/view/autofill/AutofillManager;",
        )),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/view/autofill/AutofillManager;:autofill")
        )))
    );
}

// cite: frameworks-base Context.java#getSystemService
// cite: frameworks-base SystemServiceRegistry.java#VIBRATOR_MANAGER_SERVICE, VibratorManager, SystemVibratorManager
#[test]
fn frameworks_context_system_service_by_class_uses_runtime_service_type() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getSystemService".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Ljava/lang/Object;".to_owned(),
            parameters: vec!["Ljava/lang/Class;".to_owned()],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("context")),
        DexValue::Reference(std::rc::Rc::from(
            "Ljava/lang/Class;:Landroid/os/VibratorManager;",
        )),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/os/SystemVibratorManager;:vibrator_manager")
        )))
    );
}

#[test]
fn builtin_context_get_main_looper_returns_main_looper() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getMainLooper".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/os/Looper;".to_owned(),
            parameters: Vec::new(),
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&method, None),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/os/Looper;:main")
        )))
    );
}

#[test]
// cite: frameworks-base ContextImpl.java#getClassLoader
fn builtin_context_get_class_loader_returns_application_loader() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Context;".to_owned(),
        name: "getClassLoader".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Ljava/lang/ClassLoader;".to_owned(),
            parameters: Vec::new(),
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&method, None),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Ljava/lang/ClassLoader;.app")
        )))
    );
}
