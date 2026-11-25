use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use super::super::*;
use super::helpers::*;

#[test]
fn builtin_base_bundle_contains_key_returns_false_for_bootstrap_bundle() {
    let program = DexProgram::new(Vec::new());
    let contains_key = DexMethodId {
        class_descriptor: "Landroid/os/BaseBundle;".to_owned(),
        name: "containsKey".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZL".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let put_string = DexMethodId {
        class_descriptor: "Landroid/os/BaseBundle;".to_owned(),
        name: "putString".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VLL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec![
                "Ljava/lang/String;".to_owned(),
                "Ljava/lang/String;".to_owned(),
            ],
        },
    };
    let key_set = DexMethodId {
        class_descriptor: "Landroid/os/BaseBundle;".to_owned(),
        name: "keySet".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Ljava/util/Set;".to_owned(),
            parameters: Vec::new(),
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&contains_key, None),
        Some(DexExecutionResult::Int(0))
    );
    assert_eq!(
        program.invoke_builtin_method(&put_string, None),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&key_set, None),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Ljava/util/Set;.empty")
        )))
    );
}

#[test]
fn builtin_bundle_constructor_returns_void() {
    let program = DexProgram::new(Vec::new());
    let default_constructor = DexMethodId {
        class_descriptor: "Landroid/os/Bundle;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let capacity_constructor = DexMethodId {
        class_descriptor: "Landroid/os/Bundle;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VI".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&default_constructor, None),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &capacity_constructor,
            Some(&test_invocation(vec![
                DexValue::Object(DexObjectRef {
                    id: 1,
                    class_descriptor: std::rc::Rc::from("Landroid/os/Bundle;"),
                }),
                DexValue::Int(0),
            ])),
        ),
        Some(DexExecutionResult::Void)
    );
}

#[test]
fn builtin_bundle_fresh_launch_parcelable_entries_are_empty() {
    let program = DexProgram::new(Vec::new());
    let get_parcelable = DexMethodId {
        class_descriptor: "Landroid/os/Bundle;".to_owned(),
        name: "getParcelable".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Landroid/os/Parcelable;".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let remove = DexMethodId {
        class_descriptor: "Landroid/os/Bundle;".to_owned(),
        name: "remove".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&get_parcelable, None),
        Some(DexExecutionResult::Value(DexValue::Null))
    );
    assert_eq!(
        program.invoke_builtin_method(&remove, None),
        Some(DexExecutionResult::Void)
    );
}

#[test]
fn builtin_bundle_stores_keys_and_values() {
    let program = DexProgram::new(Vec::new());
    let put_string = DexMethodId {
        class_descriptor: "Landroid/os/BaseBundle;".to_owned(),
        name: "putString".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VLL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec![
                "Ljava/lang/String;".to_owned(),
                "Ljava/lang/String;".to_owned(),
            ],
        },
    };
    let contains_key = DexMethodId {
        class_descriptor: "Landroid/os/BaseBundle;".to_owned(),
        name: "containsKey".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZL".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let key_set = DexMethodId {
        class_descriptor: "Landroid/os/BaseBundle;".to_owned(),
        name: "keySet".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Ljava/util/Set;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let size = DexMethodId {
        class_descriptor: "Landroid/os/BaseBundle;".to_owned(),
        name: "size".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "I".to_owned(),
            return_type: "I".to_owned(),
            parameters: Vec::new(),
        },
    };
    let is_empty = DexMethodId {
        class_descriptor: "Landroid/os/BaseBundle;".to_owned(),
        name: "isEmpty".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "Z".to_owned(),
            return_type: "Z".to_owned(),
            parameters: Vec::new(),
        },
    };
    let remove = DexMethodId {
        class_descriptor: "Landroid/os/Bundle;".to_owned(),
        name: "remove".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let bundle = DexValue::Object(DexObjectRef {
        id: 1,
        class_descriptor: std::rc::Rc::from("Landroid/os/Bundle;"),
    });
    let key = DexValue::String(std::rc::Rc::from("saved"));

    assert_eq!(
        program.invoke_builtin_method(&is_empty, Some(&test_invocation(vec![bundle.clone()]))),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &put_string,
            Some(&test_invocation(vec![
                bundle.clone(),
                key.clone(),
                DexValue::String(std::rc::Rc::from("value")),
            ])),
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &contains_key,
            Some(&test_invocation(vec![bundle.clone(), key.clone()])),
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.invoke_builtin_method(&size, Some(&test_invocation(vec![bundle.clone()]))),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.invoke_builtin_method(&is_empty, Some(&test_invocation(vec![bundle.clone()]))),
        Some(DexExecutionResult::Int(0))
    );

    let Some(DexExecutionResult::Value(set)) =
        program.invoke_builtin_method(&key_set, Some(&test_invocation(vec![bundle.clone()])))
    else {
        panic!("Bundle.keySet should return a Set value");
    };
    assert_eq!(
        program
            .runtime
            .borrow()
            .list_entries
            .get(&map_identity_key(&set)),
        Some(&vec![key.clone()])
    );

    assert_eq!(
        program.invoke_builtin_method(
            &remove,
            Some(&test_invocation(vec![bundle.clone(), key.clone()])),
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &contains_key,
            Some(&test_invocation(vec![bundle.clone(), key]))
        ),
        Some(DexExecutionResult::Int(0))
    );
    assert_eq!(
        program.invoke_builtin_method(&is_empty, Some(&test_invocation(vec![bundle]))),
        Some(DexExecutionResult::Int(1))
    );
}

#[test]
fn builtin_bundle_put_all_copies_source_entries() {
    let program = DexProgram::new(Vec::new());
    let put_all = DexMethodId {
        class_descriptor: "Landroid/os/Bundle;".to_owned(),
        name: "putAll".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["Landroid/os/Bundle;".to_owned()],
        },
    };
    let get_parcelable = DexMethodId {
        class_descriptor: "Landroid/os/Bundle;".to_owned(),
        name: "getParcelable".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Landroid/os/Parcelable;".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let target = DexValue::Object(DexObjectRef {
        id: 1,
        class_descriptor: std::rc::Rc::from("Landroid/os/Bundle;"),
    });
    let source = DexValue::Object(DexObjectRef {
        id: 2,
        class_descriptor: std::rc::Rc::from("Landroid/os/Bundle;"),
    });
    let parcelable = DexValue::Object(DexObjectRef {
        id: 3,
        class_descriptor: std::rc::Rc::from("Landroid/os/Parcelable;"),
    });
    let key = "state".to_owned();

    program
        .runtime
        .borrow_mut()
        .bundle_entries
        .insert((map_identity_key(&source), key.clone()), parcelable.clone());

    assert_eq!(
        program.invoke_builtin_method(
            &put_all,
            Some(&test_invocation(vec![target.clone(), source])),
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &get_parcelable,
            Some(&test_invocation(vec![
                target,
                DexValue::String(std::rc::Rc::from(key))
            ])),
        ),
        Some(DexExecutionResult::Value(parcelable))
    );
}

#[test]
fn builtin_bundle_put_bundle_stores_nested_bundle_value() {
    let program = DexProgram::new(Vec::new());
    let put_bundle = DexMethodId {
        class_descriptor: "Landroid/os/Bundle;".to_owned(),
        name: "putBundle".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VLL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec![
                "Ljava/lang/String;".to_owned(),
                "Landroid/os/Bundle;".to_owned(),
            ],
        },
    };
    let contains_key = DexMethodId {
        class_descriptor: "Landroid/os/BaseBundle;".to_owned(),
        name: "containsKey".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "ZL".to_owned(),
            return_type: "Z".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let bundle = DexValue::Object(DexObjectRef {
        id: 1,
        class_descriptor: std::rc::Rc::from("Landroid/os/Bundle;"),
    });
    let nested = DexValue::Object(DexObjectRef {
        id: 2,
        class_descriptor: std::rc::Rc::from("Landroid/os/Bundle;"),
    });
    let key = DexValue::String(std::rc::Rc::from("nested"));

    assert_eq!(
        program.invoke_builtin_method(
            &put_bundle,
            Some(&test_invocation(vec![
                bundle.clone(),
                key.clone(),
                nested.clone()
            ])),
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program
            .runtime
            .borrow()
            .bundle_entries
            .get(&(map_identity_key(&bundle), "nested".to_owned())),
        Some(&nested)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &contains_key,
            Some(&test_invocation(vec![bundle.clone(), key.clone()])),
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &put_bundle,
            Some(&test_invocation(vec![bundle.clone(), key, DexValue::Null])),
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program
            .runtime
            .borrow()
            .bundle_entries
            .get(&(map_identity_key(&bundle), "nested".to_owned())),
        Some(&DexValue::Null)
    );
}

#[test]
fn builtin_bundle_set_class_loader_returns_void() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/os/Bundle;".to_owned(),
        name: "setClassLoader".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["Ljava/lang/ClassLoader;".to_owned()],
        },
    };

    assert_eq!(
        program.invoke_builtin_method(
            &method,
            Some(&test_invocation(vec![
                DexValue::Object(DexObjectRef {
                    id: 1,
                    class_descriptor: std::rc::Rc::from("Landroid/os/Bundle;"),
                }),
                DexValue::Reference(std::rc::Rc::from("Ljava/lang/ClassLoader;.bootstrap")),
            ])),
        ),
        Some(DexExecutionResult::Void)
    );
}

