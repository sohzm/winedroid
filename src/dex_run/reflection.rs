use super::*;

impl DexProgram {
    pub(super) fn class_for_name(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<DexExecutionResult> {
        if invocation.arguments.len() != 1 {
            return Err(WinedroidError::new(format!(
                "Class.forName expected 1 argument, got {}",
                invocation.arguments.len()
            )));
        }

        // cite: art java_lang_Class.cc#Class_classForName
        if matches!(invocation.arguments[0], DexValue::Null | DexValue::Int(0)) {
            return Ok(DexExecutionResult::Throw(
                self.allocate_generated_exception(DexGeneratedException {
                    class_descriptor: "Ljava/lang/NullPointerException;".to_owned(),
                    detail_message: None,
                }),
            ));
        }

        let DexValue::String(class_name) = &invocation.arguments[0] else {
            return Err(WinedroidError::new("Class.forName name is not a String"));
        };

        self.class_for_name_result(class_name, true, false, depth)
    }

    pub(super) fn class_for_name_with_loader(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<DexExecutionResult> {
        if invocation.arguments.len() != 3 {
            return Err(WinedroidError::new(format!(
                "Class.forName with loader expected 3 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        // cite: art java_lang_Class.cc#Class_classForName
        if matches!(invocation.arguments[0], DexValue::Null | DexValue::Int(0)) {
            return Ok(DexExecutionResult::Throw(
                self.allocate_generated_exception(DexGeneratedException {
                    class_descriptor: "Ljava/lang/NullPointerException;".to_owned(),
                    detail_message: None,
                }),
            ));
        }

        let DexValue::String(class_name) = &invocation.arguments[0] else {
            return Err(WinedroidError::new("Class.forName name is not a String"));
        };
        let DexValue::Int(initialize) = invocation.arguments[1] else {
            return Err(WinedroidError::new(
                "Class.forName initialize argument is not boolean-compatible",
            ));
        };
        let bootstrap_only = match &invocation.arguments[2] {
            DexValue::Null | DexValue::Int(0) => true,
            DexValue::Reference(loader)
                if loader.as_ref() == "Ljava/lang/ClassLoader;.bootstrap" =>
            {
                true
            }
            DexValue::Reference(loader) if loader.as_ref() == "Ljava/lang/ClassLoader;.app" => {
                false
            }
            DexValue::Object(_) | DexValue::Reference(_) => {
                return Err(WinedroidError::new(
                    "Class.forName custom ClassLoader dispatch is not implemented yet\nnext missing primitive: Class.forName custom ClassLoader dispatch",
                ));
            }
            DexValue::Uninitialized
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => {
                return Err(WinedroidError::new(
                    "Class.forName loader argument is not a ClassLoader",
                ));
            }
        };

        self.class_for_name_result(class_name, initialize != 0, bootstrap_only, depth)
    }

    fn class_for_name_result(
        &self,
        class_name: &str,
        initialize: bool,
        bootstrap_only: bool,
        depth: usize,
    ) -> Result<DexExecutionResult> {
        // cite: art java_lang_Class.cc#Class_classForName
        // cite: art descriptors_names.cc#IsValidBinaryClassName, DotToDescriptor
        let Some(descriptor) = class_for_name_descriptor(class_name) else {
            return Ok(DexExecutionResult::Throw(
                self.allocate_generated_exception(DexGeneratedException {
                    class_descriptor: "Ljava/lang/ClassNotFoundException;".to_owned(),
                    detail_message: Some(format!("Invalid name: {class_name}")),
                }),
            ));
        };
        let Some(class_reference) =
            self.class_reference_for_descriptor(&descriptor, bootstrap_only)
        else {
            return Ok(DexExecutionResult::Throw(
                self.allocate_generated_exception(DexGeneratedException {
                    class_descriptor: "Ljava/lang/ClassNotFoundException;".to_owned(),
                    detail_message: Some(class_name.to_string()),
                }),
            ));
        };

        if initialize {
            let descriptor =
                class_reference_descriptor_from_value(&class_reference).ok_or_else(|| {
                    WinedroidError::new("Class.forName produced an invalid Class reference")
                })?;
            match self.ensure_class_initialized(descriptor, depth + 1)? {
                ClassInitializationResult::Complete => {}
                ClassInitializationResult::Throw(exception) => {
                    return Ok(DexExecutionResult::Throw(exception));
                }
                ClassInitializationResult::Parked(mut execution) => {
                    execution.replace_void_result_after_current_steps(class_reference);
                    return Ok(DexExecutionResult::Parked(execution));
                }
            }
        }

        Ok(DexExecutionResult::Value(class_reference))
    }

    pub(super) fn class_loader_load_class(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexExecutionResult> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let bootstrap_only = match &invocation.arguments[0] {
            DexValue::Reference(loader)
                if loader.as_ref() == "Ljava/lang/ClassLoader;.bootstrap" =>
            {
                true
            }
            DexValue::Reference(loader) if loader.as_ref() == "Ljava/lang/ClassLoader;.app" => {
                false
            }
            DexValue::Object(_) | DexValue::Reference(_) => return None,
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        };
        let DexValue::String(class_name) = &invocation.arguments[1] else {
            if matches!(invocation.arguments[1], DexValue::Null | DexValue::Int(0)) {
                return Some(DexExecutionResult::Throw(
                    self.allocate_generated_exception(DexGeneratedException {
                        class_descriptor: "Ljava/lang/NullPointerException;".to_owned(),
                        detail_message: None,
                    }),
                ));
            }
            return None;
        };

        // cite: libcore ClassLoader.java#loadClass
        let class_reference = class_for_name_descriptor(class_name)
            .filter(|_| !class_name.starts_with('['))
            .and_then(|descriptor| {
                self.class_reference_for_descriptor(&descriptor, bootstrap_only)
            });
        if let Some(class_reference) = class_reference {
            return Some(DexExecutionResult::Value(class_reference));
        }

        Some(DexExecutionResult::Throw(
            self.allocate_generated_exception(DexGeneratedException {
                class_descriptor: "Ljava/lang/ClassNotFoundException;".to_owned(),
                detail_message: Some(class_name.to_string()),
            }),
        ))
    }

    pub(super) fn class_get_class_loader(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let descriptor = class_reference_descriptor_from_value(&invocation.arguments[0])?;

        // cite: libcore Class.java#getClassLoader
        if is_primitive_descriptor(descriptor) {
            return Some(DexValue::Null);
        }

        let bootstrap_loader = self.class_descriptor_uses_bootstrap_loader(descriptor)?;

        Some(DexValue::Reference(std::rc::Rc::from(
            if bootstrap_loader {
                "Ljava/lang/ClassLoader;.bootstrap"
            } else {
                "Ljava/lang/ClassLoader;.app"
            },
        )))
    }

    fn class_reference_for_descriptor(
        &self,
        descriptor: &str,
        bootstrap_only: bool,
    ) -> Option<DexValue> {
        if !self.class_descriptor_available_to_loader(descriptor, bootstrap_only) {
            return None;
        }

        Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Ljava/lang/Class;:{descriptor}"
        ))))
    }

    fn class_descriptor_available_to_loader(&self, descriptor: &str, bootstrap_only: bool) -> bool {
        if let Some(component) = descriptor.strip_prefix('[') {
            if is_primitive_descriptor(component) {
                return component != "V";
            }
            return self.class_descriptor_available_to_loader(component, bootstrap_only);
        }

        is_bootstrap_class_descriptor(descriptor)
            || (!bootstrap_only && self.load_class(descriptor).is_some())
    }

    fn class_descriptor_uses_bootstrap_loader(&self, descriptor: &str) -> Option<bool> {
        if let Some(component) = descriptor.strip_prefix('[') {
            if is_primitive_descriptor(component) {
                return Some(true);
            }
            return self.class_descriptor_uses_bootstrap_loader(component);
        }

        if self.load_class(descriptor).is_some() {
            return Some(false);
        }
        is_bootstrap_class_descriptor(descriptor).then_some(true)
    }

    // cite: art class.cc#Class::IsInSamePackage
    fn class_descriptors_share_runtime_package(&self, left: &str, right: &str) -> bool {
        let Some(left_bootstrap_loader) = self.class_descriptor_uses_bootstrap_loader(left) else {
            return false;
        };
        let Some(right_bootstrap_loader) = self.class_descriptor_uses_bootstrap_loader(right)
        else {
            return false;
        };
        if left_bootstrap_loader != right_bootstrap_loader {
            return false;
        }

        let left_package = left
            .rsplit_once('/')
            .map(|(package, _)| package)
            .unwrap_or("");
        let right_package = right
            .rsplit_once('/')
            .map(|(package, _)| package)
            .unwrap_or("");
        left_package == right_package
    }

    // cite: art reflection.cc#VerifyAccess
    fn class_new_instance_constructor_accessible(
        &self,
        declaring_class: &str,
        access_flags: u32,
        caller_class: &str,
    ) -> bool {
        if access_flags & JAVA_ACC_PUBLIC != 0 || declaring_class == caller_class {
            return true;
        }
        if access_flags & JAVA_ACC_PRIVATE != 0 {
            return false;
        }

        self.class_descriptors_share_runtime_package(declaring_class, caller_class)
    }

    pub(super) fn class_new_instance(
        &self,
        invocation: &DexInvocation,
        caller_descriptor: Option<&str>,
        depth: usize,
    ) -> Result<DexExecutionResult> {
        if invocation.arguments.len() != 1 {
            return Err(WinedroidError::new(format!(
                "Class.newInstance expected 1 argument, got {}",
                invocation.arguments.len()
            )));
        }

        // cite: art entrypoint_utils-inl.h#FindMethodToCall
        if matches!(invocation.arguments[0], DexValue::Null | DexValue::Int(0)) {
            return Ok(DexExecutionResult::Throw(
                self.allocate_generated_exception(DexGeneratedException {
                    class_descriptor: "Ljava/lang/NullPointerException;".to_owned(),
                    detail_message: None,
                }),
            ));
        }

        let descriptor = class_reference_descriptor_from_value(&invocation.arguments[0])
            .ok_or_else(|| WinedroidError::new("Class.newInstance receiver is not a Class"))?;
        let pretty_class = class_new_instance_pretty_class(descriptor)?;
        let loaded_class = self.load_class(descriptor);

        // cite: art java_lang_Class.cc#Class_newInstance
        if is_primitive_descriptor(descriptor) || descriptor.starts_with('[') {
            return Ok(self.class_new_instance_failure(
                "Ljava/lang/InstantiationException;",
                format!("{pretty_class} cannot be instantiated"),
            ));
        }

        if loaded_class.is_none() {
            return match bootstrap_new_instance_kind(descriptor) {
                Some(BootstrapNewInstanceKind::Abstract) => {
                    Ok(self.class_new_instance_failure(
                        "Ljava/lang/InstantiationException;",
                        format!("{pretty_class} cannot be instantiated"),
                    ))
                }
                Some(BootstrapNewInstanceKind::MissingZeroArgumentConstructor) => {
                    Ok(self.class_new_instance_failure(
                        "Ljava/lang/InstantiationException;",
                        format!("{pretty_class} has no zero argument constructor"),
                    ))
                }
                Some(BootstrapNewInstanceKind::PublicZeroArgumentConstructor) => {
                    if descriptor == "Ljava/lang/String;" {
                        // cite: art java_lang_Class.cc#Class_newInstance
                        Ok(DexExecutionResult::Value(DexValue::String(
                            std::rc::Rc::from(""),
                        )))
                    } else {
                        self.class_new_instance_bootstrap(descriptor, depth)
                    }
                }
                None => Err(WinedroidError::new(format!(
                    "Class.newInstance bootstrap class metadata is not implemented yet\nclass: {descriptor}\nnext missing primitive: Class.newInstance class metadata {descriptor}"
                ))),
            };
        }

        let access_flags = loaded_class
            .as_ref()
            .map(|loaded| loaded.class_def.access_flags)
            .ok_or_else(|| WinedroidError::new("Class.newInstance lost loaded class metadata"))?;

        // cite: art java_lang_Class.cc#Class_newInstance
        if access_flags & (JAVA_ACC_INTERFACE | JAVA_ACC_ABSTRACT) != 0 {
            return Ok(self.class_new_instance_failure(
                "Ljava/lang/InstantiationException;",
                format!("{pretty_class} cannot be instantiated"),
            ));
        }

        // cite: art class-inl.h#Class::CanAccess
        // cite: art java_lang_Class.cc#Class_newInstance
        if access_flags & JAVA_ACC_PUBLIC == 0 {
            let caller_descriptor = caller_descriptor.ok_or_else(|| {
                WinedroidError::new(format!(
                    "Class.newInstance cannot check access without the Java caller\nclass: {descriptor}"
                ))
            })?;
            if !self.class_descriptors_share_runtime_package(descriptor, caller_descriptor) {
                return Ok(self.class_new_instance_failure(
                    "Ljava/lang/IllegalAccessException;",
                    format!(
                        "{pretty_class} is not accessible from {}",
                        class_new_instance_pretty_class(caller_descriptor)?
                    ),
                ));
            }
        }

        let constructor_method = DexMethodId {
            class_descriptor: descriptor.to_owned(),
            name: "<init>".to_owned(),
            proto: crate::dex::DexProto {
                shorty: "V".to_owned(),
                return_type: "V".to_owned(),
                parameters: Vec::new(),
            },
        };

        // cite: art class.cc#Class::GetDeclaredConstructor
        // cite: art java_lang_Class.cc#Class_newInstance
        let Some(constructor) = self.resolve_exact_method(&constructor_method)? else {
            return Ok(self.class_new_instance_failure(
                "Ljava/lang/InstantiationException;",
                format!("{pretty_class} has no zero argument constructor"),
            ));
        };

        let receiver = self.allocate_object(descriptor.to_owned());

        // cite: art reflection.cc#VerifyAccess
        // cite: art java_lang_Class.cc#Class_newInstance
        if constructor.encoded_method.access_flags & JAVA_ACC_PUBLIC == 0 {
            let caller_descriptor = caller_descriptor.ok_or_else(|| {
                WinedroidError::new(format!(
                    "Class.newInstance cannot check constructor access without the Java caller\nclass: {descriptor}"
                ))
            })?;
            if !self.class_new_instance_constructor_accessible(
                descriptor,
                constructor.encoded_method.access_flags,
                caller_descriptor,
            ) {
                return Ok(self.class_new_instance_failure(
                    "Ljava/lang/IllegalAccessException;",
                    format!(
                        "void {}.<init>() is not accessible from {}",
                        descriptor_to_pretty_name(descriptor).ok_or_else(|| {
                            WinedroidError::new(format!(
                                "Class.newInstance target descriptor is invalid\ndescriptor: {descriptor}"
                            ))
                        })?,
                        class_new_instance_pretty_class(caller_descriptor)?
                    ),
                ));
            }
        }

        // cite: art java_lang_Class.cc#Class_newInstance
        match self.ensure_class_initialized(descriptor, depth + 1)? {
            ClassInitializationResult::Complete => {}
            ClassInitializationResult::Throw(exception) => {
                return Ok(DexExecutionResult::Throw(exception));
            }
            ClassInitializationResult::Parked(mut execution) => {
                self.append_parked_runtime_action(
                    &mut execution,
                    ParkedRuntimeAction::InvokeExactVoid {
                        method: constructor_method,
                        arguments: vec![receiver.clone()],
                    },
                );
                execution.replace_void_result_after_current_steps(receiver);
                return Ok(DexExecutionResult::Parked(execution));
            }
        }
        let result = self.execute_method(
            None,
            &constructor,
            Some(std::slice::from_ref(&receiver)),
            depth + 1,
        )?;

        // cite: libcore Class.java#newInstance
        match result {
            DexExecutionResult::Void => Ok(DexExecutionResult::Value(receiver)),
            DexExecutionResult::Throw(exception) => Ok(DexExecutionResult::Throw(exception)),
            DexExecutionResult::Parked(mut execution) => {
                execution.replace_void_result_after_current_steps(receiver);
                Ok(DexExecutionResult::Parked(execution))
            }
            DexExecutionResult::Int(_) | DexExecutionResult::Value(_) => Err(WinedroidError::new(
                "Class.newInstance constructor returned a value",
            )),
        }
    }

    fn class_new_instance_bootstrap(
        &self,
        descriptor: &str,
        depth: usize,
    ) -> Result<DexExecutionResult> {
        let constructor = DexMethodId {
            class_descriptor: descriptor.to_owned(),
            name: "<init>".to_owned(),
            proto: crate::dex::DexProto {
                shorty: "V".to_owned(),
                return_type: "V".to_owned(),
                parameters: Vec::new(),
            },
        };
        // cite: art java_lang_Class.cc#Class_newInstance
        let receiver = self.allocate_object(descriptor.to_owned());
        let invocation = DexInvocation {
            kind: DexInvokeKind::Direct,
            range: false,
            method_index: 0,
            argument_registers: std::iter::once(0).collect(),
            arguments: std::iter::once(receiver.clone()).collect(),
        };
        match self.ensure_class_initialized(descriptor, depth + 1)? {
            ClassInitializationResult::Complete => {}
            ClassInitializationResult::Throw(exception) => {
                return Ok(DexExecutionResult::Throw(exception));
            }
            ClassInitializationResult::Parked(mut execution) => {
                self.append_parked_runtime_action(
                    &mut execution,
                    ParkedRuntimeAction::InvokeBuiltinVoid {
                        method: constructor,
                        invocation,
                    },
                );
                execution.replace_void_result_after_current_steps(receiver);
                return Ok(DexExecutionResult::Parked(execution));
            }
        }
        match self.invoke_builtin_method(&constructor, Some(&invocation)) {
            Some(DexExecutionResult::Void) => Ok(DexExecutionResult::Value(receiver)),
            None => Err(WinedroidError::new(format!(
                "Class.newInstance bootstrap constructor metadata is not implemented yet\nclass: {descriptor}\nnext missing primitive: Class.newInstance constructor metadata {descriptor}"
            ))),
            Some(DexExecutionResult::Throw(exception)) => Ok(DexExecutionResult::Throw(exception)),
            Some(DexExecutionResult::Parked(mut execution)) => {
                execution.replace_void_result_after_current_steps(receiver);
                Ok(DexExecutionResult::Parked(execution))
            }
            Some(DexExecutionResult::Int(_) | DexExecutionResult::Value(_)) => Err(
                WinedroidError::new("Class.newInstance bootstrap constructor returned a value"),
            ),
        }
    }

    fn class_new_instance_failure(
        &self,
        class_descriptor: &str,
        detail_message: String,
    ) -> DexExecutionResult {
        DexExecutionResult::Throw(self.allocate_generated_exception(DexGeneratedException {
            class_descriptor: class_descriptor.to_owned(),
            detail_message: Some(detail_message),
        }))
    }

    pub(super) fn reflect_array_new_instance(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let component_descriptor = class_reference_descriptor_from_value(&invocation.arguments[0])?;
        if component_descriptor == "V" {
            return None;
        }

        let DexValue::Int(length) = invocation.arguments[1] else {
            return None;
        };

        self.allocate_array(format!("[{component_descriptor}"), length)
    }

    pub(super) fn class_get_superclass(&self, invocation: &DexInvocation) -> Option<DexValue> {
        let descriptor = class_reference_descriptor(invocation)?;

        if descriptor == "Ljava/lang/Object;" {
            return Some(DexValue::Null);
        }

        let Some(superclass) = self.superclass_descriptor(descriptor) else {
            return Some(DexValue::Null);
        };

        Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Ljava/lang/Class;:{superclass}"
        ))))
    }

    pub(super) fn class_get_generic_superclass(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        self.class_get_superclass(invocation)
    }

    pub(super) fn class_get_interfaces(&self, invocation: &DexInvocation) -> Option<DexValue> {
        let descriptor = class_reference_descriptor(invocation)?;

        let interfaces = self
            .class_interface_descriptors(descriptor)
            .into_iter()
            .map(|interface_descriptor| {
                DexValue::Reference(std::rc::Rc::from(format!(
                    "Ljava/lang/Class;:{interface_descriptor}"
                )))
            })
            .collect();

        Some(self.allocate_filled_array("[Ljava/lang/Class;".to_owned(), interfaces))
    }

    pub(super) fn class_get_generic_interfaces(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        let descriptor = class_reference_descriptor(invocation)?;

        let interfaces = self
            .class_interface_descriptors(descriptor)
            .into_iter()
            .map(|interface_descriptor| {
                DexValue::Reference(std::rc::Rc::from(format!(
                    "Ljava/lang/Class;:{interface_descriptor}"
                )))
            })
            .collect();

        Some(self.allocate_filled_array("[Ljava/lang/reflect/Type;".to_owned(), interfaces))
    }

    pub(super) fn class_get_declared_fields(&self, invocation: &DexInvocation) -> Option<DexValue> {
        let descriptor = class_reference_descriptor(invocation)?;

        if let Some(loaded_class) = self.load_class(descriptor) {
            let mut fields = Vec::new();

            for encoded_field in loaded_class
                .class_def
                .static_fields
                .iter()
                .chain(loaded_class.class_def.instance_fields.iter())
            {
                let field = self
                    .field_id(loaded_class.dex_file_index, encoded_field.field_index)
                    .ok()?;
                fields.push(reflect_dex_field_value(&field, encoded_field.access_flags));
            }

            return Some(
                self.allocate_filled_array("[Ljava/lang/reflect/Field;".to_owned(), fields),
            );
        }

        // cite: libcore Unsafe.java#theUnsafe
        if descriptor == "Lsun/misc/Unsafe;" {
            let field = DexFieldId {
                class_descriptor: "Lsun/misc/Unsafe;".to_owned(),
                type_descriptor: "Lsun/misc/Unsafe;".to_owned(),
                name: "theUnsafe".to_owned(),
            };
            return Some(self.allocate_filled_array(
                "[Ljava/lang/reflect/Field;".to_owned(),
                vec![reflect_dex_field_value(&field, DEX_ACC_STATIC)],
            ));
        }

        Some(self.allocate_filled_array("[Ljava/lang/reflect/Field;".to_owned(), Vec::new()))
    }

    pub(super) fn class_get_declared_field(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if invocation.arguments[0] == DexValue::Null {
            return Some(DexValue::Null);
        }

        let descriptor = class_reference_descriptor_from_value(&invocation.arguments[0])?;
        let DexValue::String(field_name) = &invocation.arguments[1] else {
            return None;
        };

        if let Some(loaded_class) = self.load_class(descriptor) {
            for encoded_field in loaded_class
                .class_def
                .static_fields
                .iter()
                .chain(loaded_class.class_def.instance_fields.iter())
            {
                let field = self
                    .field_id(loaded_class.dex_file_index, encoded_field.field_index)
                    .ok()?;
                if field.name == **field_name {
                    return Some(reflect_dex_field_value(&field, encoded_field.access_flags));
                }
            }
        }

        let field = reflect_bootstrap_field(descriptor, field_name)?;

        Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Ljava/lang/reflect/Field;:{}:{}",
            field.class_descriptor, field.name
        ))))
    }

    pub(super) fn class_interface_descriptors(&self, descriptor: &str) -> Vec<String> {
        if let Some(cached) = self
            .caches
            .borrow()
            .interface_descriptors
            .get(descriptor)
            .cloned()
        {
            return cached;
        }

        let interfaces = if let Some(loaded_class) = self.load_class(descriptor) {
            loaded_class.class_def.interface_descriptors.clone()
        } else {
            bootstrap_direct_interface_descriptors(descriptor)
                .iter()
                .map(|interface_descriptor| (*interface_descriptor).to_owned())
                .collect()
        };

        self.caches
            .borrow_mut()
            .interface_descriptors
            .insert(descriptor.to_owned(), interfaces.clone());

        interfaces
    }

    pub(super) fn class_is_assignable_from(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let target = class_reference_descriptor_from_value(&invocation.arguments[0])?;
        if invocation.arguments[1] == DexValue::Null {
            return Some(0);
        }

        let candidate = class_reference_descriptor_from_value(&invocation.arguments[1])?;
        Some(i32::from(self.descriptor_matches_type(candidate, target)))
    }

    pub(super) fn class_get_modifiers(&self, invocation: &DexInvocation) -> Option<i32> {
        let descriptor = class_reference_descriptor(invocation)?;

        let mut access_flags = self
            .load_class(descriptor)
            .map(|loaded_class| loaded_class.class_def.access_flags)
            .unwrap_or_else(|| bootstrap_class_modifiers(descriptor));
        if descriptor.contains('$') {
            access_flags |= DEX_ACC_STATIC;
        }

        i32::try_from(access_flags).ok()
    }

    // cite: libcore Class.java#getAnnotation
    pub(super) fn class_get_annotation(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let descriptor = class_reference_descriptor_from_value(&invocation.arguments[0])?;
        let annotation_descriptor =
            class_reference_descriptor_from_value(&invocation.arguments[1])?;

        if let Some(annotation) =
            self.declared_runtime_annotation(descriptor, annotation_descriptor)
        {
            return self.annotation_instance(&annotation);
        }

        if !self.annotation_type_is_inherited(annotation_descriptor) {
            return Some(DexValue::Null);
        }

        let mut superclass = self
            .load_class(descriptor)
            .and_then(|loaded_class| loaded_class.class_def.superclass_descriptor.clone());
        while let Some(descriptor) = superclass {
            if let Some(annotation) =
                self.declared_runtime_annotation(&descriptor, annotation_descriptor)
            {
                return self.annotation_instance(&annotation);
            }
            superclass = self
                .load_class(&descriptor)
                .and_then(|loaded_class| loaded_class.class_def.superclass_descriptor.clone());
        }

        Some(DexValue::Null)
    }

    // cite: art dex_file_annotations.cc#GetAnnotationForClass
    fn declared_runtime_annotation(
        &self,
        descriptor: &str,
        annotation_descriptor: &str,
    ) -> Option<DexAnnotation> {
        let loaded_class = self.load_class(descriptor)?;
        let annotations = self.dex_files[loaded_class.dex_file_index]
            .class_annotations(&loaded_class.class_def)
            .ok()?;

        annotations.into_iter().find(|annotation| {
            annotation.visibility == DEX_VISIBILITY_RUNTIME
                && annotation.type_descriptor == annotation_descriptor
        })
    }

    // cite: libcore Class.java#getAnnotation
    fn annotation_type_is_inherited(&self, annotation_descriptor: &str) -> bool {
        self.declared_runtime_annotation(annotation_descriptor, "Ljava/lang/annotation/Inherited;")
            .is_some()
    }

    fn annotation_instance(&self, annotation: &DexAnnotation) -> Option<DexValue> {
        let object = self.allocate_object(annotation.type_descriptor.clone());
        let object_key = map_identity_key(&object);
        for element in &annotation.elements {
            let value = self.annotation_element_dex_value(&element.value)?;
            self.runtime
                .borrow_mut()
                .annotation_elements
                .insert((object_key.clone(), element.name.clone()), value);
        }

        Some(object)
    }

    fn annotation_element_dex_value(&self, value: &DexAnnotationValue) -> Option<DexValue> {
        match value {
            DexAnnotationValue::Int(value) => Some(DexValue::Int(*value)),
            DexAnnotationValue::Long(value) => Some(DexValue::Long(*value)),
            DexAnnotationValue::Float(value) => Some(DexValue::Float(*value)),
            DexAnnotationValue::Double(value) => Some(DexValue::Long(value.to_bits() as i64)),
            DexAnnotationValue::String(value) => {
                Some(DexValue::String(std::rc::Rc::from(value.clone())))
            }
            DexAnnotationValue::Type(descriptor) => Some(DexValue::Reference(std::rc::Rc::from(
                format!("Ljava/lang/Class;:{descriptor}"),
            ))),
            DexAnnotationValue::Boolean(value) => Some(DexValue::Int(i32::from(*value))),
            DexAnnotationValue::Null => Some(DexValue::Null),
            DexAnnotationValue::Array(values) => {
                let values = values
                    .iter()
                    .map(|value| self.annotation_element_dex_value(value))
                    .collect::<Option<Vec<DexValue>>>()?;

                Some(self.allocate_filled_array("[Ljava/lang/Object;".to_owned(), values))
            }
            DexAnnotationValue::Enum(_)
            | DexAnnotationValue::Annotation(_)
            | DexAnnotationValue::Unsupported { .. } => None,
        }
    }

    pub(super) fn annotation_element_value(
        &self,
        method: &DexMethodId,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 1 || !method.proto.parameters.is_empty() {
            return None;
        }

        let receiver_key = map_identity_key(&invocation.arguments[0]);
        self.runtime
            .borrow()
            .annotation_elements
            .get(&(receiver_key, method.name.clone()))
            .cloned()
    }

    pub(super) fn class_is_interface(&self, invocation: &DexInvocation) -> Option<i32> {
        let descriptor = class_reference_descriptor(invocation)?;

        let is_interface = self
            .load_class(descriptor)
            .map(|loaded_class| (loaded_class.class_def.access_flags & JAVA_ACC_INTERFACE) != 0)
            .unwrap_or_else(|| is_bootstrap_interface_descriptor(descriptor));

        Some(i32::from(is_interface))
    }

    pub(super) fn class_to_string(&self, invocation: &DexInvocation) -> Option<String> {
        let descriptor = class_reference_descriptor(invocation)?;
        let name = class_display_name(descriptor)?;

        if is_primitive_descriptor(descriptor) {
            return Some(name);
        }

        if self.class_is_interface(invocation)? != 0 {
            return Some(format!("interface {name}"));
        }

        Some(format!("class {name}"))
    }

    pub(super) fn class_is_instance(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let target = class_reference_descriptor_from_value(&invocation.arguments[0])?;
        Some(i32::from(
            self.value_matches_type(&invocation.arguments[1], target),
        ))
    }

    pub(super) fn class_cast(&self, invocation: &DexInvocation) -> Option<DexExecutionResult> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let target = class_reference_descriptor_from_value(&invocation.arguments[0])?;
        let value = invocation.arguments[1].clone();
        // cite: libcore Class.java#cast, cannotCastMsg, getName
        if matches!(&value, DexValue::Null | DexValue::Int(0)) {
            return Some(DexExecutionResult::Value(DexValue::Null));
        }
        if !matches!(
            &value,
            DexValue::Object(_) | DexValue::Reference(_) | DexValue::String(_)
        ) {
            return None;
        }

        if self.value_matches_type(&value, target) {
            return Some(DexExecutionResult::Value(value));
        }

        let source_name = class_display_name(value_class_descriptor(&value)?)?;
        let target_name = class_display_name(target)?;
        let exception = self.allocate_generated_exception(DexGeneratedException {
            class_descriptor: "Ljava/lang/ClassCastException;".to_owned(),
            detail_message: Some(format!("Cannot cast {source_name} to {target_name}")),
        });

        Some(DexExecutionResult::Throw(exception))
    }

    pub(super) fn reflect_method_invoke(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        let (class_descriptor, method_name) = reflect_method_reference(&invocation.arguments[0])?;
        if class_descriptor == "Landroid/os/Handler;" && method_name == "createAsync" {
            let arguments = self.reflect_method_argument_values(&invocation.arguments[2])?;
            if arguments.len() != 1 {
                return None;
            }
            let looper = match &arguments[0] {
                DexValue::Object(_) | DexValue::Reference(_) => arguments[0].clone(),
                DexValue::Uninitialized
                | DexValue::Null
                | DexValue::Int(_)
                | DexValue::Long(_)
                | DexValue::Float(_)
                | DexValue::String(_) => return None,
            };

            return Some(self.create_handler_for_looper(looper));
        }

        if class_descriptor == "Ljava/lang/Class;" && method_name == "isRecord" {
            let _target_class = class_reference_descriptor_from_value(&invocation.arguments[1])?;
            let arguments = self.reflect_method_argument_values(&invocation.arguments[2])?;
            if !arguments.is_empty() {
                return None;
            }

            return Some(DexValue::Reference(std::rc::Rc::from(
                "Ljava/lang/Boolean;:false",
            )));
        }

        reflect_method_invoke(invocation)?;
        Some(DexValue::Null)
    }

    pub(super) fn reflect_method_get_return_type(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let (class_descriptor, method_name) = reflect_method_reference(&invocation.arguments[0])?;
        let return_descriptor =
            self.reflect_method_return_descriptor(class_descriptor, method_name)?;

        Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Ljava/lang/Class;:{return_descriptor}"
        ))))
    }

    pub(super) fn reflect_method_return_descriptor(
        &self,
        class_descriptor: &str,
        method_name: &str,
    ) -> Option<String> {
        if let Some(loaded_class) = self.load_class(class_descriptor) {
            for encoded_method in loaded_class
                .class_def
                .direct_methods
                .iter()
                .chain(loaded_class.class_def.virtual_methods.iter())
            {
                let Ok(method) =
                    self.method_id(loaded_class.dex_file_index, encoded_method.method_index)
                else {
                    continue;
                };
                if method.name == method_name {
                    return Some(method.proto.return_type.clone());
                }
            }
        }

        bootstrap_method_return_descriptor(class_descriptor, method_name).map(str::to_owned)
    }

    pub(super) fn reflect_method_argument_values(&self, value: &DexValue) -> Option<Vec<DexValue>> {
        match value {
            DexValue::Null | DexValue::Int(0) => Some(Vec::new()),
            DexValue::Object(_) | DexValue::Reference(_) => {
                let array_key = map_identity_key(value);
                self.runtime.borrow().array_entries.get(&array_key).cloned()
            }
            DexValue::Uninitialized
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => None,
        }
    }

    pub(super) fn reflect_constructor_new_instance(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<DexValue> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "Constructor.newInstance expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let (class_descriptor, parameter_key) =
            reflect_constructor_reference(&invocation.arguments[0]).ok_or_else(|| {
                WinedroidError::new(format!(
                    "Constructor.newInstance requires a constructor reference\nconstructor: {}",
                    invocation.arguments[0]
                ))
            })?;
        let parameters =
            self.reflect_constructor_parameter_descriptors(parameter_key)
                .ok_or_else(|| {
                    WinedroidError::new(format!(
                        "Constructor.newInstance could not resolve constructor parameter types\nconstructor: {}",
                        invocation.arguments[0]
                    ))
                })?;
        let arguments = self
            .reflect_method_argument_values(&invocation.arguments[1])
            .ok_or_else(|| {
                WinedroidError::new(format!(
                    "Constructor.newInstance requires an Object[] argument list\narguments: {}",
                    invocation.arguments[1]
                ))
            })?;
        if arguments.len() != parameters.len() {
            return Err(WinedroidError::new(format!(
                "Constructor.newInstance argument count does not match parameter count\nconstructor: {}\narguments: {}\nparameters: {}",
                invocation.arguments[0],
                arguments.len(),
                parameters.len()
            )));
        }

        let constructor_method = DexMethodId {
            class_descriptor: class_descriptor.to_owned(),
            name: "<init>".to_owned(),
            proto: crate::dex::DexProto {
                shorty: proto_shorty("V", &parameters).ok_or_else(|| {
                    WinedroidError::new(format!(
                        "Constructor.newInstance could not build constructor shorty\nconstructor: {}",
                        invocation.arguments[0]
                    ))
                })?,
                return_type: "V".to_owned(),
                parameters,
            },
        };
        let Some(resolved) = self.resolve_exact_method(&constructor_method)? else {
            return self.allocate_reflected_no_arg_object(class_descriptor, &constructor_method);
        };

        let object = self.allocate_object(class_descriptor.to_owned());
        let mut initial_arguments = Vec::with_capacity(arguments.len() + 1);
        initial_arguments.push(object.clone());
        initial_arguments.extend(arguments);

        let result = self.execute_method(None, &resolved, Some(&initial_arguments), depth + 1)?;
        match result {
            DexExecutionResult::Void => Ok(object),
            DexExecutionResult::Int(_)
            | DexExecutionResult::Value(_)
            | DexExecutionResult::Throw(_) => {
                Err(WinedroidError::new(format!(
                    "Constructor.newInstance target constructor did not complete normally\nconstructor: {}",
                    invocation.arguments[0]
                )))
            }
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                Err(WinedroidError::new(format!(
                    "Constructor.newInstance target constructor did not complete normally\nconstructor: {}",
                    invocation.arguments[0]
                )))
            }
        }
    }

    pub(super) fn allocate_reflected_no_arg_object(
        &self,
        class_descriptor: &str,
        constructor_method: &DexMethodId,
    ) -> Result<DexValue> {
        if !constructor_method.proto.parameters.is_empty() {
            return Err(WinedroidError::new(format!(
                "Constructor.newInstance target constructor was not found\nconstructor: {}.{}",
                constructor_method.class_descriptor,
                constructor_method.display_signature()
            )));
        }

        let class_exists = self.load_class(class_descriptor).is_some()
            || is_bootstrap_class_descriptor(class_descriptor);
        if !class_exists {
            return Err(WinedroidError::new(format!(
                "Constructor.newInstance target class was not found\nclass: {class_descriptor}"
            )));
        }

        Ok(self.allocate_object(class_descriptor.to_owned()))
    }

    // cite: libcore Class.java#getConstructors
    pub(super) fn class_get_constructors(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if invocation.arguments[0] == DexValue::Null {
            return Some(DexValue::Null);
        }

        let descriptor = class_reference_descriptor_from_value(&invocation.arguments[0])?;
        let loaded = self.load_class(descriptor)?;

        let mut constructors = Vec::new();
        for encoded in &loaded.class_def.direct_methods {
            if encoded.access_flags & JAVA_ACC_PUBLIC == 0 {
                continue;
            }
            let method = self
                .method_id(loaded.dex_file_index, encoded.method_index)
                .ok()?;
            if method.name != "<init>" {
                continue;
            }

            let parameter_array = self.allocate_class_array(&method.proto.parameters)?;
            let parameter_key = map_identity_key(&parameter_array);
            constructors.push(DexValue::Reference(std::rc::Rc::from(format!(
                "Ljava/lang/reflect/Constructor;:{descriptor}:{parameter_key}"
            ))));
        }

        let array = self.allocate_array(
            "[Ljava/lang/reflect/Constructor;".to_owned(),
            i32::try_from(constructors.len()).ok()?,
        )?;
        self.runtime
            .borrow_mut()
            .array_entries
            .insert(map_identity_key(&array), constructors);

        Some(array)
    }

    // cite: libcore Constructor.java#getParameterTypes
    pub(super) fn constructor_get_parameter_types(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let (_, parameter_key) = reflect_constructor_reference(&invocation.arguments[0])?;
        let descriptors = self.reflect_constructor_parameter_descriptors(parameter_key)?;

        self.allocate_class_array(&descriptors)
    }

    fn allocate_class_array(&self, descriptors: &[String]) -> Option<DexValue> {
        let classes: Vec<DexValue> = descriptors
            .iter()
            .map(|descriptor| {
                DexValue::Reference(std::rc::Rc::from(format!("Ljava/lang/Class;:{descriptor}")))
            })
            .collect();
        let array = self.allocate_array(
            "[Ljava/lang/Class;".to_owned(),
            i32::try_from(classes.len()).ok()?,
        )?;
        self.runtime
            .borrow_mut()
            .array_entries
            .insert(map_identity_key(&array), classes);

        Some(array)
    }

    pub(super) fn reflect_constructor_parameter_descriptors(
        &self,
        parameter_key: &str,
    ) -> Option<Vec<String>> {
        if matches!(parameter_key, "int:0" | "null") {
            return Some(Vec::new());
        }

        let parameters = self
            .runtime
            .borrow()
            .array_entries
            .get(parameter_key)?
            .clone();
        let mut descriptors = Vec::with_capacity(parameters.len());

        for parameter in parameters {
            descriptors.push(class_reference_descriptor_from_value(&parameter)?.to_owned());
        }

        Some(descriptors)
    }

    pub(super) fn value_matches_type(&self, value: &DexValue, target_descriptor: &str) -> bool {
        match value {
            DexValue::Object(object) => {
                self.descriptor_matches_type(&object.class_descriptor, target_descriptor)
            }
            DexValue::Reference(reference) => {
                let Some(descriptor) = reference_descriptor(reference) else {
                    return reference.as_ref() == target_descriptor;
                };

                self.descriptor_matches_type(descriptor, target_descriptor)
            }
            DexValue::String(_) => {
                self.descriptor_matches_type("Ljava/lang/String;", target_descriptor)
            }
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_) => false,
        }
    }

    pub(super) fn descriptor_matches_type(&self, candidate: &str, target: &str) -> bool {
        // cite: libcore Class.java#isAssignableFrom
        if candidate == target {
            return true;
        }

        if target == "Ljava/lang/Object;"
            && (candidate.starts_with('L') || candidate.starts_with('['))
        {
            return true;
        }

        if candidate.starts_with('[') && target.starts_with('[') {
            return self.descriptor_matches_type(&candidate[1..], &target[1..]);
        }

        self.descriptor_matches_type_inner(candidate, target, &mut Vec::new())
    }

    pub(super) fn descriptor_matches_type_inner(
        &self,
        candidate: &str,
        target: &str,
        visited: &mut Vec<String>,
    ) -> bool {
        if visited.iter().any(|descriptor| descriptor == candidate) {
            return false;
        }
        visited.push(candidate.to_owned());

        let Some(loaded_class) = self.load_class(candidate) else {
            return self.bootstrap_descriptor_matches_type(candidate, target, visited);
        };

        for interface_descriptor in &loaded_class.class_def.interface_descriptors {
            if interface_descriptor == target
                || self.descriptor_matches_type_inner(interface_descriptor, target, visited)
            {
                return true;
            }
        }

        let Some(superclass_descriptor) = &loaded_class.class_def.superclass_descriptor else {
            return self.bootstrap_descriptor_matches_type(candidate, target, visited);
        };

        if superclass_descriptor == target
            || self.descriptor_matches_type_inner(superclass_descriptor, target, visited)
        {
            return true;
        }

        self.bootstrap_descriptor_matches_type(candidate, target, visited)
    }

    pub(super) fn bootstrap_descriptor_matches_type(
        &self,
        candidate: &str,
        target: &str,
        visited: &mut Vec<String>,
    ) -> bool {
        for interface_descriptor in bootstrap_direct_interface_descriptors(candidate) {
            if *interface_descriptor == target
                || self.descriptor_matches_type_inner(interface_descriptor, target, visited)
            {
                return true;
            }
        }

        let Some(superclass_descriptor) = bootstrap_superclass_descriptor(candidate) else {
            return false;
        };

        superclass_descriptor == target
            || self.descriptor_matches_type_inner(superclass_descriptor, target, visited)
    }
}

