use super::*;

use crate::dex::DexProto;

impl DexProgram {
    pub(super) fn read_hash_map_value(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let map_key = map_identity_key(&invocation.arguments[0]);
        let entry_key = self.map_storage_entry_key(&invocation.arguments[1]);

        Some(
            self.runtime
                .borrow()
                .map_entries
                .get(&(map_key, entry_key))
                .cloned()
                .unwrap_or(DexValue::Null),
        )
    }

    pub(super) fn write_hash_map_value(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        let map_key = map_identity_key(&invocation.arguments[0]);
        let entry_key = self.map_storage_entry_key(&invocation.arguments[1]);
        let value = invocation.arguments[2].clone();

        let mut runtime = self.runtime.borrow_mut();
        runtime.map_entry_keys.insert(
            (map_key.clone(), entry_key.clone()),
            invocation.arguments[1].clone(),
        );

        Some(
            runtime
                .map_entries
                .insert((map_key, entry_key), value)
                .unwrap_or(DexValue::Null),
        )
    }

    pub(super) fn remove_hash_map_value(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let map_key = map_identity_key(&invocation.arguments[0]);
        let entry_key = self.map_storage_entry_key(&invocation.arguments[1]);
        let mut runtime = self.runtime.borrow_mut();
        runtime
            .map_entry_keys
            .remove(&(map_key.clone(), entry_key.clone()));

        Some(
            runtime
                .map_entries
                .remove(&(map_key, entry_key))
                .unwrap_or(DexValue::Null),
        )
    }

    pub(super) fn java_read_hash_map_value(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<DexValue> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "Map.get(Object) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let value = self
            .java_hash_map_find_entry(
                &invocation.arguments[0],
                &invocation.arguments[1],
                depth + 1,
            )?
            .map(|(_entry_key, _stored_key, value)| value)
            .unwrap_or(DexValue::Null);

        Ok(value)
    }

    pub(super) fn java_write_hash_map_value(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<DexValue> {
        if invocation.arguments.len() != 3 {
            return Err(WinedroidError::new(format!(
                "Map.put(Object,Object) expected 3 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let map = &invocation.arguments[0];
        let key = invocation.arguments[1].clone();
        let value = invocation.arguments[2].clone();
        let map_key = map_identity_key(map);

        if let Some((entry_key, _stored_key, _stored_value)) =
            self.java_hash_map_find_entry(map, &key, depth + 1)?
        {
            let previous = self
                .runtime
                .borrow_mut()
                .map_entries
                .insert((map_key, entry_key), value)
                .unwrap_or(DexValue::Null);

            return Ok(previous);
        }

        let entry_key = self.map_storage_entry_key(&key);
        let mut runtime = self.runtime.borrow_mut();
        runtime
            .map_entry_keys
            .insert((map_key.clone(), entry_key.clone()), key);
        let previous = runtime
            .map_entries
            .insert((map_key, entry_key), value)
            .unwrap_or(DexValue::Null);

        Ok(previous)
    }

    pub(super) fn java_remove_hash_map_value(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<DexValue> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "Map.remove(Object) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let map = &invocation.arguments[0];
        let key = &invocation.arguments[1];
        let map_key = map_identity_key(map);
        let Some((entry_key, _stored_key, _stored_value)) =
            self.java_hash_map_find_entry(map, key, depth + 1)?
        else {
            return Ok(DexValue::Null);
        };

        let mut runtime = self.runtime.borrow_mut();
        runtime
            .map_entry_keys
            .remove(&(map_key.clone(), entry_key.clone()));
        let removed = runtime
            .map_entries
            .remove(&(map_key, entry_key))
            .unwrap_or(DexValue::Null);

        Ok(removed)
    }

    pub(super) fn clear_hash_map(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let map_key = map_identity_key(&invocation.arguments[0]);
        let mut runtime = self.runtime.borrow_mut();
        runtime
            .map_entries
            .retain(|(entry_map_key, _), _| entry_map_key != &map_key);
        runtime
            .map_entry_keys
            .retain(|(entry_map_key, _), _| entry_map_key != &map_key);

        Some(())
    }

    pub(super) fn hash_map_contains_key(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let map_key = map_identity_key(&invocation.arguments[0]);
        let entry_key = self.map_storage_entry_key(&invocation.arguments[1]);
        let contains_key = self
            .runtime
            .borrow()
            .map_entries
            .contains_key(&(map_key, entry_key));

        Some(i32::from(contains_key))
    }

    pub(super) fn java_hash_map_contains_key(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<i32> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "Map.containsKey(Object) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let contains_key = self
            .java_hash_map_find_entry(
                &invocation.arguments[0],
                &invocation.arguments[1],
                depth + 1,
            )?
            .is_some();

        Ok(i32::from(contains_key))
    }

    fn java_hash_map_find_entry(
        &self,
        map: &DexValue,
        needle: &DexValue,
        depth: usize,
    ) -> Result<Option<(String, DexValue, DexValue)>> {
        let map_key = map_identity_key(map);
        let direct_entry_key = self.map_storage_entry_key(needle);

        {
            let runtime = self.runtime.borrow();
            if let Some(value) = runtime
                .map_entries
                .get(&(map_key.clone(), direct_entry_key.clone()))
            {
                let stored_key = runtime
                    .map_entry_keys
                    .get(&(map_key.clone(), direct_entry_key.clone()))
                    .cloned()
                    .unwrap_or_else(|| needle.clone());

                return Ok(Some((direct_entry_key, stored_key, value.clone())));
            }
        }

        for (entry_key, stored_key, value) in self.runtime_map_entries_with_storage_key(map) {
            if self.runtime_values_equal(needle, &stored_key, depth + 1, true)? {
                return Ok(Some((entry_key, stored_key, value)));
            }
        }

        Ok(None)
    }

    fn runtime_map_entries_with_storage_key(
        &self,
        map: &DexValue,
    ) -> Vec<(String, DexValue, DexValue)> {
        let map_key = map_identity_key(map);
        let runtime = self.runtime.borrow();
        let mut entries = Vec::new();

        for ((entry_map_key, entry_key), value) in &runtime.map_entries {
            if entry_map_key != &map_key {
                continue;
            }
            let Some(key) = runtime
                .map_entry_keys
                .get(&(entry_map_key.clone(), entry_key.clone()))
            else {
                continue;
            };

            entries.push((entry_key.clone(), key.clone(), value.clone()));
        }

        entries
    }

    fn map_storage_entry_key(&self, value: &DexValue) -> String {
        if let Some(class_key) = self.class_backed_storage_key(value) {
            return class_key;
        }

        if let Some(value_key) = self.class_and_value_storage_key(value) {
            return value_key;
        }

        map_identity_key(value)
    }

    fn class_backed_storage_key(&self, value: &DexValue) -> Option<String> {
        let descriptor = value_class_descriptor(value)?;
        let class_field = DexFieldId {
            class_descriptor: descriptor.to_owned(),
            type_descriptor: "Ljava/lang/Class;".to_owned(),
            name: "a".to_owned(),
        };
        let DexValue::Reference(class_reference) =
            self.read_stored_instance_field(value, &class_field)?
        else {
            return None;
        };

        if !class_reference.starts_with("Ljava/lang/Class;:") {
            return None;
        }

        Some(format!("{descriptor}:class-backed:{class_reference}"))
    }

    fn class_and_value_storage_key(&self, value: &DexValue) -> Option<String> {
        let descriptor = value_class_descriptor(value)?;
        let owner_key = instance_field_owner_key(value)?;
        let class_prefix = format!("{descriptor}.a:");
        let value_prefix = format!("{descriptor}.b:");
        let mut class_part = None;
        let mut value_part = None;

        {
            let runtime = self.runtime.borrow();
            if let Some(fields) = runtime.fields_of(&owner_key) {
                for (field, field_value) in fields.iter() {
                    if field.starts_with(&class_prefix) {
                        class_part = Some(field_value.clone());
                    } else if field.starts_with(&value_prefix) {
                        value_part = Some(field_value.clone());
                    }
                }
            }
        }

        let class_part = self.class_backed_storage_key(&class_part?)?;
        let value_part = self.map_storage_entry_key(&value_part?);

        Some(format!(
            "{descriptor}:class-and-value-backed:{class_part}:{value_part}"
        ))
    }

    pub(super) fn initialize_hash_map_from_map(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let destination_key = map_identity_key(&invocation.arguments[0]);
        let source_key = map_identity_key(&invocation.arguments[1]);
        let mut copied_entries = Vec::new();

        {
            let runtime = self.runtime.borrow();
            for ((map_key, entry_key), value) in &runtime.map_entries {
                if map_key != &source_key {
                    continue;
                }

                let key = runtime
                    .map_entry_keys
                    .get(&(map_key.clone(), entry_key.clone()))?
                    .clone();
                copied_entries.push((entry_key.clone(), key, value.clone()));
            }
        }

        let mut runtime = self.runtime.borrow_mut();
        for (entry_key, key, value) in copied_entries {
            runtime
                .map_entry_keys
                .insert((destination_key.clone(), entry_key.clone()), key);
            runtime
                .map_entries
                .insert((destination_key.clone(), entry_key), value);
        }

        Some(())
    }

    pub(super) fn hash_map_size(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let map_key = map_identity_key(&invocation.arguments[0]);
        let size = self
            .runtime
            .borrow()
            .map_entries
            .keys()
            .filter(|(entry_map_key, _)| entry_map_key == &map_key)
            .count();

        i32::try_from(size).ok()
    }

    pub(super) fn hash_map_is_empty(&self, invocation: &DexInvocation) -> Option<i32> {
        let size = self.hash_map_size(invocation)?;

        Some(i32::from(size == 0))
    }

    pub(super) fn map_entry_set(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let map_key = map_identity_key(&invocation.arguments[0]);
        Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Ljava/util/Set;:mapEntrySet:{map_key}"
        ))))
    }

    pub(super) fn map_values(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let map_key = map_identity_key(&invocation.arguments[0]);
        let values = self
            .runtime
            .borrow()
            .map_entries
            .iter()
            .filter(|((entry_map_key, _), _)| entry_map_key == &map_key)
            .map(|(_, value)| value.clone())
            .collect::<Vec<_>>();
        let collection = self.allocate_object("Ljava/util/Collection;".to_owned());

        self.runtime
            .borrow_mut()
            .list_entries
            .insert(map_identity_key(&collection), values);

        Some(collection)
    }

    pub(super) fn map_key_set(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let map_key = map_identity_key(&invocation.arguments[0]);
        let keys = {
            let runtime = self.runtime.borrow();
            let mut keys = Vec::new();

            for (entry_map_key, entry_key) in runtime.map_entries.keys() {
                if entry_map_key != &map_key {
                    continue;
                }

                if let Some(key) = runtime
                    .map_entry_keys
                    .get(&(entry_map_key.clone(), entry_key.clone()))
                {
                    keys.push(key.clone());
                }
            }

            keys
        };
        let set = self.allocate_object("Ljava/util/Set;".to_owned());

        self.runtime
            .borrow_mut()
            .list_entries
            .insert(map_identity_key(&set), keys);

        Some(set)
    }

    pub(super) fn runtime_object_equals(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<i32> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "Object.equals(Object) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let allow_receiver_dispatch = matches!(
            invocation.kind,
            DexInvokeKind::Virtual | DexInvokeKind::Interface
        );
        let equal = self.runtime_values_equal(
            &invocation.arguments[0],
            &invocation.arguments[1],
            depth,
            allow_receiver_dispatch,
        )?;

        Ok(i32::from(equal))
    }

    pub(super) fn runtime_values_equal(
        &self,
        left: &DexValue,
        right: &DexValue,
        depth: usize,
        allow_receiver_dispatch: bool,
    ) -> Result<bool> {
        if left == right {
            return Ok(true);
        }

        if matches!(left, DexValue::Null) || matches!(right, DexValue::Null) {
            return Ok(false);
        }

        // cite: libcore String.java#equals
        if let Some(left_text) = self.string_text(left) {
            return Ok(self.string_text(right).as_deref() == Some(left_text.as_str()));
        }

        if let Some(equal) = self.runtime_map_equals(left, right, depth + 1)? {
            return Ok(equal);
        }

        if let Some(equal) = self.runtime_set_equals(left, right, depth + 1)? {
            return Ok(equal);
        }

        if allow_receiver_dispatch {
            if let Some(equal) = self.invoke_dex_equals(left, right, depth + 1)? {
                return Ok(equal);
            }
        }

        Ok(false)
    }

    fn runtime_map_equals(
        &self,
        left: &DexValue,
        right: &DexValue,
        depth: usize,
    ) -> Result<Option<bool>> {
        if !self.value_is_map_like(left) || !self.value_is_map_like(right) {
            return Ok(None);
        }

        let left_entries = self.runtime_map_entries(left);
        if left_entries.is_empty() && !self.runtime_map_is_directly_known(left) {
            if self.runtime_map_is_directly_known(right) {
                return self.runtime_map_equals(right, left, depth + 1);
            }

            return Ok(None);
        }

        let right_size = self.runtime_map_size(right, depth + 1)?;
        if right_size != left_entries.len() {
            return Ok(Some(false));
        }

        for (key, value) in left_entries {
            let Some(right_value) = self.runtime_map_get(right, &key, depth + 1)? else {
                return Ok(Some(false));
            };
            if !self.runtime_values_equal(&value, &right_value, depth + 1, true)? {
                return Ok(Some(false));
            }
        }

        Ok(Some(true))
    }

    fn runtime_map_entries(&self, map: &DexValue) -> Vec<(DexValue, DexValue)> {
        let map_key = map_identity_key(map);
        let runtime = self.runtime.borrow();
        let mut entries = Vec::new();

        for ((entry_map_key, entry_key), value) in &runtime.map_entries {
            if entry_map_key != &map_key {
                continue;
            }
            let Some(key) = runtime
                .map_entry_keys
                .get(&(entry_map_key.clone(), entry_key.clone()))
            else {
                continue;
            };

            entries.push((key.clone(), value.clone()));
        }

        entries
    }