#[test]
fn builtin_process_identity_returns_deterministic_values() {
    let program = DexProgram::new(Vec::new());
    let pid = DexMethodId {
        class_descriptor: "Landroid/os/Process;".to_owned(),
        name: "myPid".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "I".to_owned(),
            return_type: "I".to_owned(),
            parameters: Vec::new(),
        },
    };
    let uid = DexMethodId {
        class_descriptor: "Landroid/os/Process;".to_owned(),
        name: "myUid".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "I".to_owned(),
            return_type: "I".to_owned(),
            parameters: Vec::new(),
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&pid, None),
        Some(DexExecutionResult::Int(1000))
    );
    assert_eq!(
        program.invoke_builtin_method(&uid, None),
        Some(DexExecutionResult::Int(10000))
    );
}

#[test]
fn builtin_android_observable_constructor_returns_void() {
    let program = DexProgram::new(Vec::new());
    let constructor = DexMethodId {
        class_descriptor: "Landroid/database/Observable;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let register = DexMethodId {
        class_descriptor: "Landroid/database/Observable;".to_owned(),
        name: "registerObserver".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["Ljava/lang/Object;".to_owned()],
        },
    };
    let unregister = DexMethodId {
        class_descriptor: "Landroid/database/Observable;".to_owned(),
        name: "unregisterObserver".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["Ljava/lang/Object;".to_owned()],
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&constructor, None),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&register, None),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&unregister, None),
        Some(DexExecutionResult::Void)
    );
}

#[test]
fn builtin_sqlite_open_helper_constructor_returns_void() {
    let program = SqliteTestFixture::new("open_helper");
    let constructor = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteOpenHelper;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VLLLI".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec![
                "Landroid/content/Context;".to_owned(),
                "Ljava/lang/String;".to_owned(),
                "Landroid/database/sqlite/SQLiteDatabase$CursorFactory;".to_owned(),
                "I".to_owned(),
            ],
        },
    };
    let constructor_with_error_handler = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteOpenHelper;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VLLLIL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec![
                "Landroid/content/Context;".to_owned(),
                "Ljava/lang/String;".to_owned(),
                "Landroid/database/sqlite/SQLiteDatabase$CursorFactory;".to_owned(),
                "I".to_owned(),
                "Landroid/database/DatabaseErrorHandler;".to_owned(),
            ],
        },
    };
    let get_writable_database = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteOpenHelper;".to_owned(),
        name: "getWritableDatabase".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let set_write_ahead_logging_enabled = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteOpenHelper;".to_owned(),
        name: "setWriteAheadLoggingEnabled".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VZ".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["Z".to_owned()],
        },
    };
    let get_database_name = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteOpenHelper;".to_owned(),
        name: "getDatabaseName".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let close = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteOpenHelper;".to_owned(),
        name: "close".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let database_is_open = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "isOpen".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "Z".to_owned(),
            return_type: "Z".to_owned(),
            parameters: Vec::new(),
        },
    };
    let raw_query_with_factory = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "rawQueryWithFactory".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LLLLL".to_owned(),
            return_type: "Landroid/database/Cursor;".to_owned(),
            parameters: vec![
                "Landroid/database/sqlite/SQLiteDatabase$CursorFactory;".to_owned(),
                "Ljava/lang/String;".to_owned(),
                "[Ljava/lang/String;".to_owned(),
                "Ljava/lang/String;".to_owned(),
            ],
        },
    };
    let cursor_get_column_count = DexMethodId {
        class_descriptor: "Landroid/database/Cursor;".to_owned(),
        name: "getColumnCount".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "I".to_owned(),
            return_type: "I".to_owned(),
            parameters: Vec::new(),
        },
    };
    let cursor_get_column_name = DexMethodId {
        class_descriptor: "Landroid/database/Cursor;".to_owned(),
        name: "getColumnName".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let cursor_get_column_index = DexMethodId {
        class_descriptor: "Landroid/database/Cursor;".to_owned(),
        name: "getColumnIndex".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "IL".to_owned(),
            return_type: "I".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let cursor_close = DexMethodId {
        class_descriptor: "Landroid/database/Cursor;".to_owned(),
        name: "close".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let cursor_move_to_next = DexMethodId {
        class_descriptor: "Landroid/database/Cursor;".to_owned(),
        name: "moveToNext".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "Z".to_owned(),
            return_type: "Z".to_owned(),
            parameters: Vec::new(),
        },
    };
    let helper = DexValue::Reference(std::rc::Rc::from("helper"));
    let context = DexValue::Reference(std::rc::Rc::from("Landroid/content/Context;.bootstrap"));
    let invocation = test_invocation(vec![helper.clone()]);
    let constructor_invocation = test_invocation(vec![
        helper.clone(),
        context.clone(),
        DexValue::String(std::rc::Rc::from("timer.db")),
        DexValue::Int(0),
        DexValue::Int(1),
    ]);
    let constructor_with_error_handler_invocation = test_invocation(vec![
        helper.clone(),
        context,
        DexValue::String(std::rc::Rc::from("timer.db")),
        DexValue::Int(0),
        DexValue::Int(2),
        DexValue::Reference(std::rc::Rc::from(
            "Landroid/database/DatabaseErrorHandler;:default",
        )),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&constructor, Some(&constructor_invocation)),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &constructor_with_error_handler,
            Some(&constructor_with_error_handler_invocation)
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&get_writable_database, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Landroid/database/sqlite/SQLiteDatabase;:reference:helper")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &database_is_open,
            Some(&test_invocation(vec![DexValue::Reference(
                std::rc::Rc::from("Landroid/database/sqlite/SQLiteDatabase;:reference:helper")
            )]))
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &raw_query_with_factory,
            Some(&test_invocation(vec![
                DexValue::Reference(
                    std::rc::Rc::from("Landroid/database/sqlite/SQLiteDatabase;:reference:helper")
                ),
                DexValue::Int(0),
                DexValue::String(std::rc::Rc::from("SELECT 1")),
                DexValue::Int(0),
                DexValue::Null,
            ]))
        ),
        Some(DexExecutionResult::Value(DexValue::Reference(std::rc::Rc::from(format!(
            "Landroid/database/Cursor;:sqlite:reference:Landroid/database/sqlite/SQLiteDatabase;:reference:helper:{}",
            stable_hash_i32("SELECT 1")
        )))))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &cursor_get_column_count,
            Some(&test_invocation(vec![DexValue::Reference(std::rc::Rc::from(format!(
                "Landroid/database/Cursor;:sqlite:reference:Landroid/database/sqlite/SQLiteDatabase;:reference:helper:{}",
                stable_hash_i32("SELECT 1")
            )))]))
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &cursor_get_column_name,
            Some(&test_invocation(vec![
                DexValue::Reference(std::rc::Rc::from(format!(
                    "Landroid/database/Cursor;:sqlite:reference:Landroid/database/sqlite/SQLiteDatabase;:reference:helper:{}",
                    stable_hash_i32("SELECT 1")
                ))),
                DexValue::Int(0)
            ]))
        ),
        Some(DexExecutionResult::Value(DexValue::String(std::rc::Rc::from("1"))))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &cursor_get_column_index,
            Some(&test_invocation(vec![
                DexValue::Reference(std::rc::Rc::from(format!(
                    "Landroid/database/Cursor;:sqlite:reference:Landroid/database/sqlite/SQLiteDatabase;:reference:helper:{}",
                    stable_hash_i32("SELECT 1")
                ))),
                DexValue::String(std::rc::Rc::from("1"))
            ]))
        ),
        Some(DexExecutionResult::Int(0))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &cursor_move_to_next,
            Some(&test_invocation(vec![DexValue::Reference(std::rc::Rc::from(format!(
                "Landroid/database/Cursor;:sqlite:reference:Landroid/database/sqlite/SQLiteDatabase;:reference:helper:{}",
                stable_hash_i32("SELECT 1")
            )))]))
        ),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &cursor_close,
            Some(&test_invocation(vec![DexValue::Reference(std::rc::Rc::from(format!(
                "Landroid/database/Cursor;:sqlite:reference:Landroid/database/sqlite/SQLiteDatabase;:reference:helper:{}",
                stable_hash_i32("SELECT 1")
            )))]))
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &set_write_ahead_logging_enabled,
            Some(&test_invocation(vec![helper.clone(), DexValue::Int(1)]))
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.read_stored_instance_field(
            &helper,
            &DexFieldId {
                class_descriptor: "Landroid/database/sqlite/SQLiteOpenHelper;".to_owned(),
                type_descriptor: "Z".to_owned(),
                name: "mEnableWriteAheadLogging".to_owned(),
            }
        ),
        Some(DexValue::Int(1))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_database_name, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from("timer.db")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(&close, Some(&invocation)),
        Some(DexExecutionResult::Void)
    );
}