// cite: libcore Class.java#getName
// cite: art class.cc#ComputeName
pub(super) fn class_get_name(invocation: &DexInvocation) -> Option<String> {
    if invocation.arguments.len() != 1 {
        return None;
    }

    let descriptor = class_reference_descriptor(invocation)?;

    class_display_name(descriptor)
}

fn class_display_name(descriptor: &str) -> Option<String> {
    primitive_descriptor_to_java_name(descriptor)
        .map(str::to_owned)
        .or_else(|| descriptor_to_java_name(descriptor))
}

// cite: art descriptors_names.cc#IsValidBinaryClassName, DotToDescriptor
fn class_for_name_descriptor(class_name: &str) -> Option<String> {
    let mut name = class_name;
    let mut array_dimensions = 0usize;
    while let Some(component) = name.strip_prefix('[') {
        array_dimensions += 1;
        if array_dimensions > 255 {
            return None;
        }
        name = component;
    }

    if array_dimensions != 0 {
        let valid_component = match name {
            "B" | "C" | "D" | "F" | "I" | "J" | "S" | "Z" => true,
            _ => name
                .strip_prefix('L')
                .and_then(|name| name.strip_suffix(';'))
                .is_some_and(valid_binary_class_components),
        };
        if !valid_component {
            return None;
        }
    } else if !valid_binary_class_components(name) {
        return None;
    }

    let descriptor = class_name.replace('.', "/");
    if array_dimensions == 0 {
        Some(format!("L{descriptor};"))
    } else {
        Some(descriptor)
    }
}

