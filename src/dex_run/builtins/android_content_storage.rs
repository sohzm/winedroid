use super::super::*;

impl DexProgram {
    pub(super) fn invoke_android_content_storage_builtin(
        &self,
        method: &DexMethodId,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        if method.class_descriptor == "Landroid/content/ContentValues;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/content/ContentValues;"
            && method.name == "put"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;Ljava/lang/String;)V")
        {
            let invocation = invocation?;
            self.write_content_values_string(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/content/ContentValues;"
            && method.name == "put"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;Ljava/lang/Long;)V")
        {
            let invocation = invocation?;
            self.write_content_values_long(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/content/Intent;"
            && method.name == "getFlags"
            && method.proto.descriptor_is("()I")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Landroid/content/Intent;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/content/Intent;"
            && method.name == "putIntegerArrayListExtra"
            && method.proto.descriptor()
                == "(Ljava/lang/String;Ljava/util/ArrayList;)Landroid/content/Intent;"
        {
            let invocation = invocation?;
            let intent = self.put_intent_extra(invocation)?;

            return Some(DexExecutionResult::Value(intent));
        }

        if method.class_descriptor == "Landroid/content/Intent;"
            && method.name == "getIntegerArrayListExtra"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Ljava/util/ArrayList;")
        {
            let invocation = invocation?;
            let value = self.get_intent_extra(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/content/Intent;"
            && method.name == "<init>"
            && method
                .proto
                .descriptor_is("(Landroid/content/Context;Ljava/lang/Class;)V")
        {
            let invocation = invocation?;
            self.initialize_intent_for_class(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/content/Intent;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(Ljava/lang/String;)V")
        {
            let invocation = invocation?;
            self.initialize_intent_for_action(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/content/Intent;"
            && method.name == "<init>"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;Landroid/net/Uri;)V")
        {
            let invocation = invocation?;
            self.initialize_intent_for_action_and_uri(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/content/Intent;"
            && method.name == "getData"
            && method.proto.descriptor_is("()Landroid/net/Uri;")
        {
            let invocation = invocation?;
            let value = self.intent_data(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/content/Intent;"
            && method.name == "setData"
            && method
                .proto
                .descriptor_is("(Landroid/net/Uri;)Landroid/content/Intent;")
        {
            let invocation = invocation?;
            let value = self.set_intent_data(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: frameworks-base Intent.java#setAction
        if method.class_descriptor == "Landroid/content/Intent;"
            && method.name == "setAction"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Landroid/content/Intent;")
        {
            let invocation = invocation?;
            let value = self.set_intent_action(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/content/Intent;"
            && method.name == "getAction"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.intent_action(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: frameworks-base Intent.java#setComponent
        if method.matches_builtin_signature(
            "Landroid/content/Intent;",
            "setComponent",
            "(Landroid/content/ComponentName;)Landroid/content/Intent;",
        ) {
            let invocation = invocation?;
            return Some(DexExecutionResult::Value(
                invocation.arguments.first()?.clone(),
            ));
        }

        // cite: frameworks-base Intent.java#setDataAndType
        if method.class_descriptor == "Landroid/content/Intent;"
            && method.name == "setDataAndType"
            && method.proto.descriptor()
                == "(Landroid/net/Uri;Ljava/lang/String;)Landroid/content/Intent;"
        {
            let invocation = invocation?;
            let value = self.set_intent_data_and_type(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: frameworks-base Intent.java#getType, mType
        if method.class_descriptor == "Landroid/content/Intent;"
            && method.name == "getType"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            if !dex_value_is_object_like(invocation.arguments.first()?) {
                return None;
            }

            return Some(DexExecutionResult::Value(
                self.read_stored_instance_field(
                    &invocation.arguments[0],
                    &DexFieldId {
                        class_descriptor: "Landroid/content/Intent;".to_owned(),
                        type_descriptor: "Ljava/lang/String;".to_owned(),
                        name: "mType".to_owned(),
                    },
                )
                .unwrap_or(DexValue::Null),
            ));
        }

        if method.class_descriptor == "Landroid/content/Intent;"
            && method.name == "getType"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.intent_type(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/content/Intent;"
            && method.name == "getExtras"
            && method.proto.descriptor_is("()Landroid/os/Bundle;")
        {
            return Some(DexExecutionResult::Value(DexValue::Null));
        }

        if method.class_descriptor == "Landroid/net/Uri;"
            && method.name == "parse"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Landroid/net/Uri;")
        {
            let invocation = invocation?;
            let value = self.parse_uri(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: frameworks-base Uri.java#getScheme, getQuery, getFragment, getHost, getPath
        if method.class_descriptor == "Landroid/net/Uri;"
            && method.proto.descriptor_is("()Ljava/lang/String;")
            && matches!(
                method.name.as_str(),
                "getScheme"
                    | "getQuery"
                    | "getFragment"
                    | "getHost"
                    | "getAuthority"
                    | "getPath"
                    | "getLastPathSegment"
            )
        {
            let invocation = invocation?;
            let component = match method.name.as_str() {
                "getScheme" => "scheme",
                "getQuery" => "query",
                "getFragment" => "fragment",
                "getHost" => "host",
                "getAuthority" => "authority",
                "getPath" => "path",
                "getLastPathSegment" => "lastPathSegment",
                _ => return None,
            };
            let value = self.uri_component(invocation, component)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: frameworks-base Uri.java#getPathSegments
        if method.class_descriptor == "Landroid/net/Uri;"
            && method.name == "getPathSegments"
            && method.proto.descriptor_is("()Ljava/util/List;")
        {
            let invocation = invocation?;
            let value = self.uri_path_segments(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: frameworks-base Uri.java#getQueryParameterNames
        if method.class_descriptor == "Landroid/net/Uri;"
            && method.name == "getQueryParameterNames"
            && method.proto.descriptor_is("()Ljava/util/Set;")
        {
            let invocation = invocation?;
            let pairs = self.uri_query_pairs(invocation)?;
            let mut names = Vec::new();
            for (name, _) in pairs {
                let value = DexValue::String(std::rc::Rc::from(name));
                if !names.contains(&value) {
                    names.push(value);
                }
            }
            let set = self.allocate_object("Ljava/util/LinkedHashSet;".to_owned());
            self.runtime
                .borrow_mut()
                .list_entries
                .insert(map_identity_key(&set), names);

            return Some(DexExecutionResult::Value(set));
        }

        // cite: frameworks-base Uri.java#getQueryParameters, getQueryParameter
        if method.class_descriptor == "Landroid/net/Uri;"
            && method.name == "getQueryParameters"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Ljava/util/List;")
        {
            let invocation = invocation?;
            let DexValue::String(wanted) = invocation.arguments.get(1)? else {
                return None;
            };
            let pairs = self.uri_query_pairs(invocation)?;
            let values: Vec<DexValue> = pairs
                .into_iter()
                .filter(|(name, _)| name.as_str() == wanted.as_ref())
                .map(|(_, value)| DexValue::String(std::rc::Rc::from(value)))
                .collect();
            let list = self.allocate_object("Ljava/util/ArrayList;".to_owned());
            self.runtime
                .borrow_mut()
                .list_entries
                .insert(map_identity_key(&list), values);

            return Some(DexExecutionResult::Value(list));
        }

        if method.class_descriptor == "Landroid/net/Uri;"
            && method.name == "getQueryParameter"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let DexValue::String(wanted) = invocation.arguments.get(1)? else {
                return None;
            };
            let pairs = self.uri_query_pairs(invocation)?;
            let value = pairs
                .into_iter()
                .find(|(name, _)| name.as_str() == wanted.as_ref())
                .map(|(_, value)| DexValue::String(std::rc::Rc::from(value)))
                .unwrap_or(DexValue::Null);

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/net/Uri;"
            && method.name == "toString"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.uri_to_string(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/content/IntentFilter;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/content/IntentFilter;"
            && method.name == "addAction"
            && method.proto.descriptor_is("(Ljava/lang/String;)V")
        {
            let invocation = invocation?;
            self.add_intent_filter_action(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/content/IntentFilter;"
            && method.name == "countActions"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.intent_filter_action_count(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: frameworks-base IntentFilter.java#addDataScheme
        if method.matches_builtin_signature(
            "Landroid/content/IntentFilter;",
            "addDataScheme",
            "(Ljava/lang/String;)V",
        ) {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/content/BroadcastReceiver;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base ComponentName.java#ComponentName
        if method.class_descriptor == "Landroid/content/ComponentName;"
            && method.name == "<init>"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Landroid/content/Context;Ljava/lang/Class;)V"
                    | "(Landroid/content/Context;Ljava/lang/String;)V"
                    | "(Ljava/lang/String;Ljava/lang/String;)V"
            )
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base PackageManager.java#getComponentEnabledSetting, setComponentEnabledSetting
        if method.class_descriptor == "Landroid/content/pm/PackageManager;"
            && method.name == "getComponentEnabledSetting"
            && method
                .proto
                .descriptor_is("(Landroid/content/ComponentName;)I")
        {
            return Some(DexExecutionResult::Int(0));
        }

        // cite: frameworks-base PackageManager.java#setComponentEnabledSetting
        if method.class_descriptor == "Landroid/content/pm/PackageManager;"
            && method.name == "setComponentEnabledSetting"
            && method
                .proto
                .descriptor_is("(Landroid/content/ComponentName;II)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "getApplicationContext"
            && method.proto.descriptor_is("()Landroid/content/Context;")
        {
            return Some(DexExecutionResult::Value(self.application_value()));
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "getCacheDir"
            && method.proto.descriptor_is("()Ljava/io/File;")
        {
            let invocation = invocation?;
            let value = self.context_private_directory_file(invocation, "cache")?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "getFilesDir"
            && method.proto.descriptor_is("()Ljava/io/File;")
        {
            let invocation = invocation?;
            let value = self.context_private_directory_file(invocation, "files")?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "getExternalFilesDirs"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)[Ljava/io/File;")
        {
            let invocation = invocation?;
            let value = self.context_external_files_dirs(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "getDataDir"
            && method.proto.descriptor_is("()Ljava/io/File;")
        {
            let invocation = invocation?;
            let value = self.context_data_dir_file(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "getNoBackupFilesDir"
            && method.proto.descriptor_is("()Ljava/io/File;")
        {
            let invocation = invocation?;
            let value = self.context_private_directory_file(invocation, "no_backup")?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "getCodeCacheDir"
            && method.proto.descriptor_is("()Ljava/io/File;")
        {
            let invocation = invocation?;
            let value = self.context_private_directory_file(invocation, "code_cache")?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "getSharedPreferencesPath"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Ljava/io/File;")
        {
            let invocation = invocation?;
            return self.context_shared_preferences_path_file(invocation);
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "createDeviceProtectedStorageContext"
            && method.proto.descriptor_is("()Landroid/content/Context;")
        {
            let invocation = invocation?;
            let context = context_receiver(invocation)?;
            let value = self.create_private_storage_context(
                context,
                crate::guest_filesystem::PrivateStorageKind::DeviceProtected,
            )?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "createCredentialProtectedStorageContext"
            && method.proto.descriptor_is("()Landroid/content/Context;")
        {
            let invocation = invocation?;
            let context = context_receiver(invocation)?;
            let value = self.create_private_storage_context(
                context,
                crate::guest_filesystem::PrivateStorageKind::CredentialProtected,
            )?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "isDeviceProtectedStorage"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let context = context_receiver(invocation)?;
            let value = self.context_is_private_storage(
                context,
                crate::guest_filesystem::PrivateStorageKind::DeviceProtected,
            )?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "isCredentialProtectedStorage"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let context = context_receiver(invocation)?;
            let value = self.context_is_private_storage(
                context,
                crate::guest_filesystem::PrivateStorageKind::CredentialProtected,
            )?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "getDatabasePath"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Ljava/io/File;")
        {
            let invocation = invocation?;
            return self.context_database_path_file(invocation);
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "registerComponentCallbacks"
            && method
                .proto
                .descriptor_is("(Landroid/content/ComponentCallbacks;)V")
        {
            let invocation = invocation?;
            self.register_component_callback(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "unregisterComponentCallbacks"
            && method
                .proto
                .descriptor_is("(Landroid/content/ComponentCallbacks;)V")
        {
            let invocation = invocation?;
            self.unregister_component_callback(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "unregisterReceiver"
            && method
                .proto
                .descriptor_is("(Landroid/content/BroadcastReceiver;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base Context.java#registerReceiver
        if method.class_descriptor == "Landroid/content/Context;"
            && method.name == "registerReceiver"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Landroid/content/BroadcastReceiver;Landroid/content/IntentFilter;)Landroid/content/Intent;"
                    | "(Landroid/content/BroadcastReceiver;Landroid/content/IntentFilter;I)Landroid/content/Intent;"
                    | "(Landroid/content/BroadcastReceiver;Landroid/content/IntentFilter;Ljava/lang/String;Landroid/os/Handler;)Landroid/content/Intent;"
                    | "(Landroid/content/BroadcastReceiver;Landroid/content/IntentFilter;Ljava/lang/String;Landroid/os/Handler;I)Landroid/content/Intent;"
            )
        {
            return Some(DexExecutionResult::Value(DexValue::Null));
        }

        if method.class_descriptor == "Landroid/content/pm/PackageManager;"
            && method.name == "hasSystemFeature"
            && method.proto.descriptor_is("(Ljava/lang/String;)Z")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Landroid/content/pm/PackageManager;"
            && method.name == "checkPermission"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;Ljava/lang/String;)I")
        {
            let invocation = invocation?;
            let value = self.package_manager_check_permission(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/content/pm/PackageManager;"
            && method.name == "getActivityInfo"
            && method.proto.descriptor()
                == "(Landroid/content/ComponentName;I)Landroid/content/pm/ActivityInfo;"
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroid/content/pm/ActivityInfo;.bootstrap"),
            )));
        }

        if method.class_descriptor == "Landroid/content/pm/PackageManager;"
            && method.name == "getPackageInfo"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;I)Landroid/content/pm/PackageInfo;")
        {
            let invocation = invocation?;
            return Some(self.package_manager_get_package_info(invocation)?);
        }

        if method.class_descriptor == "Landroid/content/pm/PackageManager;"
            && method.name == "getApplicationInfo"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;I)Landroid/content/pm/ApplicationInfo;")
        {
            let invocation = invocation?;
            return Some(self.package_manager_get_application_info(invocation)?);
        }

        if method.class_descriptor == "Landroid/content/pm/PackageManager;"
            && method.name == "resolveService"
            && method
                .proto
                .descriptor_is("(Landroid/content/Intent;I)Landroid/content/pm/ResolveInfo;")
        {
            let invocation = invocation?;
            return Some(self.package_manager_resolve_service(invocation)?);
        }

        if method.class_descriptor == "Landroid/content/pm/PackageManager;"
            && method.name == "resolveContentProvider"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;I)Landroid/content/pm/ProviderInfo;")
        {
            let invocation = invocation?;
            return Some(self.package_manager_resolve_content_provider(invocation)?);
        }

        if method.class_descriptor == "Landroid/content/pm/PackageManager;"
            && method.name == "getInstalledPackages"
            && method.proto.descriptor_is("(I)Ljava/util/List;")
        {
            let invocation = invocation?;
            return Some(self.package_manager_get_installed_packages(invocation)?);
        }

        if method.class_descriptor == "Landroid/content/pm/PackageManager;"
            && method.name == "getInstalledApplications"
            && method.proto.descriptor_is("(I)Ljava/util/List;")
        {
            let invocation = invocation?;
            return Some(self.package_manager_get_installed_applications(invocation)?);
        }

        if method.class_descriptor == "Landroid/content/pm/PackageManager;"
            && method.name == "getPackageInstaller"
            && method
                .proto
                .descriptor_is("()Landroid/content/pm/PackageInstaller;")
        {
            return Some(DexExecutionResult::Value(self.allocate_object(
                "Landroid/content/pm/PackageInstaller;".to_owned(),
            )));
        }

        if method.class_descriptor == "Landroid/content/pm/PackageInstaller;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("createSession", "(Landroid/content/pm/PackageInstaller$SessionParams;)I") => {
                    let invocation = invocation?;
                    return Some(self.package_installer_create_session(invocation)?);
                }
                ("openSession", "(I)Landroid/content/pm/PackageInstaller$Session;") => {
                    let invocation = invocation?;
                    return Some(self.package_installer_open_session(invocation)?);
                }
                ("abandonSession", "(I)V") => {
                    return Some(DexExecutionResult::Void);
                }
                // cite: frameworks-base PackageInstaller.java#getMySessions
                ("getMySessions", "()Ljava/util/List;") => {
                    let list = self.allocate_object("Ljava/util/ArrayList;".to_owned());
                    self.runtime
                        .borrow_mut()
                        .list_entries
                        .insert(map_identity_key(&list), Vec::new());
                    return Some(DexExecutionResult::Value(list));
                }
                _ => {}
            }
        }

        if method.class_descriptor == "Landroid/content/pm/PackageInstaller$Session;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("openWrite", "(Ljava/lang/String;JJ)Ljava/io/OutputStream;") => {
                    let invocation = invocation?;
                    return Some(self.package_installer_session_open_write(invocation)?);
                }
                ("fsync", "(Ljava/io/OutputStream;)V") | ("close", "()V") => {
                    return Some(DexExecutionResult::Void);
                }
                ("commit", "(Landroid/content/IntentSender;)V") => {
                    let invocation = invocation?;
                    return Some(self.package_installer_session_commit(invocation)?);
                }
                _ => {}
            }
        }

        if method.class_descriptor == "Landroid/content/pm/Signature;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("toByteArray", "()[B") => {
                    let invocation = invocation?;
                    return Some(self.pm_signature_to_byte_array(invocation)?);
                }
                ("toCharsString", "()Ljava/lang/String;") => {
                    let invocation = invocation?;
                    return Some(self.pm_signature_to_chars_string(invocation)?);
                }
                ("hashCode", "()I") => {
                    let invocation = invocation?;
                    return Some(self.pm_signature_hash_code(invocation)?);
                }
                ("equals", "(Ljava/lang/Object;)Z") => {
                    let invocation = invocation?;
                    return Some(self.pm_signature_equals(invocation)?);
                }
                _ => {}
            }
        }

        if method.class_descriptor == "Landroid/content/pm/SigningInfo;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                (
                    "getApkContentsSigners" | "getSigningCertificateHistory",
                    "()[Landroid/content/pm/Signature;",
                ) => {
                    let invocation = invocation?;
                    return Some(self.pm_signing_info_signers(invocation)?);
                }
                ("hasMultipleSigners", "()Z") => {
                    let invocation = invocation?;
                    return Some(self.pm_signing_info_has_multiple(invocation)?);
                }
                _ => {}
            }
        }

        if method.class_descriptor == "Landroid/content/SharedPreferences;"
            && method.name == "contains"
            && method.proto.descriptor_is("(Ljava/lang/String;)Z")
        {
            let invocation = invocation?;
            let value = self.shared_preferences_contains(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/content/SharedPreferences;"
            && method.name == "getBoolean"
            && method.proto.descriptor_is("(Ljava/lang/String;Z)Z")
        {
            let invocation = invocation?;
            let default_value = self.shared_preferences_get_boolean(invocation)?;

            return Some(DexExecutionResult::Int(default_value));
        }

        if method.class_descriptor == "Landroid/content/SharedPreferences;"
            && method.name == "getString"
            && method.proto.descriptor()
                == "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;"
        {
            let invocation = invocation?;
            let default_value = self.shared_preferences_get_string(invocation)?;

            return Some(DexExecutionResult::Value(default_value));
        }

        if method.class_descriptor == "Landroid/content/SharedPreferences;"
            && method.name == "getInt"
            && method.proto.descriptor_is("(Ljava/lang/String;I)I")
        {
            let invocation = invocation?;
            let default_value = self.shared_preferences_get_int(invocation)?;

            return Some(DexExecutionResult::Int(default_value));
        }

        if method.class_descriptor == "Landroid/content/SharedPreferences;"
            && method.name == "getLong"
            && method.proto.descriptor_is("(Ljava/lang/String;J)J")
        {
            let invocation = invocation?;
            let default_value = self.shared_preferences_get_long(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(default_value)));
        }

        if method.class_descriptor == "Landroid/content/SharedPreferences;"
            && method.name == "edit"
            && method
                .proto
                .descriptor_is("()Landroid/content/SharedPreferences$Editor;")
        {
            let invocation = invocation?;
            let editor = self.shared_preferences_edit(invocation)?;

            return Some(DexExecutionResult::Value(editor));
        }

        if method.class_descriptor == "Landroid/content/SharedPreferences$Editor;"
            && method.name == "putBoolean"
            && method.proto.descriptor()
                == "(Ljava/lang/String;Z)Landroid/content/SharedPreferences$Editor;"
        {
            let invocation = invocation?;
            let editor = self.shared_preferences_editor_put_boolean(invocation)?;

            return Some(DexExecutionResult::Value(editor));
        }

        if method.class_descriptor == "Landroid/content/SharedPreferences$Editor;"
            && method.name == "putString"
            && method.proto.descriptor()
                == "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;"
        {
            let invocation = invocation?;
            let editor = self.shared_preferences_editor_put_string(invocation)?;

            return Some(DexExecutionResult::Value(editor));
        }

        if method.class_descriptor == "Landroid/content/SharedPreferences$Editor;"
            && method.name == "putInt"
            && method.proto.descriptor()
                == "(Ljava/lang/String;I)Landroid/content/SharedPreferences$Editor;"
        {
            let invocation = invocation?;
            let editor = self.shared_preferences_editor_put_int(invocation)?;

            return Some(DexExecutionResult::Value(editor));
        }

        if method.class_descriptor == "Landroid/content/SharedPreferences$Editor;"
            && method.name == "putLong"
            && method.proto.descriptor()
                == "(Ljava/lang/String;J)Landroid/content/SharedPreferences$Editor;"
        {
            let invocation = invocation?;
            let editor = self.shared_preferences_editor_put_long(invocation)?;

            return Some(DexExecutionResult::Value(editor));
        }

        // cite: frameworks-base SharedPreferences.java#putStringSet
        if method.class_descriptor == "Landroid/content/SharedPreferences$Editor;"
            && method.name == "putStringSet"
            && method.proto.descriptor()
                == "(Ljava/lang/String;Ljava/util/Set;)Landroid/content/SharedPreferences$Editor;"
        {
            let invocation = invocation?;
            return Some(DexExecutionResult::Value(
                invocation.arguments.first()?.clone(),
            ));
        }

        if method.class_descriptor == "Landroid/content/SharedPreferences$Editor;"
            && method.name == "remove"
            && method.proto.descriptor()
                == "(Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;"
        {
            let invocation = invocation?;
            let editor = self.shared_preferences_editor_remove(invocation)?;

            return Some(DexExecutionResult::Value(editor));
        }

        if method.class_descriptor == "Landroid/content/SharedPreferences$Editor;"
            && method.name == "clear"
            && method
                .proto
                .descriptor_is("()Landroid/content/SharedPreferences$Editor;")
        {
            let invocation = invocation?;
            let editor = self.shared_preferences_editor_clear(invocation)?;

            return Some(DexExecutionResult::Value(editor));
        }

        if method.class_descriptor == "Landroid/content/SharedPreferences$Editor;"
            && method.name == "apply"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.shared_preferences_editor_apply(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/content/SharedPreferences$Editor;"
            && method.name == "commit"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = self.shared_preferences_editor_commit(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        None
    }

    fn context_data_dir_file(&self, invocation: &DexInvocation) -> Option<DexValue> {
        // cite: frameworks-base ContextImpl.java#getDataDir
        let context = context_receiver(invocation)?;
        let storage = self.context_private_storage_kind(context)?;
        let path = self
            .guest_filesystem
            .as_ref()?
            .private_guest_path(storage, "");
        Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Ljava/io/File;:{path}"
        ))))
    }

    fn context_private_directory_file(
        &self,
        invocation: &DexInvocation,
        child: &str,
    ) -> Option<DexValue> {
        // cite: frameworks-base ContextImpl.java#getFilesDir, getNoBackupFilesDir, getCacheDir, getCodeCacheDir, getDataDir
        let context = context_receiver(invocation)?;
        let path = self.ensure_context_private_directory(context, child)?;
        Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Ljava/io/File;:{path}"
        ))))
    }

    // cite: frameworks-base ContextImpl.java#getExternalFilesDirs
    // cite: frameworks-base Environment.java#buildExternalStorageAppFilesDirs, buildPaths
    fn context_external_files_dirs(&self, invocation: &DexInvocation) -> Option<DexValue> {
        let _context = context_receiver(invocation)?;
        match &invocation.arguments[1] {
            DexValue::String(type_name) => {
                self.build_external_files_dirs_array(Some(type_name.as_ref()))
            }
            DexValue::Null | DexValue::Int(0) => self.build_external_files_dirs_array(None),
            _ => None,
        }
    }

    fn build_external_files_dirs_array(&self, type_name: Option<&str>) -> Option<DexValue> {
        let package_name = self.package_name.as_deref()?;
        let mut guest_path = format!(
            "{}/Android/data/{package_name}/files",
            crate::external_storage::GUEST_PRIMARY_EXTERNAL_ROOT
        );
        if let Some(type_name) = type_name {
            guest_path.push('/');
            guest_path.push_str(type_name);
        }

        if let Some(filesystem) = self.guest_filesystem.as_ref() {
            if let Ok(resolved) = filesystem.resolve(
                &guest_path,
                crate::external_storage::ExternalStorageAccess::Write,
                &self.external_storage_policy,
            ) {
                let _ = resolved.mkdir_all();
            }
        }

        Some(self.allocate_filled_array(
            "[Ljava/io/File;".to_owned(),
            vec![DexValue::Reference(std::rc::Rc::from(format!(
                "Ljava/io/File;:{guest_path}"
            )))],
        ))
    }

    fn context_database_path_file(&self, invocation: &DexInvocation) -> Option<DexExecutionResult> {
        if invocation.arguments.len() != 2 {
            return None;
        }
        let context = context_receiver(invocation)?;
        let storage = self.context_private_storage_kind(context)?;

        let DexValue::String(name) = &invocation.arguments[1] else {
            return None;
        };
        if name.is_empty() {
            return Some(DexExecutionResult::Throw(self.allocate_object(
                "Ljava/lang/StringIndexOutOfBoundsException;".to_owned(),
            )));
        }
        let filesystem = self.guest_filesystem.as_ref()?;

        // cite: frameworks-base ContextImpl.java#getDatabasePath, getDatabasesDir, makeFilename
        let path = if name.starts_with('/') {
            let path = crate::guest_filesystem::normalize_guest_file_path(name);
            self.try_create_absolute_database_parent(&path);
            path
        } else {
            if name.contains('/') {
                return Some(DexExecutionResult::Throw(
                    self.allocate_object("Ljava/lang/IllegalArgumentException;".to_owned()),
                ));
            }
            let database_dir = filesystem.private_guest_path(storage, "databases");
            if let Ok(directory) = filesystem.resolve(
                &database_dir,
                crate::external_storage::ExternalStorageAccess::Write,
                &self.external_storage_policy,
            ) {
                let _ = directory.mkdir_all();
            }
            format!("{database_dir}/{name}")
        };

        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from(format!("Ljava/io/File;:{path}")),
        )))
    }

    fn package_manager_check_permission(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 3 {
            return None;
        }
        let DexValue::String(permission) = &invocation.arguments[1] else {
            return None;
        };
        let DexValue::String(package_name) = &invocation.arguments[2] else {
            return None;
        };

        // cite: frameworks-base ApplicationPackageManager.java#checkPermission
        // cite: frameworks-base PermissionManager.java#checkPackageNamePermission
        if package_name.as_ref() != self.bootstrap_process_name() {
            return Some(ANDROID_PERMISSION_DENIED);
        }

        let granted = self
            .external_storage_policy
            .permission_granted(permission)
            .unwrap_or(true);

        Some(if granted {
            ANDROID_PERMISSION_GRANTED
        } else {
            ANDROID_PERMISSION_DENIED
        })
    }

    fn context_shared_preferences_path_file(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexExecutionResult> {
        if invocation.arguments.len() != 2 {
            return None;
        }
        let context = context_receiver(invocation)?;
        let DexValue::String(name) = &invocation.arguments[1] else {
            return None;
        };
        if name.contains('/') {
            return Some(DexExecutionResult::Throw(
                self.allocate_object("Ljava/lang/IllegalArgumentException;".to_owned()),
            ));
        }
        let directory = self.ensure_context_private_directory(context, "shared_prefs")?;
        let path = format!("{directory}/{name}.xml");

        Some(DexExecutionResult::Value(DexValue::Reference(
            std::rc::Rc::from(format!("Ljava/io/File;:{path}")),
        )))
    }

    fn ensure_context_private_directory(&self, context: &DexValue, child: &str) -> Option<String> {
        let filesystem = self.guest_filesystem.as_ref()?;
        let storage = self.context_private_storage_kind(context)?;
        let guest_path = filesystem.private_guest_path(storage, child);
        if let Ok(path) = filesystem.resolve(
            &guest_path,
            crate::external_storage::ExternalStorageAccess::Write,
            &self.external_storage_policy,
        ) {
            let _ = path.mkdir_all();
        }

        Some(guest_path)
    }

    fn try_create_absolute_database_parent(&self, guest_path: &str) {
        let Some((parent, _)) = guest_path.rsplit_once('/') else {
            return;
        };
        let parent = if parent.is_empty() { "/" } else { parent };
        let Some(filesystem) = self.guest_filesystem.as_ref() else {
            return;
        };
        let Ok(parent) = filesystem.resolve(
            parent,
            crate::external_storage::ExternalStorageAccess::Write,
            &self.external_storage_policy,
        ) else {
            return;
        };
        if !parent
            .metadata()
            .is_ok_and(|metadata| metadata.kind == crate::capability_fs::MetadataKind::Directory)
        {
            let _ = parent.mkdir();
        }
    }

    fn ensure_package_registry(&self) {
        if self.caches.borrow().package_registry.is_some() {
            return;
        }
        let loaded = crate::prefix::WinedroidPrefix::default()
            .and_then(|prefix| {
                crate::package_registry::PackageRegistry::load(&prefix, &self.config)
            })
            .unwrap_or_else(|_| {
                crate::package_registry::PackageRegistry::from_artifacts(
                    Vec::new(),
                    self.config.signature_spoofing(),
                )
            });
        self.caches.borrow_mut().package_registry = Some(loaded);
    }

    fn package_manager_get_package_info(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexExecutionResult> {
        if invocation.arguments.len() != 3 {
            return None;
        }
        let DexValue::String(package_name) = &invocation.arguments[1] else {
            return None;
        };
        let DexValue::Int(flags) = invocation.arguments[2] else {
            return None;
        };

        self.ensure_package_registry();
        let artifact = self
            .caches
            .borrow()
            .package_registry
            .as_ref()?
            .get_package(package_name.as_ref())
            .cloned();
        let Some(artifact) = artifact else {
            // cite: frameworks-base ApplicationPackageManager.java#getPackageInfo
            return Some(DexExecutionResult::Throw(
                self.allocate_generated_exception(DexGeneratedException {
                    class_descriptor: "Landroid/content/pm/PackageManager$NameNotFoundException;"
                        .to_owned(),
                    detail_message: Some(package_name.to_string()),
                }),
            ));
        };

        let package_info = self.allocate_object("Landroid/content/pm/PackageInfo;".to_owned());
        self.set_object_string_field(
            &package_info,
            "Landroid/content/pm/PackageInfo;",
            "packageName",
            &artifact.package_name,
        );
        if let Some(version_name) = &artifact.version_name {
            self.set_object_string_field(
                &package_info,
                "Landroid/content/pm/PackageInfo;",
                "versionName",
                version_name,
            );
        }
        let version_code = artifact
            .version_code
            .as_deref()
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(0);
        let _ = self.write_stored_instance_field(
            &package_info,
            &DexFieldId {
                class_descriptor: "Landroid/content/pm/PackageInfo;".to_owned(),
                type_descriptor: "I".to_owned(),
                name: "versionCode".to_owned(),
            },
            DexValue::Int(version_code),
        );

        let app_info = self.build_application_info_object(&artifact, flags);
        let _ = self.write_stored_instance_field(
            &package_info,
            &DexFieldId {
                class_descriptor: "Landroid/content/pm/PackageInfo;".to_owned(),
                type_descriptor: "Landroid/content/pm/ApplicationInfo;".to_owned(),
                name: "applicationInfo".to_owned(),
            },
            app_info,
        );

        if (flags & crate::package_registry::GET_SIGNATURES) != 0
            || (flags & crate::package_registry::GET_SIGNING_CERTIFICATES) != 0
        {
            // cite: frameworks-base PackageManager.java#GET_SIGNATURES, GET_SIGNING_CERTIFICATES
            let certs = self
                .caches
                .borrow()
                .package_registry
                .as_ref()?
                .signing_certificates(&artifact.package_name)
                .unwrap_or_default();
            let sig_objects: Vec<DexValue> = certs
                .into_iter()
                .map(|der| self.allocate_pm_signature(&der))
                .collect();
            let sig_array = self.allocate_filled_array(
                "[Landroid/content/pm/Signature;".to_owned(),
                sig_objects.clone(),
            );
            if (flags & crate::package_registry::GET_SIGNATURES) != 0 {
                let _ = self.write_stored_instance_field(
                    &package_info,
                    &DexFieldId {
                        class_descriptor: "Landroid/content/pm/PackageInfo;".to_owned(),
                        type_descriptor: "[Landroid/content/pm/Signature;".to_owned(),
                        name: "signatures".to_owned(),
                    },
                    sig_array.clone(),
                );
            }
            if (flags & crate::package_registry::GET_SIGNING_CERTIFICATES) != 0 {
                let signing_info =
                    self.allocate_object("Landroid/content/pm/SigningInfo;".to_owned());
                let _ = self.write_stored_instance_field(
                    &signing_info,
                    &DexFieldId {
                        class_descriptor: "Landroid/content/pm/SigningInfo;".to_owned(),
                        type_descriptor: "[Landroid/content/pm/Signature;".to_owned(),
                        name: "mSigningDetails".to_owned(),
                    },
                    sig_array,
                );
                let _ = self.write_stored_instance_field(
                    &package_info,
                    &DexFieldId {
                        class_descriptor: "Landroid/content/pm/PackageInfo;".to_owned(),
                        type_descriptor: "Landroid/content/pm/SigningInfo;".to_owned(),
                        name: "signingInfo".to_owned(),
                    },
                    signing_info,
                );
            }
        }

        Some(DexExecutionResult::Value(package_info))
    }

    fn package_manager_get_application_info(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexExecutionResult> {
        if invocation.arguments.len() != 3 {
            return None;
        }
        let DexValue::String(package_name) = &invocation.arguments[1] else {
            return None;
        };
        let DexValue::Int(flags) = invocation.arguments[2] else {
            return None;
        };

        self.ensure_package_registry();
        let artifact = self
            .caches
            .borrow()
            .package_registry
            .as_ref()?
            .get_package(package_name.as_ref())
            .cloned();
        let Some(artifact) = artifact else {
            // cite: frameworks-base ApplicationPackageManager.java#getApplicationInfo
            return Some(DexExecutionResult::Throw(
                self.allocate_generated_exception(DexGeneratedException {
                    class_descriptor: "Landroid/content/pm/PackageManager$NameNotFoundException;"
                        .to_owned(),
                    detail_message: Some(package_name.to_string()),
                }),
            ));
        };

        Some(DexExecutionResult::Value(
            self.build_application_info_object(&artifact, flags),
        ))
    }

    fn build_application_info_object(
        &self,
        artifact: &crate::install::InstalledPackageArtifact,
        flags: i32,
    ) -> DexValue {
        let app_info = self.allocate_object("Landroid/content/pm/ApplicationInfo;".to_owned());
        self.set_object_string_field(
            &app_info,
            "Landroid/content/pm/PackageItemInfo;",
            "packageName",
            &artifact.package_name,
        );
        self.set_object_string_field(
            &app_info,
            "Landroid/content/pm/ApplicationInfo;",
            "packageName",
            &artifact.package_name,
        );
        self.set_object_string_field(
            &app_info,
            "Landroid/content/pm/ApplicationInfo;",
            "sourceDir",
            &artifact.source_apk.display().to_string(),
        );
        self.set_object_string_field(
            &app_info,
            "Landroid/content/pm/ApplicationInfo;",
            "publicSourceDir",
            &artifact.source_apk.display().to_string(),
        );
        if (flags & crate::package_registry::GET_META_DATA) != 0 {
            let bundle = self.allocate_object("Landroid/os/Bundle;".to_owned());
            let bundle_key = map_identity_key(&bundle);
            let entries = self
                .caches
                .borrow()
                .package_registry
                .as_ref()
                .map(|registry| registry.application_meta_data(&artifact.package_name))
                .unwrap_or_default();
            for (name, value, resource) in entries {
                if let Some(text) = value {
                    self.runtime.borrow_mut().bundle_entries.insert(
                        (bundle_key.clone(), name),
                        DexValue::String(std::rc::Rc::from(text)),
                    );
                } else if let Some(res_id) = resource {
                    self.runtime
                        .borrow_mut()
                        .bundle_entries
                        .insert((bundle_key.clone(), name), DexValue::Int(res_id as i32));
                }
            }
            let _ = self.write_stored_instance_field(
                &app_info,
                &DexFieldId {
                    class_descriptor: "Landroid/content/pm/PackageItemInfo;".to_owned(),
                    type_descriptor: "Landroid/os/Bundle;".to_owned(),
                    name: "metaData".to_owned(),
                },
                bundle,
            );
        }
        app_info
    }

    fn package_manager_resolve_service(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexExecutionResult> {
        if invocation.arguments.len() != 3 {
            return None;
        }
        let intent = &invocation.arguments[1];
        let action = self
            .read_stored_instance_field(
                intent,
                &DexFieldId {
                    class_descriptor: "Landroid/content/Intent;".to_owned(),
                    type_descriptor: "Ljava/lang/String;".to_owned(),
                    name: "mAction".to_owned(),
                },
            )
            .and_then(|v| match v {
                DexValue::String(s) => Some(s.to_string()),
                _ => None,
            });
        let package = self
            .read_stored_instance_field(
                intent,
                &DexFieldId {
                    class_descriptor: "Landroid/content/Intent;".to_owned(),
                    type_descriptor: "Ljava/lang/String;".to_owned(),
                    name: "mPackage".to_owned(),
                },
            )
            .and_then(|v| match v {
                DexValue::String(s) => Some(s.to_string()),
                _ => None,
            });

        self.ensure_package_registry();
        let resolved = self
            .caches
            .borrow()
            .package_registry
            .as_ref()?
            .resolve_service(action.as_deref(), package.as_deref(), None);
        let Some((pkg_name, service_name)) = resolved else {
            return Some(DexExecutionResult::Value(DexValue::Null));
        };

        let resolve_info = self.allocate_object("Landroid/content/pm/ResolveInfo;".to_owned());
        let service_info = self.allocate_object("Landroid/content/pm/ServiceInfo;".to_owned());
        self.set_object_string_field(
            &service_info,
            "Landroid/content/pm/PackageItemInfo;",
            "packageName",
            &pkg_name,
        );
        self.set_object_string_field(
            &service_info,
            "Landroid/content/pm/PackageItemInfo;",
            "name",
            &service_name,
        );
        let _ = self.write_stored_instance_field(
            &resolve_info,
            &DexFieldId {
                class_descriptor: "Landroid/content/pm/ResolveInfo;".to_owned(),
                type_descriptor: "Landroid/content/pm/ServiceInfo;".to_owned(),
                name: "serviceInfo".to_owned(),
            },
            service_info,
        );
        Some(DexExecutionResult::Value(resolve_info))
    }

    fn package_manager_resolve_content_provider(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexExecutionResult> {
        if invocation.arguments.len() != 3 {
            return None;
        }
        let DexValue::String(authority) = &invocation.arguments[1] else {
            return None;
        };
        self.ensure_package_registry();
        let resolved = self
            .caches
            .borrow()
            .package_registry
            .as_ref()?
            .resolve_content_provider(authority.as_ref());
        let Some((pkg_name, provider_name)) = resolved else {
            return Some(DexExecutionResult::Value(DexValue::Null));
        };

        let provider_info = self.allocate_object("Landroid/content/pm/ProviderInfo;".to_owned());
        self.set_object_string_field(
            &provider_info,
            "Landroid/content/pm/PackageItemInfo;",
            "packageName",
            &pkg_name,
        );
        self.set_object_string_field(
            &provider_info,
            "Landroid/content/pm/PackageItemInfo;",
            "name",
            &provider_name,
        );
        self.set_object_string_field(
            &provider_info,
            "Landroid/content/pm/ProviderInfo;",
            "authority",
            authority.as_ref(),
        );
        Some(DexExecutionResult::Value(provider_info))
    }

    fn allocate_pm_signature(&self, der: &[u8]) -> DexValue {
        let sig = self.allocate_object("Landroid/content/pm/Signature;".to_owned());
        let bytes: Vec<DexValue> = der
            .iter()
            .map(|b| DexValue::Int((*b as i8) as i32))
            .collect();
        let byte_array = self.allocate_filled_array("[B".to_owned(), bytes);
        let _ = self.write_stored_instance_field(
            &sig,
            &DexFieldId {
                class_descriptor: "Landroid/content/pm/Signature;".to_owned(),
                type_descriptor: "[B".to_owned(),
                name: "mSignature".to_owned(),
            },
            byte_array,
        );
        sig
    }

    fn pm_signature_bytes(&self, signature: &DexValue) -> Option<Vec<u8>> {
        let array = self.read_stored_instance_field(
            signature,
            &DexFieldId {
                class_descriptor: "Landroid/content/pm/Signature;".to_owned(),
                type_descriptor: "[B".to_owned(),
                name: "mSignature".to_owned(),
            },
        )?;
        let entries = self
            .runtime
            .borrow()
            .array_entries
            .get(&map_identity_key(&array))?
            .clone();
        Some(
            entries
                .into_iter()
                .filter_map(|v| match v {
                    DexValue::Int(b) => Some(b as u8),
                    _ => None,
                })
                .collect(),
        )
    }

    fn pm_signature_to_byte_array(&self, invocation: &DexInvocation) -> Option<DexExecutionResult> {
        let receiver = invocation.arguments.first()?;
        let bytes = self.pm_signature_bytes(receiver).unwrap_or_default();
        let elements: Vec<DexValue> = bytes
            .into_iter()
            .map(|b| DexValue::Int((b as i8) as i32))
            .collect();
        Some(DexExecutionResult::Value(
            self.allocate_filled_array("[B".to_owned(), elements),
        ))
    }

    fn pm_signature_to_chars_string(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexExecutionResult> {
        let receiver = invocation.arguments.first()?;
        let bytes = self.pm_signature_bytes(receiver).unwrap_or_default();
        Some(DexExecutionResult::Value(DexValue::String(
            std::rc::Rc::from(crate::apk_signature::hex_encode(&bytes)),
        )))
    }

    fn pm_signature_hash_code(&self, invocation: &DexInvocation) -> Option<DexExecutionResult> {
        // cite: frameworks-base Signature.java#hashCode
        let receiver = invocation.arguments.first()?;
        let bytes = self.pm_signature_bytes(receiver).unwrap_or_default();
        let mut hash: i32 = 1;
        for byte in bytes {
            hash = hash.wrapping_mul(31).wrapping_add(i32::from(byte as i8));
        }
        Some(DexExecutionResult::Int(hash))
    }

    fn pm_signature_equals(&self, invocation: &DexInvocation) -> Option<DexExecutionResult> {
        // cite: frameworks-base Signature.java#equals
        if invocation.arguments.len() != 2 {
            return None;
        }
        let left = self.pm_signature_bytes(&invocation.arguments[0])?;
        let right = self.pm_signature_bytes(&invocation.arguments[1]);
        Some(DexExecutionResult::Int(i32::from(Some(left) == right)))
    }

    fn pm_signing_info_signers(&self, invocation: &DexInvocation) -> Option<DexExecutionResult> {
        let receiver = invocation.arguments.first()?;
        let array = self
            .read_stored_instance_field(
                receiver,
                &DexFieldId {
                    class_descriptor: "Landroid/content/pm/SigningInfo;".to_owned(),
                    type_descriptor: "[Landroid/content/pm/Signature;".to_owned(),
                    name: "mSigningDetails".to_owned(),
                },
            )
            .unwrap_or_else(|| {
                self.allocate_filled_array("[Landroid/content/pm/Signature;".to_owned(), Vec::new())
            });
        Some(DexExecutionResult::Value(array))
    }

    fn pm_signing_info_has_multiple(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexExecutionResult> {
        let receiver = invocation.arguments.first()?;
        let count = self
            .read_stored_instance_field(
                receiver,
                &DexFieldId {
                    class_descriptor: "Landroid/content/pm/SigningInfo;".to_owned(),
                    type_descriptor: "[Landroid/content/pm/Signature;".to_owned(),
                    name: "mSigningDetails".to_owned(),
                },
            )
            .and_then(|arr| {
                self.runtime
                    .borrow()
                    .array_entries
                    .get(&map_identity_key(&arr))
                    .map(Vec::len)
            })
            .unwrap_or(0);
        Some(DexExecutionResult::Int(i32::from(count > 1)))
    }

    fn package_manager_get_installed_packages(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexExecutionResult> {
        // cite: frameworks-base ApplicationPackageManager.java#getInstalledPackages
        if invocation.arguments.len() != 2 {
            return None;
        }
        let DexValue::Int(flags) = invocation.arguments[1] else {
            return None;
        };
        self.ensure_package_registry();
        let artifacts: Vec<crate::install::InstalledPackageArtifact> = self
            .caches
            .borrow()
            .package_registry
            .as_ref()?
            .all_packages()
            .into_iter()
            .cloned()
            .collect();
        let items: Vec<DexValue> = artifacts
            .iter()
            .filter_map(|artifact| {
                let info = self.allocate_object("Landroid/content/pm/PackageInfo;".to_owned());
                self.set_object_string_field(
                    &info,
                    "Landroid/content/pm/PackageInfo;",
                    "packageName",
                    &artifact.package_name,
                );
                if let Some(version_name) = &artifact.version_name {
                    self.set_object_string_field(
                        &info,
                        "Landroid/content/pm/PackageInfo;",
                        "versionName",
                        version_name,
                    );
                }
                let version_code = artifact
                    .version_code
                    .as_deref()
                    .and_then(|v| v.parse::<i32>().ok())
                    .unwrap_or(0);
                let _ = self.write_stored_instance_field(
                    &info,
                    &DexFieldId {
                        class_descriptor: "Landroid/content/pm/PackageInfo;".to_owned(),
                        type_descriptor: "I".to_owned(),
                        name: "versionCode".to_owned(),
                    },
                    DexValue::Int(version_code),
                );
                let app_info = self.build_application_info_object(artifact, flags);
                let _ = self.write_stored_instance_field(
                    &info,
                    &DexFieldId {
                        class_descriptor: "Landroid/content/pm/PackageInfo;".to_owned(),
                        type_descriptor: "Landroid/content/pm/ApplicationInfo;".to_owned(),
                        name: "applicationInfo".to_owned(),
                    },
                    app_info,
                );
                Some(info)
            })
            .collect();
        let list = self.allocate_object("Ljava/util/ArrayList;".to_owned());
        self.runtime
            .borrow_mut()
            .list_entries
            .insert(map_identity_key(&list), items);
        Some(DexExecutionResult::Value(list))
    }

    fn package_manager_get_installed_applications(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexExecutionResult> {
        // cite: frameworks-base ApplicationPackageManager.java#getInstalledApplications
        if invocation.arguments.len() != 2 {
            return None;
        }
        let DexValue::Int(flags) = invocation.arguments[1] else {
            return None;
        };
        self.ensure_package_registry();
        let artifacts: Vec<crate::install::InstalledPackageArtifact> = self
            .caches
            .borrow()
            .package_registry
            .as_ref()?
            .all_packages()
            .into_iter()
            .cloned()
            .collect();
        let items: Vec<DexValue> = artifacts
            .iter()
            .map(|artifact| self.build_application_info_object(artifact, flags))
            .collect();
        let list = self.allocate_object("Ljava/util/ArrayList;".to_owned());
        self.runtime
            .borrow_mut()
            .list_entries
            .insert(map_identity_key(&list), items);
        Some(DexExecutionResult::Value(list))
    }

    fn package_installer_create_session(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexExecutionResult> {
        // cite: frameworks-base PackageInstaller.java#createSession
        if invocation.arguments.len() != 2 {
            return None;
        }
        let session_id = {
            let mut runtime = self.runtime.borrow_mut();
            let id = runtime.next_object_id as i32;
            runtime.next_object_id += 1;
            id
        };
        if let Some(filesystem) = self.guest_filesystem.as_ref() {
            let session_dir = filesystem.private_guest_path(
                self.default_private_storage,
                &format!("files/package_installer_sessions/{session_id}"),
            );
            if let Ok(resolved) = filesystem.resolve(
                &session_dir,
                crate::external_storage::ExternalStorageAccess::Write,
                &self.external_storage_policy,
            ) {
                let _ = resolved.mkdir_all();
            }
        }
        Some(DexExecutionResult::Int(session_id))
    }

    fn package_installer_open_session(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexExecutionResult> {
        // cite: frameworks-base PackageInstaller.java#openSession
        if invocation.arguments.len() != 2 {
            return None;
        }
        let DexValue::Int(session_id) = invocation.arguments[1] else {
            return None;
        };
        let session =
            self.allocate_object("Landroid/content/pm/PackageInstaller$Session;".to_owned());
        let _ = self.write_stored_instance_field(
            &session,
            &DexFieldId {
                class_descriptor: "Landroid/content/pm/PackageInstaller$Session;".to_owned(),
                type_descriptor: "I".to_owned(),
                name: "mSessionId".to_owned(),
            },
            DexValue::Int(session_id),
        );
        Some(DexExecutionResult::Value(session))
    }

    fn package_installer_session_open_write(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexExecutionResult> {
        // cite: frameworks-base PackageInstaller.java#openWrite
        if invocation.arguments.len() != 4 {
            return None;
        }
        let session = &invocation.arguments[0];
        let DexValue::String(name) = &invocation.arguments[1] else {
            return None;
        };
        let Some(DexValue::Int(session_id)) = self.read_stored_instance_field(
            session,
            &DexFieldId {
                class_descriptor: "Landroid/content/pm/PackageInstaller$Session;".to_owned(),
                type_descriptor: "I".to_owned(),
                name: "mSessionId".to_owned(),
            },
        ) else {
            return None;
        };
        let sanitized = name.replace('/', "_").replace('\\', "_");
        let file_name = if sanitized.ends_with(".apk") {
            sanitized
        } else {
            format!("{sanitized}.apk")
        };
        let filesystem = self.guest_filesystem.as_ref()?;
        let session_dir = filesystem.private_guest_path(
            self.default_private_storage,
            &format!("files/package_installer_sessions/{session_id}"),
        );
        if let Ok(resolved_dir) = filesystem.resolve(
            &session_dir,
            crate::external_storage::ExternalStorageAccess::Write,
            &self.external_storage_policy,
        ) {
            let _ = resolved_dir.mkdir_all();
        }
        let guest_path = format!("{session_dir}/{file_name}");
        let resolved = match filesystem.resolve(
            &guest_path,
            crate::external_storage::ExternalStorageAccess::Write,
            &self.external_storage_policy,
        ) {
            Ok(path) => path,
            Err(detail) => {
                return Some(DexExecutionResult::Throw(
                    self.allocate_generated_exception(DexGeneratedException {
                        class_descriptor: "Ljava/io/IOException;".to_owned(),
                        detail_message: Some(detail.to_string()),
                    }),
                ));
            }
        };
        if let Err(error) = resolved.open_truncate() {
            return Some(DexExecutionResult::Throw(
                self.allocate_generated_exception(DexGeneratedException {
                    class_descriptor: "Ljava/io/IOException;".to_owned(),
                    detail_message: Some(error.to_string()),
                }),
            ));
        }
        let stream = self.allocate_object("Ljava/io/FileOutputStream;".to_owned());
        let _ = self.write_stored_instance_field(
            &stream,
            &DexFieldId {
                class_descriptor: "Ljava/io/FileOutputStream;".to_owned(),
                type_descriptor: "Ljava/lang/String;".to_owned(),
                name: "path".to_owned(),
            },
            DexValue::String(std::rc::Rc::from(guest_path)),
        );
        Some(DexExecutionResult::Value(stream))
    }

    fn package_installer_session_commit(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexExecutionResult> {
        // cite: frameworks-base PackageInstaller.java#commit, STATUS_SUCCESS, STATUS_FAILURE
        if invocation.arguments.len() != 2 {
            return None;
        }
        let session = &invocation.arguments[0];
        let Some(DexValue::Int(session_id)) = self.read_stored_instance_field(
            session,
            &DexFieldId {
                class_descriptor: "Landroid/content/pm/PackageInstaller$Session;".to_owned(),
                type_descriptor: "I".to_owned(),
                name: "mSessionId".to_owned(),
            },
        ) else {
            return None;
        };

        let filesystem = self.guest_filesystem.as_ref()?;
        let session_guest_dir = filesystem.private_guest_path(
            self.default_private_storage,
            &format!("files/package_installer_sessions/{session_id}"),
        );
        let resolved_session_dir = filesystem
            .resolve(
                &session_guest_dir,
                crate::external_storage::ExternalStorageAccess::Read,
                &self.external_storage_policy,
            )
            .ok()?;
        let mut entry_names = resolved_session_dir.read_dir_names().unwrap_or_default();
        entry_names.retain(|name| name.ends_with(".apk"));
        entry_names.sort();
        if entry_names.is_empty() {
            return Some(DexExecutionResult::Throw(
                self.allocate_generated_exception(DexGeneratedException {
                    class_descriptor: "Ljava/lang/IllegalStateException;".to_owned(),
                    detail_message: Some("no APK written to PackageInstaller session".to_owned()),
                }),
            ));
        }

        let staging_dir = std::env::temp_dir().join(format!(
            "winedroid-pkg-session-{}-{}",
            std::process::id(),
            session_id
        ));
        if let Err(error) = std::fs::create_dir_all(&staging_dir) {
            return Some(DexExecutionResult::Throw(
                self.allocate_generated_exception(DexGeneratedException {
                    class_descriptor: "Ljava/lang/IllegalStateException;".to_owned(),
                    detail_message: Some(error.to_string()),
                }),
            ));
        }

        let mut apk_paths = Vec::with_capacity(entry_names.len());
        for name in &entry_names {
            let guest_apk = format!("{session_guest_dir}/{name}");
            let bytes = match filesystem
                .resolve(
                    &guest_apk,
                    crate::external_storage::ExternalStorageAccess::Read,
                    &self.external_storage_policy,
                )
                .map_err(|e| e.to_string())
                .and_then(|p| p.read().map_err(|e| e.to_string()))
            {
                Ok(bytes) => bytes,
                Err(error) => {
                    let _ = std::fs::remove_dir_all(&staging_dir);
                    return Some(DexExecutionResult::Throw(
                        self.allocate_generated_exception(DexGeneratedException {
                            class_descriptor: "Ljava/lang/IllegalStateException;".to_owned(),
                            detail_message: Some(error.to_string()),
                        }),
                    ));
                }
            };
            let staged_path = staging_dir.join(name);
            if let Err(error) = std::fs::write(&staged_path, bytes) {
                let _ = std::fs::remove_dir_all(&staging_dir);
                return Some(DexExecutionResult::Throw(
                    self.allocate_generated_exception(DexGeneratedException {
                        class_descriptor: "Ljava/lang/IllegalStateException;".to_owned(),
                        detail_message: Some(error.to_string()),
                    }),
                ));
            }
            apk_paths.push(staged_path);
        }

        let base_idx = apk_paths
            .iter()
            .position(|p| p.file_name().and_then(|n| n.to_str()) == Some("base.apk"))
            .unwrap_or(0);
        let base_apk = apk_paths.remove(base_idx);
        let split_apks = apk_paths;

        let prefix = match crate::prefix::WinedroidPrefix::default() {
            Ok(prefix) => prefix,
            Err(error) => {
                let _ = std::fs::remove_dir_all(&staging_dir);
                return Some(DexExecutionResult::Throw(
                    self.allocate_generated_exception(DexGeneratedException {
                        class_descriptor: "Ljava/lang/IllegalStateException;".to_owned(),
                        detail_message: Some(error.to_string()),
                    }),
                ));
            }
        };
        let report = match crate::inspect::inspect_apk(&base_apk) {
            Ok(report) => report,
            Err(error) => {
                let _ = std::fs::remove_dir_all(&staging_dir);
                return Some(DexExecutionResult::Throw(
                    self.allocate_generated_exception(DexGeneratedException {
                        class_descriptor: "Ljava/lang/IllegalArgumentException;".to_owned(),
                        detail_message: Some(error.to_string()),
                    }),
                ));
            }
        };
        if let Err(error) =
            crate::install::install_apk_set(&prefix, &base_apk, &split_apks, &report)
        {
            let _ = std::fs::remove_dir_all(&staging_dir);
            return Some(DexExecutionResult::Throw(
                self.allocate_generated_exception(DexGeneratedException {
                    class_descriptor: "Ljava/lang/IllegalStateException;".to_owned(),
                    detail_message: Some(error.to_string()),
                }),
            ));
        }
        self.caches.borrow_mut().package_registry = None;
        let _ = std::fs::remove_dir_all(&staging_dir);

        Some(DexExecutionResult::Void)
    }

    fn set_object_string_field(
        &self,
        object: &DexValue,
        class_descriptor: &str,
        field_name: &str,
        value: &str,
    ) {
        let _ = self.write_stored_instance_field(
            object,
            &DexFieldId {
                class_descriptor: class_descriptor.to_owned(),
                type_descriptor: "Ljava/lang/String;".to_owned(),
                name: field_name.to_owned(),
            },
            DexValue::String(std::rc::Rc::from(value)),
        );
    }
}

fn context_receiver(invocation: &DexInvocation) -> Option<&DexValue> {
    if invocation.arguments.len() != 1 && invocation.arguments.len() != 2 {
        return None;
    }
    let context = invocation.arguments.first()?;
    dex_value_is_object_like(context).then_some(context)
}