#[test]
fn builtin_intent_get_flags_returns_zero_for_bootstrap_intent() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Intent;".to_owned(),
        name: "getFlags".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "I".to_owned(),
            return_type: "I".to_owned(),
            parameters: Vec::new(),
        },
    };
    let invocation = test_invocation(vec![DexValue::Reference(std::rc::Rc::from(
        "Landroid/content/Intent;.bootstrap",
    ))]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Int(0))
    );
}

#[test]
fn builtin_intent_class_constructor_stores_target_class() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Intent;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VLL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec![
                "Landroid/content/Context;".to_owned(),
                "Ljava/lang/Class;".to_owned(),
            ],
        },
    };
    let intent = DexValue::Reference(std::rc::Rc::from("Landroid/content/Intent;:settings"));
    let target = DexValue::Reference(std::rc::Rc::from(
        "Ljava/lang/Class;:Lcom/darkempire78/opencalculator/activities/SettingsActivity;"
            .to_owned(),
    ));
    let invocation = test_invocation(vec![
        intent.clone(),
        DexValue::Reference(std::rc::Rc::from("Landroid/content/Context;.bootstrap")),
        target.clone(),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.read_stored_instance_field(&intent, &intent_target_class_field()),
        Some(target)
    );
}

#[test]
fn builtin_intent_get_data_returns_null_for_bootstrap_intent() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Intent;".to_owned(),
        name: "getData".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/net/Uri;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let invocation = test_invocation(vec![DexValue::Reference(std::rc::Rc::from(
        "Landroid/content/Intent;.bootstrap",
    ))]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::Null))
    );
}

#[test]
fn builtin_uri_parse_stores_uri_string() {
    let program = DexProgram::new(Vec::new());
    let parse = DexMethodId {
        class_descriptor: "Landroid/net/Uri;".to_owned(),
        name: "parse".to_owned(),
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
    let invocation = test_invocation(vec![DexValue::String(std::rc::Rc::from(
        "https://github.com/Darkempire78/OpenCalc",
    ))]);
    let Some(DexExecutionResult::Value(uri)) =
        program.invoke_builtin_method(&parse, Some(&invocation))
    else {
        panic!("Uri.parse should return a Uri value");
    };

    assert_eq!(
        program.read_stored_instance_field(&uri, &uri_string_field()),
        Some(DexValue::String(std::rc::Rc::from(
            "https://github.com/Darkempire78/OpenCalc"
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(&to_string, Some(&test_invocation(vec![uri]))),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from("https://github.com/Darkempire78/OpenCalc")
        )))
    );
}

#[test]
fn builtin_intent_action_data_constructor_round_trips_values() {
    let program = DexProgram::new(Vec::new());
    let constructor = DexMethodId {
        class_descriptor: "Landroid/content/Intent;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VLL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec![
                "Ljava/lang/String;".to_owned(),
                "Landroid/net/Uri;".to_owned(),
            ],
        },
    };
    let get_action = DexMethodId {
        class_descriptor: "Landroid/content/Intent;".to_owned(),
        name: "getAction".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let get_data = DexMethodId {
        class_descriptor: "Landroid/content/Intent;".to_owned(),
        name: "getData".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/net/Uri;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let intent = DexValue::Reference(std::rc::Rc::from("Landroid/content/Intent;:view"));
    let uri = DexValue::Reference(std::rc::Rc::from("Landroid/net/Uri;:github"));
    let invocation = test_invocation(vec![
        intent.clone(),
        DexValue::String(std::rc::Rc::from("android.intent.action.VIEW")),
        uri.clone(),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&constructor, Some(&invocation)),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&get_action, Some(&test_invocation(vec![intent.clone()]))),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from("android.intent.action.VIEW")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(&get_data, Some(&test_invocation(vec![intent]))),
        Some(DexExecutionResult::Value(uri))
    );
}

#[test]
fn builtin_intent_get_extras_returns_null_for_bootstrap_intent() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Landroid/content/Intent;".to_owned(),
        name: "getExtras".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Landroid/os/Bundle;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let invocation = test_invocation(vec![DexValue::Reference(std::rc::Rc::from(
        "Landroid/content/Intent;.bootstrap",
    ))]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::Null))
    );
}

#[test]
fn builtin_sqlite_database_insert_stores_content_values() {
    let program = SqliteTestFixture::new("insert");
    let exec_sql = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "execSQL".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let put_string = DexMethodId {
        class_descriptor: "Landroid/content/ContentValues;".to_owned(),
        name: "put".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VLL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec![
                "Ljava/lang/String;".to_owned(),
                "Ljava/lang/String;".to_owned(),
            ],
        },
    };
    let put_long = DexMethodId {
        class_descriptor: "Landroid/content/ContentValues;".to_owned(),
        name: "put".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VLL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec![
                "Ljava/lang/String;".to_owned(),
                "Ljava/lang/Long;".to_owned(),
            ],
        },
    };
    let insert = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "insert".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "JLLL".to_owned(),
            return_type: "J".to_owned(),
            parameters: vec![
                "Ljava/lang/String;".to_owned(),
                "Ljava/lang/String;".to_owned(),
                "Landroid/content/ContentValues;".to_owned(),
            ],
        },
    };
    let content_values = DexValue::Reference(std::rc::Rc::from("content-values"));
    let database = DexValue::Reference(std::rc::Rc::from("database"));
    program.register_database(&database);
    let put_string_invocation = test_invocation(vec![
        content_values.clone(),
        DexValue::String(std::rc::Rc::from("name")),
        DexValue::String(std::rc::Rc::from("calculator")),
    ]);
    let put_long_invocation = test_invocation(vec![
        content_values.clone(),
        DexValue::String(std::rc::Rc::from("timestamp")),
        DexValue::Reference(std::rc::Rc::from("Ljava/lang/Long;:42")),
    ]);
    let insert_invocation = test_invocation(vec![
        database.clone(),
        DexValue::String(std::rc::Rc::from("events")),
        DexValue::Int(0),
        content_values,
    ]);

    assert_eq!(
        program.invoke_builtin_method(
            &exec_sql,
            Some(&test_invocation(vec![
                database,
                DexValue::String(std::rc::Rc::from(
                    "CREATE TABLE events (name TEXT NOT NULL, timestamp INTEGER NOT NULL)"
                        .to_owned()
                )),
            ]))
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&put_string, Some(&put_string_invocation)),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&put_long, Some(&put_long_invocation)),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&insert, Some(&insert_invocation)),
        Some(DexExecutionResult::Value(DexValue::Long(1)))
    );
}