fn valid_binary_class_components(name: &str) -> bool {
    !name.is_empty()
        && name
            .split('.')
            .all(|component| !component.is_empty() && component.chars().all(valid_dex_name_char))
}

fn valid_dex_name_char(character: char) -> bool {
    if character.is_ascii() {
        return character == ' '
            || character == '$'
            || character == '-'
            || character == '_'
            || character.is_ascii_alphanumeric();
    }

    let code = character as u32;
    !(0x80..=0x9f).contains(&code)
        && !(0x200b..=0x200f).contains(&code)
        && !(0x2028..=0x202e).contains(&code)
        && !(0xfff0..=0xffff).contains(&code)
}

fn class_new_instance_pretty_class(descriptor: &str) -> Result<String> {
    let name = descriptor_to_pretty_name(descriptor).ok_or_else(|| {
        WinedroidError::new(format!(
            "Class.newInstance target descriptor is invalid\ndescriptor: {descriptor}"
        ))
    })?;

    // cite: art class.cc#Class::PrettyClass
    Ok(format!("java.lang.Class<{name}>"))
}

fn primitive_descriptor_to_java_name(descriptor: &str) -> Option<&'static str> {
    match descriptor {
        "Z" => Some("boolean"),
        "B" => Some("byte"),
        "C" => Some("char"),
        "S" => Some("short"),
        "I" => Some("int"),
        "J" => Some("long"),
        "F" => Some("float"),
        "D" => Some("double"),
        "V" => Some("void"),
        _ => None,
    }
}