    fn runtime_map_size(&self, map: &DexValue, depth: usize) -> Result<usize> {
        if self.runtime_map_is_directly_known(map) {
            let invocation = DexInvocation {
                kind: DexInvokeKind::Interface,
                range: false,
                method_index: 0,
                argument_registers: Default::default(),
                arguments: smallvec::smallvec![map.clone()],
            };
            let Some(size) = self.hash_map_size(&invocation) else {
                return Err(WinedroidError::new(format!(
                    "Map.size() failed for runtime map\nmap: {map}"
                )));
            };

            return usize::try_from(size).map_err(|_| {
                WinedroidError::new(format!("Map.size() returned a negative value\nmap: {map}"))
            });
        }

        let Some(result) =
            self.invoke_map_method(map, "size", "I", "I", Vec::new(), Vec::new(), depth + 1)?
        else {
            return Err(WinedroidError::new(format!(
                "Map.size() method was not found\nmap: {map}\nnext missing primitive: Java Map equality dispatch"
            )));
        };

        match result {
            DexExecutionResult::Int(value) => usize::try_from(value).map_err(|_| {
                WinedroidError::new(format!("Map.size() returned a negative value\nmap: {map}"))
            }),
            DexExecutionResult::Throw(value) => Err(WinedroidError::new(format!(
                "Map.size() threw while comparing maps\nmap: {map}\nthrowable: {value}"
            ))),
            DexExecutionResult::Void | DexExecutionResult::Value(_) => Err(WinedroidError::new(
                "Map.size() did not return an int while comparing maps",
            )),
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                Err(WinedroidError::new(
                    "Map.size() did not return an int while comparing maps",
                ))
            }
        }
    }

    fn runtime_map_get(
        &self,
        map: &DexValue,
        key: &DexValue,
        depth: usize,
    ) -> Result<Option<DexValue>> {
        if self.runtime_direct_map_uses_java_key_equality(map) {
            return Ok(self
                .java_hash_map_find_entry(map, key, depth + 1)?
                .map(|(_entry_key, _stored_key, value)| value));
        }

        if let Some(value) = self.runtime_map_get_direct(map, key) {
            return Ok(value);
        }

        let contains = self.runtime_map_contains_key(map, key, depth + 1)?;
        if !contains {
            return Ok(None);
        }

        let Some(result) = self.invoke_map_method(
            map,
            "get",
            "LL",
            "Ljava/lang/Object;",
            vec!["Ljava/lang/Object;".to_owned()],
            vec![key.clone()],
            depth + 1,
        )?
        else {
            return Err(WinedroidError::new(format!(
                "Map.get(Object) method was not found\nmap: {map}\nnext missing primitive: Java Map equality dispatch"
            )));
        };

        match result {
            DexExecutionResult::Value(value) => Ok(Some(value)),
            DexExecutionResult::Throw(value) => Err(WinedroidError::new(format!(
                "Map.get(Object) threw while comparing maps\nmap: {map}\nthrowable: {value}"
            ))),
            DexExecutionResult::Void | DexExecutionResult::Int(_) => Err(WinedroidError::new(
                "Map.get(Object) did not return an object while comparing maps",
            )),
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                Err(WinedroidError::new(
                    "Map.get(Object) did not return an object while comparing maps",
                ))
            }
        }
    }

    fn runtime_map_get_direct(&self, map: &DexValue, key: &DexValue) -> Option<Option<DexValue>> {
        let map_key = map_identity_key(map);
        let entry_key = self.map_storage_entry_key(key);
        let runtime = self.runtime.borrow();
        if let Some(value) = runtime.map_entries.get(&(map_key.clone(), entry_key)) {
            return Some(Some(value.clone()));
        }

        if runtime
            .map_entries
            .keys()
            .any(|(entry_map_key, _)| entry_map_key == &map_key)
            || self
                .runtime_map_descriptor(map)
                .map(|descriptor| is_supported_map_class(&descriptor))
                .unwrap_or(false)
        {
            return Some(None);
        }

        None
    }

    fn runtime_map_contains_key(
        &self,
        map: &DexValue,
        key: &DexValue,
        depth: usize,
    ) -> Result<bool> {
        if self.runtime_direct_map_uses_java_key_equality(map) {
            return Ok(self
                .java_hash_map_find_entry(map, key, depth + 1)?
                .is_some());
        }

        if let Some(value) = self.runtime_map_get_direct(map, key) {
            return Ok(value.is_some());
        }

        let Some(result) = self.invoke_map_method(
            map,
            "containsKey",
            "ZL",
            "Z",
            vec!["Ljava/lang/Object;".to_owned()],
            vec![key.clone()],
            depth + 1,
        )?
        else {
            return Err(WinedroidError::new(format!(
                "Map.containsKey(Object) method was not found\nmap: {map}\nnext missing primitive: Java Map equality dispatch"
            )));
        };

        match result {
            DexExecutionResult::Int(value) => Ok(value != 0),
            DexExecutionResult::Throw(value) => Err(WinedroidError::new(format!(
                "Map.containsKey(Object) threw while comparing maps\nmap: {map}\nthrowable: {value}"
            ))),
            DexExecutionResult::Void | DexExecutionResult::Value(_) => Err(WinedroidError::new(
                "Map.containsKey(Object) did not return a boolean while comparing maps",
            )),
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                Err(WinedroidError::new(
                    "Map.containsKey(Object) did not return a boolean while comparing maps",
                ))
            }
        }
    }

    fn invoke_map_method(
        &self,
        receiver: &DexValue,
        method_name: &str,
        shorty: &str,
        return_type: &str,
        parameters: Vec<String>,
        arguments: Vec<DexValue>,
        depth: usize,
    ) -> Result<Option<DexExecutionResult>> {
        if depth > MAX_INVOCATION_DEPTH {
            return Err(WinedroidError::new(format!(
                "Map equality dispatch depth exceeded {MAX_INVOCATION_DEPTH}"
            )));
        }

        let invocation = DexInvocation {
            kind: DexInvokeKind::Interface,
            range: false,
            method_index: 0,
            argument_registers: Default::default(),
            arguments: std::iter::once(receiver.clone()).chain(arguments).collect(),
        };

        if let Some(class_descriptor) = self.runtime_map_descriptor(receiver) {
            if !is_supported_map_class(&class_descriptor) {
                let method = DexMethodId {
                    class_descriptor: class_descriptor.clone(),
                    name: method_name.to_owned(),
                    proto: crate::dex::DexProto {
                        shorty: shorty.to_owned(),
                        return_type: return_type.to_owned(),
                        parameters: parameters.clone(),
                    },
                };
                if let Some(resolved) =
                    self.resolve_exact_method_in_class_hierarchy(&class_descriptor, &method)?
                {
                    let result = self.execute_method(
                        None,
                        &resolved,
                        Some(&invocation.arguments),
                        depth + 1,
                    )?;
                    return Ok(Some(result));
                }

                return Ok(None);
            }
        }

        let method = DexMethodId {
            class_descriptor: "Ljava/util/Map;".to_owned(),
            name: method_name.to_owned(),
            proto: crate::dex::DexProto {
                shorty: shorty.to_owned(),
                return_type: return_type.to_owned(),
                parameters,
            },
        };

        Ok(self.invoke_builtin_method(&method, Some(&invocation)))
    }

    fn invoke_dex_equals(
        &self,
        left: &DexValue,
        right: &DexValue,
        depth: usize,
    ) -> Result<Option<bool>> {
        if depth > MAX_INVOCATION_DEPTH {
            return Err(WinedroidError::new(format!(
                "Object.equals dispatch depth exceeded {MAX_INVOCATION_DEPTH}"
            )));
        }

        let Some(class_descriptor) = value_class_descriptor(left).map(str::to_owned) else {
            return Ok(None);
        };
        let method = DexMethodId {
            class_descriptor: class_descriptor.clone(),
            name: "equals".to_owned(),
            proto: crate::dex::DexProto {
                shorty: "ZL".to_owned(),
                return_type: "Z".to_owned(),
                parameters: vec!["Ljava/lang/Object;".to_owned()],
            },
        };
        let Some(resolved) =
            self.resolve_exact_method_in_class_hierarchy(&class_descriptor, &method)?
        else {
            return Ok(None);
        };
        let result = self.execute_method(
            None,
            &resolved,
            Some(&[left.clone(), right.clone()]),
            depth + 1,
        )?;

        match result {
            DexExecutionResult::Int(value) => Ok(Some(value != 0)),
            DexExecutionResult::Throw(value) => Err(WinedroidError::new(format!(
                "Object.equals(Object) threw\nreceiver: {left}\nargument: {right}\nthrowable: {value}"
            ))),
            DexExecutionResult::Void | DexExecutionResult::Value(_) => Err(WinedroidError::new(
                "Object.equals(Object) did not return a boolean",
            )),
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                Err(WinedroidError::new(
                    "Object.equals(Object) did not return a boolean",
                ))
            }
        }
    }

    fn value_is_map_like(&self, value: &DexValue) -> bool {
        if self.runtime_map_is_directly_known(value) {
            return true;
        }

        value_class_descriptor(value)
            .map(|descriptor| self.descriptor_matches_type(descriptor, "Ljava/util/Map;"))
            .unwrap_or(false)
    }

    fn runtime_map_is_directly_known(&self, value: &DexValue) -> bool {
        let map_key = map_identity_key(value);
        if self
            .runtime
            .borrow()
            .map_entries
            .keys()
            .any(|(entry_map_key, _)| entry_map_key == &map_key)
        {
            return true;
        }

        self.runtime_map_descriptor(value)
            .map(|descriptor| is_supported_map_class(&descriptor))
            .unwrap_or(false)
    }

    fn runtime_map_descriptor(&self, value: &DexValue) -> Option<String> {
        value_class_descriptor(value).map(str::to_owned)
    }

    fn runtime_direct_map_uses_java_key_equality(&self, value: &DexValue) -> bool {
        let Some(descriptor) = self.runtime_map_descriptor(value) else {
            return false;
        };

        is_java_key_equality_map_class(&descriptor)
    }

    fn runtime_set_equals(
        &self,
        left: &DexValue,
        right: &DexValue,
        depth: usize,
    ) -> Result<Option<bool>> {
        if !self.value_is_set_like(left) || !self.value_is_set_like(right) {
            return Ok(None);
        }

        let left_values = self.runtime_set_values(left);
        if left_values.is_empty() && !self.runtime_set_is_directly_known(left) {
            if self.runtime_set_is_directly_known(right) {
                return self.runtime_set_equals(right, left, depth + 1);
            }

            return Ok(None);
        }

        let right_size = self.runtime_set_size(right, depth + 1)?;
        if right_size != left_values.len() {
            return Ok(Some(false));
        }

        for value in left_values {
            if !self.runtime_set_contains(right, &value, depth + 1)? {
                return Ok(Some(false));
            }
        }

        Ok(Some(true))
    }

    fn runtime_set_values(&self, set: &DexValue) -> Vec<DexValue> {
        let set_key = map_identity_key(set);
        self.runtime
            .borrow()
            .list_entries
            .get(&set_key)
            .cloned()
            .unwrap_or_default()
    }

    fn runtime_set_size(&self, set: &DexValue, depth: usize) -> Result<usize> {
        if self.runtime_set_is_directly_known(set) {
            let invocation = DexInvocation {
                kind: DexInvokeKind::Interface,
                range: false,
                method_index: 0,
                argument_registers: Default::default(),
                arguments: smallvec::smallvec![set.clone()],
            };
            let Some(size) = self.set_size(Some(&invocation)) else {
                return Err(WinedroidError::new(format!(
                    "Set.size() failed for runtime set\nset: {set}"
                )));
            };

            return usize::try_from(size).map_err(|_| {
                WinedroidError::new(format!("Set.size() returned a negative value\nset: {set}"))
            });
        }

        let Some(result) =
            self.invoke_set_method(set, "size", "I", "I", Vec::new(), Vec::new(), depth + 1)?
        else {
            return Err(WinedroidError::new(format!(
                "Set.size() method was not found\nset: {set}\nnext missing primitive: Java Set equality dispatch"
            )));
        };

        match result {
            DexExecutionResult::Int(value) => usize::try_from(value).map_err(|_| {
                WinedroidError::new(format!("Set.size() returned a negative value\nset: {set}"))
            }),
            DexExecutionResult::Throw(value) => Err(WinedroidError::new(format!(
                "Set.size() threw while comparing sets\nset: {set}\nthrowable: {value}"
            ))),
            DexExecutionResult::Void | DexExecutionResult::Value(_) => Err(WinedroidError::new(
                "Set.size() did not return an int while comparing sets",
            )),
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                Err(WinedroidError::new(
                    "Set.size() did not return an int while comparing sets",
                ))
            }
        }
    }

    fn runtime_set_contains(&self, set: &DexValue, value: &DexValue, depth: usize) -> Result<bool> {
        if self.runtime_set_is_directly_known(set) {
            return self.java_collection_contains_equal(
                &self.runtime_set_values(set),
                value,
                depth + 1,
            );
        }

        let Some(result) = self.invoke_set_method(
            set,
            "contains",
            "ZL",
            "Z",
            vec!["Ljava/lang/Object;".to_owned()],
            vec![value.clone()],
            depth + 1,
        )?
        else {
            return Err(WinedroidError::new(format!(
                "Set.contains(Object) method was not found\nset: {set}\nnext missing primitive: Java Set equality dispatch"
            )));
        };

        match result {
            DexExecutionResult::Int(value) => Ok(value != 0),
            DexExecutionResult::Throw(value) => Err(WinedroidError::new(format!(
                "Set.contains(Object) threw while comparing sets\nset: {set}\nthrowable: {value}"
            ))),
            DexExecutionResult::Void | DexExecutionResult::Value(_) => Err(WinedroidError::new(
                "Set.contains(Object) did not return a boolean while comparing sets",
            )),
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                Err(WinedroidError::new(
                    "Set.contains(Object) did not return a boolean while comparing sets",
                ))
            }
        }
    }

    fn invoke_set_method(
        &self,
        receiver: &DexValue,
        method_name: &str,
        shorty: &str,
        return_type: &str,
        parameters: Vec<String>,
        arguments: Vec<DexValue>,
        depth: usize,
    ) -> Result<Option<DexExecutionResult>> {
        if depth > MAX_INVOCATION_DEPTH {
            return Err(WinedroidError::new(format!(
                "Set equality dispatch depth exceeded {MAX_INVOCATION_DEPTH}"
            )));
        }

        let invocation = DexInvocation {
            kind: DexInvokeKind::Interface,
            range: false,
            method_index: 0,
            argument_registers: Default::default(),
            arguments: std::iter::once(receiver.clone()).chain(arguments).collect(),
        };

        if let Some(class_descriptor) = self.runtime_set_descriptor(receiver) {
            if !is_supported_set_class(&class_descriptor) {
                let method = DexMethodId {
                    class_descriptor: class_descriptor.clone(),
                    name: method_name.to_owned(),
                    proto: crate::dex::DexProto {
                        shorty: shorty.to_owned(),
                        return_type: return_type.to_owned(),
                        parameters: parameters.clone(),
                    },
                };
                if let Some(resolved) =
                    self.resolve_exact_method_in_class_hierarchy(&class_descriptor, &method)?
                {
                    let result = self.execute_method(
                        None,
                        &resolved,
                        Some(&invocation.arguments),
                        depth + 1,
                    )?;
                    return Ok(Some(result));
                }

                return Ok(None);
            }
        }

        let method = DexMethodId {
            class_descriptor: "Ljava/util/Set;".to_owned(),
            name: method_name.to_owned(),
            proto: crate::dex::DexProto {
                shorty: shorty.to_owned(),
                return_type: return_type.to_owned(),
                parameters,
            },
        };

        Ok(self.invoke_builtin_method(&method, Some(&invocation)))
    }

    fn value_is_set_like(&self, value: &DexValue) -> bool {
        if self.runtime_set_is_directly_known(value) {
            return true;
        }

        value_class_descriptor(value)
            .map(|descriptor| self.descriptor_matches_type(descriptor, "Ljava/util/Set;"))
            .unwrap_or(false)
    }

    fn runtime_set_is_directly_known(&self, value: &DexValue) -> bool {
        let set_key = map_identity_key(value);
        if self.runtime.borrow().list_entries.contains_key(&set_key) {
            return true;
        }

        self.runtime_set_descriptor(value)
            .map(|descriptor| is_supported_set_class(&descriptor))
            .unwrap_or(false)
    }

    fn runtime_set_descriptor(&self, value: &DexValue) -> Option<String> {
        value_class_descriptor(value).map(str::to_owned)
    }

    pub(super) fn set_iterator(&self, invocation: Option<&DexInvocation>) -> Option<DexValue> {
        let Some(invocation) = invocation else {
            return Some(empty_iterator());
        };

        if invocation.arguments.len() != 1 {
            return None;
        }

        self.collection_iterator(&invocation.arguments[0])
    }

    pub(super) fn iterable_iterator(&self, invocation: Option<&DexInvocation>) -> Option<DexValue> {
        let Some(invocation) = invocation else {
            return Some(empty_iterator());
        };

        if invocation.arguments.len() != 1 {
            return None;
        }

        self.collection_iterator(&invocation.arguments[0])
    }

    pub(super) fn collection_iterator(&self, collection: &DexValue) -> Option<DexValue> {
        if let DexValue::Reference(reference) = collection {
            if let Some(map_key) = reference.strip_prefix("Ljava/util/Set;:mapEntrySet:") {
                return self.create_map_entry_iterator(map_key);
            }
        }

        let collection_key = map_identity_key(collection);
        let tracked = runtime_list_values(&self.runtime.borrow(), &collection_key);

        if tracked.is_none() && matches!(collection, DexValue::Object(_)) {
            return None;
        }

        let values = tracked.unwrap_or_default();
        if values.is_empty() {
            return Some(empty_iterator());
        }

        self.create_value_iterator_with_position(
            values,
            0,
            "Ljava/util/Iterator;",
            Some(collection_key),
        )
    }

    pub(super) fn list_iterator(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if let DexValue::Reference(reference) = &invocation.arguments[0] {
            if reference.starts_with("Ljava/util/List;:workInfosByTag:")
                && reference.ends_with(":empty")
            {
                return Some(empty_iterator());
            }
        }

        self.collection_iterator(&invocation.arguments[0])
    }

    pub(super) fn list_iterator_at(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let collection_key = map_identity_key(&invocation.arguments[0]);
        let position = primitive_int_argument(&invocation.arguments[1])?;
        if position < 0 {
            return None;
        }

        let tracked = self
            .runtime
            .borrow()
            .list_entries
            .get(&collection_key)
            .cloned();
        let values = match tracked {
            Some(values) => values,
            None if matches!(invocation.arguments[0], DexValue::Object(_)) => {
                match self.snapshot_dex_list_values(&invocation.arguments[0]) {
                    Ok(Some(values)) => values,
                    Ok(None) => return None,
                    Err(error) => {
                        eprintln!("List.listIterator snapshot failed\n  error: {error}");
                        return None;
                    }
                }
            }
            None => Vec::new(),
        };
        let position = usize::try_from(position).ok()?;
        if position > values.len() {
            return None;
        }

        self.create_value_iterator_with_position(
            values,
            position,
            "Ljava/util/ListIterator;",
            Some(collection_key),
        )
    }

    // cite: libcore AbstractList.java#listIterator, ListItr
    pub(super) fn dex_backed_list_iterator_at(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<Option<DexValue>> {
        if invocation.arguments.len() != 2 {
            return Ok(None);
        }

        let receiver = &invocation.arguments[0];
        let DexValue::Int(position) = invocation.arguments[1] else {
            return Ok(None);
        };
        let Ok(position) = usize::try_from(position) else {
            return Ok(None);
        };
        let Some(values) = self.dex_backed_list_snapshot(receiver, depth)? else {
            return Ok(None);
        };
        if position > values.len() {
            return Ok(None);
        }

        Ok(self.create_value_iterator_with_position(
            values,
            position,
            "Ljava/util/ListIterator;",
            None,
        ))
    }

    // cite: libcore AbstractList.java#iterator, Itr
    pub(super) fn dex_backed_list_iterator(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<Option<DexValue>> {
        if invocation.arguments.len() != 1 {
            return Ok(None);
        }

        let Some(values) = self.dex_backed_list_snapshot(&invocation.arguments[0], depth)? else {
            return Ok(None);
        };

        Ok(self.create_value_iterator_with_position(values, 0, "Ljava/util/Iterator;", None))
    }

    fn dex_backed_list_snapshot(
        &self,
        receiver: &DexValue,
        depth: usize,
    ) -> Result<Option<Vec<DexValue>>> {
        let Some(class_descriptor) = value_class_descriptor(receiver) else {
            return Ok(None);
        };
        let class_descriptor = class_descriptor.to_owned();

        let size_method = DexMethodId {
            class_descriptor: class_descriptor.clone(),
            name: "size".to_owned(),
            proto: DexProto {
                shorty: "I".to_owned(),
                return_type: "I".to_owned(),
                parameters: Vec::new(),
            },
        };
        let Some(resolved_size) =
            self.resolve_exact_method_in_class_hierarchy(&class_descriptor, &size_method)?
        else {
            return Ok(None);
        };
        let size_result = self.execute_method(
            None,
            &resolved_size,
            Some(std::slice::from_ref(receiver)),
            depth,
        )?;
        let size = match size_result {
            DexExecutionResult::Int(size) => size,
            DexExecutionResult::Value(DexValue::Int(size)) => size,
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                return Ok(None);
            }
            _ => return Ok(None),
        };
        let Ok(size) = usize::try_from(size) else {
            return Ok(None);
        };

        let get_method = DexMethodId {
            class_descriptor: class_descriptor.clone(),
            name: "get".to_owned(),
            proto: DexProto {
                shorty: "LI".to_owned(),
                return_type: "Ljava/lang/Object;".to_owned(),
                parameters: vec!["I".to_owned()],
            },
        };
        let Some(resolved_get) =
            self.resolve_exact_method_in_class_hierarchy(&class_descriptor, &get_method)?
        else {
            return Ok(None);
        };
        let mut values = Vec::with_capacity(size);
        for index in 0..size {
            let arguments = [receiver.clone(), DexValue::Int(index as i32)];
            let result = self.execute_method(None, &resolved_get, Some(&arguments), depth)?;
            let value = match result {
                DexExecutionResult::Value(value) => value,
                DexExecutionResult::Int(value) => DexValue::Int(value),
                DexExecutionResult::Parked(execution) => {
                    self.discard_parked_execution(&execution);
                    return Ok(None);
                }
                _ => return Ok(None),
            };
            values.push(value);
        }

        Ok(Some(values))
    }

    // cite: libcore AbstractList.java#iterator, Itr
    pub(super) fn abstract_list_snapshot_iterator(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !matches!(invocation.arguments[0], DexValue::Object(_)) {
            return None;
        }

        let values = match self.snapshot_dex_list_values(&invocation.arguments[0]) {
            Ok(Some(values)) => values,
            Ok(None) => return None,
            Err(error) => {
                eprintln!("AbstractList.iterator snapshot failed\n  error: {error}");
                return None;
            }
        };

        self.create_value_iterator_with_position(values, 0, "Ljava/util/Iterator;", None)
    }

    // cite: libcore AbstractList.java#listIterator, ListItr
    fn snapshot_dex_list_values(&self, list: &DexValue) -> Result<Option<Vec<DexValue>>> {
        let Some(class_descriptor) = value_class_descriptor(list).map(str::to_owned) else {
            return Ok(None);
        };
        let size_method = DexMethodId {
            class_descriptor: class_descriptor.clone(),
            name: "size".to_owned(),
            proto: crate::dex::DexProto {
                shorty: "I".to_owned(),
                return_type: "I".to_owned(),
                parameters: Vec::new(),
            },
        };
        let Some(resolved_size) =
            self.resolve_exact_method_in_class_hierarchy(&class_descriptor, &size_method)?
        else {
            return Ok(None);
        };
        let size_result = self.execute_method(None, &resolved_size, Some(&[list.clone()]), 0)?;
        let size = match size_result {
            DexExecutionResult::Int(size) => size,
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                return Err(WinedroidError::new(format!(
                    "List.size() did not return an int\nlist class: {class_descriptor}"
                )));
            }
            _ => {
                return Err(WinedroidError::new(format!(
                    "List.size() did not return an int\nlist class: {class_descriptor}"
                )));
            }
        };

        let get_method = DexMethodId {
            class_descriptor: class_descriptor.clone(),
            name: "get".to_owned(),
            proto: crate::dex::DexProto {
                shorty: "LI".to_owned(),
                return_type: "Ljava/lang/Object;".to_owned(),
                parameters: vec!["I".to_owned()],
            },
        };
        let Some(resolved_get) =
            self.resolve_exact_method_in_class_hierarchy(&class_descriptor, &get_method)?
        else {
            return self.snapshot_dex_iterable_values(list, size);
        };

        let mut values = Vec::with_capacity(usize::try_from(size).unwrap_or(0));
        for index in 0..size {
            let result = self.execute_method(
                None,
                &resolved_get,
                Some(&[list.clone(), DexValue::Int(index)]),
                0,
            )?;
            let value = match result {
                DexExecutionResult::Value(value) => value,
                DexExecutionResult::Int(value) => DexValue::Int(value),
                DexExecutionResult::Void | DexExecutionResult::Throw(_) => {
                    return Err(WinedroidError::new(format!(
                        "List.get(int) did not complete normally\nlist class: {class_descriptor}"
                    )));
                }
                DexExecutionResult::Parked(execution) => {
                    self.discard_parked_execution(&execution);
                    return Err(WinedroidError::new(format!(
                        "List.get(int) did not complete normally\nlist class: {class_descriptor}"
                    )));
                }
            };
            values.push(value);
        }

        Ok(Some(values))
    }

    // cite: libcore AbstractCollection.java#toArray
    fn snapshot_dex_iterable_values(
        &self,
        collection: &DexValue,
        size: i32,
    ) -> Result<Option<Vec<DexValue>>> {
        let Some(class_descriptor) = value_class_descriptor(collection).map(str::to_owned) else {
            return Ok(None);
        };
        let iterator_method = DexMethodId {
            class_descriptor: class_descriptor.clone(),
            name: "iterator".to_owned(),
            proto: crate::dex::DexProto {
                shorty: "L".to_owned(),
                return_type: "Ljava/util/Iterator;".to_owned(),
                parameters: Vec::new(),
            },
        };
        let Some(resolved_iterator) =
            self.resolve_exact_method_in_class_hierarchy(&class_descriptor, &iterator_method)?
        else {
            return Err(WinedroidError::new(format!(
                "Collection iteration was not possible: neither get(int) nor iterator() was found\ncollection class: {class_descriptor}"
            )));
        };
        let iterator_result =
            self.execute_method(None, &resolved_iterator, Some(&[collection.clone()]), 0)?;
        let iterator = match iterator_result {
            DexExecutionResult::Value(value) => value,
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                return Err(WinedroidError::new(format!(
                    "Collection.iterator() did not return an iterator\ncollection class: {class_descriptor}\nresult: parked"
                )));
            }
            other => {
                return Err(WinedroidError::new(format!(
                    "Collection.iterator() did not return an iterator\ncollection class: {class_descriptor}\nresult: {other:?}"
                )))
            }
        };

        let mut values = Vec::with_capacity(usize::try_from(size).unwrap_or(0));
        loop {
            let has_next_result = self.call_iterator_method(&iterator, "hasNext", "Z")?;
            let has_next = match has_next_result {
                DexExecutionResult::Int(has_next) => has_next,
                DexExecutionResult::Parked(execution) => {
                    self.discard_parked_execution(&execution);
                    return Err(WinedroidError::new(
                        "Iterator.hasNext() did not return a boolean".to_owned(),
                    ));
                }
                _ => {
                    return Err(WinedroidError::new(
                        "Iterator.hasNext() did not return a boolean".to_owned(),
                    ));
                }
            };
            if has_next == 0 {
                break;
            }
            let value = match self.call_iterator_method(&iterator, "next", "L")? {
                DexExecutionResult::Value(value) => value,
                DexExecutionResult::Int(value) => DexValue::Int(value),
                DexExecutionResult::Parked(execution) => {
                    self.discard_parked_execution(&execution);
                    return Err(WinedroidError::new(
                        "Iterator.next() did not return a value\nresult: parked".to_owned(),
                    ));
                }
                other => {
                    return Err(WinedroidError::new(format!(
                        "Iterator.next() did not return a value\nresult: {other:?}"
                    )))
                }
            };
            values.push(value);
            if values.len() > usize::try_from(size.max(0)).unwrap_or(0) {
                return Err(WinedroidError::new(format!(
                    "Collection iterator returned more elements than size()\ncollection class: {class_descriptor}"
                )));
            }
        }

        Ok(Some(values))
    }

    fn call_iterator_method(
        &self,
        iterator: &DexValue,
        name: &str,
        shorty: &str,
    ) -> Result<DexExecutionResult> {
        let (return_type, parameters) = match shorty {
            "Z" => ("Z".to_owned(), Vec::new()),
            _ => ("Ljava/lang/Object;".to_owned(), Vec::new()),
        };
        let method = DexMethodId {
            class_descriptor: "Ljava/util/Iterator;".to_owned(),
            name: name.to_owned(),
            proto: crate::dex::DexProto {
                shorty: shorty.to_owned(),
                return_type,
                parameters,
            },
        };

        if let Some(receiver_descriptor) = value_class_descriptor(iterator) {
            let receiver_method = DexMethodId {
                class_descriptor: receiver_descriptor.to_owned(),
                ..method.clone()
            };
            if let Some(resolved) =
                self.resolve_exact_method_in_class_hierarchy(receiver_descriptor, &receiver_method)?
            {
                return self.execute_method(None, &resolved, Some(&[iterator.clone()]), 0);
            }
        }

        let invocation = DexInvocation {
            kind: crate::dex_interpreter::DexInvokeKind::Interface,
            range: false,
            method_index: 0,
            argument_registers: Default::default(),
            arguments: smallvec::smallvec![iterator.clone()],
        };
        self.invoke_builtin_method(&method, Some(&invocation))
            .ok_or_else(|| {
                WinedroidError::new(format!(
                    "Iterator.{name}() was not resolvable\niterator: {iterator}"
                ))
            })
    }

    fn create_value_iterator_with_position(
        &self,
        values: Vec<DexValue>,
        position: usize,
        descriptor: &str,
        source_key: Option<String>,
    ) -> Option<DexValue> {
        let mut runtime = self.runtime.borrow_mut();
        let iterator_key = format!("iterator:{}", runtime.next_object_id);
        runtime.next_object_id += 1;

        runtime
            .iterator_entries
            .insert(iterator_key.clone(), values);
        runtime
            .iterator_positions
            .insert(iterator_key.clone(), position);
        if let Some(source_key) = source_key {
            runtime
                .iterator_sources
                .insert(iterator_key.clone(), source_key);
        }

        Some(DexValue::Reference(std::rc::Rc::from(format!(
            "{descriptor}:{iterator_key}"
        ))))
    }

    pub(super) fn set_size(&self, invocation: Option<&DexInvocation>) -> Option<i32> {
        let Some(invocation) = invocation else {
            return Some(0);
        };

        if invocation.arguments.len() != 1 {
            return None;
        }

        if let DexValue::Reference(reference) = &invocation.arguments[0] {
            if let Some(map_key) = reference.strip_prefix("Ljava/util/Set;:mapEntrySet:") {
                let size = self
                    .runtime
                    .borrow()
                    .map_entries
                    .keys()
                    .filter(|(entry_map_key, _)| entry_map_key == map_key)
                    .count();

                return i32::try_from(size).ok();
            }
        }

        let Some(size) = self.managed_list_len_or_empty(&invocation.arguments[0], true) else {
            return None;
        };

        i32::try_from(size).ok()
    }

    pub(super) fn collection_is_empty(&self, invocation: Option<&DexInvocation>) -> Option<i32> {
        let Some(invocation) = invocation else {
            return Some(1);
        };

        if invocation.arguments.len() != 1 {
            return None;
        }

        let Some(size) = self.managed_list_len_or_empty(&invocation.arguments[0], false) else {
            return None;
        };

        Some(i32::from(size == 0))
    }

    pub(super) fn list_size(&self, invocation: Option<&DexInvocation>) -> Option<i32> {
        let Some(invocation) = invocation else {
            return Some(0);
        };

        if invocation.arguments.len() != 1 {
            return None;
        }

        let Some(size) = self.managed_list_len_or_empty(&invocation.arguments[0], false) else {
            return None;
        };

        i32::try_from(size).ok()
    }

    pub(super) fn list_is_empty(&self, invocation: Option<&DexInvocation>) -> Option<i32> {
        let Some(invocation) = invocation else {
            return Some(1);
        };

        if invocation.arguments.len() != 1 {
            return None;
        }

        let Some(size) = self.managed_list_len_or_empty(&invocation.arguments[0], false) else {
            return None;
        };

        Some(i32::from(size == 0))
    }

    pub(super) fn concrete_list_size(&self, invocation: Option<&DexInvocation>) -> Option<i32> {
        let Some(invocation) = invocation else {
            return Some(0);
        };

        if invocation.arguments.len() != 1 {
            return None;
        }

        let Some(size) = self.managed_list_len_or_empty(&invocation.arguments[0], true) else {
            return None;
        };

        i32::try_from(size).ok()
    }

    pub(super) fn concrete_list_is_empty(&self, invocation: Option<&DexInvocation>) -> Option<i32> {
        let Some(invocation) = invocation else {
            return Some(1);
        };

        if invocation.arguments.len() != 1 {
            return None;
        }

        let Some(size) = self.managed_list_len_or_empty(&invocation.arguments[0], true) else {
            return None;
        };

        Some(i32::from(size == 0))
    }

    fn managed_list_len_or_empty(
        &self,
        value: &DexValue,
        object_without_entries_is_empty: bool,
    ) -> Option<usize> {
        let list_key = map_identity_key(value);
        {
            let runtime = self.runtime.borrow();
            if let Some(size) = runtime_list_len(&runtime, &list_key) {
                return Some(size);
            }
        }

        match value {
            DexValue::Object(object)
                if is_managed_concrete_list_descriptor(&object.class_descriptor) =>
            {
                Some(0)
            }
            DexValue::Object(_) if object_without_entries_is_empty => Some(0),
            DexValue::Object(_) => None,
            DexValue::Reference(_)
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_)
            | DexValue::Uninitialized => Some(0),
        }
    }

    pub(super) fn add_collection_value(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let collection_key = map_identity_key(&invocation.arguments[0]);
        let value = invocation.arguments[1].clone();

        self.runtime
            .borrow_mut()
            .list_entries
            .entry(collection_key)
            .or_default()
            .push(value);

        Some(())
    }

    pub(super) fn array_deque_add_last<A: MethodIntrinsicArguments + ?Sized>(
        &self,
        invocation: &A,
    ) -> Result<Option<()>> {
        if invocation.argument_count() != 2 {
            return Ok(None);
        }

        let Some(receiver) = invocation.argument_value(0)? else {
            return Ok(None);
        };
        if !dex_value_is_object_like(&receiver) {
            return Ok(None);
        }

        let Some(value) = invocation.argument_value(1)? else {
            return Ok(None);
        };
        if matches!(value, DexValue::Null | DexValue::Uninitialized) {
            return Ok(None);
        }

        self.runtime
            .borrow_mut()
            .list_entries
            .entry(map_identity_key(&receiver))
            .or_default()
            .push(value);

        Ok(Some(()))
    }

    pub(super) fn add_set_value(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let set_key = map_identity_key(&invocation.arguments[0]);
        let value = invocation.arguments[1].clone();
        let mut runtime = self.runtime.borrow_mut();
        let values = runtime.list_entries.entry(set_key).or_default();
        if values.contains(&value) {
            return Some(0);
        }

        values.push(value);
        Some(1)
    }

    pub(super) fn java_add_set_value(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<i32> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "Set.add(Object) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let set_key = map_identity_key(&invocation.arguments[0]);
        let value = invocation.arguments[1].clone();
        let mut values = self
            .runtime
            .borrow()
            .list_entries
            .get(&set_key)
            .cloned()
            .unwrap_or_default();

        if self.java_collection_contains_equal(&values, &value, depth + 1)? {
            return Ok(0);
        }

        values.push(value);
        self.runtime
            .borrow_mut()
            .list_entries
            .insert(set_key, values);

        Ok(1)
    }

    pub(super) fn priority_queue_add(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        match &invocation.arguments[1] {
            DexValue::Object(_) | DexValue::Reference(_) | DexValue::String(_) => {}
            DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::Uninitialized => return None,
        }

        self.add_collection_value(invocation)?;

        Some(1)
    }

    pub(super) fn initialize_tree_set(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 && invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let set_key = map_identity_key(&invocation.arguments[0]);
        let comparator = invocation
            .arguments
            .get(1)
            .cloned()
            .unwrap_or(DexValue::Null);
        let mut runtime = self.runtime.borrow_mut();
        runtime.list_entries.entry(set_key.clone()).or_default();
        runtime.tree_set_comparators.insert(set_key, comparator);

        Some(())
    }

    pub(super) fn initialize_priority_queue(&self, invocation: &DexInvocation) -> Option<()> {
        if !matches!(invocation.arguments.len(), 1 | 2 | 3) {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        if invocation.arguments.len() >= 2 {
            match &invocation.arguments[1] {
                DexValue::Int(_) if invocation.arguments.len() != 2 => {}
                DexValue::Int(_) => {}
                value if invocation.arguments.len() == 2 && dex_value_is_object_like(value) => {}
                DexValue::Null if invocation.arguments.len() == 2 => {}
                _ => return None,
            }
        }

        if invocation.arguments.len() == 3 {
            match &invocation.arguments[2] {
                DexValue::Null | DexValue::Int(0) => {}
                value if dex_value_is_object_like(value) => {}
                _ => return None,
            }
        }

        let values = if invocation.arguments.len() == 2
            && !matches!(invocation.arguments[1], DexValue::Int(_))
        {
            self.projected_collection_values(&invocation.arguments[1])
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let queue_key = map_identity_key(&invocation.arguments[0]);
        self.runtime
            .borrow_mut()
            .list_entries
            .insert(queue_key, values);

        Some(())
    }

    pub(super) fn tree_set_add_value(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<i32> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "TreeSet.add(Object) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let set_key = map_identity_key(&invocation.arguments[0]);
        let value = invocation.arguments[1].clone();
        let (comparator, mut values) = {
            let runtime = self.runtime.borrow();
            let comparator = runtime
                .tree_set_comparators
                .get(&set_key)
                .cloned()
                .unwrap_or(DexValue::Null);
            let values = runtime
                .list_entries
                .get(&set_key)
                .cloned()
                .unwrap_or_default();

            (comparator, values)
        };

        if comparator == DexValue::Null {
            return Err(WinedroidError::new(
                "TreeSet.add(Object) natural ordering is not implemented\nnext missing primitive: TreeSet natural ordering",
            ));
        }

        let mut insert_at = values.len();
        for (index, existing) in values.iter().enumerate() {
            let order = self.compare_with_comparator(&comparator, &value, existing, depth + 1)?;
            if order == 0 {
                return Ok(0);
            }
            if order < 0 {
                insert_at = index;
                break;
            }
        }

        values.insert(insert_at, value);
        self.runtime
            .borrow_mut()
            .list_entries
            .insert(set_key, values);

        Ok(1)
    }

    pub(super) fn tree_set_first(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let set_key = map_identity_key(&invocation.arguments[0]);
        let first = self
            .runtime
            .borrow()
            .list_entries
            .get(&set_key)
            .and_then(|values| values.first().cloned())
            .unwrap_or(DexValue::Null);

        Some(first)
    }

    pub(super) fn initialize_array_list_from_collection(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let target_key = map_identity_key(&invocation.arguments[0]);
        let source_values = match self.projected_collection_values(&invocation.arguments[1]) {
            Some(values) => values,
            None if matches!(invocation.arguments[1], DexValue::Object(_)) => {
                match self.snapshot_dex_list_values(&invocation.arguments[1]) {
                    Ok(Some(values)) => values,
                    Ok(None) => return None,
                    Err(error) => {
                        eprintln!("ArrayList(Collection) snapshot failed\n  error: {error}");
                        return None;
                    }
                }
            }
            None => Vec::new(),
        };

        self.runtime
            .borrow_mut()
            .list_entries
            .insert(target_key, source_values);

        Some(())
    }

    pub(super) fn initialize_empty_list(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 && invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        if let Some(DexValue::Int(capacity)) = invocation.arguments.get(1) {
            if *capacity < 0 {
                return None;
            }
        }

        self.runtime
            .borrow_mut()
            .list_entries
            .insert(map_identity_key(&invocation.arguments[0]), Vec::new());

        Some(())
    }

    pub(super) fn initialize_array_list_from_collection_iterating(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<()> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "ArrayList(Collection) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let target_key = map_identity_key(&invocation.arguments[0]);
        let source_values =
            self.collection_values_by_contract(&invocation.arguments[1], depth + 1)?;

        self.runtime
            .borrow_mut()
            .list_entries
            .insert(target_key, source_values);

        Ok(())
    }

    pub(super) fn initialize_set_from_collection(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<()> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "Set(Collection) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let target_key = map_identity_key(&invocation.arguments[0]);
        let source_values =
            self.collection_values_by_contract(&invocation.arguments[1], depth + 1)?;
        let mut set_values = Vec::new();
        for value in source_values {
            if !self.java_collection_contains_equal(&set_values, &value, depth + 1)? {
                set_values.push(value);
            }
        }

        self.runtime
            .borrow_mut()
            .list_entries
            .insert(target_key, set_values);

        Ok(())
    }

    pub(super) fn initialize_empty_set_with_capacity(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(capacity) = invocation.arguments[1] else {
            return None;
        };
        if capacity < 0 {
            return None;
        }

        self.runtime
            .borrow_mut()
            .list_entries
            .insert(map_identity_key(&invocation.arguments[0]), Vec::new());

        Some(())
    }

    pub(super) fn initialize_empty_set(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        self.runtime
            .borrow_mut()
            .list_entries
            .insert(map_identity_key(&invocation.arguments[0]), Vec::new());

        Some(())
    }

    fn array_backed_collection_values(&self, value: &DexValue) -> Option<Vec<DexValue>> {
        let owner_key = instance_field_owner_key(value)?;
        let runtime = self.runtime.borrow();
        let mut array_values = Vec::new();

        let fields = runtime.fields_of(&owner_key)?;
        for field_value in fields.values() {
            let Some(descriptor) = value_class_descriptor(field_value) else {
                continue;
            };
            if !descriptor.starts_with('[') {
                continue;
            }

            let array_key = map_identity_key(field_value);
            let values = runtime.array_entries.get(&array_key)?;
            array_values.push(values.clone());
        }

        if array_values.len() != 1 {
            return None;
        }

        array_values.pop()
    }

    fn projected_collection_values(&self, value: &DexValue) -> Option<Vec<DexValue>> {
        // cite: libcore Collections.java#EMPTY_LIST, EMPTY_SET
        if let DexValue::Reference(reference) = value {
            if matches!(
                reference.as_ref(),
                "Ljava/util/List;:collections:emptyList" | "Ljava/util/Set;:collections:emptySet"
            ) {
                return Some(Vec::new());
            }
        }
        let list_key = map_identity_key(value);
        {
            let runtime = self.runtime.borrow();
            if let Some(values) = runtime_list_values(&runtime, &list_key) {
                return Some(values);
            }
        }

        // cite: libcore AbstractList.java#iterator, Itr
        if matches!(value, DexValue::Object(_)) {
            match self.snapshot_dex_list_values(value) {
                Ok(Some(values)) => return Some(values),
                Ok(None) => {}
                Err(error) => {
                    eprintln!("collection projection snapshot failed\n  error: {error}");
                    return None;
                }
            }
        }

        self.array_backed_collection_values(value)
    }

    fn collection_values_by_contract(
        &self,
        value: &DexValue,
        depth: usize,
    ) -> Result<Vec<DexValue>> {
        if let Some(values) = self.projected_collection_values(value) {
            return Ok(values);
        }

        self.dex_iterable_values(value, depth + 1)
    }

    fn dex_iterable_values(&self, value: &DexValue, depth: usize) -> Result<Vec<DexValue>> {
        if depth > MAX_INVOCATION_DEPTH {
            return Err(WinedroidError::new(format!(
                "Collection iterator dispatch depth exceeded {MAX_INVOCATION_DEPTH}"
            )));
        }

        if value == &DexValue::Null {
            return Err(WinedroidError::new(
                "ArrayList(Collection) received null source collection",
            ));
        }

        let Some(class_descriptor) = value_class_descriptor(value).map(str::to_owned) else {
            return Err(WinedroidError::new(format!(
                "ArrayList(Collection) source is not an object-backed collection\nsource: {value}"
            )));
        };
        let iterator_method = DexMethodId {
            class_descriptor: class_descriptor.clone(),
            name: "iterator".to_owned(),
            proto: crate::dex::DexProto {
                shorty: "L".to_owned(),
                return_type: "Ljava/util/Iterator;".to_owned(),
                parameters: Vec::new(),
            },
        };
        let Some(resolved) =
            self.resolve_exact_method_in_class_hierarchy(&class_descriptor, &iterator_method)?
        else {
            // cite: libcore AbstractList.java#iterator, Itr
            if let Some(values) = self.dex_backed_list_snapshot(value, depth + 1)? {
                return Ok(values);
            }

            return Err(WinedroidError::new(format!(
                "ArrayList(Collection) source does not expose iterator()\nsource class: {class_descriptor}"
            )));
        };
        let result = self.execute_method(
            None,
            &resolved,
            Some(std::slice::from_ref(value)),
            depth + 1,
        )?;
        let iterator = match result {
            DexExecutionResult::Value(iterator) if dex_value_is_object_like(&iterator) => iterator,
            DexExecutionResult::Throw(value) => {
                return Err(WinedroidError::new(format!(
                    "Collection.iterator() threw while constructing ArrayList\nsource: {value}"
                )));
            }
            DexExecutionResult::Void => {
                return Err(WinedroidError::new(
                    "Collection.iterator() returned void while constructing ArrayList",
                ));
            }
            DexExecutionResult::Value(value) => {
                return Err(WinedroidError::new(format!(
                    "Collection.iterator() returned a non-iterator value\nvalue: {value}"
                )));
            }
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                return Err(WinedroidError::new(
                    "Collection.iterator() parked while constructing ArrayList",
                ));
            }
            DexExecutionResult::Int(value) => {
                return Err(WinedroidError::new(format!(
                    "Collection.iterator() returned int {value} while constructing ArrayList"
                )));
            }
        };

        let mut values = Vec::new();
        while self.dex_iterator_has_next(&iterator, depth + 1)? {
            values.push(self.dex_iterator_next(&iterator, depth + 1)?);
        }

        Ok(values)
    }

    fn dex_iterator_has_next(&self, iterator: &DexValue, depth: usize) -> Result<bool> {
        if let DexValue::Reference(reference) = iterator {
            if iterator_storage_key(reference).is_some() {
                let invocation = DexInvocation {
                    kind: DexInvokeKind::Interface,
                    range: false,
                    method_index: 0,
                    argument_registers: Default::default(),
                    arguments: smallvec::smallvec![iterator.clone()],
                };
                return Ok(self.iterator_has_next(Some(&invocation)).unwrap_or(0) != 0);
            }
        }

        let Some(class_descriptor) = value_class_descriptor(iterator).map(str::to_owned) else {
            return Err(WinedroidError::new(format!(
                "Iterator.hasNext() receiver is not object-backed\niterator: {iterator}"
            )));
        };
        let method = DexMethodId {
            class_descriptor: class_descriptor.clone(),
            name: "hasNext".to_owned(),
            proto: crate::dex::DexProto {
                shorty: "Z".to_owned(),
                return_type: "Z".to_owned(),
                parameters: Vec::new(),
            },
        };
        let Some(resolved) =
            self.resolve_exact_method_in_class_hierarchy(&class_descriptor, &method)?
        else {
            return Err(WinedroidError::new(format!(
                "Iterator.hasNext() was not found\niterator class: {class_descriptor}"
            )));
        };
        let result = self.execute_method(
            None,
            &resolved,
            Some(std::slice::from_ref(iterator)),
            depth + 1,
        )?;

        match result {
            DexExecutionResult::Int(value) => Ok(value != 0),
            DexExecutionResult::Throw(value) => Err(WinedroidError::new(format!(
                "Iterator.hasNext() threw while constructing ArrayList\niterator: {iterator}\nthrowable: {value}"
            ))),
            DexExecutionResult::Void => Err(WinedroidError::new(
                "Iterator.hasNext() returned void while constructing ArrayList",
            )),
            DexExecutionResult::Value(value) => Err(WinedroidError::new(format!(
                "Iterator.hasNext() returned an object while constructing ArrayList\nvalue: {value}"
            ))),
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                Err(WinedroidError::new(
                    "Iterator.hasNext() parked while constructing ArrayList",
                ))
            }
        }
    }

    fn dex_iterator_next(&self, iterator: &DexValue, depth: usize) -> Result<DexValue> {
        if let DexValue::Reference(reference) = iterator {
            if iterator_storage_key(reference).is_some() {
                let invocation = DexInvocation {
                    kind: DexInvokeKind::Interface,
                    range: false,
                    method_index: 0,
                    argument_registers: Default::default(),
                    arguments: smallvec::smallvec![iterator.clone()],
                };
                return self.iterator_next(&invocation).ok_or_else(|| {
                    WinedroidError::new(
                        "Iterator.next() reached the end while constructing ArrayList",
                    )
                });
            }
        }

        let Some(class_descriptor) = value_class_descriptor(iterator).map(str::to_owned) else {
            return Err(WinedroidError::new(format!(
                "Iterator.next() receiver is not object-backed\niterator: {iterator}"
            )));
        };
        let method = DexMethodId {
            class_descriptor: class_descriptor.clone(),
            name: "next".to_owned(),
            proto: crate::dex::DexProto {
                shorty: "L".to_owned(),
                return_type: "Ljava/lang/Object;".to_owned(),
                parameters: Vec::new(),
            },
        };
        let Some(resolved) =
            self.resolve_exact_method_in_class_hierarchy(&class_descriptor, &method)?
        else {
            return Err(WinedroidError::new(format!(
                "Iterator.next() was not found\niterator class: {class_descriptor}"
            )));
        };
        let result = self.execute_method(
            None,
            &resolved,
            Some(std::slice::from_ref(iterator)),
            depth + 1,
        )?;

        match result {
            DexExecutionResult::Value(value) => Ok(value),
            DexExecutionResult::Int(value) => Err(WinedroidError::new(format!(
                "Iterator.next() returned int {value} while constructing ArrayList"
            ))),
            DexExecutionResult::Throw(value) => Err(WinedroidError::new(format!(
                "Iterator.next() threw while constructing ArrayList\niterator: {iterator}\nthrowable: {value}"
            ))),
            DexExecutionResult::Void => Err(WinedroidError::new(
                "Iterator.next() returned void while constructing ArrayList",
            )),
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                Err(WinedroidError::new(
                    "Iterator.next() parked while constructing ArrayList",
                ))
            }
        }
    }

    // cite: libcore AbstractCollection.java#retainAll
    pub(super) fn retain_all_collection_values(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let collection_key = map_identity_key(&invocation.arguments[0]);
        let retained_values = self.projected_collection_values(&invocation.arguments[1])?;
        let mut runtime = self.runtime.borrow_mut();
        let entries = runtime.list_entries.get_mut(&collection_key)?;
        let before = entries.len();
        entries.retain(|value| retained_values.contains(value));

        Some(i32::from(before != entries.len()))
    }

    pub(super) fn remove_all_collection_values(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let collection_key = map_identity_key(&invocation.arguments[0]);
        let removed_key = map_identity_key(&invocation.arguments[1]);
        let removed_values = self
            .runtime
            .borrow()
            .list_entries
            .get(&removed_key)
            .cloned()
            .unwrap_or_default();

        if removed_values.is_empty() {
            return Some(0);
        }

        let mut runtime = self.runtime.borrow_mut();
        let values = runtime.list_entries.entry(collection_key).or_default();
        let original_len = values.len();

        values.retain(|value| !removed_values.contains(value));

        Some(i32::from(values.len() != original_len))
    }

    pub(super) fn java_remove_all_collection_values(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<i32> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "Collection.removeAll(Collection) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let collection_key = map_identity_key(&invocation.arguments[0]);
        let removed_values = self
            .projected_collection_values(&invocation.arguments[1])
            .unwrap_or_default();

        if removed_values.is_empty() {
            return Ok(0);
        }

        let original_values = self
            .runtime
            .borrow()
            .list_entries
            .get(&collection_key)
            .cloned()
            .unwrap_or_default();
        let mut kept_values = Vec::with_capacity(original_values.len());
        let mut removed_any = false;

        for value in original_values {
            if self.java_collection_contains_equal(&removed_values, &value, depth + 1)? {
                removed_any = true;
            } else {
                kept_values.push(value);
            }
        }

        if removed_any {
            self.runtime
                .borrow_mut()
                .list_entries
                .insert(collection_key, kept_values);
        }

        Ok(i32::from(removed_any))
    }

    pub(super) fn remove_collection_value(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let collection_key = map_identity_key(&invocation.arguments[0]);
        let needle = &invocation.arguments[1];
        let mut runtime = self.runtime.borrow_mut();
        let values = runtime.list_entries.entry(collection_key).or_default();
        let Some(index) = values.iter().position(|value| value == needle) else {
            return Some(0);
        };

        values.remove(index);
        Some(1)
    }

    pub(super) fn java_remove_collection_value(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<i32> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "Collection.remove(Object) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let collection_key = map_identity_key(&invocation.arguments[0]);
        let values = self
            .runtime
            .borrow()
            .list_entries
            .get(&collection_key)
            .cloned()
            .unwrap_or_default();
        let Some(index) =
            self.java_collection_index_of(&values, &invocation.arguments[1], depth + 1)?
        else {
            return Ok(0);
        };

        if let Some(values) = self
            .runtime
            .borrow_mut()
            .list_entries
            .get_mut(&collection_key)
        {
            if index < values.len() {
                values.remove(index);
            }
        }

        Ok(1)
    }

    pub(super) fn add_all_collection_values(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<i32> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "Collection.addAll(Collection) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let target_key = map_identity_key(&invocation.arguments[0]);
        let source_values =
            self.collection_values_by_contract(&invocation.arguments[1], depth + 1)?;

        if source_values.is_empty() {
            return Ok(0);
        }

        self.runtime
            .borrow_mut()
            .list_entries
            .entry(target_key)
            .or_default()
            .extend(source_values);

        Ok(1)
    }

    pub(super) fn java_add_all_set_values(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<i32> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "Set.addAll(Collection) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let target_key = map_identity_key(&invocation.arguments[0]);
        let source_values =
            self.collection_values_by_contract(&invocation.arguments[1], depth + 1)?;

        if source_values.is_empty() {
            return Ok(0);
        }

        let mut values = self
            .runtime
            .borrow()
            .list_entries
            .get(&target_key)
            .cloned()
            .unwrap_or_default();
        let mut changed = false;

        for value in source_values {
            if self.java_collection_contains_equal(&values, &value, depth + 1)? {
                continue;
            }

            values.push(value);
            changed = true;
        }

        if changed {
            self.runtime
                .borrow_mut()
                .list_entries
                .insert(target_key, values);
        }

        Ok(i32::from(changed))
    }

    pub(super) fn create_map_entry_iterator(&self, source_map_key: &str) -> Option<DexValue> {
        let mut entries = Vec::new();

        {
            let runtime = self.runtime.borrow();
            for ((map_key, entry_key), value) in &runtime.map_entries {
                if map_key != source_map_key {
                    continue;
                }

                let key = runtime
                    .map_entry_keys
                    .get(&(map_key.clone(), entry_key.clone()))?
                    .clone();
                entries.push((entry_key.clone(), key, value.clone()));
            }
        }

        entries.sort_by(|left, right| left.0.cmp(&right.0));

        let mut runtime = self.runtime.borrow_mut();
        let iterator_key = format!("iterator:{}", runtime.next_object_id);
        runtime.next_object_id += 1;

        let mut iterator_entries = Vec::new();
        for (index, (_entry_key, key, value)) in entries.into_iter().enumerate() {
            let entry_reference = format!("Ljava/util/Map$Entry;:{iterator_key}:{index}");
            runtime
                .map_entry_objects
                .insert(entry_reference.clone(), (key, value));
            iterator_entries.push(DexValue::Reference(std::rc::Rc::from(entry_reference)));
        }

        runtime
            .iterator_entries
            .insert(iterator_key.clone(), iterator_entries);
        runtime.iterator_positions.insert(iterator_key.clone(), 0);

        Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Ljava/util/Iterator;:{iterator_key}"
        ))))
    }

    pub(super) fn iterator_has_next(&self, invocation: Option<&DexInvocation>) -> Option<i32> {
        let Some(invocation) = invocation else {
            return Some(0);
        };

        if invocation.arguments.len() != 1 {
            return None;
        }

        let DexValue::Reference(reference) = &invocation.arguments[0] else {
            if matches!(invocation.arguments[0], DexValue::Object(_)) {
                return None;
            }
            return Some(0);
        };

        let Some(iterator_key) = iterator_storage_key(reference) else {
            return Some(0);
        };

        let runtime = self.runtime.borrow();
        let position = runtime
            .iterator_positions
            .get(iterator_key)
            .copied()
            .unwrap_or(0);
        let length = runtime
            .iterator_entries
            .get(iterator_key)
            .map(Vec::len)
            .unwrap_or(0);

        Some(i32::from(position < length))
    }

    pub(super) fn iterator_next(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let DexValue::Reference(reference) = &invocation.arguments[0] else {
            return None;
        };
        let iterator_key = iterator_storage_key(reference)?;

        let mut runtime = self.runtime.borrow_mut();
        let position = runtime
            .iterator_positions
            .get(iterator_key)
            .copied()
            .unwrap_or(0);
        let value = runtime
            .iterator_entries
            .get(iterator_key)
            .and_then(|entries| entries.get(position))
            .cloned()?;

        runtime
            .iterator_positions
            .insert(iterator_key.to_owned(), position + 1);
        runtime
            .iterator_last_returned_indices
            .insert(iterator_key.to_owned(), position);

        Some(value)
    }

    pub(super) fn iterator_has_previous(&self, invocation: Option<&DexInvocation>) -> Option<i32> {
        let Some(invocation) = invocation else {
            return Some(0);
        };

        if invocation.arguments.len() != 1 {
            return None;
        }

        let DexValue::Reference(reference) = &invocation.arguments[0] else {
            if matches!(invocation.arguments[0], DexValue::Object(_)) {
                return None;
            }
            return Some(0);
        };

        let Some(iterator_key) = iterator_storage_key(reference) else {
            return Some(0);
        };

        let position = self
            .runtime
            .borrow()
            .iterator_positions
            .get(iterator_key)
            .copied()
            .unwrap_or(0);

        Some(i32::from(position > 0))
    }

    pub(super) fn iterator_previous(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let DexValue::Reference(reference) = &invocation.arguments[0] else {
            return None;
        };
        let iterator_key = iterator_storage_key(reference)?;

        let mut runtime = self.runtime.borrow_mut();
        let position = runtime
            .iterator_positions
            .get(iterator_key)
            .copied()
            .unwrap_or(0);
        if position == 0 {
            return None;
        }

        let next_position = position - 1;
        let value = runtime
            .iterator_entries
            .get(iterator_key)
            .and_then(|entries| entries.get(next_position))
            .cloned()?;

        runtime
            .iterator_positions
            .insert(iterator_key.to_owned(), next_position);
        runtime
            .iterator_last_returned_indices
            .insert(iterator_key.to_owned(), next_position);

        Some(value)
    }

    pub(super) fn iterator_remove(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let DexValue::Reference(reference) = &invocation.arguments[0] else {
            return None;
        };
        let iterator_key = iterator_storage_key(reference)?;

        let mut runtime = self.runtime.borrow_mut();
        let source_key = runtime.iterator_sources.get(iterator_key)?.clone();
        let removed_index = runtime
            .iterator_last_returned_indices
            .remove(iterator_key)?;

        if let Some(values) = runtime.list_entries.get_mut(&source_key) {
            if removed_index < values.len() {
                values.remove(removed_index);
            }
        }

        if let Some(values) = runtime.iterator_entries.get_mut(iterator_key) {
            if removed_index < values.len() {
                values.remove(removed_index);
            }
        }

        if let Some(position) = runtime.iterator_positions.get_mut(iterator_key) {
            if removed_index < *position {
                *position -= 1;
            }
        }

        Some(())
    }

    pub(super) fn map_entry_key(&self, invocation: &DexInvocation) -> Option<DexValue> {
        let entry_key = map_entry_object_key(invocation)?;
        self.runtime
            .borrow()
            .map_entry_objects
            .get(&entry_key)
            .map(|(key, _value)| key.clone())
    }

    pub(super) fn map_entry_value(&self, invocation: &DexInvocation) -> Option<DexValue> {
        let entry_key = map_entry_object_key(invocation)?;
        self.runtime
            .borrow()
            .map_entry_objects
            .get(&entry_key)
            .map(|(_key, value)| value.clone())
    }

    pub(super) fn write_json_object_value(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        let object = invocation.arguments[0].clone();
        let object_key = map_identity_key(&object);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        let value = invocation.arguments[2].clone();

        self.runtime
            .borrow_mut()
            .json_entries
            .insert((object_key, entry_key.clone().to_string()), value);

        Some(object)
    }

    pub(super) fn write_json_object_long(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 4 {
            return None;
        }

        let object = invocation.arguments[0].clone();
        let object_key = map_identity_key(&object);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        let DexValue::Long(value) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Long(high_slot_value) = invocation.arguments[3] else {
            return None;
        };

        if high_slot_value != value {
            return None;
        }

        self.runtime.borrow_mut().json_entries.insert(
            (object_key, entry_key.clone().to_string()),
            DexValue::Long(value),
        );

        Some(object)
    }

    pub(super) fn write_content_values_string(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        let values_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        let value = match &invocation.arguments[2] {
            DexValue::String(value) => DexValue::String(value.clone()),
            DexValue::Null => DexValue::Null,
            DexValue::Uninitialized
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };

        self.runtime
            .borrow_mut()
            .content_values_entries
            .insert((values_key, entry_key.clone().to_string()), value);

        Some(())
    }

    pub(super) fn write_content_values_long(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        let values_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        let value = match &invocation.arguments[2] {
            DexValue::Reference(reference) => {
                DexValue::Long(boxed_long_reference_value(reference)?)
            }
            DexValue::Null => DexValue::Null,
            DexValue::Uninitialized
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_)
            | DexValue::Object(_) => return None,
        };

        self.runtime
            .borrow_mut()
            .content_values_entries
            .insert((values_key, entry_key.clone().to_string()), value);

        Some(())
    }

    pub(super) fn initialize_bundle(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.is_empty() {
            return None;
        }

        let bundle_key = map_identity_key(&invocation.arguments[0]);
        self.runtime
            .borrow_mut()
            .bundle_entries
            .retain(|(entry_bundle_key, _), _| entry_bundle_key != &bundle_key);

        Some(())
    }

    // cite: frameworks-base Bundle.java#Bundle
    pub(super) fn initialize_bundle_copy(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let bundle_key = map_identity_key(&invocation.arguments[0]);
        let source = match &invocation.arguments[1] {
            DexValue::Null | DexValue::Int(0) => {
                self.runtime
                    .borrow_mut()
                    .bundle_entries
                    .retain(|(entry_bundle_key, _), _| entry_bundle_key != &bundle_key);
                return Some(());
            }
            DexValue::Object(_) | DexValue::Reference(_) => {
                map_identity_key(&invocation.arguments[1])
            }
            DexValue::Uninitialized
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        };

        let entries: Vec<((String, String), DexValue)> = self
            .runtime
            .borrow()
            .bundle_entries
            .iter()
            .filter(|((entry_bundle_key, _), _)| entry_bundle_key == &source)
            .map(|((_, entry_key), value)| ((bundle_key.clone(), entry_key.clone()), value.clone()))
            .collect();

        let mut runtime = self.runtime.borrow_mut();
        runtime
            .bundle_entries
            .retain(|(entry_bundle_key, _), _| entry_bundle_key != &bundle_key);
        for (key, value) in entries {
            runtime.bundle_entries.insert(key, value);
        }

        Some(())
    }

    pub(super) fn bundle_contains_key(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let bundle_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };

        Some(i32::from(
            self.runtime
                .borrow()
                .bundle_entries
                .contains_key(&(bundle_key, entry_key.clone().to_string())),
        ))
    }

    pub(super) fn bundle_size(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let bundle_key = map_identity_key(&invocation.arguments[0]);
        let count = self
            .runtime
            .borrow()
            .bundle_entries
            .keys()
            .filter(|(entry_bundle_key, _)| entry_bundle_key == &bundle_key)
            .count();

        i32::try_from(count).ok()
    }

    pub(super) fn bundle_is_empty(&self, invocation: &DexInvocation) -> Option<i32> {
        let size = self.bundle_size(invocation)?;

        Some(i32::from(size == 0))
    }

    // cite: frameworks-base BaseBundle.java#putBoolean, putInt, putLong, putDouble, getInt, getLong, getFloat, getDouble, getBoolean
    pub(super) fn read_bundle_scalar(
        &self,
        invocation: &DexInvocation,
        box_descriptor: &str,
        default_value: DexValue,
    ) -> Option<DexValue> {
        if !matches!(invocation.arguments.len(), 2 | 3) {
            return None;
        }

        let bundle_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        let default_value = match invocation.arguments.get(2) {
            Some(value) => value.clone(),
            None => default_value,
        };

        let Some(stored) = self
            .runtime
            .borrow()
            .bundle_entries
            .get(&(bundle_key, entry_key.clone().to_string()))
            .cloned()
        else {
            return Some(default_value);
        };

        // cite: frameworks-base BaseBundle.java#getInt
        let DexValue::Reference(reference) = &stored else {
            return Some(default_value);
        };
        let Some(payload) = reference
            .strip_prefix(box_descriptor)
            .and_then(|rest| rest.strip_prefix(':'))
        else {
            return Some(default_value);
        };

        let unboxed = match box_descriptor {
            "Ljava/lang/Boolean;" => Some(DexValue::Int(i32::from(payload == "true"))),
            "Ljava/lang/Integer;" => payload.parse().ok().map(DexValue::Int),
            "Ljava/lang/Long;" => payload.parse().ok().map(DexValue::Long),
            "Ljava/lang/Float;" => u32::from_str_radix(payload, 16)
                .ok()
                .map(|bits| DexValue::Float(f32::from_bits(bits))),
            "Ljava/lang/Double;" => u64::from_str_radix(payload, 16)
                .ok()
                .map(|bits| DexValue::Long(bits as i64)),
            _ => None,
        };

        Some(unboxed.unwrap_or(default_value))
    }

    // cite: frameworks-base BaseBundle.java#getString
    pub(super) fn read_bundle_string(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if !matches!(invocation.arguments.len(), 2 | 3) {
            return None;
        }

        let bundle_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        let default_value = match invocation.arguments.get(2) {
            Some(DexValue::String(value)) => DexValue::String(value.clone()),
            Some(DexValue::Null) => DexValue::Null,
            Some(_) => return None,
            None => DexValue::Null,
        };

        Some(
            self.runtime
                .borrow()
                .bundle_entries
                .get(&(bundle_key, entry_key.clone().to_string()))
                .cloned()
                .unwrap_or(default_value),
        )
    }

    pub(super) fn write_bundle_scalar(
        &self,
        invocation: &DexInvocation,
        box_descriptor: &str,
    ) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }
        let bundle_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        // cite: frameworks-base BaseBundle.java#putInt
        let value = match (box_descriptor, &invocation.arguments[2]) {
            ("Ljava/lang/Boolean;", DexValue::Int(value)) => DexValue::Reference(
                std::rc::Rc::from(format!("Ljava/lang/Boolean;:{}", *value != 0)),
            ),
            ("Ljava/lang/Integer;", DexValue::Int(value)) => {
                DexValue::Reference(std::rc::Rc::from(format!("Ljava/lang/Integer;:{value}")))
            }
            ("Ljava/lang/Long;", DexValue::Long(value)) => {
                DexValue::Reference(std::rc::Rc::from(format!("Ljava/lang/Long;:{value}")))
            }
            ("Ljava/lang/Float;", DexValue::Float(value)) => DexValue::Reference(
                std::rc::Rc::from(format!("Ljava/lang/Float;:{:08x}", value.to_bits())),
            ),
            ("Ljava/lang/Double;", DexValue::Long(bits)) => DexValue::Reference(std::rc::Rc::from(
                format!("Ljava/lang/Double;:{:016x}", *bits as u64),
            )),
            _ => return None,
        };

        self.runtime
            .borrow_mut()
            .bundle_entries
            .insert((bundle_key, entry_key.clone().to_string()), value);

        Some(())
    }

    // cite: frameworks-base Bundle.java#putSerializable, putParcelable
    pub(super) fn write_bundle_object(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        let bundle_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        let value = match &invocation.arguments[2] {
            value @ (DexValue::Null
            | DexValue::String(_)
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::Object(_)
            | DexValue::Reference(_)) => value.clone(),
            DexValue::Uninitialized => return None,
        };

        self.runtime
            .borrow_mut()
            .bundle_entries
            .insert((bundle_key, entry_key.clone().to_string()), value);

        Some(())
    }

    pub(super) fn write_bundle_string(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        let bundle_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        let value = match &invocation.arguments[2] {
            DexValue::String(value) => DexValue::String(value.clone()),
            DexValue::Null => DexValue::Null,
            DexValue::Uninitialized
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };

        self.runtime
            .borrow_mut()
            .bundle_entries
            .insert((bundle_key, entry_key.clone().to_string()), value);

        Some(())
    }

    // cite: frameworks-base Bundle.java#putCharSequence
    pub(super) fn write_bundle_char_sequence(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        let bundle_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        let value = match &invocation.arguments[2] {
            DexValue::String(_) | DexValue::Object(_) | DexValue::Reference(_) => {
                invocation.arguments[2].clone()
            }
            DexValue::Null => DexValue::Null,
            DexValue::Uninitialized | DexValue::Int(_) | DexValue::Long(_) | DexValue::Float(_) => {
                return None
            }
        };

        self.runtime
            .borrow_mut()
            .bundle_entries
            .insert((bundle_key, entry_key.clone().to_string()), value);

        Some(())
    }

    pub(super) fn write_bundle_bundle(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        let bundle_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        let value = match &invocation.arguments[2] {
            DexValue::Object(_) | DexValue::Reference(_) => invocation.arguments[2].clone(),
            DexValue::Null | DexValue::Int(0) => DexValue::Null,
            DexValue::Uninitialized
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        };

        self.runtime
            .borrow_mut()
            .bundle_entries
            .insert((bundle_key, entry_key.clone().to_string()), value);

        Some(())
    }

    // cite: frameworks-base BaseBundle.java#get
    pub(super) fn bundle_get(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let bundle_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };

        Some(
            self.runtime
                .borrow()
                .bundle_entries
                .get(&(bundle_key, entry_key.clone().to_string()))
                .cloned()
                .unwrap_or(DexValue::Null),
        )
    }

    pub(super) fn bundle_key_set(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let bundle_key = map_identity_key(&invocation.arguments[0]);
        let keys = self
            .runtime
            .borrow()
            .bundle_entries
            .keys()
            .filter(|(entry_bundle_key, _)| entry_bundle_key == &bundle_key)
            .map(|(_, entry_key)| DexValue::String(std::rc::Rc::from(entry_key.clone())))
            .collect::<Vec<_>>();
        let set = self.allocate_object("Ljava/util/Set;".to_owned());

        self.runtime
            .borrow_mut()
            .list_entries
            .insert(map_identity_key(&set), keys);

        Some(set)
    }

    pub(super) fn bundle_get_object_entry(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 && invocation.arguments.len() != 3 {
            return None;
        }

        let bundle_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };

        Some(
            self.runtime
                .borrow()
                .bundle_entries
                .get(&(bundle_key, entry_key.clone().to_string()))
                .cloned()
                .unwrap_or(DexValue::Null),
        )
    }

    pub(super) fn remove_bundle_entry(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let bundle_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };

        self.runtime
            .borrow_mut()
            .bundle_entries
            .remove(&(bundle_key, entry_key.clone().to_string()));

        Some(())
    }

    pub(super) fn bundle_put_all(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let target_key = map_identity_key(&invocation.arguments[0]);
        let source_key = map_identity_key(&invocation.arguments[1]);
        let copied = self
            .runtime
            .borrow()
            .bundle_entries
            .iter()
            .filter(|((entry_bundle_key, _), _)| entry_bundle_key == &source_key)
            .map(|((_, entry_key), value)| (entry_key.clone(), value.clone()))
            .collect::<Vec<_>>();
        let mut runtime = self.runtime.borrow_mut();

        for (entry_key, value) in copied {
            runtime
                .bundle_entries
                .insert((target_key.clone(), entry_key), value);
        }

        Some(())
    }

    pub(super) fn initialize_sparse_array(&self, invocation: &DexInvocation) -> Option<()> {
        if !matches!(invocation.arguments.len(), 1 | 2) {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        self.runtime
            .borrow_mut()
            .sparse_array_entries
            .entry(array_key)
            .or_default()
            .clear();

        Some(())
    }

    pub(super) fn sparse_array_size(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let size = self
            .runtime
            .borrow()
            .sparse_array_entries
            .get(&array_key)
            .map(Vec::len)
            .unwrap_or(0);

        i32::try_from(size).ok()
    }

    pub(super) fn sparse_array_put(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::Int(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        let entry_key = *entry_key;
        let value = invocation.arguments[2].clone();
        let mut runtime = self.runtime.borrow_mut();
        let entries = runtime.sparse_array_entries.entry(array_key).or_default();

        if let Some((_, stored_value)) = entries.iter_mut().find(|(key, _)| *key == entry_key) {
            *stored_value = value;
        } else {
            entries.push((entry_key, value));
        }

        Some(())
    }

    pub(super) fn sparse_array_get(
        &self,
        invocation: &DexInvocation,
        default_value: Option<DexValue>,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 2 && invocation.arguments.len() != 3 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::Int(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        let entry_key = *entry_key;
        let value = self
            .runtime
            .borrow()
            .sparse_array_entries
            .get(&array_key)
            .and_then(|entries| {
                entries
                    .iter()
                    .find(|(key, _)| *key == entry_key)
                    .map(|(_, value)| value.clone())
            })
            .unwrap_or_else(|| default_value.unwrap_or(DexValue::Null));

        Some(value)
    }

    pub(super) fn sparse_array_remove(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::Int(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        let entry_key = *entry_key;
        let mut runtime = self.runtime.borrow_mut();
        let entries = runtime.sparse_array_entries.entry(array_key).or_default();

        entries.retain(|(key, _)| *key != entry_key);

        Some(())
    }

    pub(super) fn sparse_array_remove_at(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::Int(index) = &invocation.arguments[1] else {
            return None;
        };
        let index = *index;
        if index < 0 {
            return None;
        }

        let mut runtime = self.runtime.borrow_mut();
        let entries = runtime.sparse_array_entries.entry(array_key).or_default();
        let index = usize::try_from(index).ok()?;
        if index < entries.len() {
            entries.remove(index);
        }

        Some(())
    }

    pub(super) fn sparse_array_clear(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        self.runtime
            .borrow_mut()
            .sparse_array_entries
            .entry(array_key)
            .or_default()
            .clear();

        Some(())
    }

    pub(super) fn sparse_array_value_at(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::Int(index) = &invocation.arguments[1] else {
            return None;
        };
        let index = *index;
        if index < 0 {
            return None;
        }

        let index = usize::try_from(index).ok()?;
        let value = self
            .runtime
            .borrow()
            .sparse_array_entries
            .get(&array_key)
            .and_then(|entries| entries.get(index))
            .map(|(_, value)| value.clone())
            .unwrap_or(DexValue::Null);

        Some(value)
    }

    pub(super) fn sparse_array_key_at(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::Int(index) = &invocation.arguments[1] else {
            return None;
        };
        let index = *index;
        if index < 0 {
            return None;
        }

        let index = usize::try_from(index).ok()?;
        let value = self
            .runtime
            .borrow()
            .sparse_array_entries
            .get(&array_key)
            .and_then(|entries| entries.get(index))
            .map(|(key, _)| *key)
            .unwrap_or(0);

        Some(value)
    }

    pub(super) fn sparse_array_index_of_key(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::Int(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        let entry_key = *entry_key;
        let index = self
            .runtime
            .borrow()
            .sparse_array_entries
            .get(&array_key)
            .and_then(|entries| entries.iter().position(|(key, _)| *key == entry_key));

        match index {
            Some(index) => i32::try_from(index).ok(),
            None => Some(-1),
        }
    }

    pub(super) fn sparse_long_array_put(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 4 {
            return None;
        }

        let DexValue::Long(value) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Long(value_high_slot) = invocation.arguments[3] else {
            return None;
        };
        if value != value_high_slot {
            return None;
        }

        let sparse_invocation = DexInvocation {
            arguments: smallvec::smallvec![
                invocation.arguments[0].clone(),
                invocation.arguments[1].clone(),
                DexValue::Long(value),
            ],
            ..invocation.clone()
        };

        self.sparse_array_put(&sparse_invocation)
    }

    pub(super) fn sparse_long_array_get(
        &self,
        invocation: &DexInvocation,
        default_value: i64,
    ) -> Option<i64> {
        if invocation.arguments.len() != 2 && invocation.arguments.len() != 4 {
            return None;
        }

        let mut effective_default = default_value;
        let sparse_invocation = if invocation.arguments.len() == 4 {
            let DexValue::Long(value) = invocation.arguments[2] else {
                return None;
            };
            let DexValue::Long(value_high_slot) = invocation.arguments[3] else {
                return None;
            };
            if value != value_high_slot {
                return None;
            }
            effective_default = value;

            DexInvocation {
                arguments: smallvec::smallvec![
                    invocation.arguments[0].clone(),
                    invocation.arguments[1].clone(),
                    DexValue::Long(value),
                ],
                ..invocation.clone()
            }
        } else {
            invocation.clone()
        };

        match self.sparse_array_get(&sparse_invocation, Some(DexValue::Long(effective_default)))? {
            DexValue::Long(value) => Some(value),
            DexValue::Int(value) => Some(i64::from(value)),
            DexValue::Null
            | DexValue::Uninitialized
            | DexValue::Float(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => None,
        }
    }

    pub(super) fn sparse_long_array_value_at(&self, invocation: &DexInvocation) -> Option<i64> {
        match self.sparse_array_value_at(invocation)? {
            DexValue::Long(value) => Some(value),
            DexValue::Int(value) => Some(i64::from(value)),
            DexValue::Null
            | DexValue::Uninitialized
            | DexValue::Float(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => Some(0),
        }
    }

    pub(super) fn sparse_boolean_array_put(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        let DexValue::Int(value) = invocation.arguments[2] else {
            return None;
        };
        let sparse_invocation = DexInvocation {
            arguments: smallvec::smallvec![
                invocation.arguments[0].clone(),
                invocation.arguments[1].clone(),
                DexValue::Int(normalize_sparse_boolean(value)),
            ],
            ..invocation.clone()
        };

        self.sparse_array_put(&sparse_invocation)
    }

    pub(super) fn sparse_boolean_array_get(
        &self,
        invocation: &DexInvocation,
        default_value: i32,
    ) -> Option<i32> {
        if invocation.arguments.len() != 2 && invocation.arguments.len() != 3 {
            return None;
        }

        let mut effective_default = normalize_sparse_boolean(default_value);
        let sparse_invocation = if invocation.arguments.len() == 3 {
            let DexValue::Int(value) = invocation.arguments[2] else {
                return None;
            };
            effective_default = normalize_sparse_boolean(value);

            DexInvocation {
                arguments: smallvec::smallvec![
                    invocation.arguments[0].clone(),
                    invocation.arguments[1].clone(),
                    DexValue::Int(effective_default),
                ],
                ..invocation.clone()
            }
        } else {
            invocation.clone()
        };

        match self.sparse_array_get(&sparse_invocation, Some(DexValue::Int(effective_default)))? {
            DexValue::Int(value) => Some(normalize_sparse_boolean(value)),
            DexValue::Long(value) => Some(i32::from(value != 0)),
            DexValue::Null
            | DexValue::Uninitialized
            | DexValue::Float(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => None,
        }
    }

    pub(super) fn sparse_boolean_array_value_at(&self, invocation: &DexInvocation) -> Option<i32> {
        match self.sparse_array_value_at(invocation)? {
            DexValue::Int(value) => Some(normalize_sparse_boolean(value)),
            DexValue::Long(value) => Some(i32::from(value != 0)),
            DexValue::Null
            | DexValue::Uninitialized
            | DexValue::Float(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => Some(0),
        }
    }

    pub(super) fn json_object_length(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let object_key = map_identity_key(&invocation.arguments[0]);
        let count = self
            .runtime
            .borrow()
            .json_entries
            .keys()
            .filter(|(json_key, _entry_key)| json_key == &object_key)
            .count();

        i32::try_from(count).ok()
    }

    pub(super) fn json_object_is_null(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let object_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };
        let value = self
            .runtime
            .borrow()
            .json_entries
            .get(&(object_key, entry_key.clone().to_string()))
            .cloned()
            .unwrap_or(DexValue::Null);

        Some(i32::from(value == DexValue::Null))
    }

    pub(super) fn json_object_to_string(&self, invocation: &DexInvocation) -> Option<String> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let object_key = map_identity_key(&invocation.arguments[0]);
        self.json_object_to_string_by_key(&object_key, 0)
    }

    pub(super) fn json_object_to_string_by_key(
        &self,
        object_key: &str,
        depth: usize,
    ) -> Option<String> {
        if depth > 16 {
            return None;
        }

        let mut entries = Vec::new();
        let runtime = self.runtime.borrow();

        for ((json_key, entry_key), value) in &runtime.json_entries {
            if json_key != object_key {
                continue;
            }

            entries.push((entry_key.clone(), value.clone()));
        }

        drop(runtime);

        entries.sort_by(|left, right| left.0.cmp(&right.0));

        let mut encoded_entries = Vec::new();
        for (entry_key, value) in entries {
            let encoded_key = json_quote(&entry_key);
            let encoded_value = self.json_value_to_string(&value, depth + 1)?;
            encoded_entries.push(format!("{encoded_key}:{encoded_value}"));
        }

        Some(format!("{{{}}}", encoded_entries.join(",")))
    }

    pub(super) fn json_value_to_string(&self, value: &DexValue, depth: usize) -> Option<String> {
        match value {
            DexValue::Null => Some("null".to_owned()),
            DexValue::Int(value) => Some(value.to_string()),
            DexValue::Long(value) => Some(value.to_string()),
            DexValue::String(value) => Some(json_quote(value)),
            DexValue::Object(object)
                if object.class_descriptor == std::rc::Rc::from("Lorg/json/JSONObject;") =>
            {
                self.json_object_to_string_by_key(&map_identity_key(value), depth)
            }
            DexValue::Object(object)
                if object.class_descriptor == std::rc::Rc::from("Lorg/json/JSONArray;") =>
            {
                self.json_array_to_string_by_key(&map_identity_key(value), depth)
            }
            DexValue::Reference(reference) => boxed_reference_display_value(reference),
            DexValue::Uninitialized | DexValue::Float(_) | DexValue::Object(_) => None,
        }
    }

    pub(super) fn init_json_array_from_collection(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let collection_key = map_identity_key(&invocation.arguments[1]);
        let values = self
            .runtime
            .borrow()
            .list_entries
            .get(&collection_key)
            .cloned()
            .unwrap_or_default();
        self.runtime
            .borrow_mut()
            .list_entries
            .insert(array_key, values);

        Some(())
    }

    pub(super) fn init_json_array_from_string(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let DexValue::String(source) = &invocation.arguments[1] else {
            return None;
        };
        let array_key = map_identity_key(&invocation.arguments[0]);

        let mut chars = source.chars().peekable();
        skip_json_whitespace(&mut chars);
        let values = self.parse_json_array_body(&mut chars)?;
        self.runtime
            .borrow_mut()
            .list_entries
            .insert(array_key, values);

        Some(())
    }

    pub(super) fn init_json_object_from_string(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let DexValue::String(source) = &invocation.arguments[1] else {
            return None;
        };
        let object_key = map_identity_key(&invocation.arguments[0]);

        let mut chars = source.chars().peekable();
        skip_json_whitespace(&mut chars);
        let entries = self.parse_json_object_body(&mut chars)?;
        let mut runtime = self.runtime.borrow_mut();
        for (entry_key, value) in entries {
            runtime
                .json_entries
                .insert((object_key.clone(), entry_key), value);
        }

        Some(())
    }

    pub(super) fn json_object_get_value(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let object_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::String(entry_key) = &invocation.arguments[1] else {
            return None;
        };

        self.runtime
            .borrow()
            .json_entries
            .get(&(object_key, entry_key.clone().to_string()))
            .cloned()
    }

    pub(super) fn json_array_get_value(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::Int(index) = invocation.arguments[1] else {
            return None;
        };
        let index = usize::try_from(index).ok()?;

        self.runtime
            .borrow()
            .list_entries
            .get(&array_key)?
            .get(index)
            .cloned()
    }

    pub(super) fn json_array_get_string(&self, invocation: &DexInvocation) -> Option<DexValue> {
        let value = self.json_array_get_value(invocation)?;

        json_coerce_to_string(&value).map(|value| DexValue::String(value.into()))
    }

    pub(super) fn json_array_is_null(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::Int(index) = invocation.arguments[1] else {
            return None;
        };
        let Ok(index) = usize::try_from(index) else {
            return Some(1);
        };
        let is_null = match self.runtime.borrow().list_entries.get(&array_key) {
            Some(values) => {
                !matches!(values.get(index), Some(value) if !matches!(value, DexValue::Null))
            }
            None => true,
        };

        Some(i32::from(is_null))
    }

    fn parse_json_value(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    ) -> Option<DexValue> {
        skip_json_whitespace(chars);
        match *chars.peek()? {
            '"' => parse_json_string(chars).map(|value| DexValue::String(value.into())),
            '[' => {
                let values = self.parse_json_array_body(chars)?;
                let array = self.allocate_object("Lorg/json/JSONArray;".to_owned());
                self.runtime
                    .borrow_mut()
                    .list_entries
                    .insert(map_identity_key(&array), values);
                Some(array)
            }
            '{' => {
                let entries = self.parse_json_object_body(chars)?;
                let object = self.allocate_object("Lorg/json/JSONObject;".to_owned());
                let object_key = map_identity_key(&object);
                let mut runtime = self.runtime.borrow_mut();
                for (entry_key, value) in entries {
                    runtime
                        .json_entries
                        .insert((object_key.clone(), entry_key), value);
                }
                Some(object)
            }
            't' | 'f' => parse_json_literal(chars),
            'n' => parse_json_literal(chars),
            c if c == '-' || c.is_ascii_digit() => parse_json_number(chars),
            _ => None,
        }
    }

    fn parse_json_array_body(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    ) -> Option<Vec<DexValue>> {
        if chars.next()? != '[' {
            return None;
        }
        let mut values = Vec::new();
        skip_json_whitespace(chars);
        if chars.peek() == Some(&']') {
            chars.next();
            return Some(values);
        }
        loop {
            let value = self.parse_json_value(chars)?;
            values.push(value);
            skip_json_whitespace(chars);
            match chars.next()? {
                ',' => skip_json_whitespace(chars),
                ']' => break,
                _ => return None,
            }
        }
        Some(values)
    }

    fn parse_json_object_body(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    ) -> Option<Vec<(String, DexValue)>> {
        if chars.next()? != '{' {
            return None;
        }
        let mut entries = Vec::new();
        skip_json_whitespace(chars);
        if chars.peek() == Some(&'}') {
            chars.next();
            return Some(entries);
        }
        loop {
            skip_json_whitespace(chars);
            let key = parse_json_string(chars)?;
            skip_json_whitespace(chars);
            if chars.next()? != ':' {
                return None;
            }
            let value = self.parse_json_value(chars)?;
            entries.push((key, value));
            skip_json_whitespace(chars);
            match chars.next()? {
                ',' => {}
                '}' => break,
                _ => return None,
            }
        }
        Some(entries)
    }

    pub(super) fn json_array_put_value(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let array = invocation.arguments[0].clone();
        let array_key = map_identity_key(&array);
        let value = invocation.arguments[1].clone();
        self.runtime
            .borrow_mut()
            .list_entries
            .entry(array_key)
            .or_default()
            .push(value);

        Some(array)
    }

    pub(super) fn json_array_length(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let length = self
            .runtime
            .borrow()
            .list_entries
            .get(&array_key)
            .map(|values| values.len())
            .unwrap_or(0);

        i32::try_from(length).ok()
    }

    pub(super) fn json_array_to_string(&self, invocation: &DexInvocation) -> Option<String> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        self.json_array_to_string_by_key(&array_key, 0)
    }

    pub(super) fn json_array_to_string_by_key(
        &self,
        array_key: &str,
        depth: usize,
    ) -> Option<String> {
        if depth > 16 {
            return None;
        }

        let values = self
            .runtime
            .borrow()
            .list_entries
            .get(array_key)
            .cloned()
            .unwrap_or_default();

        let mut encoded = Vec::with_capacity(values.len());
        for value in values {
            encoded.push(self.json_value_to_string(&value, depth + 1)?);
        }

        Some(format!("[{}]", encoded.join(",")))
    }

    pub(super) fn add_array_list_value(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let list_key = map_identity_key(&invocation.arguments[0]);
        let value = invocation.arguments[1].clone();

        self.runtime
            .borrow_mut()
            .list_entries
            .entry(list_key)
            .or_default()
            .push(value);

        Some(())
    }

    pub(super) fn add_array_list_value_at(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        let list_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::Int(index) = invocation.arguments[1] else {
            return None;
        };
        let index = usize::try_from(index).ok()?;
        let value = invocation.arguments[2].clone();

        let mut runtime = self.runtime.borrow_mut();
        let values = runtime.list_entries.entry(list_key).or_default();
        if index > values.len() {
            return None;
        }

        values.insert(index, value);

        Some(())
    }

    pub(super) fn set_array_list_value(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        let list_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::Int(index) = invocation.arguments[1] else {
            return None;
        };
        if index < 0 {
            return None;
        }

        let mut runtime = self.runtime.borrow_mut();
        let values = runtime.list_entries.get_mut(&list_key)?;
        let slot = values.get_mut(usize::try_from(index).ok()?)?;
        let previous = slot.clone();
        *slot = invocation.arguments[2].clone();

        Some(previous)
    }

    pub(super) fn add_all_array_list_values_at(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<i32> {
        if invocation.arguments.len() != 3 {
            return Err(WinedroidError::new(format!(
                "List.addAll(int, Collection) expected 3 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let list_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::Int(index) = invocation.arguments[1] else {
            return Err(WinedroidError::new(
                "List.addAll(int, Collection) index argument was not an int",
            ));
        };
        let index = usize::try_from(index).map_err(|_| {
            WinedroidError::new(format!(
                "List.addAll(int, Collection) index was negative: {index}"
            ))
        })?;
        let source_values =
            self.collection_values_by_contract(&invocation.arguments[2], depth + 1)?;
        let changed = !source_values.is_empty();

        let mut runtime = self.runtime.borrow_mut();
        let values = runtime.list_entries.entry(list_key).or_default();
        if index > values.len() {
            return Err(WinedroidError::new(format!(
                "List.addAll(int, Collection) index {index} exceeds size {}",
                values.len()
            )));
        }

        values.splice(index..index, source_values);

        Ok(i32::from(changed))
    }

    pub(super) fn array_list_sub_list(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        let list_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::Int(from_index) = invocation.arguments[1] else {
            return None;
        };
        let DexValue::Int(to_index) = invocation.arguments[2] else {
            return None;
        };
        let from_index = usize::try_from(from_index).ok()?;
        let to_index = usize::try_from(to_index).ok()?;
        if from_index > to_index {
            return None;
        }

        {
            let runtime = self.runtime.borrow();
            let values = runtime.list_entries.get(&list_key)?;
            values.get(from_index..to_index)?;
        }

        let sub_list = self.allocate_object("Ljava/util/ArrayList;".to_owned());

        self.runtime.borrow_mut().list_sub_lists.insert(
            map_identity_key(&sub_list),
            RuntimeSubList {
                parent_key: list_key,
                from_index,
                to_index,
            },
        );

        Some(sub_list)
    }

    pub(super) fn list_contains(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let list_key = map_identity_key(&invocation.arguments[0]);
        let contains = runtime_list_values(&self.runtime.borrow(), &list_key)
            .map(|values| values.contains(&invocation.arguments[1]))
            .unwrap_or(false);

        Some(i32::from(contains))
    }

    pub(super) fn java_collection_contains_value(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<i32> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "Collection.contains(Object) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let values = self.collection_values_for_java_equality(&invocation.arguments[0]);
        let contains = self
            .java_collection_index_of(&values, &invocation.arguments[1], depth + 1)?
            .is_some();

        Ok(i32::from(contains))
    }

    pub(super) fn list_index_of(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let list_key = map_identity_key(&invocation.arguments[0]);
        let index = runtime_list_values(&self.runtime.borrow(), &list_key)
            .and_then(|values| {
                values
                    .iter()
                    .position(|value| value == &invocation.arguments[1])
            })
            .map(|index| index as i32)
            .unwrap_or(-1);

        Some(index)
    }

    // cite: libcore AbstractList.java#lastIndexOf
    pub(super) fn list_last_index_of(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let list_key = map_identity_key(&invocation.arguments[0]);
        let index = runtime_list_values(&self.runtime.borrow(), &list_key)
            .and_then(|values| {
                values
                    .iter()
                    .rposition(|value| value == &invocation.arguments[1])
            })
            .map(|index| index as i32)
            .unwrap_or(-1);

        Some(index)
    }

    pub(super) fn java_collection_index_of_value(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<i32> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "List.indexOf(Object) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let values = self.collection_values_for_java_equality(&invocation.arguments[0]);
        let index = self
            .java_collection_index_of(&values, &invocation.arguments[1], depth + 1)?
            .map(|index| index as i32)
            .unwrap_or(-1);

        Ok(index)
    }

    fn collection_values_for_java_equality(&self, collection: &DexValue) -> Vec<DexValue> {
        let collection_key = map_identity_key(collection);
        self.runtime
            .borrow()
            .list_entries
            .get(&collection_key)
            .cloned()
            .unwrap_or_default()
    }

    fn java_collection_contains_equal(
        &self,
        values: &[DexValue],
        needle: &DexValue,
        depth: usize,
    ) -> Result<bool> {
        Ok(self
            .java_collection_index_of(values, needle, depth + 1)?
            .is_some())
    }

    fn java_collection_index_of(
        &self,
        values: &[DexValue],
        needle: &DexValue,
        depth: usize,
    ) -> Result<Option<usize>> {
        for (index, value) in values.iter().enumerate() {
            if self.runtime_values_equal(needle, value, depth + 1, true)? {
                return Ok(Some(index));
            }
        }

        Ok(None)
    }

    pub(super) fn collections_binary_search(&self, invocation: &DexInvocation) -> Option<i32> {
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

        let list_key = map_identity_key(&invocation.arguments[0]);
        let needle = &invocation.arguments[1];
        let values = self
            .runtime
            .borrow()
            .list_entries
            .get(&list_key)
            .cloned()
            .unwrap_or_default();

        if let Some(index) = values.iter().position(|value| value == needle) {
            return i32::try_from(index).ok();
        }

        let mut low = 0_usize;
        let mut high = values.len();
        while low < high {
            let middle = low + ((high - low) / 2);
            match self.compare_binary_search_values(&values[middle], needle) {
                Some(std::cmp::Ordering::Greater) => high = middle,
                Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal) | None => {
                    low = middle + 1;
                }
            }
        }

        let insertion_point = i32::try_from(low).ok()?;
        insertion_point.checked_add(1)?.checked_neg()
    }

    // cite: libcore Collections.java#reverse
    pub(super) fn collections_reverse_list(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let list_key = map_identity_key(&invocation.arguments[0]);
        let mut runtime = self.runtime.borrow_mut();
        let values = runtime.list_entries.get_mut(&list_key)?;
        values.reverse();

        Some(())
    }

    pub(super) fn collections_sort_list(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
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

        let list_key = map_identity_key(&invocation.arguments[0]);
        let mut values = self
            .runtime
            .borrow()
            .list_entries
            .get(&list_key)
            .cloned()
            .unwrap_or_default();
        if values.len() > 1 {
            values.sort_by(|left, right| {
                self.compare_binary_search_values(left, right)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        self.runtime
            .borrow_mut()
            .list_entries
            .insert(list_key, values);
        Some(())
    }

    pub(super) fn collections_sort_list_with_comparator(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<()> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "Collections.sort(List,Comparator) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return Err(WinedroidError::new(format!(
                "Collections.sort(List,Comparator) requires an object-backed List\nargument: {}",
                invocation.arguments[0]
            )));
        }

        if invocation.arguments[1] == DexValue::Null {
            return self
                .collections_sort_list(&DexInvocation {
                    kind: DexInvokeKind::Static,
                    range: false,
                    method_index: invocation.method_index,
                    argument_registers: invocation.argument_registers.clone(),
                    arguments: smallvec::smallvec![invocation.arguments[0].clone()],
                })
                .ok_or_else(|| {
                    WinedroidError::new(
                        "Collections.sort(List,Comparator) natural sort failed for List",
                    )
                });
        }

        if !dex_value_is_object_like(&invocation.arguments[1]) {
            return Err(WinedroidError::new(format!(
                "Collections.sort(List,Comparator) requires an object-backed Comparator\nargument: {}",
                invocation.arguments[1]
            )));
        }

        let list_key = map_identity_key(&invocation.arguments[0]);
        let comparator = invocation.arguments[1].clone();
        let mut values = self
            .runtime
            .borrow()
            .list_entries
            .get(&list_key)
            .cloned()
            .unwrap_or_default();

        for index in 1..values.len() {
            let value = values[index].clone();
            let mut insert_at = index;

            while insert_at > 0
                && self.compare_with_comparator(
                    &comparator,
                    &value,
                    &values[insert_at - 1],
                    depth + 1,
                )? < 0
            {
                values[insert_at] = values[insert_at - 1].clone();
                insert_at -= 1;
            }

            values[insert_at] = value;
        }

        self.runtime
            .borrow_mut()
            .list_entries
            .insert(list_key, values);
        Ok(())
    }

    pub(super) fn compare_binary_search_values(
        &self,
        left: &DexValue,
        right: &DexValue,
    ) -> Option<std::cmp::Ordering> {
        match (left, right) {
            (DexValue::String(left), DexValue::String(right)) => return Some(left.cmp(right)),
            (DexValue::Int(left), DexValue::Int(right)) => return Some(left.cmp(right)),
            _ => {}
        }

        let left_order = self.preference_order_value(left);
        let right_order = self.preference_order_value(right);
        match (left_order, right_order) {
            (Some(left), Some(right)) => Some(left.cmp(&right)),
            _ => None,
        }
    }

    pub(super) fn preference_order_value(&self, value: &DexValue) -> Option<i32> {
        let descriptor = value_class_descriptor(value)?;
        if !descriptor.starts_with("Landroidx/preference/") {
            return None;
        }

        let field = DexFieldId {
            class_descriptor: "Landroidx/preference/Preference;".to_owned(),
            type_descriptor: "I".to_owned(),
            name: "f".to_owned(),
        };

        match self.read_stored_instance_field(value, &field) {
            Some(DexValue::Int(order)) => Some(order),
            Some(
                DexValue::Uninitialized
                | DexValue::Null
                | DexValue::Long(_)
                | DexValue::Float(_)
                | DexValue::String(_)
                | DexValue::Object(_)
                | DexValue::Reference(_),
            ) => None,
            None => Some(i32::MAX),
        }
    }

    pub(super) fn collections_singleton_list(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let value = invocation.arguments[0].clone();
        let mut runtime = self.runtime.borrow_mut();
        let list_key = format!("collections:singletonList:{}", runtime.next_object_id);
        runtime.next_object_id += 1;
        let list = DexValue::Reference(std::rc::Rc::from(format!("Ljava/util/List;:{list_key}")));

        runtime
            .list_entries
            .insert(map_identity_key(&list), vec![value]);

        Some(list)
    }

    pub(super) fn collections_empty_list(&self) -> DexValue {
        let list = DexValue::Reference(std::rc::Rc::from("Ljava/util/List;:collections:emptyList"));

        self.runtime
            .borrow_mut()
            .list_entries
            .entry(map_identity_key(&list))
            .or_default();

        list
    }

    pub(super) fn arrays_as_list(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let values = self.runtime.borrow().array_entries.get(&array_key)?.clone();
        let list = self.allocate_object("Ljava/util/Arrays$ArrayList;".to_owned());
        let list_key = map_identity_key(&list);

        self.runtime
            .borrow_mut()
            .list_entries
            .insert(list_key, values);

        Some(list)
    }

    pub(super) fn arrays_fill_long_range(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 5 {
            return None;
        }

        let DexValue::Int(from_index) = invocation.arguments[1] else {
            return None;
        };
        let DexValue::Int(to_index) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Long(value) = invocation.arguments[3] else {
            return None;
        };
        let DexValue::Long(value_wide_slot) = invocation.arguments[4] else {
            return None;
        };

        if value != value_wide_slot || from_index < 0 || to_index < from_index {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let from_index = usize::try_from(from_index).ok()?;
        let to_index = usize::try_from(to_index).ok()?;
        let mut runtime = self.runtime.borrow_mut();
        let values = runtime.array_entries.get_mut(&array_key)?;

        if to_index > values.len() {
            return None;
        }

        for slot in &mut values[from_index..to_index] {
            *slot = DexValue::Long(value);
        }

        Some(())
    }

    pub(super) fn arrays_fill_int_range(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 4 {
            return None;
        }

        let DexValue::Int(from_index) = invocation.arguments[1] else {
            return None;
        };
        let DexValue::Int(to_index) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Int(value) = invocation.arguments[3] else {
            return None;
        };

        if from_index < 0 || to_index < from_index {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let from_index = usize::try_from(from_index).ok()?;
        let to_index = usize::try_from(to_index).ok()?;
        let mut runtime = self.runtime.borrow_mut();
        let values = runtime.array_entries.get_mut(&array_key)?;

        if to_index > values.len() {
            return None;
        }

        for slot in &mut values[from_index..to_index] {
            *slot = DexValue::Int(value);
        }

        Some(())
    }

    pub(super) fn arrays_fill_object_range(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 4 {
            return None;
        }

        let DexValue::Int(from_index) = invocation.arguments[1] else {
            return None;
        };
        let DexValue::Int(to_index) = invocation.arguments[2] else {
            return None;
        };

        if from_index < 0 || to_index < from_index {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let from_index = usize::try_from(from_index).ok()?;
        let to_index = usize::try_from(to_index).ok()?;
        let fill_value = invocation.arguments[3].clone();
        let mut runtime = self.runtime.borrow_mut();
        let values = runtime.array_entries.get_mut(&array_key)?;

        if to_index > values.len() {
            return None;
        }

        for slot in &mut values[from_index..to_index] {
            *slot = fill_value.clone();
        }

        Some(())
    }

    pub(super) fn arrays_equals_object(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if invocation.arguments[0] == invocation.arguments[1] {
            return Some(1);
        }
        if invocation.arguments[0] == DexValue::Null || invocation.arguments[1] == DexValue::Null {
            return Some(0);
        }

        let left_key = map_identity_key(&invocation.arguments[0]);
        let right_key = map_identity_key(&invocation.arguments[1]);
        let runtime = self.runtime.borrow();
        let left = runtime.array_entries.get(&left_key)?;
        let right = runtime.array_entries.get(&right_key)?;

        Some(i32::from(left == right))
    }

    pub(super) fn arrays_sort_int_range(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }
        let DexValue::Int(from_index) = invocation.arguments[1] else {
            return None;
        };
        let DexValue::Int(to_index) = invocation.arguments[2] else {
            return None;
        };
        if from_index < 0 || to_index < from_index {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let from_index = usize::try_from(from_index).ok()?;
        let to_index = usize::try_from(to_index).ok()?;
        let mut runtime = self.runtime.borrow_mut();
        let values = runtime.array_entries.get_mut(&array_key)?;
        if to_index > values.len() {
            return None;
        }

        let mut ints = Vec::with_capacity(to_index.saturating_sub(from_index));
        for value in &values[from_index..to_index] {
            let DexValue::Int(value) = value else {
                return None;
            };
            ints.push(*value);
        }
        ints.sort_unstable();
        for (slot, value) in values[from_index..to_index].iter_mut().zip(ints) {
            *slot = DexValue::Int(value);
        }

        Some(())
    }

    pub(super) fn arrays_copy_of_object(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let DexValue::Int(new_length) = invocation.arguments[1] else {
            return None;
        };
        if new_length < 0 {
            return None;
        }

        let array_descriptor = value_class_descriptor(&invocation.arguments[0])?;
        if !array_descriptor.starts_with('[') {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let original = self.runtime.borrow().array_entries.get(&array_key)?.clone();
        let mut copied = Vec::with_capacity(usize::try_from(new_length).ok()?);

        for index in 0..usize::try_from(new_length).ok()? {
            copied.push(original.get(index).cloned().unwrap_or(DexValue::Null));
        }

        Some(self.allocate_filled_array(array_descriptor.to_owned(), copied))
    }

    // cite: libcore Arrays.java#copyOfRange
    pub(super) fn arrays_copy_of_range_object(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if !matches!(invocation.arguments.len(), 3 | 4) {
            return None;
        }

        let DexValue::Int(from) = invocation.arguments[1] else {
            return None;
        };
        let DexValue::Int(to) = invocation.arguments[2] else {
            return None;
        };
        if from < 0 || to < from {
            return None;
        }

        let source_descriptor = value_class_descriptor(&invocation.arguments[0])?;
        if !source_descriptor.starts_with('[') {
            return None;
        }
        let copy_descriptor = match invocation.arguments.get(3) {
            Some(new_type) => {
                crate::dex_run::reflection::class_reference_descriptor_from_value(new_type)?
                    .to_owned()
            }
            None => source_descriptor.to_owned(),
        };

        let array_key = map_identity_key(&invocation.arguments[0]);
        let original = self.runtime.borrow().array_entries.get(&array_key)?.clone();
        let from = usize::try_from(from).ok()?;
        let to = usize::try_from(to).ok()?;
        if from > original.len() {
            return None;
        }

        let mut copied = Vec::with_capacity(to - from);
        for index in from..to {
            copied.push(original.get(index).cloned().unwrap_or(DexValue::Null));
        }

        Some(self.allocate_filled_array(copy_descriptor, copied))
    }

    pub(super) fn arrays_copy_of_int(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let DexValue::Int(new_length) = invocation.arguments[1] else {
            return None;
        };
        if new_length < 0 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let original = self.runtime.borrow().array_entries.get(&array_key)?.clone();
        let mut copied = Vec::with_capacity(usize::try_from(new_length).ok()?);

        for index in 0..usize::try_from(new_length).ok()? {
            copied.push(original.get(index).cloned().unwrap_or(DexValue::Int(0)));
        }

        Some(self.allocate_filled_array("[I".to_owned(), copied))
    }

    pub(super) fn arrays_copy_of_long(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let DexValue::Int(new_length) = invocation.arguments[1] else {
            return None;
        };
        if new_length < 0 {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let original = self.runtime.borrow().array_entries.get(&array_key)?.clone();
        let mut copied = Vec::with_capacity(usize::try_from(new_length).ok()?);

        for index in 0..usize::try_from(new_length).ok()? {
            copied.push(original.get(index).cloned().unwrap_or(DexValue::Long(0)));
        }

        Some(self.allocate_filled_array("[J".to_owned(), copied))
    }

    pub(super) fn arrays_sort_object_with_comparator(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<()> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "Arrays.sort(Object[],Comparator) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let array_len = {
            let runtime = self.runtime.borrow();
            let values = runtime.array_entries.get(&array_key).ok_or_else(|| {
                WinedroidError::new("Arrays.sort(Object[],Comparator) received an unknown array")
            })?;
            i32::try_from(values.len()).map_err(|_| {
                WinedroidError::new("Arrays.sort(Object[],Comparator) array length overflowed")
            })?
        };

        self.arrays_sort_object_range_with_comparator(
            &DexInvocation {
                kind: DexInvokeKind::Static,
                range: false,
                method_index: invocation.method_index,
                argument_registers: invocation.argument_registers.clone(),
                arguments: smallvec::smallvec![
                    invocation.arguments[0].clone(),
                    DexValue::Int(0),
                    DexValue::Int(array_len),
                    invocation.arguments[1].clone(),
                ],
            },
            depth + 1,
        )
    }

    pub(super) fn arrays_sort_object_range_with_comparator(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<()> {
        if invocation.arguments.len() != 4 {
            return Err(WinedroidError::new(format!(
                "Arrays.sort(Object[],int,int,Comparator) expected 4 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let DexValue::Int(from_index) = invocation.arguments[1] else {
            return Err(WinedroidError::new(
                "Arrays.sort(Object[],int,int,Comparator) requires an int fromIndex",
            ));
        };
        let DexValue::Int(to_index) = invocation.arguments[2] else {
            return Err(WinedroidError::new(
                "Arrays.sort(Object[],int,int,Comparator) requires an int toIndex",
            ));
        };
        if from_index < 0 || to_index < from_index {
            return Err(WinedroidError::new(format!(
                "Arrays.sort(Object[],int,int,Comparator) received invalid range {from_index}..{to_index}"
            )));
        }

        let array_key = map_identity_key(&invocation.arguments[0]);
        let from_index = usize::try_from(from_index).map_err(|_| {
            WinedroidError::new("Arrays.sort(Object[],int,int,Comparator) fromIndex overflowed")
        })?;
        let to_index = usize::try_from(to_index).map_err(|_| {
            WinedroidError::new("Arrays.sort(Object[],int,int,Comparator) toIndex overflowed")
        })?;
        let mut range = {
            let runtime = self.runtime.borrow();
            let values = runtime.array_entries.get(&array_key).ok_or_else(|| {
                WinedroidError::new(
                    "Arrays.sort(Object[],int,int,Comparator) received an unknown array",
                )
            })?;
            if to_index > values.len() {
                return Err(WinedroidError::new(format!(
                    "Arrays.sort(Object[],int,int,Comparator) range end {to_index} exceeds array length {}",
                    values.len()
                )));
            }

            values[from_index..to_index].to_vec()
        };

        if range.len() > 1 {
            let comparator = invocation.arguments[3].clone();
            if comparator == DexValue::Null {
                return Err(WinedroidError::new(
                    "Arrays.sort(Object[],int,int,Comparator) natural ordering is not implemented\nnext missing primitive: Arrays.sort natural ordering",
                ));
            }

            for index in 1..range.len() {
                let value = range[index].clone();
                let mut insert_at = index;

                while insert_at > 0
                    && self.compare_with_comparator(
                        &comparator,
                        &value,
                        &range[insert_at - 1],
                        depth + 1,
                    )? < 0
                {
                    range[insert_at] = range[insert_at - 1].clone();
                    insert_at -= 1;
                }

                range[insert_at] = value;
            }
        }

        let mut runtime = self.runtime.borrow_mut();
        let values = runtime.array_entries.get_mut(&array_key).ok_or_else(|| {
            WinedroidError::new(
                "Arrays.sort(Object[],int,int,Comparator) array disappeared during sort",
            )
        })?;
        for (offset, value) in range.into_iter().enumerate() {
            values[from_index + offset] = value;
        }

        Ok(())
    }

    fn compare_with_comparator(
        &self,
        comparator: &DexValue,
        left: &DexValue,
        right: &DexValue,
        depth: usize,
    ) -> Result<i32> {
        let Some(class_descriptor) = value_class_descriptor(comparator) else {
            return Err(WinedroidError::new(format!(
                "Comparator.compare requires an object-backed comparator\nargument: {comparator}"
            )));
        };
        let method = DexMethodId {
            class_descriptor: class_descriptor.to_owned(),
            name: "compare".to_owned(),
            proto: crate::dex::DexProto {
                shorty: "ILL".to_owned(),
                return_type: "I".to_owned(),
                parameters: vec![
                    "Ljava/lang/Object;".to_owned(),
                    "Ljava/lang/Object;".to_owned(),
                ],
            },
        };
        let Some(resolved) =
            self.resolve_exact_method_in_class_hierarchy(class_descriptor, &method)?
        else {
            return Err(WinedroidError::new(format!(
                "Comparator.compare(Object,Object) was not found\ncomparator class: {class_descriptor}"
            )));
        };
        let result = self.execute_method(
            None,
            &resolved,
            Some(&[comparator.clone(), left.clone(), right.clone()]),
            depth + 1,
        )?;

        match result {
            DexExecutionResult::Int(value) => Ok(value),
            DexExecutionResult::Throw(value) => Err(WinedroidError::new(format!(
                "Comparator.compare(Object,Object) threw\ncomparator: {comparator}\nleft: {left}\nright: {right}\nthrowable: {value}"
            ))),
            DexExecutionResult::Void => Err(WinedroidError::new(
                "Comparator.compare(Object,Object) returned void",
            )),
            DexExecutionResult::Value(value) => Err(WinedroidError::new(format!(
                "Comparator.compare(Object,Object) returned an object\nvalue: {value}"
            ))),
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                Err(WinedroidError::new(
                    "Comparator.compare(Object,Object) parked while sorting",
                ))
            }
        }
    }

    pub(super) fn collections_singleton_map(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let map = self.allocate_object("Ljava/util/LinkedHashMap;".to_owned());
        let map_key = map_identity_key(&map);
        let entry_key = self.map_storage_entry_key(&invocation.arguments[0]);
        let mut runtime = self.runtime.borrow_mut();
        runtime.map_entry_keys.insert(
            (map_key.clone(), entry_key.clone()),
            invocation.arguments[0].clone(),
        );
        runtime
            .map_entries
            .insert((map_key, entry_key), invocation.arguments[1].clone());

        Some(map)
    }

    pub(super) fn collections_singleton_set(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let value = invocation.arguments[0].clone();
        let mut runtime = self.runtime.borrow_mut();
        let set_key = format!("collections:singleton:{}", runtime.next_object_id);
        runtime.next_object_id += 1;
        let set = DexValue::Reference(std::rc::Rc::from(format!("Ljava/util/Set;:{set_key}")));

        runtime
            .list_entries
            .insert(map_identity_key(&set), vec![value]);

        Some(set)
    }

    pub(super) fn collections_new_set_from_map(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let backing_map_key = map_identity_key(&invocation.arguments[0]);
        let mut runtime = self.runtime.borrow_mut();
        let set_key = format!(
            "collections:newSetFromMap:{}:{}",
            runtime.next_object_id, backing_map_key
        );
        runtime.next_object_id += 1;
        let set = DexValue::Reference(std::rc::Rc::from(format!("Ljava/util/Set;:{set_key}")));

        runtime
            .list_entries
            .insert(map_identity_key(&set), Vec::new());

        Some(set)
    }

    pub(super) fn collections_synchronized_map(invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        Some(invocation.arguments[0].clone())
    }

    pub(super) fn collections_identity(invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        Some(invocation.arguments[0].clone())
    }

    pub(super) fn initialize_random<A: MethodIntrinsicArguments + ?Sized>(
        &self,
        invocation: &A,
    ) -> Result<Option<()>> {
        if !matches!(invocation.argument_count(), 1 | 2) {
            return Ok(None);
        }

        let Some(receiver) = invocation.argument_value(0)? else {
            return Ok(None);
        };
        if !dex_value_is_object_like(&receiver) {
            return Ok(None);
        }

        let seed = if invocation.argument_count() == 2 {
            let Some(value) = invocation.argument_value(1)? else {
                return Ok(None);
            };
            let Some(seed) = read_long_argument(&value) else {
                return Ok(None);
            };
            seed
        } else {
            let runtime = self.runtime.borrow();
            runtime.boot_wall_clock_millis ^ runtime.next_object_id as i64
        };

        self.runtime.borrow_mut().random_seeds.insert(
            map_identity_key(&receiver),
            java_random_initial_scramble(seed),
        );

        Ok(Some(()))
    }

    pub(super) fn random_next_unbounded_int<A: MethodIntrinsicArguments + ?Sized>(
        &self,
        invocation: &A,
    ) -> Result<Option<i32>> {
        if invocation.argument_count() != 1 {
            return Ok(None);
        }

        let Some(receiver) = invocation.argument_value(0)? else {
            return Ok(None);
        };
        if !dex_value_is_object_like(&receiver) {
            return Ok(None);
        }

        let random_key = map_identity_key(&receiver);
        let mut runtime = self.runtime.borrow_mut();
        let seed = runtime
            .random_seeds
            .entry(random_key)
            .or_insert_with(|| java_random_initial_scramble(0));

        Ok(Some(java_random_next_bits(seed, 32)))
    }

    pub(super) fn random_next_int<A: MethodIntrinsicArguments + ?Sized>(
        &self,
        invocation: &A,
    ) -> Result<Option<i32>> {
        if invocation.argument_count() != 2 {
            return Ok(None);
        }

        let Some(receiver) = invocation.argument_value(0)? else {
            return Ok(None);
        };
        if !dex_value_is_object_like(&receiver) {
            return Ok(None);
        }

        let Some(bound_value) = invocation.argument_value(1)? else {
            return Ok(None);
        };
        let Some(bound) = read_int_argument(&bound_value) else {
            return Ok(None);
        };
        if bound <= 0 {
            return Ok(None);
        }

        let random_key = map_identity_key(&receiver);
        let mut runtime = self.runtime.borrow_mut();
        let seed = runtime
            .random_seeds
            .entry(random_key)
            .or_insert_with(|| java_random_initial_scramble(0));
        let value = java_random_next_int_bound(seed, bound);

        Ok(Some(value))
    }

    pub(super) fn read_array_list_value(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let list_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::Int(index) = invocation.arguments[1] else {
            return None;
        };

        if index < 0 {
            return None;
        }

        Some(
            runtime_list_values(&self.runtime.borrow(), &list_key)
                .and_then(|values| values.get(index as usize).cloned())
                .unwrap_or(DexValue::Null),
        )
    }

    pub(super) fn remove_array_list_value(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let list_key = map_identity_key(&invocation.arguments[0]);
        let DexValue::Int(index) = invocation.arguments[1] else {
            return None;
        };

        if index < 0 {
            return None;
        }

        let mut runtime = self.runtime.borrow_mut();
        let values = runtime.list_entries.get_mut(&list_key)?;

        if index as usize >= values.len() {
            return None;
        }

        Some(values.remove(index as usize))
    }

    pub(super) fn array_list_to_array(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let list_key = map_identity_key(&invocation.arguments[0]);
        let values = runtime_list_values(&self.runtime.borrow(), &list_key).unwrap_or_default();

        self.collection_values_to_array(values, &invocation.arguments[1])
    }

    pub(super) fn java_collection_to_array(
        &self,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<DexValue> {
        if invocation.arguments.len() != 2 {
            return Err(WinedroidError::new(format!(
                "Collection.toArray(T[]) expected 2 arguments, got {}",
                invocation.arguments.len()
            )));
        }

        let values = self.collection_values_by_contract(&invocation.arguments[0], depth + 1)?;
        self.collection_values_to_array(values, &invocation.arguments[1])
            .ok_or_else(|| {
                WinedroidError::new(format!(
                    "Collection.toArray(T[]) could not write result array\ncollection: {}\narray: {}",
                    invocation.arguments[0], invocation.arguments[1]
                ))
            })
    }

    fn collection_values_to_array(
        &self,
        values: Vec<DexValue>,
        array: &DexValue,
    ) -> Option<DexValue> {
        let value_count = values.len();
        let value_count_i32 = i32::try_from(value_count).ok()?;

        let array_descriptor = value_class_descriptor(&array)?;
        if !array_descriptor.starts_with('[') {
            return None;
        }

        let array_key = map_identity_key(&array);
        let target_length = {
            let runtime = self.runtime.borrow();
            runtime.array_entries.get(&array_key)?.len()
        };

        let target = if target_length >= value_count {
            array.clone()
        } else {
            self.allocate_array(array_descriptor.to_owned(), value_count_i32)?
        };

        let target_key = map_identity_key(&target);
        let mut runtime = self.runtime.borrow_mut();
        let target_values = runtime.array_entries.get_mut(&target_key)?;

        for (index, value) in values.into_iter().enumerate() {
            let slot = target_values.get_mut(index)?;
            *slot = value;
        }

        if target_values.len() > value_count {
            let null_slot = target_values.get_mut(value_count)?;
            *null_slot = DexValue::Null;
        }

        Some(target)
    }

    pub(super) fn clear_list(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let list_key = map_identity_key(&invocation.arguments[0]);
        let mut runtime = self.runtime.borrow_mut();
        if let Some(sub_list) = runtime.list_sub_lists.get(&list_key).cloned() {
            let values = runtime.list_entries.get_mut(&sub_list.parent_key)?;
            if sub_list.from_index > sub_list.to_index || sub_list.to_index > values.len() {
                return None;
            }

            values.drain(sub_list.from_index..sub_list.to_index);
            if let Some(stored_sub_list) = runtime.list_sub_lists.get_mut(&list_key) {
                stored_sub_list.to_index = stored_sub_list.from_index;
            }

            return Some(());
        }

        runtime.list_entries.entry(list_key).or_default().clear();

        Some(())
    }
}

fn runtime_list_len(runtime: &DexRuntimeState, list_key: &str) -> Option<usize> {
    if let Some(values) = runtime.list_entries.get(list_key) {
        return Some(values.len());
    }

    let sub_list = runtime.list_sub_lists.get(list_key)?;
    let values = runtime.list_entries.get(&sub_list.parent_key)?;
    values
        .get(sub_list.from_index..sub_list.to_index)
        .map(<[DexValue]>::len)
}

fn runtime_list_values(runtime: &DexRuntimeState, list_key: &str) -> Option<Vec<DexValue>> {
    if let Some(values) = runtime.list_entries.get(list_key) {
        return Some(values.clone());
    }

    let sub_list = runtime.list_sub_lists.get(list_key)?;
    let values = runtime.list_entries.get(&sub_list.parent_key)?;
    values
        .get(sub_list.from_index..sub_list.to_index)
        .map(<[DexValue]>::to_vec)
}

const JAVA_RANDOM_MULTIPLIER: i64 = 0x5DEECE66D;
const JAVA_RANDOM_ADDEND: i64 = 0xB;
const JAVA_RANDOM_MASK: i64 = (1_i64 << 48) - 1;

// cite: libcore Random.java#initialScramble
fn java_random_initial_scramble(seed: i64) -> i64 {
    (seed ^ JAVA_RANDOM_MULTIPLIER) & JAVA_RANDOM_MASK
}

// cite: libcore Random.java#next
fn java_random_next_bits(seed: &mut i64, bits: i32) -> i32 {
    *seed = seed
        .wrapping_mul(JAVA_RANDOM_MULTIPLIER)
        .wrapping_add(JAVA_RANDOM_ADDEND)
        & JAVA_RANDOM_MASK;
    (*seed >> (48 - bits)) as i32
}

// cite: libcore Random.java#nextInt
fn java_random_next_int_bound(seed: &mut i64, bound: i32) -> i32 {
    let mut r = java_random_next_bits(seed, 31);
    let m = bound - 1;
    if bound & m == 0 {
        return ((bound as i64 * r as i64) >> 31) as i32;
    }

    loop {
        let value = r % bound;
        if r.wrapping_sub(value).wrapping_add(m) >= 0 {
            return value;
        }
        r = java_random_next_bits(seed, 31);
    }
}

fn normalize_sparse_boolean(value: i32) -> i32 {
    if value == 0 {
        0
    } else {
        1
    }
}

impl DexProgram {
    pub(super) fn invoke_java_collection_equality_method(
        &self,
        method: &DexMethodId,
        invocation: &DexInvocation,
        depth: usize,
    ) -> Result<Option<DexExecutionResult>> {
        let uses_key_eq = self.map_invocation_uses_java_key_equality(method, invocation);
        if uses_key_eq
            && method.name == "get"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;)Ljava/lang/Object;")
        {
            let value = self.java_read_hash_map_value(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Value(value)));
        }

        if uses_key_eq
            && method.name == "containsKey"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let value = self.java_hash_map_contains_key(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Int(value)));
        }

        if uses_key_eq
            && method.name == "put"
            && method.proto.descriptor()
                == "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;"
        {
            let previous = self.java_write_hash_map_value(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Value(previous)));
        }

        if uses_key_eq
            && method.name == "remove"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;)Ljava/lang/Object;")
        {
            let previous = self.java_remove_hash_map_value(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Value(previous)));
        }

        if is_supported_set_class(&method.class_descriptor)
            && method.name == "contains"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let value = self.java_collection_contains_value(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Int(value)));
        }

        if is_supported_set_class(&method.class_descriptor)
            && method.name == "remove"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let value = self.java_remove_collection_value(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Int(value)));
        }

        if is_supported_set_class(&method.class_descriptor)
            && method.name == "add"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
            && method.class_descriptor != "Ljava/util/TreeSet;"
        {
            let value = self.java_add_set_value(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Int(value)));
        }

        if is_supported_set_class(&method.class_descriptor)
            && method.name == "addAll"
            && method.proto.descriptor_is("(Ljava/util/Collection;)Z")
        {
            let value = self.java_add_all_set_values(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Int(value)));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/ArrayList;"
                | "Ljava/util/Collection;"
                | "Ljava/util/AbstractCollection;"
                | "Ljava/util/AbstractSet;"
                | "Ljava/util/Set;"
                | "Ljava/util/HashSet;"
                | "Ljava/util/LinkedHashSet;"
                | "Ljava/util/TreeSet;"
                | "Ljava/util/concurrent/CopyOnWriteArraySet;"
        ) && method.name == "toArray"
            && method
                .proto
                .descriptor_is("([Ljava/lang/Object;)[Ljava/lang/Object;")
        {
            let value = self.java_collection_to_array(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Value(value)));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/HashSet;"
                | "Ljava/util/LinkedHashSet;"
                | "Ljava/util/concurrent/CopyOnWriteArraySet;"
        ) && method.name == "<init>"
            && method.proto.descriptor_is("(Ljava/util/Collection;)V")
        {
            self.initialize_set_from_collection(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Void));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/Collection;"
                | "Ljava/util/AbstractCollection;"
                | "Ljava/util/List;"
                | "Ljava/util/AbstractList;"
                | "Ljava/util/ArrayList;"
                | "Ljava/util/concurrent/CopyOnWriteArrayList;"
        ) && method.name == "addAll"
            && method.proto.descriptor_is("(Ljava/util/Collection;)Z")
        {
            let value = self.add_all_collection_values(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Int(value)));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/List;" | "Ljava/util/AbstractList;" | "Ljava/util/ArrayList;"
        ) && method.name == "addAll"
            && method.proto.descriptor_is("(ILjava/util/Collection;)Z")
        {
            let value = self.add_all_array_list_values_at(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Int(value)));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/List;"
                | "Ljava/util/AbstractList;"
                | "Ljava/util/ArrayList;"
                | "Ljava/util/concurrent/CopyOnWriteArrayList;"
        ) && method.name == "contains"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let value = self.java_collection_contains_value(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Int(value)));
        }

        // cite: libcore AbstractCollection.java#contains
        if method.class_descriptor == "Ljava/util/AbstractCollection;"
            && method.name == "contains"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let value = self.java_collection_contains_value(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Int(value)));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/List;"
                | "Ljava/util/AbstractList;"
                | "Ljava/util/ArrayList;"
                | "Ljava/util/concurrent/CopyOnWriteArrayList;"
        ) && method.name == "indexOf"
            && method.proto.descriptor_is("(Ljava/lang/Object;)I")
        {
            let value = self.java_collection_index_of_value(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Int(value)));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/Collection;"
                | "Ljava/util/AbstractCollection;"
                | "Ljava/util/List;"
                | "Ljava/util/AbstractList;"
                | "Ljava/util/ArrayList;"
                | "Ljava/util/concurrent/CopyOnWriteArrayList;"
        ) && method.name == "remove"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let value = self.java_remove_collection_value(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Int(value)));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/Collection;" | "Ljava/util/AbstractCollection;"
        ) && method.name == "removeAll"
            && method.proto.descriptor_is("(Ljava/util/Collection;)Z")
        {
            let value = self.java_remove_all_collection_values(invocation, depth + 1)?;
            return Ok(Some(DexExecutionResult::Int(value)));
        }

        Ok(None)
    }

    fn map_invocation_uses_java_key_equality(
        &self,
        method: &DexMethodId,
        invocation: &DexInvocation,
    ) -> bool {
        if !is_supported_map_class(&method.class_descriptor) {
            return false;
        }

        let receiver_descriptor = invocation
            .arguments
            .first()
            .and_then(value_class_descriptor)
            .unwrap_or(method.class_descriptor.as_str());

        is_java_key_equality_map_class(receiver_descriptor)
    }
}