#[test]
fn builtin_sqlite_raw_query_reads_inserted_rows() {
    let program = SqliteTestFixture::new("raw_query");
    let exec_sql = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "execSQL".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let put_string = DexMethodId {
        class_descriptor: "Landroid/content/ContentValues;".to_owned(),
        name: "put".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VLL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec![
                "Ljava/lang/String;".to_owned(),
                "Ljava/lang/String;".to_owned(),
            ],
        },
    };
    let put_long = DexMethodId {
        class_descriptor: "Landroid/content/ContentValues;".to_owned(),
        name: "put".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VLL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec![
                "Ljava/lang/String;".to_owned(),
                "Ljava/lang/Long;".to_owned(),
            ],
        },
    };
    let insert = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "insert".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "JLLL".to_owned(),
            return_type: "J".to_owned(),
            parameters: vec![
                "Ljava/lang/String;".to_owned(),
                "Ljava/lang/String;".to_owned(),
                "Landroid/content/ContentValues;".to_owned(),
            ],
        },
    };
    let raw_query_with_factory = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "rawQueryWithFactory".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LLLLL".to_owned(),
            return_type: "Landroid/database/Cursor;".to_owned(),
            parameters: vec![
                "Landroid/database/sqlite/SQLiteDatabase$CursorFactory;".to_owned(),
                "Ljava/lang/String;".to_owned(),
                "[Ljava/lang/String;".to_owned(),
                "Ljava/lang/String;".to_owned(),
            ],
        },
    };
    let move_to_next = DexMethodId {
        class_descriptor: "Landroid/database/Cursor;".to_owned(),
        name: "moveToNext".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "Z".to_owned(),
            return_type: "Z".to_owned(),
            parameters: Vec::new(),
        },
    };
    let get_string = DexMethodId {
        class_descriptor: "Landroid/database/Cursor;".to_owned(),
        name: "getString".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LI".to_owned(),
            return_type: "Ljava/lang/String;".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let get_long = DexMethodId {
        class_descriptor: "Landroid/database/Cursor;".to_owned(),
        name: "getLong".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "JI".to_owned(),
            return_type: "J".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let database = DexValue::Reference(std::rc::Rc::from("database"));
    let content_values = DexValue::Reference(std::rc::Rc::from("content-values"));
    program.register_database(&database);

    assert_eq!(
        program.invoke_builtin_method(
            &exec_sql,
            Some(&test_invocation(vec![
                database.clone(),
                DexValue::String(std::rc::Rc::from(
                    "CREATE TABLE events (name TEXT NOT NULL, timestamp INTEGER NOT NULL)"
                        .to_owned()
                )),
            ]))
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &put_string,
            Some(&test_invocation(vec![
                content_values.clone(),
                DexValue::String(std::rc::Rc::from("name")),
                DexValue::String(std::rc::Rc::from("calculator")),
            ]))
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &put_long,
            Some(&test_invocation(vec![
                content_values.clone(),
                DexValue::String(std::rc::Rc::from("timestamp")),
                DexValue::Reference(std::rc::Rc::from("Ljava/lang/Long;:42")),
            ]))
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &insert,
            Some(&test_invocation(vec![
                database.clone(),
                DexValue::String(std::rc::Rc::from("events")),
                DexValue::Int(0),
                content_values,
            ]))
        ),
        Some(DexExecutionResult::Value(DexValue::Long(1)))
    );

    let query = "SELECT name, timestamp FROM events";
    let cursor = DexValue::Reference(std::rc::Rc::from(format!(
        "Landroid/database/Cursor;:sqlite:{}:{}",
        map_identity_key(&database),
        stable_hash_i32(query)
    )));
    assert_eq!(
        program.invoke_builtin_method(
            &raw_query_with_factory,
            Some(&test_invocation(vec![
                database,
                DexValue::Int(0),
                DexValue::String(std::rc::Rc::from(query.to_owned())),
                DexValue::Int(0),
                DexValue::Null,
            ]))
        ),
        Some(DexExecutionResult::Value(cursor.clone()))
    );
    assert_eq!(
        program.invoke_builtin_method(&move_to_next, Some(&test_invocation(vec![cursor.clone()]))),
        Some(DexExecutionResult::Int(1))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &get_string,
            Some(&test_invocation(vec![cursor.clone(), DexValue::Int(0)]))
        ),
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from("calculator")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(
            &get_long,
            Some(&test_invocation(vec![cursor.clone(), DexValue::Int(1)]))
        ),
        Some(DexExecutionResult::Value(DexValue::Long(42)))
    );
    assert_eq!(
        program.invoke_builtin_method(&move_to_next, Some(&test_invocation(vec![cursor]))),
        Some(DexExecutionResult::Int(0))
    );
}

#[test]
fn builtin_sqlite_program_bind_string_accepts_sqlite_query_receiver() {
    let program = SqliteTestFixture::new("bind_string");
    let bind_string = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteProgram;".to_owned(),
        name: "bindString".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VIL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["I".to_owned(), "Ljava/lang/String;".to_owned()],
        },
    };
    let database = DexValue::Reference(std::rc::Rc::from("database"));
    let query = DexValue::Reference(std::rc::Rc::from(
        "Landroid/database/sqlite/SQLiteQuery;:query",
    ));
    program.register_database(&database);

    assert_eq!(
        program.invoke_builtin_method(
            &bind_string,
            Some(&test_invocation(vec![
                query.clone(),
                DexValue::Int(1),
                DexValue::String(std::rc::Rc::from("bound")),
            ])),
        ),
        Some(DexExecutionResult::Void)
    );

    let bind_args = program
        .sqlite_bound_query_args(&database, "SELECT ? AS value", &query)
        .expect("query bind args should read SQLiteQuery bindString values");
    let cursor = program
        .sqlite_raw_query_result(&database, "SELECT ? AS value", &bind_args)
        .expect("bound query should execute");

    assert_eq!(cursor.columns, vec!["value".to_owned()]);
    assert_eq!(
        cursor.rows,
        vec![vec![DexValue::String(std::rc::Rc::from("bound"))]]
    );
}

#[test]
fn cts_sqlite_database_test_transaction_commits_and_rolls_back() {
    let program = SqliteTestFixture::new("transaction_commit_rollback");
    let methods = SqliteTransactionMethods::new();
    let database = sqlite_transaction_database(&program);

    assert_eq!(sqlite_in_transaction(&program, &methods, &database), 0);
    assert_sqlite_transaction_void(&program, &methods.begin, &database);
    assert_eq!(sqlite_in_transaction(&program, &methods, &database), 1);
    sqlite_set_transaction_num(&program, &database, 1);
    assert_sqlite_transaction_void(&program, &methods.successful, &database);
    assert_sqlite_transaction_void(&program, &methods.end, &database);
    assert_eq!(sqlite_in_transaction(&program, &methods, &database), 0);
    evict_sqlite_database_sessions(&program, &database);
    assert_eq!(sqlite_transaction_num(&program, &database), 1);

    sqlite_set_transaction_num(&program, &database, 0);
    assert_sqlite_transaction_void(&program, &methods.begin_nonexclusive, &database);
    sqlite_set_transaction_num(&program, &database, 1);
    assert_sqlite_transaction_void(&program, &methods.end, &database);
    evict_sqlite_database_sessions(&program, &database);
    assert_eq!(sqlite_transaction_num(&program, &database), 0);
}

#[test]
fn aosp_sqlite_database_transaction_modes_control_reader_access() {
    // cite: frameworks-base SQLiteDatabase.java#beginTransaction, beginTransactionNonExclusive
    // cite: frameworks-base SQLiteSession.java#beginTransactionUnchecked
    let program = SqliteTestFixture::new("transaction_modes");
    let methods = SqliteTransactionMethods::new();
    let database = sqlite_transaction_database(&program);
    let observer = rusqlite::Connection::open(program.data_dir.join("databases/test.db"))
        .expect("observer database should open");
    observer
        .busy_timeout(std::time::Duration::ZERO)
        .expect("observer busy timeout should configure");

    assert_sqlite_transaction_void(&program, &methods.begin_nonexclusive, &database);
    assert_eq!(
        observer
            .query_row("SELECT num FROM test", [], |row| row.get::<_, i64>(0))
            .expect("immediate transaction should allow an independent reader"),
        0
    );
    assert_sqlite_transaction_void(&program, &methods.end, &database);

    assert_sqlite_transaction_void(&program, &methods.begin, &database);
    let error = observer
        .query_row("SELECT num FROM test", [], |row| row.get::<_, i64>(0))
        .expect_err("exclusive transaction should block an independent reader");
    assert!(matches!(
        error.sqlite_error_code(),
        Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked)
    ));
    assert_sqlite_transaction_void(&program, &methods.end, &database);
}

#[test]
fn cts_sqlite_database_test_transaction_propagates_nested_failure() {
    let program = SqliteTestFixture::new("transaction_nested");
    let methods = SqliteTransactionMethods::new();
    let database = sqlite_transaction_database(&program);

    assert_sqlite_transaction_void(&program, &methods.begin, &database);
    assert_sqlite_transaction_void(&program, &methods.begin_nonexclusive, &database);
    sqlite_set_transaction_num(&program, &database, 1);
    assert_sqlite_transaction_void(&program, &methods.successful, &database);
    assert_sqlite_transaction_void(&program, &methods.end, &database);
    assert_eq!(sqlite_in_transaction(&program, &methods, &database), 1);
    assert_sqlite_transaction_void(&program, &methods.successful, &database);
    assert_sqlite_transaction_void(&program, &methods.end, &database);
    assert_eq!(sqlite_transaction_num(&program, &database), 1);

    sqlite_set_transaction_num(&program, &database, 0);
    assert_sqlite_transaction_void(&program, &methods.begin, &database);
    assert_sqlite_transaction_void(&program, &methods.begin, &database);
    sqlite_set_transaction_num(&program, &database, 1);
    assert_sqlite_transaction_void(&program, &methods.end, &database);
    assert_sqlite_transaction_void(&program, &methods.successful, &database);
    assert_sqlite_transaction_void(&program, &methods.end, &database);
    assert_eq!(sqlite_transaction_num(&program, &database), 0);

    assert_sqlite_transaction_void(&program, &methods.begin, &database);
    assert_sqlite_transaction_void(&program, &methods.begin, &database);
    sqlite_set_transaction_num(&program, &database, 1);
    assert_sqlite_transaction_void(&program, &methods.successful, &database);
    assert_sqlite_transaction_void(&program, &methods.end, &database);
    assert_sqlite_transaction_void(&program, &methods.end, &database);
    assert_eq!(sqlite_transaction_num(&program, &database), 0);
}