pub(super) fn class_get_simple_name(invocation: &DexInvocation) -> Option<String> {
    let name = class_get_name(invocation)?;
    let simple_start = name.rfind(['.', '$']).map(|index| index + 1).unwrap_or(0);

    Some(name[simple_start..].to_owned())
}

pub(super) fn class_is_array(invocation: &DexInvocation) -> Option<i32> {
    let descriptor = class_reference_descriptor(invocation)?;

    Some(i32::from(descriptor.starts_with('[')))
}

pub(super) fn class_is_member_class(invocation: &DexInvocation) -> Option<i32> {
    let descriptor = class_reference_descriptor(invocation)?;

    Some(i32::from(descriptor.contains('$')))
}

pub(super) fn class_is_primitive(invocation: &DexInvocation) -> Option<i32> {
    let descriptor = class_reference_descriptor(invocation)?;

    Some(i32::from(matches!(
        descriptor,
        "Z" | "B" | "C" | "S" | "I" | "J" | "F" | "D" | "V"
    )))
}

pub(super) fn class_get_component_type(invocation: &DexInvocation) -> Option<DexValue> {
    let descriptor = class_reference_descriptor(invocation)?;
    let Some(component_descriptor) = array_component_descriptor(descriptor) else {
        return Some(DexValue::Null);
    };

    Some(DexValue::Reference(std::rc::Rc::from(format!(
        "Ljava/lang/Class;:{component_descriptor}"
    ))))
}