pub(super) fn empty_iterator() -> DexValue {
    DexValue::Reference(std::rc::Rc::from("Ljava/util/Iterator;.empty"))
}

fn iterator_storage_key(reference: &str) -> Option<&str> {
    reference
        .strip_prefix("Ljava/util/Iterator;:")
        .or_else(|| reference.strip_prefix("Ljava/util/ListIterator;:"))
}

pub(super) fn map_entry_object_key(invocation: &DexInvocation) -> Option<String> {
    if invocation.arguments.len() != 1 {
        return None;
    }

    let DexValue::Reference(reference) = &invocation.arguments[0] else {
        return None;
    };

    Some(reference.clone().to_string())
}

pub(super) fn is_supported_map_class(class_descriptor: &str) -> bool {
    matches!(
        class_descriptor,
        "Ljava/util/Map;"
            | "Ljava/util/AbstractMap;"
            | "Ljava/util/HashMap;"
            | "Ljava/util/IdentityHashMap;"
            | "Ljava/util/LinkedHashMap;"
            | "Ljava/util/TreeMap;"
            | "Ljava/util/WeakHashMap;"
            | "Ljava/util/concurrent/ConcurrentHashMap;"
    )
}

pub(super) fn is_supported_map_implementation_class(class_descriptor: &str) -> bool {
    matches!(
        class_descriptor,
        "Ljava/util/HashMap;"
            | "Ljava/util/IdentityHashMap;"
            | "Ljava/util/LinkedHashMap;"
            | "Ljava/util/TreeMap;"
            | "Ljava/util/WeakHashMap;"
            | "Ljava/util/concurrent/ConcurrentHashMap;"
    )
}