#[test]
fn cts_sqlite_database_test_transaction_rejects_invalid_state() {
    let program = SqliteTestFixture::new("transaction_invalid_state");
    let methods = SqliteTransactionMethods::new();
    let database = sqlite_transaction_database(&program);

    assert_sqlite_transaction_error(
        &program,
        &methods.end,
        &database,
        "Ljava/lang/IllegalStateException;",
        SQLITE_NO_TRANSACTION_MESSAGE,
    );
    assert_sqlite_transaction_error(
        &program,
        &methods.successful,
        &database,
        "Ljava/lang/IllegalStateException;",
        SQLITE_NO_TRANSACTION_MESSAGE,
    );
    assert_sqlite_transaction_void(&program, &methods.begin, &database);
    assert_sqlite_transaction_void(&program, &methods.successful, &database);
    assert_sqlite_transaction_error(
        &program,
        &methods.successful,
        &database,
        "Ljava/lang/IllegalStateException;",
        SQLITE_ALREADY_SUCCESSFUL_MESSAGE,
    );
    assert_sqlite_transaction_error(
        &program,
        &methods.begin,
        &database,
        "Ljava/lang/IllegalStateException;",
        SQLITE_ALREADY_SUCCESSFUL_MESSAGE,
    );
    assert_sqlite_transaction_void(&program, &methods.end, &database);
}

#[test]
fn aosp_sqlite_thread_ownership_reports_unimplemented_connection_wait_explicitly() {
    // cite: frameworks-base SQLiteDatabase.java#mThreadSession, getThreadSession, inTransaction
    // cite: frameworks-base SQLiteSession.java#hasTransaction, throwIfNoTransaction
    // cite: frameworks-base SQLiteConnectionPool.java#waitForConnection, setMaxConnectionPoolSizeLocked
    // cite: libcore Thread.java#start, run, currentThread
    let dex_file = crate::dex::parse_dex_file(
        "transaction_thread.dex",
        include_bytes!("fixtures/transaction_thread.dex").to_vec(),
    )
    .expect("transaction thread DEX should parse");
    let program = SqliteTestFixture::with_dex_files("transaction_owner", vec![dex_file]);
    let methods = SqliteTransactionMethods::new();
    let database = sqlite_transaction_database(&program);
    let thread_class = "Lwinedroid/tests/TransactionThread;";
    program
        .ensure_class_initialized(thread_class, 0)
        .expect("transaction thread class should initialize");
    let database_field = DexFieldId {
        class_descriptor: thread_class.to_owned(),
        type_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "database".to_owned(),
    };
    let rejected_begin_field = DexFieldId {
        class_descriptor: thread_class.to_owned(),
        type_descriptor: "Z".to_owned(),
        name: "rejectedBegin".to_owned(),
    };
    let rejected_begin_as_runtime_field = DexFieldId {
        class_descriptor: thread_class.to_owned(),
        type_descriptor: "Z".to_owned(),
        name: "rejectedBeginAsRuntime".to_owned(),
    };
    let rejected_success_field = DexFieldId {
        class_descriptor: thread_class.to_owned(),
        type_descriptor: "Z".to_owned(),
        name: "rejectedSuccess".to_owned(),
    };
    let saw_transaction_field = DexFieldId {
        class_descriptor: thread_class.to_owned(),
        type_descriptor: "Z".to_owned(),
        name: "sawTransaction".to_owned(),
    };
    program.write_stored_static_field(&database_field, database.clone());
    let worker_thread = program.allocate_object(thread_class.to_owned());

    assert_sqlite_transaction_void(&program, &methods.begin_nonexclusive, &database);
    sqlite_set_transaction_num(&program, &database, 1);
    program
        .run_started_thread(&worker_thread, 0)
        .expect("transaction worker should execute");

    assert_eq!(
        program.read_stored_static_field(&saw_transaction_field),
        Some(DexValue::Int(0))
    );
    assert_eq!(
        program.read_stored_static_field(&rejected_success_field),
        Some(DexValue::Int(1))
    );
    assert_eq!(
        program.read_stored_static_field(&rejected_begin_field),
        Some(DexValue::Int(1))
    );
    assert_eq!(
        program.read_stored_static_field(&rejected_begin_as_runtime_field),
        Some(DexValue::Int(1))
    );
    let contention = program.with_current_guest_thread(worker_thread.clone(), || {
        invoke_sqlite_transaction(&program, &methods.begin, &database)
    });
    let DexExecutionResult::Throw(exception) = contention else {
        panic!("contended begin should report the missing connection wait primitive");
    };
    assert_eq!(
        value_class_descriptor(&exception),
        Some("Ljava/lang/UnsupportedOperationException;")
    );
    assert!(program.value_matches_type(&exception, "Ljava/lang/RuntimeException;"));
    assert_eq!(
        program.read_stored_instance_field(
            &exception,
            &DexFieldId {
                class_descriptor: "Ljava/lang/Throwable;".to_owned(),
                type_descriptor: "Ljava/lang/String;".to_owned(),
                name: "detailMessage".to_owned(),
            },
        ),
        Some(DexValue::String(std::rc::Rc::from(
            super::super::sqlite::SQLITE_CONNECTION_WAIT_MESSAGE
        )))
    );
    assert_eq!(
        program.runtime.borrow().current_guest_thread,
        DexValue::Reference(std::rc::Rc::from("Ljava/lang/Thread;:main"))
    );
    assert_eq!(sqlite_in_transaction(&program, &methods, &database), 1);
    assert_sqlite_transaction_void(&program, &methods.successful, &database);
    assert_sqlite_transaction_void(&program, &methods.end, &database);
    assert_eq!(sqlite_transaction_num(&program, &database), 1);
}

#[test]
fn aosp_activity_thread_main_loop_uses_main_guest_thread_and_restores_worker() {
    // cite: frameworks-base ActivityThread.java#main
    // cite: libcore Thread.java#currentThread
    let dex_file = crate::dex::parse_dex_file(
        "transaction_thread.dex",
        include_bytes!("fixtures/transaction_thread.dex").to_vec(),
    )
    .expect("main thread probe DEX should parse");
    let program = SqliteTestFixture::with_dex_files("main_thread_probe", vec![dex_file]);
    let probe_class = "Lwinedroid/tests/MainThreadProbe;";
    program
        .ensure_class_initialized(probe_class, 0)
        .expect("main thread probe class should initialize");
    let probe = program.allocate_object(probe_class.to_owned());
    let observed_thread_field = DexFieldId {
        class_descriptor: probe_class.to_owned(),
        type_descriptor: "Ljava/lang/Thread;".to_owned(),
        name: "observedThread".to_owned(),
    };
    let worker = program.allocate_object("Ljava/lang/Thread;".to_owned());

    program.with_current_guest_thread(worker.clone(), || {
        program.runtime.borrow_mut().post_handler_runnable(
            "reference:main-handler".to_owned(),
            probe,
            0,
            false,
        );
        program
            .dispatch_pending_main_loop(420, 760, 0)
            .expect("nested main loop should dispatch probe");
        assert_eq!(program.runtime.borrow().current_guest_thread, worker);
    });

    assert_eq!(
        program.read_stored_static_field(&observed_thread_field),
        Some(DexValue::Reference(std::rc::Rc::from(
            "Ljava/lang/Thread;:main"
        )))
    );
    assert_eq!(
        program.runtime.borrow().current_guest_thread,
        DexValue::Reference(std::rc::Rc::from("Ljava/lang/Thread;:main"))
    );
}

#[test]
fn aosp_parked_thread_resume_keeps_sqlite_transaction_owner() {
    // cite: frameworks-base SQLiteDatabase.java#mThreadSession, getThreadSession, inTransaction
    // cite: libcore LockSupport.java#parkNanos
    // cite: libcore Thread.java#currentThread
    let dex_file = crate::dex::parse_dex_file(
        "transaction_thread.dex",
        include_bytes!("fixtures/transaction_thread.dex").to_vec(),
    )
    .expect("parked transaction thread DEX should parse");
    let program = SqliteTestFixture::with_dex_files("parked_transaction", vec![dex_file]);
    let methods = SqliteTransactionMethods::new();
    let database = sqlite_transaction_database(&program);
    let thread_class = "Lwinedroid/tests/ParkedTransactionThread;";
    program
        .ensure_class_initialized(thread_class, 0)
        .expect("parked transaction thread class should initialize");
    let database_field = DexFieldId {
        class_descriptor: thread_class.to_owned(),
        type_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "database".to_owned(),
    };
    let saw_transaction_field = DexFieldId {
        class_descriptor: thread_class.to_owned(),
        type_descriptor: "Z".to_owned(),
        name: "sawTransactionAfterResume".to_owned(),
    };
    program.write_stored_static_field(&database_field, database.clone());
    let worker = program.allocate_object(thread_class.to_owned());

    program
        .run_started_thread(&worker, 0)
        .expect("transaction worker should park");

    assert_eq!(sqlite_in_transaction(&program, &methods, &database), 0);
    let worker_key = SqliteSessionKey {
        database: map_identity_key(&database),
        thread: map_identity_key(&worker),
    };
    assert!(program
        .sqlite_sessions
        .borrow()
        .get(&worker_key)
        .is_some_and(|session| session.transaction.is_some()));
    program
        .dispatch_pending_main_loop(420, 760, 0)
        .expect("parked transaction worker should resume");

    assert_eq!(
        program.read_stored_static_field(&saw_transaction_field),
        Some(DexValue::Int(1))
    );
    assert_eq!(sqlite_in_transaction(&program, &methods, &database), 0);
    assert!(program
        .sqlite_sessions
        .borrow()
        .get(&worker_key)
        .is_some_and(|session| session.transaction.is_none()));
    evict_sqlite_database_sessions(&program, &database);
    assert_eq!(sqlite_transaction_num(&program, &database), 3);
}