pub(super) fn array_component_descriptor(descriptor: &str) -> Option<&str> {
    descriptor.strip_prefix('[')
}

pub(super) fn calendar_get_field(invocation: &DexInvocation) -> Option<i32> {
    if invocation.arguments.len() != 2 {
        return None;
    }

    let DexValue::Int(field) = invocation.arguments[1] else {
        return None;
    };

    match field {
        ANDROID_CALENDAR_HOUR_OF_DAY => Some(BOOTSTRAP_HOUR_OF_DAY),
        _ => None,
    }
}

pub(super) fn class_get_package(invocation: &DexInvocation) -> Option<DexValue> {
    let descriptor = class_reference_descriptor(invocation)?;
    let package_name = descriptor_package_name(descriptor)?;

    Some(DexValue::Reference(std::rc::Rc::from(format!(
        "Ljava/lang/Package;:{package_name}"
    ))))
}

pub(super) fn class_get_declared_constructor(invocation: &DexInvocation) -> Option<DexValue> {
    if invocation.arguments.len() != 2 {
        return None;
    }

    if invocation.arguments[0] == DexValue::Null {
        return Some(DexValue::Null);
    }

    let descriptor = class_reference_descriptor_from_value(&invocation.arguments[0])?;
    let parameter_key = map_identity_key(&invocation.arguments[1]);

    Some(DexValue::Reference(std::rc::Rc::from(format!(
        "Ljava/lang/reflect/Constructor;:{descriptor}:{parameter_key}"
    ))))
}