fn is_java_key_equality_map_class(class_descriptor: &str) -> bool {
    matches!(
        class_descriptor,
        "Ljava/util/Map;"
            | "Ljava/util/AbstractMap;"
            | "Ljava/util/HashMap;"
            | "Ljava/util/LinkedHashMap;"
            | "Ljava/util/WeakHashMap;"
            | "Ljava/util/concurrent/ConcurrentHashMap;"
    )
}

pub(super) fn is_supported_set_class(class_descriptor: &str) -> bool {
    matches!(
        class_descriptor,
        "Ljava/util/Set;"
            | "Ljava/util/HashSet;"
            | "Ljava/util/LinkedHashSet;"
            | "Ljava/util/TreeSet;"
            | "Ljava/util/concurrent/CopyOnWriteArraySet;"
            | "Ls/e;"
    )
}

fn is_managed_concrete_list_descriptor(class_descriptor: &str) -> bool {
    matches!(
        class_descriptor,
        "Ljava/util/ArrayList;"
            | "Ljava/util/Arrays$ArrayList;"
            | "Ljava/util/PriorityQueue;"
            | "Ljava/util/concurrent/CopyOnWriteArrayList;"
    )
}

fn skip_json_whitespace(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while matches!(chars.peek(), Some(' ' | '\t' | '\n' | '\r')) {
        chars.next();
    }
}