#[test]
fn aosp_sqlite_database_objects_own_independent_thread_sessions() {
    // cite: frameworks-base SQLiteDatabase.java#mThreadSession, getThreadSession
    let program = SqliteTestFixture::new("transaction_database_owner");
    let methods = SqliteTransactionMethods::new();
    let first_database = sqlite_transaction_database(&program);
    let second_database = DexValue::Reference(std::rc::Rc::from("second-database"));
    program.register_database(&second_database);

    assert_sqlite_transaction_void(&program, &methods.begin_nonexclusive, &first_database);
    sqlite_set_transaction_num(&program, &first_database, 1);
    assert_eq!(
        sqlite_in_transaction(&program, &methods, &second_database),
        0
    );
    assert_eq!(sqlite_transaction_num(&program, &second_database), 0);
    assert_eq!(
        sqlite_in_transaction(&program, &methods, &first_database),
        1
    );
    assert_sqlite_transaction_void(&program, &methods.successful, &first_database);
    assert_sqlite_transaction_void(&program, &methods.end, &first_database);
    assert_eq!(sqlite_transaction_num(&program, &second_database), 1);
}

#[test]
fn aosp_sqlite_session_exec_sql_maps_transaction_statements() {
    // cite: frameworks-base SQLiteSession.java#executeSpecial
    // cite: frameworks-base DatabaseUtils.java#getSqlStatementType
    let program = SqliteTestFixture::new("transaction_engine_error");
    let methods = SqliteTransactionMethods::new();
    let database = sqlite_transaction_database(&program);

    assert_sqlite_exec_sql(&program, &database, "BEGIN");
    assert_eq!(sqlite_in_transaction(&program, &methods, &database), 1);
    sqlite_set_transaction_num(&program, &database, 1);
    assert_sqlite_exec_sql(&program, &database, "END");
    assert_eq!(sqlite_transaction_num(&program, &database), 1);

    assert_sqlite_exec_sql(&program, &database, "BEGIN IMMEDIATE");
    sqlite_set_transaction_num(&program, &database, 2);
    assert_sqlite_exec_sql(&program, &database, "ROLLBACK");
    assert_eq!(sqlite_transaction_num(&program, &database), 1);

    assert_sqlite_exec_sql_error(
        &program,
        &database,
        "COMMIT",
        "Ljava/lang/IllegalStateException;",
        SQLITE_NO_TRANSACTION_MESSAGE,
    );
}

#[test]
fn sqlite_commit_failure_rolls_back_and_releases_the_session() {
    // cite: frameworks-base SQLiteSession.java#endTransactionUnchecked
    let program = SqliteTestFixture::new("transaction_commit_failure");
    let methods = SqliteTransactionMethods::new();
    let database = sqlite_transaction_database(&program);
    program
        .sqlite_execute_database_sql(&database, "PRAGMA foreign_keys=ON", &[])
        .expect("foreign keys should enable");
    program
        .sqlite_execute_database_sql(
            &database,
            "CREATE TABLE parent(id INTEGER PRIMARY KEY)",
            &[],
        )
        .expect("parent table should exist");
    program
        .sqlite_execute_database_sql(
            &database,
            "CREATE TABLE child(parent_id INTEGER REFERENCES parent(id) DEFERRABLE INITIALLY DEFERRED)",
            &[],
        )
        .expect("child table should exist");

    assert_sqlite_transaction_void(&program, &methods.begin, &database);
    program
        .sqlite_execute_database_sql(&database, "INSERT INTO child VALUES (9)", &[])
        .expect("deferred constraint insert should execute");
    assert_sqlite_transaction_void(&program, &methods.successful, &database);
    assert_sqlite_transaction_class(
        &program,
        &methods.end,
        &database,
        "Landroid/database/sqlite/SQLiteException;",
    );
    assert_eq!(sqlite_in_transaction(&program, &methods, &database), 0);
    let key = program.sqlite_session_key(&database);
    assert!(program
        .sqlite_sessions
        .borrow()
        .get(&key)
        .expect("SQLite session should remain reusable")
        .connection
        .connection()
        .is_autocommit());
    assert_sqlite_transaction_void(&program, &methods.begin, &database);
    assert_sqlite_transaction_void(&program, &methods.end, &database);
}

#[test]
fn builtin_sqlite_pragma_table_info_uses_executed_schema() {
    let program = SqliteTestFixture::new("pragma_table_info");
    let compile_statement = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "compileStatement".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Landroid/database/sqlite/SQLiteStatement;".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let execute = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteStatement;".to_owned(),
        name: "execute".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let raw_query_with_factory = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "rawQueryWithFactory".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LLLLL".to_owned(),
            return_type: "Landroid/database/Cursor;".to_owned(),
            parameters: vec![
                "Landroid/database/sqlite/SQLiteDatabase$CursorFactory;".to_owned(),
                "Ljava/lang/String;".to_owned(),
                "[Ljava/lang/String;".to_owned(),
                "Ljava/lang/String;".to_owned(),
            ],
        },
    };
    let database = DexValue::Reference(std::rc::Rc::from("database"));
    program.register_database(&database);
    let current_schema =
        "CREATE TABLE IF NOT EXISTS `Timer` (`name` TEXT NOT NULL, `duration` INTEGER NOT NULL)";

    let statement = program
        .invoke_builtin_method(
            &compile_statement,
            Some(&test_invocation(vec![
                database.clone(),
                DexValue::String(std::rc::Rc::from(current_schema.to_owned())),
            ])),
        )
        .expect("compileStatement should be implemented");
    let DexExecutionResult::Value(statement) = statement else {
        panic!("compileStatement should return a statement");
    };

    assert_eq!(
        program.invoke_builtin_method(&execute, Some(&test_invocation(vec![statement]))),
        Some(DexExecutionResult::Void)
    );

    let cursor = program
        .invoke_builtin_method(
            &raw_query_with_factory,
            Some(&test_invocation(vec![
                database,
                DexValue::Null,
                DexValue::String(std::rc::Rc::from("PRAGMA table_info(`Timer`)")),
                DexValue::Null,
                DexValue::Null,
            ])),
        )
        .expect("rawQueryWithFactory should be implemented");
    let DexExecutionResult::Value(cursor) = cursor else {
        panic!("rawQueryWithFactory should return a cursor");
    };
    let cursor_key = map_identity_key(&cursor);
    let runtime = program.runtime.borrow();
    let cursor_state = runtime
        .sqlite_cursors
        .get(&cursor_key)
        .expect("PRAGMA query should create cursor state");

    assert_eq!(cursor_state.rows.len(), 2);
    assert_eq!(
        cursor_state.rows[1][1],
        DexValue::String(std::rc::Rc::from("duration"))
    );
}