pub(super) fn class_get_method(invocation: &DexInvocation) -> Option<DexValue> {
    if invocation.arguments.len() != 3 {
        return None;
    }

    if invocation.arguments[0] == DexValue::Null {
        return Some(DexValue::Null);
    }

    let descriptor = class_reference_descriptor_from_value(&invocation.arguments[0])?;
    let DexValue::String(method_name) = &invocation.arguments[1] else {
        return None;
    };
    let parameter_key = map_identity_key(&invocation.arguments[2]);

    Some(DexValue::Reference(std::rc::Rc::from(format!(
        "Ljava/lang/reflect/Method;:{descriptor}:{method_name}:{parameter_key}"
    ))))
}

pub(super) fn reflect_method_reference(value: &DexValue) -> Option<(&str, &str)> {
    let DexValue::Reference(reference) = value else {
        return None;
    };
    let method_reference = reference.strip_prefix("Ljava/lang/reflect/Method;:")?;
    let (class_descriptor, rest) = method_reference.split_once(':')?;
    let (method_name, _parameter_key) = rest.split_once(':')?;

    Some((class_descriptor, method_name))
}

pub(super) fn reflect_constructor_reference(value: &DexValue) -> Option<(&str, &str)> {
    let DexValue::Reference(reference) = value else {
        return None;
    };
    let constructor_reference = reference.strip_prefix("Ljava/lang/reflect/Constructor;:")?;

    constructor_reference.split_once(':')
}

pub(super) fn class_get_field(invocation: &DexInvocation) -> Option<DexValue> {
    if invocation.arguments.len() != 2 {
        return None;
    }

    if invocation.arguments[0] == DexValue::Null {
        return Some(DexValue::Null);
    }

    let descriptor = class_reference_descriptor_from_value(&invocation.arguments[0])?;
    let DexValue::String(field_name) = &invocation.arguments[1] else {
        return None;
    };
    let field = reflect_bootstrap_field(descriptor, field_name)?;

    Some(DexValue::Reference(std::rc::Rc::from(format!(
        "Ljava/lang/reflect/Field;:{}:{}",
        field.class_descriptor, field.name
    ))))
}