fn parse_json_string(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<String> {
    if chars.next()? != '"' {
        return None;
    }
    let mut text = String::new();
    loop {
        match chars.next()? {
            '"' => return Some(text),
            '\\' => match chars.next()? {
                '"' => text.push('"'),
                '\\' => text.push('\\'),
                '/' => text.push('/'),
                'n' => text.push('\n'),
                'r' => text.push('\r'),
                't' => text.push('\t'),
                'b' => text.push('\u{8}'),
                'f' => text.push('\u{c}'),
                'u' => {
                    let mut code = 0u32;
                    for _ in 0..4 {
                        code = code * 16 + chars.next()?.to_digit(16)?;
                    }
                    text.push(char::from_u32(code)?);
                }
                _ => return None,
            },
            other => text.push(other),
        }
    }
}

fn parse_json_number(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<DexValue> {
    let mut text = String::new();
    while let Some(&c) = chars.peek() {
        if c == '-' || c == '+' || c == '.' || c == 'e' || c == 'E' || c.is_ascii_digit() {
            text.push(c);
            chars.next();
        } else {
            break;
        }
    }

    if !text.contains(['.', 'e', 'E']) {
        if let Ok(value) = text.parse::<i32>() {
            return Some(DexValue::Int(value));
        }
        if let Ok(value) = text.parse::<i64>() {
            return Some(DexValue::Long(value));
        }
    }

    text.parse::<f32>().ok().map(DexValue::Float)
}

fn parse_json_literal(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<DexValue> {
    let mut word = String::new();
    while matches!(chars.peek(), Some(c) if c.is_ascii_alphabetic()) {
        word.push(chars.next()?);
    }
    match word.as_str() {
        "true" => Some(DexValue::Int(1)),
        "false" => Some(DexValue::Int(0)),
        "null" => Some(DexValue::Null),
        _ => None,
    }
}

pub(in crate::dex_run) fn json_coerce_to_string(value: &DexValue) -> Option<String> {
    match value {
        DexValue::String(text) => Some(text.clone().to_string()),
        DexValue::Int(value) => Some(value.to_string()),
        DexValue::Long(value) => Some(value.to_string()),
        DexValue::Float(value) => Some(value.to_string()),
        DexValue::Null | DexValue::Object(_) | DexValue::Reference(_) | DexValue::Uninitialized => {
            None
        }
    }
}

pub(super) fn json_quote(value: &str) -> String {
    let mut encoded = String::from("\"");

    for character in value.chars() {
        match character {
            '"' => encoded.push_str("\\\""),
            '\\' => encoded.push_str("\\\\"),
            '\n' => encoded.push_str("\\n"),
            '\r' => encoded.push_str("\\r"),
            '\t' => encoded.push_str("\\t"),
            _ => encoded.push(character),
        }
    }

    encoded.push('"');
    encoded
}