#[test]
fn builtin_sqlite_master_query_returns_recorded_views() {
    let program = SqliteTestFixture::new("master_query");
    let compile_statement = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "compileStatement".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Landroid/database/sqlite/SQLiteStatement;".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let execute = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteStatement;".to_owned(),
        name: "execute".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let raw_query_with_factory = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "rawQueryWithFactory".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LLLLL".to_owned(),
            return_type: "Landroid/database/Cursor;".to_owned(),
            parameters: vec![
                "Landroid/database/sqlite/SQLiteDatabase$CursorFactory;".to_owned(),
                "Ljava/lang/String;".to_owned(),
                "[Ljava/lang/String;".to_owned(),
                "Ljava/lang/String;".to_owned(),
            ],
        },
    };
    let database = DexValue::Reference(std::rc::Rc::from("database"));
    program.register_database(&database);
    let table_sql = "CREATE TABLE `timer` (`uid` INTEGER NOT NULL, `name` TEXT NOT NULL)";
    let view_sql = "CREATE VIEW `TimerSummary` AS SELECT name, COUNT(uid) AS usageCount FROM timer GROUP BY name";

    for sql in [table_sql, view_sql] {
        let statement = program
            .invoke_builtin_method(
                &compile_statement,
                Some(&test_invocation(vec![
                    database.clone(),
                    DexValue::String(std::rc::Rc::from(sql.to_owned())),
                ])),
            )
            .expect("compileStatement should be implemented");
        let DexExecutionResult::Value(statement) = statement else {
            panic!("compileStatement should return a statement");
        };

        assert_eq!(
            program.invoke_builtin_method(&execute, Some(&test_invocation(vec![statement]))),
            Some(DexExecutionResult::Void)
        );
    }

    let query = "SELECT name, sql FROM sqlite_master WHERE type = 'view' AND name = 'TimerSummary'";
    let cursor = program
        .invoke_builtin_method(
            &raw_query_with_factory,
            Some(&test_invocation(vec![
                database,
                DexValue::Null,
                DexValue::String(std::rc::Rc::from(query.to_owned())),
                DexValue::Null,
                DexValue::Null,
            ])),
        )
        .expect("rawQueryWithFactory should be implemented");
    let DexExecutionResult::Value(cursor) = cursor else {
        panic!("rawQueryWithFactory should return a cursor");
    };
    let cursor_key = map_identity_key(&cursor);
    let runtime = program.runtime.borrow();
    let cursor_state = runtime
        .sqlite_cursors
        .get(&cursor_key)
        .expect("sqlite_master query should create cursor state");

    assert_eq!(cursor_state.rows.len(), 1);
    assert_eq!(
        cursor_state.rows[0][0],
        DexValue::String(std::rc::Rc::from("TimerSummary"))
    );
    assert_eq!(
        cursor_state.rows[0][1],
        DexValue::String(std::rc::Rc::from(view_sql.to_owned()))
    );
}

#[test]
fn builtin_sqlite_temp_table_persists_for_database_object() {
    let program = DexProgram::new(Vec::new());
    let exec_sql = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "execSQL".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    let raw_query_with_factory = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "rawQueryWithFactory".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LLLLL".to_owned(),
            return_type: "Landroid/database/Cursor;".to_owned(),
            parameters: vec![
                "Landroid/database/sqlite/SQLiteDatabase$CursorFactory;".to_owned(),
                "Ljava/lang/String;".to_owned(),
                "[Ljava/lang/String;".to_owned(),
                "Ljava/lang/String;".to_owned(),
            ],
        },
    };
    let writer_database = DexValue::Reference(std::rc::Rc::from("database-writer"));
    let database_path = std::env::temp_dir().join(format!(
        "winedroid-shared-sqlite-{}.db",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    program
        .register_sqlite_database_path(&writer_database, &database_path)
        .unwrap();

    assert_eq!(
        program.invoke_builtin_method(
            &exec_sql,
            Some(&test_invocation(vec![
                writer_database.clone(),
                DexValue::String(
                    std::rc::Rc::from("CREATE TEMP TABLE room_table_modification_log(table_id INTEGER PRIMARY KEY, invalidated INTEGER NOT NULL DEFAULT 0)"
                        .to_owned())
                )
            ])),
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &exec_sql,
            Some(&test_invocation(vec![
                writer_database.clone(),
                DexValue::String(std::rc::Rc::from(
                    "INSERT INTO room_table_modification_log(table_id, invalidated) VALUES(1, 0)"
                        .to_owned()
                ))
            ])),
        ),
        Some(DexExecutionResult::Void)
    );

    let cursor = program
        .invoke_builtin_method(
            &raw_query_with_factory,
            Some(&test_invocation(vec![
                writer_database,
                DexValue::Null,
                DexValue::String(std::rc::Rc::from(
                    "SELECT invalidated FROM room_table_modification_log WHERE table_id = 1"
                        .to_owned(),
                )),
                DexValue::Null,
                DexValue::Null,
            ])),
        )
        .expect("rawQueryWithFactory should be implemented");
    let DexExecutionResult::Value(cursor) = cursor else {
        panic!("rawQueryWithFactory should return a cursor");
    };
    let cursor_key = map_identity_key(&cursor);
    let runtime = program.runtime.borrow();
    let cursor_state = runtime
        .sqlite_cursors
        .get(&cursor_key)
        .expect("temp table query should create cursor state");

    assert_eq!(cursor_state.columns, vec!["invalidated".to_owned()]);
    assert_eq!(cursor_state.rows, vec![vec![DexValue::Long(0)]]);
}

#[test]
fn builtin_set_iterator_returns_empty_iterator() {
    let program = DexProgram::new(Vec::new());
    let set_iterator = DexMethodId {
        class_descriptor: "Ljava/util/Set;".to_owned(),
        name: "iterator".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Ljava/util/Iterator;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let iterable_iterator = DexMethodId {
        class_descriptor: "Ljava/lang/Iterable;".to_owned(),
        name: "iterator".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "L".to_owned(),
            return_type: "Ljava/util/Iterator;".to_owned(),
            parameters: Vec::new(),
        },
    };
    let set_size = DexMethodId {
        class_descriptor: "Ljava/util/Set;".to_owned(),
        name: "size".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "I".to_owned(),
            return_type: "I".to_owned(),
            parameters: Vec::new(),
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&set_iterator, None),
        Some(DexExecutionResult::Value(empty_iterator()))
    );
    assert_eq!(
        program.invoke_builtin_method(&iterable_iterator, None),
        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from("Ljava/util/Iterator;.empty")
        )))
    );
    assert_eq!(
        program.invoke_builtin_method(&set_size, None),
        Some(DexExecutionResult::Int(0))
    );
}

#[test]
fn builtin_empty_iterator_has_next_returns_false() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Ljava/util/Iterator;".to_owned(),
        name: "hasNext".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "Z".to_owned(),
            return_type: "Z".to_owned(),
            parameters: Vec::new(),
        },
    };

    assert_eq!(
        program.invoke_builtin_method(&method, None),
        Some(DexExecutionResult::Int(0))
    );
}

#[test]
fn builtin_map_get_returns_null_for_empty_bootstrap_map() {
    let program = DexProgram::new(Vec::new());
    let method = DexMethodId {
        class_descriptor: "Ljava/util/Map;".to_owned(),
        name: "get".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "LL".to_owned(),
            return_type: "Ljava/lang/Object;".to_owned(),
            parameters: vec!["Ljava/lang/Object;".to_owned()],
        },
    };
    let invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("map")),
        DexValue::Reference(std::rc::Rc::from("key")),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&method, Some(&invocation)),
        Some(DexExecutionResult::Value(DexValue::Null))
    );
}

#[test]
fn builtin_hash_map_constructor_returns_void() {
    let program = DexProgram::new(Vec::new());
    let hash_map = DexMethodId {
        class_descriptor: "Ljava/util/HashMap;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let linked_hash_map = DexMethodId {
        class_descriptor: "Ljava/util/LinkedHashMap;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let linked_hash_map_with_capacity = DexMethodId {
        class_descriptor: "Ljava/util/LinkedHashMap;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VI".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let linked_hash_map_with_capacity_load_and_order = DexMethodId {
        class_descriptor: "Ljava/util/LinkedHashMap;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VIFZ".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["I".to_owned(), "F".to_owned(), "Z".to_owned()],
        },
    };
    let identity_hash_map = DexMethodId {
        class_descriptor: "Ljava/util/IdentityHashMap;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let weak_hash_map = DexMethodId {
        class_descriptor: "Ljava/util/WeakHashMap;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let weak_hash_map_with_capacity = DexMethodId {
        class_descriptor: "Ljava/util/WeakHashMap;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VI".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let tree_map = DexMethodId {
        class_descriptor: "Ljava/util/TreeMap;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let abstract_set = DexMethodId {
        class_descriptor: "Ljava/util/AbstractSet;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let concurrent_hash_map = DexMethodId {
        class_descriptor: "Ljava/util/concurrent/ConcurrentHashMap;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let linked_hash_set = DexMethodId {
        class_descriptor: "Ljava/util/LinkedHashSet;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let hash_set = DexMethodId {
        class_descriptor: "Ljava/util/HashSet;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let linked_hash_set_with_capacity = DexMethodId {
        class_descriptor: "Ljava/util/LinkedHashSet;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VI".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["I".to_owned()],
        },
    };
    let copy_on_write_array_set = DexMethodId {
        class_descriptor: "Ljava/util/concurrent/CopyOnWriteArraySet;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let array_deque = DexMethodId {
        class_descriptor: "Ljava/util/ArrayDeque;".to_owned(),
        name: "<init>".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "V".to_owned(),
            return_type: "V".to_owned(),
            parameters: Vec::new(),
        },
    };
    let capacity_invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("set")),
        DexValue::Int(4),
    ]);
    let linked_map_full_invocation = test_invocation(vec![
        DexValue::Reference(std::rc::Rc::from("linked-map")),
        DexValue::Int(4),
        DexValue::Float(0.75),
        DexValue::Int(1),
    ]);

    assert_eq!(
        program.invoke_builtin_method(&hash_map, None),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&linked_hash_map, None),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&linked_hash_map_with_capacity, Some(&capacity_invocation)),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(
            &linked_hash_map_with_capacity_load_and_order,
            Some(&linked_map_full_invocation)
        ),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&identity_hash_map, None),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&weak_hash_map, None),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&weak_hash_map_with_capacity, Some(&capacity_invocation)),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&tree_map, None),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&abstract_set, None),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&concurrent_hash_map, None),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&linked_hash_set, None),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&hash_set, None),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&linked_hash_set_with_capacity, Some(&capacity_invocation)),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&copy_on_write_array_set, None),
        Some(DexExecutionResult::Void)
    );
    assert_eq!(
        program.invoke_builtin_method(&array_deque, None),
        Some(DexExecutionResult::Void)
    );
}