pub(super) fn reflect_dex_field_value(field: &DexFieldId, access_flags: u32) -> DexValue {
    DexValue::Reference(std::rc::Rc::from(format!(
        "Ljava/lang/reflect/Field;:{}:{}:{}:{}",
        field.class_descriptor, field.name, field.type_descriptor, access_flags
    )))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ReflectFieldReference {
    field: DexFieldId,
    access_flags: Option<u32>,
}

impl DexProgram {
    pub(super) fn reflect_field_get_int(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let field = reflect_field_reference(&invocation.arguments[0])?;
        let value = self.reflect_field_read_value(&field, &invocation.arguments[1])?;

        match value {
            DexValue::Int(value) => Some(value),
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => None,
        }
    }

    pub(super) fn reflect_field_get(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let field = reflect_field_reference(&invocation.arguments[0])?;
        let value = self.reflect_field_read_value(&field, &invocation.arguments[1])?;

        box_reflected_field_value(&field.field, value)
    }

    // cite: libcore Field.java#set
    pub(super) fn reflect_field_set(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        let field = reflect_field_reference(&invocation.arguments[0])?;
        let unboxed = unbox_reflected_field_value(&field.field, &invocation.arguments[2])?;
        let is_static = field
            .access_flags
            .is_some_and(|flags| flags & DEX_ACC_STATIC != 0);

        if is_static {
            self.write_stored_static_field(&field.field, unboxed);
            return Some(());
        }

        match &invocation.arguments[1] {
            DexValue::Object(_) | DexValue::Reference(_) => {}
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        }

        self.write_stored_instance_field(&invocation.arguments[1], &field.field, unboxed)
            .then_some(())
    }

    pub(super) fn reflect_field_read_value(
        &self,
        field: &ReflectFieldReference,
        object: &DexValue,
    ) -> Option<DexValue> {
        let Some(access_flags) = field.access_flags else {
            return bootstrap_static_field(&field.field);
        };

        if access_flags & DEX_ACC_STATIC != 0 {
            return self
                .read_stored_static_field(&field.field)
                .or_else(|| bootstrap_static_field(&field.field))
                .or_else(|| default_field_value(&field.field));
        }

        match object {
            DexValue::Object(_) | DexValue::Reference(_) => {}
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        }

        self.read_stored_instance_field(object, &field.field)
            .or_else(|| self.read_bootstrap_instance_field(object, &field.field))
            .or_else(|| default_field_value(&field.field))
    }

    pub(super) fn unsafe_object_field_offset(&self, invocation: &DexInvocation) -> Option<i64> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        match &invocation.arguments[0] {
            DexValue::Object(_) | DexValue::Reference(_) => {}
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        }

        let field = reflect_field_reference(&invocation.arguments[1])?.field;
        let offset = i64::from(stable_hash_i32(&field.display_signature()));

        self.runtime
            .borrow_mut()
            .unsafe_field_offsets
            .insert(offset, field);

        Some(offset)
    }

    pub(super) fn unsafe_compare_and_swap_object(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 6 {
            return None;
        }

        match &invocation.arguments[0] {
            DexValue::Object(_) | DexValue::Reference(_) => {}
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        }

        let target = &invocation.arguments[1];
        let DexValue::Long(offset) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Long(high_slot_offset) = invocation.arguments[3] else {
            return None;
        };
        if offset != high_slot_offset {
            return None;
        }

        let expected = atomic_reference_value(&invocation.arguments[4])?;
        let next = atomic_reference_value(&invocation.arguments[5])?;

        // cite: libcore Unsafe.java#compareAndSwapObject
        if let Some(index) = unsafe_object_array_index(target, offset) {
            let array_key = map_identity_key(target);
            let mut runtime = self.runtime.borrow_mut();
            let entries = runtime.array_entries.get_mut(&array_key)?;
            let slot = entries.get_mut(index)?;
            if *slot != expected {
                return Some(0);
            }
            *slot = next;
            return Some(1);
        }

        let field = self
            .runtime
            .borrow()
            .unsafe_field_offsets
            .get(&offset)?
            .clone();
        if !field.type_descriptor.starts_with('L') && !field.type_descriptor.starts_with('[') {
            return None;
        }

        let current = self
            .read_stored_instance_field(target, &field)
            .or_else(|| self.read_bootstrap_instance_field(target, &field))
            .or_else(|| default_field_value(&field))?;

        if current != expected {
            return Some(0);
        }

        self.write_stored_instance_field(target, &field, next)
            .then_some(1)
    }

    // cite: libcore Unsafe.java#compareAndSwapInt
    pub(super) fn unsafe_compare_and_swap_int(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 6 {
            return None;
        }

        match &invocation.arguments[0] {
            DexValue::Object(_) | DexValue::Reference(_) => {}
            _ => return None,
        }

        let target = &invocation.arguments[1];
        let DexValue::Long(offset) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Long(high_slot_offset) = invocation.arguments[3] else {
            return None;
        };
        if offset != high_slot_offset {
            return None;
        }
        let DexValue::Int(expected) = invocation.arguments[4] else {
            return None;
        };
        let DexValue::Int(next) = invocation.arguments[5] else {
            return None;
        };

        let field = self
            .runtime
            .borrow()
            .unsafe_field_offsets
            .get(&offset)?
            .clone();
        let current = match self
            .read_stored_instance_field(target, &field)
            .or_else(|| self.read_bootstrap_instance_field(target, &field))
            .or_else(|| default_field_value(&field))?
        {
            DexValue::Int(value) => value,
            _ => return None,
        };

        if current != expected {
            return Some(0);
        }

        self.write_stored_instance_field(target, &field, DexValue::Int(next))
            .then_some(1)
    }

    // cite: libcore Unsafe.java#compareAndSwapLong
    pub(super) fn unsafe_compare_and_swap_long(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 8 {
            return None;
        }

        match &invocation.arguments[0] {
            DexValue::Object(_) | DexValue::Reference(_) => {}
            _ => return None,
        }

        let target = &invocation.arguments[1];
        let DexValue::Long(offset) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Long(high_slot_offset) = invocation.arguments[3] else {
            return None;
        };
        if offset != high_slot_offset {
            return None;
        }
        let DexValue::Long(expected) = invocation.arguments[4] else {
            return None;
        };
        let DexValue::Long(next) = invocation.arguments[6] else {
            return None;
        };

        let field = self
            .runtime
            .borrow()
            .unsafe_field_offsets
            .get(&offset)?
            .clone();
        let current = match self
            .read_stored_instance_field(target, &field)
            .or_else(|| self.read_bootstrap_instance_field(target, &field))
            .or_else(|| default_field_value(&field))?
        {
            DexValue::Long(value) => value,
            _ => return None,
        };

        if current != expected {
            return Some(0);
        }

        self.write_stored_instance_field(target, &field, DexValue::Long(next))
            .then_some(1)
    }

    // cite: libcore Unsafe.java#arrayBaseOffset
    pub(super) fn unsafe_array_base_offset(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }
        Some(16)
    }

    // cite: libcore Unsafe.java#arrayIndexScale
    pub(super) fn unsafe_array_index_scale(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }
        let descriptor = class_reference_descriptor_from_value(&invocation.arguments[1]);
        let scale = match descriptor {
            Some("[B") | Some("[Z") => 1,
            Some("[C") | Some("[S") => 2,
            Some("[J") | Some("[D") => 8,
            _ => 4,
        };
        Some(scale)
    }

    // cite: libcore Unsafe.java#putLong
    pub(super) fn unsafe_put_long(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 6 {
            return None;
        }
        let target = &invocation.arguments[1];
        let DexValue::Long(offset) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Long(value) = invocation.arguments[4] else {
            return None;
        };
        let field = self
            .runtime
            .borrow()
            .unsafe_field_offsets
            .get(&offset)?
            .clone();
        self.write_stored_instance_field(target, &field, DexValue::Long(value))
            .then_some(())
    }

    // cite: libcore Unsafe.java#getLong
    pub(super) fn unsafe_get_long(&self, invocation: &DexInvocation) -> Option<i64> {
        if invocation.arguments.len() != 4 {
            return None;
        }
        let target = &invocation.arguments[1];
        let DexValue::Long(offset) = invocation.arguments[2] else {
            return None;
        };
        let field = self
            .runtime
            .borrow()
            .unsafe_field_offsets
            .get(&offset)?
            .clone();
        match self
            .read_stored_instance_field(target, &field)
            .or_else(|| self.read_bootstrap_instance_field(target, &field))
            .or_else(|| default_field_value(&field))?
        {
            DexValue::Long(value) => Some(value),
            _ => None,
        }
    }

    // cite: libcore Unsafe.java#putInt
    pub(super) fn unsafe_put_int(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 5 {
            return None;
        }
        let target = &invocation.arguments[1];
        let DexValue::Long(offset) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Int(value) = invocation.arguments[4] else {
            return None;
        };
        let field = self
            .runtime
            .borrow()
            .unsafe_field_offsets
            .get(&offset)?
            .clone();
        self.write_stored_instance_field(target, &field, DexValue::Int(value))
            .then_some(())
    }

    // cite: libcore Unsafe.java#getInt
    pub(super) fn unsafe_get_int(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 4 {
            return None;
        }
        let target = &invocation.arguments[1];
        let DexValue::Long(offset) = invocation.arguments[2] else {
            return None;
        };
        let field = self
            .runtime
            .borrow()
            .unsafe_field_offsets
            .get(&offset)?
            .clone();
        match self
            .read_stored_instance_field(target, &field)
            .or_else(|| self.read_bootstrap_instance_field(target, &field))
            .or_else(|| default_field_value(&field))?
        {
            DexValue::Int(value) => Some(value),
            _ => None,
        }
    }

    pub(super) fn unsafe_put_object_volatile(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 5 {
            return None;
        }

        match &invocation.arguments[0] {
            DexValue::Object(_) | DexValue::Reference(_) => {}
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        }

        let target = &invocation.arguments[1];
        let DexValue::Long(offset) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Long(high_slot_offset) = invocation.arguments[3] else {
            return None;
        };
        if offset != high_slot_offset {
            return None;
        }

        let value = atomic_reference_value(&invocation.arguments[4])?;

        // cite: libcore Unsafe.java#putObjectVolatile
        if let Some(index) = unsafe_object_array_index(target, offset) {
            let array_key = map_identity_key(target);
            let mut runtime = self.runtime.borrow_mut();
            let entries = runtime.array_entries.get_mut(&array_key)?;
            let slot = entries.get_mut(index)?;
            *slot = value;
            return Some(());
        }

        let field = self
            .runtime
            .borrow()
            .unsafe_field_offsets
            .get(&offset)?
            .clone();
        if !field.type_descriptor.starts_with('L') && !field.type_descriptor.starts_with('[') {
            return None;
        }

        self.write_stored_instance_field(target, &field, value)
            .then_some(())
    }

    pub(super) fn unsafe_get_object_volatile(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 4 {
            return None;
        }

        match &invocation.arguments[0] {
            DexValue::Object(_) | DexValue::Reference(_) => {}
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        }

        let target = &invocation.arguments[1];
        let DexValue::Long(offset) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Long(high_slot_offset) = invocation.arguments[3] else {
            return None;
        };
        if offset != high_slot_offset {
            return None;
        }

        // cite: libcore Unsafe.java#getObjectVolatile
        if let Some(index) = unsafe_object_array_index(target, offset) {
            let array_key = map_identity_key(target);
            let runtime = self.runtime.borrow();
            let entries = runtime.array_entries.get(&array_key)?;
            return entries.get(index).cloned();
        }

        let field = self
            .runtime
            .borrow()
            .unsafe_field_offsets
            .get(&offset)?
            .clone();
        if !field.type_descriptor.starts_with('L') && !field.type_descriptor.starts_with('[') {
            return None;
        }

        self.read_stored_instance_field(target, &field)
            .or_else(|| self.read_bootstrap_instance_field(target, &field))
            .or_else(|| default_field_value(&field))
    }

    pub(super) fn unsafe_get_and_set_object(&self, invocation: &DexInvocation) -> Option<DexValue> {
        // cite: libcore Unsafe.java#getAndSetObject, getObjectVolatile, compareAndSwapObject
        if invocation.arguments.len() != 5 {
            return None;
        }

        match &invocation.arguments[0] {
            DexValue::Object(_) | DexValue::Reference(_) => {}
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        }

        let target = &invocation.arguments[1];
        let DexValue::Long(offset) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Long(high_slot_offset) = invocation.arguments[3] else {
            return None;
        };
        if offset != high_slot_offset {
            return None;
        }

        let field = self
            .runtime
            .borrow()
            .unsafe_field_offsets
            .get(&offset)?
            .clone();
        if !field.type_descriptor.starts_with('L') && !field.type_descriptor.starts_with('[') {
            return None;
        }

        let next = atomic_reference_value(&invocation.arguments[4])?;
        let current = self
            .read_stored_instance_field(target, &field)
            .or_else(|| self.read_bootstrap_instance_field(target, &field))
            .or_else(|| default_field_value(&field))?;
        self.write_stored_instance_field(target, &field, next)
            .then_some(current)
    }
}

pub(super) fn box_reflected_field_value(field: &DexFieldId, value: DexValue) -> Option<DexValue> {
    match field.type_descriptor.as_str() {
        "Z" => match value {
            DexValue::Int(0) => Some(DexValue::Reference(std::rc::Rc::from(
                "Ljava/lang/Boolean;:false",
            ))),
            DexValue::Int(_) => Some(DexValue::Reference(std::rc::Rc::from(
                "Ljava/lang/Boolean;:true",
            ))),
            _ => None,
        },
        "B" => match value {
            DexValue::Int(value) => Some(DexValue::Reference(std::rc::Rc::from(format!(
                "Ljava/lang/Byte;:{value}"
            )))),
            _ => None,
        },
        "C" => match value {
            DexValue::Int(value) => Some(DexValue::Reference(std::rc::Rc::from(format!(
                "Ljava/lang/Character;:{value}"
            )))),
            _ => None,
        },
        "S" => match value {
            DexValue::Int(value) => Some(DexValue::Reference(std::rc::Rc::from(format!(
                "Ljava/lang/Short;:{value}"
            )))),
            _ => None,
        },
        "I" => match value {
            DexValue::Int(value) => Some(DexValue::Reference(std::rc::Rc::from(format!(
                "Ljava/lang/Integer;:{value}"
            )))),
            _ => None,
        },
        "J" => match value {
            DexValue::Long(value) => Some(DexValue::Reference(std::rc::Rc::from(format!(
                "Ljava/lang/Long;:{value}"
            )))),
            _ => None,
        },
        "F" => match value {
            DexValue::Float(value) => Some(DexValue::Reference(std::rc::Rc::from(format!(
                "Ljava/lang/Float;:{:08x}",
                value.to_bits()
            )))),
            DexValue::Int(value) => Some(DexValue::Reference(std::rc::Rc::from(format!(
                "Ljava/lang/Float;:{:08x}",
                value
            )))),
            _ => None,
        },
        _ => match value {
            DexValue::String(_) | DexValue::Object(_) | DexValue::Reference(_) | DexValue::Null => {
                Some(value)
            }
            DexValue::Uninitialized | DexValue::Int(_) | DexValue::Long(_) | DexValue::Float(_) => {
                None
            }
        },
    }
}

