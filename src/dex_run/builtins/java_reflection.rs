use super::super::*;

impl DexProgram {
    pub(super) fn invoke_java_reflection_builtin(
        &self,
        method: &DexMethodId,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "toString"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.class_to_string(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getName"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = class_get_name(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getCanonicalName"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = class_get_name(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getSimpleName"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = class_get_simple_name(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getClassLoader"
            && method.proto.descriptor_is("()Ljava/lang/ClassLoader;")
        {
            let invocation = invocation?;
            let value = self.class_get_class_loader(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/ClassLoader;"
            && method.name == "loadClass"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Ljava/lang/Class;")
        {
            let invocation = invocation?;
            return self.class_loader_load_class(invocation);
        }

        if method.class_descriptor == "Ljava/lang/reflect/Array;"
            && method.name == "newInstance"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Class;I)Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.reflect_array_new_instance(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getPackage"
            && method.proto.descriptor_is("()Ljava/lang/Package;")
        {
            let invocation = invocation?;
            let value = class_get_package(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getConstructors"
            && method
                .proto
                .descriptor_is("()[Ljava/lang/reflect/Constructor;")
        {
            let invocation = invocation?;
            let value = self.class_get_constructors(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Constructor;"
            && method.name == "getParameterTypes"
            && method.proto.descriptor_is("()[Ljava/lang/Class;")
        {
            let invocation = invocation?;
            let value = self.constructor_get_parameter_types(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getDeclaredConstructor"
            && method
                .proto
                .descriptor_is("([Ljava/lang/Class;)Ljava/lang/reflect/Constructor;")
        {
            let invocation = invocation?;
            let value = class_get_declared_constructor(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getConstructor"
            && method
                .proto
                .descriptor_is("([Ljava/lang/Class;)Ljava/lang/reflect/Constructor;")
        {
            let invocation = invocation?;
            let value = class_get_declared_constructor(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getMethod"
            && method.proto.descriptor()
                == "(Ljava/lang/String;[Ljava/lang/Class;)Ljava/lang/reflect/Method;"
        {
            let invocation = invocation?;
            let value = class_get_method(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getDeclaredMethod"
            && method.proto.descriptor()
                == "(Ljava/lang/String;[Ljava/lang/Class;)Ljava/lang/reflect/Method;"
        {
            let invocation = invocation?;
            let value = class_get_method(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getField"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Ljava/lang/reflect/Field;")
        {
            let invocation = invocation?;
            let value = class_get_field(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getDeclaredField"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Ljava/lang/reflect/Field;")
        {
            let invocation = invocation?;
            let value = self.class_get_declared_field(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getDeclaredFields"
            && method.proto.descriptor_is("()[Ljava/lang/reflect/Field;")
        {
            let invocation = invocation?;
            let value = self.class_get_declared_fields(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getDeclaredMethods"
            && method.proto.descriptor_is("()[Ljava/lang/reflect/Method;")
        {
            let value =
                self.allocate_filled_array("[Ljava/lang/reflect/Method;".to_owned(), Vec::new());

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getInterfaces"
            && method.proto.descriptor_is("()[Ljava/lang/Class;")
        {
            let invocation = invocation?;
            let value = self.class_get_interfaces(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getGenericInterfaces"
            && method.proto.descriptor_is("()[Ljava/lang/reflect/Type;")
        {
            let invocation = invocation?;
            let value = self.class_get_generic_interfaces(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getSuperclass"
            && method.proto.descriptor_is("()Ljava/lang/Class;")
        {
            let invocation = invocation?;
            let value = self.class_get_superclass(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getGenericSuperclass"
            && method.proto.descriptor_is("()Ljava/lang/reflect/Type;")
        {
            let invocation = invocation?;
            let value = self.class_get_generic_superclass(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getComponentType"
            && method.proto.descriptor_is("()Ljava/lang/Class;")
        {
            let invocation = invocation?;
            let value = class_get_component_type(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "isAssignableFrom"
            && method.proto.descriptor_is("(Ljava/lang/Class;)Z")
        {
            let invocation = invocation?;
            let value = self.class_is_assignable_from(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "cast"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;)Ljava/lang/Object;")
        {
            let invocation = invocation?;
            return self.class_cast(invocation);
        }

        // cite: libcore Class.java#asSubclass
        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "asSubclass"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Class;)Ljava/lang/Class;")
        {
            let invocation = invocation?;
            let receiver = invocation.arguments.first()?;
            let target = invocation.arguments.get(1)?;
            let self_desc = class_reference_descriptor_from_value(receiver)?;
            let target_desc = class_reference_descriptor_from_value(target)?;
            if self.descriptor_matches_type(self_desc, target_desc) {
                return Some(DexExecutionResult::Value(receiver.clone()));
            }
            let exception = self.allocate_object("Ljava/lang/ClassCastException;".to_owned());
            return Some(DexExecutionResult::Throw(exception));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getModifiers"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.class_get_modifiers(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore Class.java#getAnnotation
        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "getAnnotation"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Class;)Ljava/lang/annotation/Annotation;")
        {
            let invocation = invocation?;
            let value = self.class_get_annotation(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "isAnonymousClass"
            && method.proto.descriptor_is("()Z")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "isLocalClass"
            && method.proto.descriptor_is("()Z")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "isMemberClass"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = class_is_member_class(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "isArray"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = class_is_array(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "isPrimitive"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = class_is_primitive(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "isInterface"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = self.class_is_interface(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Class;"
            && method.name == "isInstance"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let invocation = invocation?;
            let value = self.class_is_instance(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Package;"
            && method.name == "getName"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = package_get_name(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/reflect/AccessibleObject;"
            && method.name == "isAccessible"
            && method.proto.descriptor_is("()Z")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Ljava/lang/reflect/AccessibleObject;"
            && method.name == "setAccessible"
            && method.proto.descriptor_is("(Z)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/reflect/AccessibleObject;"
            && method.name == "getAnnotations"
            && method
                .proto
                .descriptor_is("()[Ljava/lang/annotation/Annotation;")
        {
            let value = self
                .allocate_filled_array("[Ljava/lang/annotation/Annotation;".to_owned(), Vec::new());

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Method;"
            && method.name == "invoke"
            && method.proto.descriptor()
                == "(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;"
        {
            let invocation = invocation?;
            let value = self.reflect_method_invoke(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Method;"
            && method.name == "getReturnType"
            && method.proto.descriptor_is("()Ljava/lang/Class;")
        {
            let invocation = invocation?;
            let value = self.reflect_method_get_return_type(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Field;"
            && method.name == "getInt"
            && method.proto.descriptor_is("(Ljava/lang/Object;)I")
        {
            let invocation = invocation?;
            let value = self.reflect_field_get_int(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Field;"
            && method.name == "get"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;)Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.reflect_field_get(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: libcore Field.java#set
        if method.class_descriptor == "Ljava/lang/reflect/Field;"
            && method.name == "set"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;Ljava/lang/Object;)V")
        {
            let invocation = invocation?;
            self.reflect_field_set(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/reflect/Field;"
            && method.name == "getName"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = reflect_field_get_name(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Field;"
            && method.name == "getType"
            && method.proto.descriptor_is("()Ljava/lang/Class;")
        {
            let invocation = invocation?;
            let value = reflect_field_get_type(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Field;"
            && method.name == "getGenericType"
            && method.proto.descriptor_is("()Ljava/lang/reflect/Type;")
        {
            let invocation = invocation?;
            let value = reflect_field_get_type(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Field;"
            && method.name == "getModifiers"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = reflect_field_get_modifiers(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Field;"
            && method.name == "isSynthetic"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = reflect_field_is_synthetic(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Field;"
            && method.name == "getAnnotation"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Class;)Ljava/lang/annotation/Annotation;")
        {
            return Some(DexExecutionResult::Value(DexValue::Null));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Field;"
            && method.name == "setAccessible"
            && method.proto.descriptor_is("(Z)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/reflect/Field;"
            && method.name == "isAccessible"
            && method.proto.descriptor_is("()Z")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Field;"
            && method.name == "equals"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let invocation = invocation?;
            let value = reflect_field_equals(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Field;"
            && method.name == "hashCode"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = reflect_field_hash_code(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Modifier;"
            && method.name == "isAbstract"
            && method.proto.descriptor_is("(I)Z")
        {
            let invocation = invocation?;
            let value = modifier_has_flag(invocation, JAVA_ACC_ABSTRACT)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Modifier;"
            && method.name == "isStatic"
            && method.proto.descriptor_is("(I)Z")
        {
            let invocation = invocation?;
            let value = modifier_has_flag(invocation, DEX_ACC_STATIC)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Modifier;"
            && method.name == "isTransient"
            && method.proto.descriptor_is("(I)Z")
        {
            let invocation = invocation?;
            let value = modifier_has_flag(invocation, JAVA_ACC_TRANSIENT)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Modifier;"
            && method.name == "isFinal"
            && method.proto.descriptor_is("(I)Z")
        {
            let invocation = invocation?;
            let value = modifier_has_flag(invocation, JAVA_ACC_FINAL)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Modifier;"
            && method.name == "isPrivate"
            && method.proto.descriptor_is("(I)Z")
        {
            let invocation = invocation?;
            let value = modifier_has_flag(invocation, JAVA_ACC_PRIVATE)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Modifier;"
            && method.name == "isProtected"
            && method.proto.descriptor_is("(I)Z")
        {
            let invocation = invocation?;
            let value = modifier_has_flag(invocation, JAVA_ACC_PROTECTED)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/reflect/Modifier;"
            && method.name == "isPublic"
            && method.proto.descriptor_is("(I)Z")
        {
            let invocation = invocation?;
            let value = modifier_has_flag(invocation, JAVA_ACC_PUBLIC)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/concurrent/TimeUnit;"
            && matches!(method.name.as_str(), "toMillis" | "toNanos")
            && method.proto.descriptor_is("(J)J")
        {
            let invocation = invocation?;
            let value = if method.name == "toMillis" {
                time_unit_to_millis(invocation)?
            } else {
                time_unit_to_nanos(invocation)?
            };

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/TimeUnit;",
            "convert",
            "(JLjava/util/concurrent/TimeUnit;)J",
        ) {
            let invocation = invocation?;
            let value = time_unit_convert(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        // cite: libcore ServiceLoader.java#load, iterator
        if method.class_descriptor == "Ljava/util/ServiceLoader;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("load", "(Ljava/lang/Class;)Ljava/util/ServiceLoader;")
                | ("load", "(Ljava/lang/Class;Ljava/lang/ClassLoader;)Ljava/util/ServiceLoader;") =>
                {
                    let invocation = invocation?;
                    let service_class = invocation.arguments.first()?;
                    let service_name = match service_class {
                        DexValue::Reference(reference) => reference
                            .strip_prefix("Ljava/lang/Class;:")
                            .and_then(descriptor_to_java_name)
                            .unwrap_or_default(),
                        _ => String::new(),
                    };
                    let loader = self.allocate_object("Ljava/util/ServiceLoader;".to_owned());
                    self.write_stored_instance_field(
                        &loader,
                        &service_loader_service_field(),
                        DexValue::String(std::rc::Rc::from(service_name)),
                    );
                    return Some(DexExecutionResult::Value(loader));
                }
                ("iterator", "()Ljava/util/Iterator;") => {
                    let invocation = invocation?;
                    let receiver = invocation.arguments.first()?;
                    let service_name = match self
                        .read_stored_instance_field(receiver, &service_loader_service_field())
                    {
                        Some(DexValue::String(name)) => name.to_string(),
                        _ => String::new(),
                    };
                    let providers = self.load_service_providers(&service_name);
                    let mut runtime = self.runtime.borrow_mut();
                    let iterator_key = format!("iterator:{}", runtime.next_object_id);
                    runtime.next_object_id += 1;
                    runtime
                        .iterator_entries
                        .insert(iterator_key.clone(), providers);
                    runtime.iterator_positions.insert(iterator_key.clone(), 0);
                    return Some(DexExecutionResult::Value(DexValue::Reference(
                        std::rc::Rc::from(format!("Ljava/util/Iterator;:{iterator_key}")),
                    )));
                }
                _ => {}
            }
        }

        None
    }

    fn load_service_providers(&self, service_name: &str) -> Vec<DexValue> {
        let mut providers = Vec::new();
        if service_name.is_empty() {
            return providers;
        }
        let Some(apk_path) = self.apk_path.as_ref() else {
            return providers;
        };
        let Ok(mut apk) = ApkArchive::open(apk_path) else {
            return providers;
        };
        let entry_path = format!("META-INF/services/{service_name}");
        let Ok(Some(bytes)) = apk.read_optional_file(&entry_path) else {
            return providers;
        };
        let text = String::from_utf8_lossy(&bytes);
        for raw_line in text.lines() {
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let descriptor = format!("L{};", line.replace('.', "/"));
            let instance = self.allocate_object(descriptor.clone());
            let init_method = DexMethodId {
                class_descriptor: descriptor.clone(),
                name: "<init>".to_owned(),
                proto: crate::dex::DexProto {
                    shorty: "V".to_owned(),
                    return_type: "V".to_owned(),
                    parameters: Vec::new(),
                },
            };
            if let Ok(Some(constructor)) = self.resolve_exact_method(&init_method) {
                let _ = self.ensure_class_initialized(&descriptor, 1);
                if let Ok(DexExecutionResult::Void) = self.execute_method(
                    None,
                    &constructor,
                    Some(std::slice::from_ref(&instance)),
                    1,
                ) {
                    providers.push(instance);
                }
            }
        }
        providers
    }
}

fn service_loader_service_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/util/ServiceLoader;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "service".to_owned(),
    }
}