const SQLITE_NO_TRANSACTION_MESSAGE: &str =
    "Cannot perform this operation because there is no current transaction.";
const SQLITE_ALREADY_SUCCESSFUL_MESSAGE: &str = "Cannot perform this operation because the transaction has already been marked successful.  The only thing you can do now is call endTransaction().";

struct SqliteTransactionMethods {
    begin: DexMethodId,
    begin_nonexclusive: DexMethodId,
    successful: DexMethodId,
    end: DexMethodId,
    in_transaction: DexMethodId,
}

impl SqliteTransactionMethods {
    fn new() -> Self {
        Self {
            begin: sqlite_database_transaction_method("beginTransaction", "V"),
            begin_nonexclusive: sqlite_database_transaction_method(
                "beginTransactionNonExclusive",
                "V",
            ),
            successful: sqlite_database_transaction_method("setTransactionSuccessful", "V"),
            end: sqlite_database_transaction_method("endTransaction", "V"),
            in_transaction: sqlite_database_transaction_method("inTransaction", "Z"),
        }
    }
}

fn sqlite_database_transaction_method(name: &str, shorty: &str) -> DexMethodId {
    DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: name.to_owned(),
        proto: crate::dex::DexProto {
            shorty: shorty.to_owned(),
            return_type: shorty.to_owned(),
            parameters: Vec::new(),
        },
    }
}

fn sqlite_transaction_database(program: &SqliteTestFixture) -> DexValue {
    let database = DexValue::Reference(std::rc::Rc::from("database"));
    program.register_database(&database);
    program
        .sqlite_execute_database_sql(&database, "CREATE TABLE test (num INTEGER)", &[])
        .expect("transaction table should exist");
    program
        .sqlite_execute_database_sql(&database, "INSERT INTO test (num) VALUES (0)", &[])
        .expect("transaction row should exist");
    database
}

fn invoke_sqlite_transaction(
    program: &DexProgram,
    method: &DexMethodId,
    database: &DexValue,
) -> DexExecutionResult {
    program
        .invoke_builtin_method(method, Some(&test_invocation(vec![database.clone()])))
        .expect("SQLite transaction builtin should be handled")
}

fn assert_sqlite_transaction_void(program: &DexProgram, method: &DexMethodId, database: &DexValue) {
    assert_eq!(
        invoke_sqlite_transaction(program, method, database),
        DexExecutionResult::Void
    );
}

fn assert_sqlite_transaction_error(
    program: &DexProgram,
    method: &DexMethodId,
    database: &DexValue,
    class_descriptor: &str,
    message: &str,
) {
    let result = invoke_sqlite_transaction(program, method, database);
    let DexExecutionResult::Throw(exception) = result else {
        panic!("SQLite transaction call should throw {class_descriptor}");
    };
    assert_eq!(value_class_descriptor(&exception), Some(class_descriptor));
    assert_eq!(
        program.read_stored_instance_field(
            &exception,
            &DexFieldId {
                class_descriptor: "Ljava/lang/Throwable;".to_owned(),
                type_descriptor: "Ljava/lang/String;".to_owned(),
                name: "detailMessage".to_owned(),
            },
        ),
        Some(DexValue::String(std::rc::Rc::from(message)))
    );
}

fn assert_sqlite_transaction_class(
    program: &DexProgram,
    method: &DexMethodId,
    database: &DexValue,
    class_descriptor: &str,
) {
    let result = invoke_sqlite_transaction(program, method, database);
    let DexExecutionResult::Throw(exception) = result else {
        panic!("SQLite transaction call should throw {class_descriptor}");
    };
    assert_eq!(value_class_descriptor(&exception), Some(class_descriptor));
}

fn invoke_sqlite_exec_sql(
    program: &DexProgram,
    database: &DexValue,
    sql: &str,
) -> DexExecutionResult {
    let method = DexMethodId {
        class_descriptor: "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
        name: "execSQL".to_owned(),
        proto: crate::dex::DexProto {
            shorty: "VL".to_owned(),
            return_type: "V".to_owned(),
            parameters: vec!["Ljava/lang/String;".to_owned()],
        },
    };
    program
        .invoke_builtin_method(
            &method,
            Some(&test_invocation(vec![
                database.clone(),
                DexValue::String(std::rc::Rc::from(sql)),
            ])),
        )
        .expect("SQLite execSQL builtin should be handled")
}

fn assert_sqlite_exec_sql(program: &DexProgram, database: &DexValue, sql: &str) {
    assert_eq!(
        invoke_sqlite_exec_sql(program, database, sql),
        DexExecutionResult::Void
    );
}

fn assert_sqlite_exec_sql_error(
    program: &DexProgram,
    database: &DexValue,
    sql: &str,
    class_descriptor: &str,
    message: &str,
) {
    let DexExecutionResult::Throw(exception) = invoke_sqlite_exec_sql(program, database, sql)
    else {
        panic!("SQLite execSQL should throw {class_descriptor}");
    };
    assert_eq!(value_class_descriptor(&exception), Some(class_descriptor));
    assert_eq!(
        program.read_stored_instance_field(
            &exception,
            &DexFieldId {
                class_descriptor: "Ljava/lang/Throwable;".to_owned(),
                type_descriptor: "Ljava/lang/String;".to_owned(),
                name: "detailMessage".to_owned(),
            },
        ),
        Some(DexValue::String(std::rc::Rc::from(message)))
    );
}

fn evict_sqlite_database_sessions(program: &DexProgram, database: &DexValue) {
    let database_key = map_identity_key(database);
    program
        .sqlite_sessions
        .borrow_mut()
        .retain(|session, _| session.database != database_key);
}

fn sqlite_in_transaction(
    program: &DexProgram,
    methods: &SqliteTransactionMethods,
    database: &DexValue,
) -> i32 {
    let result = invoke_sqlite_transaction(program, &methods.in_transaction, database);
    let DexExecutionResult::Int(value) = result else {
        panic!("SQLite inTransaction should return boolean int");
    };
    value
}

fn sqlite_set_transaction_num(program: &DexProgram, database: &DexValue, value: i64) {
    program
        .sqlite_execute_database_sql(database, &format!("UPDATE test SET num = {value}"), &[])
        .expect("transaction update should execute");
}

fn sqlite_transaction_num(program: &DexProgram, database: &DexValue) -> i64 {
    let result = program
        .sqlite_raw_query_result(database, "SELECT num FROM test", &[])
        .expect("transaction value query should execute");
    let value = result
        .rows
        .first()
        .and_then(|row| row.first())
        .expect("transaction table should contain one row");
    match value {
        DexValue::Long(value) => *value,
        DexValue::Int(value) => i64::from(*value),
        _ => panic!("transaction value should be an integer"),
    }
}

struct SqliteTestFixture {
    program: DexProgram,
    data_dir: PathBuf,
}

impl SqliteTestFixture {
    fn new(name: &str) -> Self {
        Self::with_dex_files(name, Vec::new())
    }

    fn with_dex_files(name: &str, dex_files: Vec<DexFile>) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let data_dir = std::env::temp_dir().join(format!(
            "winedroid-content-storage-{name}-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&data_dir).expect("SQLite test data directory should exist");
        let program = DexProgram::with_app_data_dir(dex_files, data_dir.clone())
            .expect("SQLite test program should have private storage");

        Self { program, data_dir }
    }

    fn register_database(&self, database: &DexValue) {
        self.program
            .register_sqlite_database_name(
                database,
                &DexValue::Reference(std::rc::Rc::from("Landroid/content/Context;.bootstrap")),
                Some("test.db"),
            )
            .expect("SQLite test database should have a Context path");
    }
}

impl Deref for SqliteTestFixture {
    type Target = DexProgram;

    fn deref(&self) -> &Self::Target {
        &self.program
    }
}

impl Drop for SqliteTestFixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.data_dir).ok();
    }
}