// cite: libcore Field.java#set
pub(super) fn unbox_reflected_field_value(
    field: &DexFieldId,
    value: &DexValue,
) -> Option<DexValue> {
    match field.type_descriptor.as_str() {
        "Z" => match value {
            DexValue::Int(v) => Some(DexValue::Int(if *v == 0 { 0 } else { 1 })),
            DexValue::Reference(r) => match r.as_ref() {
                "Ljava/lang/Boolean;:true" => Some(DexValue::Int(1)),
                "Ljava/lang/Boolean;:false" => Some(DexValue::Int(0)),
                _ => None,
            },
            _ => None,
        },
        "B" => match value {
            DexValue::Int(v) => Some(DexValue::Int(*v)),
            DexValue::Reference(r) => r
                .strip_prefix("Ljava/lang/Byte;:")
                .and_then(|s| s.parse::<i32>().ok())
                .map(DexValue::Int),
            _ => None,
        },
        "C" => match value {
            DexValue::Int(v) => Some(DexValue::Int(*v)),
            DexValue::Reference(r) => r
                .strip_prefix("Ljava/lang/Character;:")
                .and_then(|s| s.parse::<i32>().ok())
                .map(DexValue::Int),
            _ => None,
        },
        "S" => match value {
            DexValue::Int(v) => Some(DexValue::Int(*v)),
            DexValue::Reference(r) => r
                .strip_prefix("Ljava/lang/Short;:")
                .and_then(|s| s.parse::<i32>().ok())
                .map(DexValue::Int),
            _ => None,
        },
        "I" => match value {
            DexValue::Int(v) => Some(DexValue::Int(*v)),
            DexValue::Reference(r) => r
                .strip_prefix("Ljava/lang/Integer;:")
                .and_then(|s| s.parse::<i32>().ok())
                .map(DexValue::Int),
            _ => None,
        },
        "J" => match value {
            DexValue::Long(v) => Some(DexValue::Long(*v)),
            DexValue::Reference(r) => r
                .strip_prefix("Ljava/lang/Long;:")
                .and_then(|s| s.parse::<i64>().ok())
                .map(DexValue::Long),
            _ => None,
        },
        "F" => match value {
            DexValue::Float(v) => Some(DexValue::Float(*v)),
            DexValue::Reference(r) => r
                .strip_prefix("Ljava/lang/Float;:")
                .and_then(|s| u32::from_str_radix(s, 16).ok())
                .map(|bits| DexValue::Float(f32::from_bits(bits))),
            _ => None,
        },
        "D" => match value {
            DexValue::Long(v) => Some(DexValue::Long(*v)),
            DexValue::Reference(r) => r
                .strip_prefix("Ljava/lang/Double;:")
                .and_then(|s| u64::from_str_radix(s, 16).ok())
                .map(|bits| DexValue::Long(bits as i64)),
            _ => None,
        },
        _ => Some(value.clone()),
    }
}

pub(super) fn reflect_field_get_name(invocation: &DexInvocation) -> Option<String> {
    if invocation.arguments.len() != 1 {
        return None;
    }

    Some(
        reflect_field_reference(&invocation.arguments[0])?
            .field
            .name,
    )
}

pub(super) fn reflect_field_get_type(invocation: &DexInvocation) -> Option<DexValue> {
    if invocation.arguments.len() != 1 {
        return None;
    }

    let field = reflect_field_reference(&invocation.arguments[0])?.field;

    Some(DexValue::Reference(std::rc::Rc::from(format!(
        "Ljava/lang/Class;:{}",
        field.type_descriptor
    ))))
}

pub(super) fn reflect_field_get_modifiers(invocation: &DexInvocation) -> Option<i32> {
    if invocation.arguments.len() != 1 {
        return None;
    }

    let field = reflect_field_reference(&invocation.arguments[0])?;
    let access_flags = field
        .access_flags
        .unwrap_or(JAVA_ACC_PUBLIC | DEX_ACC_STATIC | JAVA_ACC_FINAL);

    i32::try_from(access_flags).ok()
}

pub(super) fn reflect_field_is_synthetic(invocation: &DexInvocation) -> Option<i32> {
    if invocation.arguments.len() != 1 {
        return None;
    }

    let field = reflect_field_reference(&invocation.arguments[0])?;
    let access_flags = field.access_flags.unwrap_or(0);

    Some(i32::from(access_flags & JAVA_ACC_SYNTHETIC != 0))
}

pub(super) fn reflect_field_equals(invocation: &DexInvocation) -> Option<i32> {
    if invocation.arguments.len() != 2 {
        return None;
    }

    Some(i32::from(
        invocation.arguments[0] == invocation.arguments[1],
    ))
}

pub(super) fn reflect_field_hash_code(invocation: &DexInvocation) -> Option<i32> {
    if invocation.arguments.len() != 1 {
        return None;
    }

    let DexValue::Reference(reference) = &invocation.arguments[0] else {
        return None;
    };

    Some(stable_hash_i32(reference))
}

pub(super) fn reflect_field_reference(value: &DexValue) -> Option<ReflectFieldReference> {
    let DexValue::Reference(reference) = value else {
        return None;
    };
    let field_reference = reference.strip_prefix("Ljava/lang/reflect/Field;:")?;
    if let Some((field_prefix, access_flags)) = field_reference.rsplit_once(':') {
        if let Ok(access_flags) = access_flags.parse::<u32>() {
            let (field_prefix, type_descriptor) = field_prefix.rsplit_once(':')?;
            let (class_descriptor, field_name) = field_prefix.rsplit_once(':')?;

            return Some(ReflectFieldReference {
                field: DexFieldId {
                    class_descriptor: class_descriptor.to_owned(),
                    type_descriptor: type_descriptor.to_owned(),
                    name: field_name.to_owned(),
                },
                access_flags: Some(access_flags),
            });
        }
    }

    let (class_descriptor, field_name) = field_reference.rsplit_once(':')?;
    let field = reflect_bootstrap_field(class_descriptor, field_name)?;

    Some(ReflectFieldReference {
        field,
        access_flags: None,
    })
}

pub(super) fn reflect_bootstrap_field(
    class_descriptor: &str,
    field_name: &str,
) -> Option<DexFieldId> {
    if class_descriptor == "Landroid/os/Build$VERSION;" && field_name == "SDK_INT" {
        return Some(DexFieldId {
            class_descriptor: class_descriptor.to_owned(),
            type_descriptor: "I".to_owned(),
            name: field_name.to_owned(),
        });
    }

    if class_descriptor == "Lsun/misc/Unsafe;" && field_name == "theUnsafe" {
        return Some(DexFieldId {
            class_descriptor: class_descriptor.to_owned(),
            type_descriptor: "Lsun/misc/Unsafe;".to_owned(),
            name: field_name.to_owned(),
        });
    }

    None
}

pub(super) fn modifier_has_flag(invocation: &DexInvocation, flag: u32) -> Option<i32> {
    if invocation.arguments.len() != 1 {
        return None;
    }

    let DexValue::Int(modifiers) = invocation.arguments[0] else {
        return None;
    };

    let flag = i32::try_from(flag).ok()?;

    Some(i32::from((modifiers & flag) != 0))
}

pub(super) fn reflect_method_invoke(invocation: &DexInvocation) -> Option<()> {
    if invocation.arguments.len() != 3 {
        return None;
    }

    match &invocation.arguments[0] {
        DexValue::Object(_) | DexValue::Reference(_) => {}
        DexValue::Uninitialized
        | DexValue::Null
        | DexValue::Int(_)
        | DexValue::Long(_)
        | DexValue::Float(_)
        | DexValue::String(_) => return None,
    }

    match &invocation.arguments[1] {
        DexValue::Object(_) | DexValue::Reference(_) | DexValue::Null | DexValue::Int(0) => {}
        DexValue::Uninitialized
        | DexValue::Int(_)
        | DexValue::Long(_)
        | DexValue::Float(_)
        | DexValue::String(_) => return None,
    }

    match &invocation.arguments[2] {
        DexValue::Object(_) | DexValue::Reference(_) | DexValue::Null | DexValue::Int(0) => {
            Some(())
        }
        DexValue::Uninitialized
        | DexValue::Int(_)
        | DexValue::Long(_)
        | DexValue::Float(_)
        | DexValue::String(_) => None,
    }
}

pub(super) fn package_get_name(invocation: &DexInvocation) -> Option<String> {
    if invocation.arguments.len() != 1 {
        return None;
    }

    let DexValue::Reference(reference) = &invocation.arguments[0] else {
        return None;
    };

    reference
        .strip_prefix("Ljava/lang/Package;:")
        .map(str::to_owned)
}

pub(super) fn class_reference_descriptor(invocation: &DexInvocation) -> Option<&str> {
    if invocation.arguments.len() != 1 {
        return None;
    }

    class_reference_descriptor_from_value(&invocation.arguments[0])
}

pub(super) fn class_reference_descriptor_from_value(value: &DexValue) -> Option<&str> {
    let DexValue::Reference(reference) = value else {
        return None;
    };

    reference.strip_prefix("Ljava/lang/Class;:")
}

pub(super) fn object_get_class(invocation: &DexInvocation) -> Option<DexValue> {
    if invocation.arguments.len() != 1 {
        return None;
    }

    if invocation.arguments[0] == DexValue::Null {
        return Some(DexValue::Null);
    }

    let descriptor = value_class_descriptor(&invocation.arguments[0])?;

    Some(DexValue::Reference(std::rc::Rc::from(format!(
        "Ljava/lang/Class;:{descriptor}"
    ))))
}

pub(super) fn object_hash_code(invocation: &DexInvocation) -> Option<i32> {
    if invocation.arguments.len() != 1 {
        return None;
    }

    Some(stable_hash_i32(&map_identity_key(&invocation.arguments[0])))
}

// cite: libcore Unsafe.java#arrayBaseOffset, arrayIndexScale
fn unsafe_object_array_index(target: &DexValue, offset: i64) -> Option<usize> {
    let descriptor = value_class_descriptor(target)?;
    if !descriptor.starts_with('[') || offset < 16 {
        return None;
    }
    let rel = offset - 16;
    if rel % 4 != 0 {
        return None;
    }
    usize::try_from(rel / 4).ok()
}
