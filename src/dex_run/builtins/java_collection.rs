use super::super::*;

impl DexProgram {
    pub(super) fn invoke_java_collection_builtin(
        &self,
        method: &DexMethodId,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        if is_supported_set_class(&method.class_descriptor)
            && method.name == "iterator"
            && method.proto.descriptor_is("()Ljava/util/Iterator;")
        {
            let value = self.set_iterator(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if is_supported_set_class(&method.class_descriptor)
            && method.name == "size"
            && method.proto.descriptor_is("()I")
        {
            let value = self.set_size(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if is_supported_set_class(&method.class_descriptor)
            && method.name == "isEmpty"
            && method.proto.descriptor_is("()Z")
        {
            let value = i32::from(self.set_size(invocation)? == 0);

            return Some(DexExecutionResult::Int(value));
        }

        if is_supported_set_class(&method.class_descriptor)
            && method.name == "contains"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let invocation = invocation?;
            let value = self.list_contains(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if is_supported_set_class(&method.class_descriptor)
            && method.name == "remove"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let invocation = invocation?;
            let value = self.remove_collection_value(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if is_supported_set_class(&method.class_descriptor)
            && method.name == "clear"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.clear_list(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/TreeSet;"
            && method.name == "first"
            && method.proto.descriptor_is("()Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.tree_set_first(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if is_supported_set_class(&method.class_descriptor)
            && method.name == "add"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            if method.class_descriptor == "Ljava/util/TreeSet;" {
                return None;
            }

            let invocation = invocation?;
            let value = self.add_set_value(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if is_supported_set_class(&method.class_descriptor)
            && method.name == "addAll"
            && method.proto.descriptor_is("(Ljava/util/Collection;)Z")
        {
            let invocation = invocation?;
            let value = self.java_add_all_set_values(invocation, 0).ok()?;

            return Some(DexExecutionResult::Int(value));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/Collection;" | "Ljava/util/AbstractCollection;"
        ) && method.name == "add"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let invocation = invocation?;
            self.add_collection_value(invocation)?;

            return Some(DexExecutionResult::Int(1));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/Collection;" | "Ljava/util/AbstractCollection;"
        ) && method.name == "isEmpty"
            && method.proto.descriptor_is("()Z")
        {
            let value = self.collection_is_empty(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/Collection;" | "Ljava/util/AbstractCollection;"
        ) && method.name == "size"
            && method.proto.descriptor_is("()I")
        {
            let value = self.list_size(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/PriorityQueue;"
            && method.name == "size"
            && method.proto.descriptor_is("()I")
        {
            let value = self.list_size(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/PriorityQueue;"
            && method.name == "add"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let invocation = invocation?;
            let value = self.priority_queue_add(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/PriorityQueue;"
            && method.name == "iterator"
            && method.proto.descriptor_is("()Ljava/util/Iterator;")
        {
            let Some(invocation) = invocation else {
                return Some(DexExecutionResult::Value(empty_iterator()));
            };
            let value = self.collection_iterator(&invocation.arguments[0])?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: libcore AbstractList.java#iterator, Itr
        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/Collection;" | "Ljava/util/AbstractCollection;" | "Ljava/lang/Iterable;"
        ) && method.name == "iterator"
            && method.proto.descriptor_is("()Ljava/util/Iterator;")
        {
            let Some(invocation) = invocation else {
                return Some(DexExecutionResult::Value(empty_iterator()));
            };
            let value = self
                .collection_iterator(&invocation.arguments[0])
                .or_else(|| self.abstract_list_snapshot_iterator(invocation))?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/Collection;"
            && method.name == "removeAll"
            && method.proto.descriptor_is("(Ljava/util/Collection;)Z")
        {
            let invocation = invocation?;
            let removed = self.remove_all_collection_values(invocation)?;

            return Some(DexExecutionResult::Int(removed));
        }

        // cite: libcore AbstractCollection.java#retainAll
        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/Collection;" | "Ljava/util/Set;" | "Ljava/util/List;"
        ) && method.name == "retainAll"
            && method.proto.descriptor_is("(Ljava/util/Collection;)Z")
        {
            let invocation = invocation?;
            let changed = self.retain_all_collection_values(invocation)?;

            return Some(DexExecutionResult::Int(changed));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/Collection;" | "Ljava/util/AbstractCollection;"
        ) && method.name == "remove"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let invocation = invocation?;
            let removed = self.remove_collection_value(invocation)?;

            return Some(DexExecutionResult::Int(removed));
        }

        if method.class_descriptor == "Ljava/util/Collections;"
            && method.name == "singletonList"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;)Ljava/util/List;")
        {
            let invocation = invocation?;
            let value = self.collections_singleton_list(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/Collections;"
            && method.name == "singletonMap"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;Ljava/lang/Object;)Ljava/util/Map;")
        {
            // cite: libcore Collections.java#singletonMap
            let invocation = invocation?;
            let value = self.collections_singleton_map(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/Collections;"
            && method.name == "emptyList"
            && method.proto.descriptor_is("()Ljava/util/List;")
        {
            let value = self.collections_empty_list();

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/Collections;"
            && method.name == "reverse"
            && method.proto.descriptor_is("(Ljava/util/List;)V")
        {
            let invocation = invocation?;
            self.collections_reverse_list(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/Collections;"
            && method.name == "sort"
            && method.proto.descriptor_is("(Ljava/util/List;)V")
        {
            let invocation = invocation?;
            self.collections_sort_list(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/Collections;"
            && method.name == "unmodifiableList"
            && method
                .proto
                .descriptor_is("(Ljava/util/List;)Ljava/util/List;")
        {
            let invocation = invocation?;
            let value = Self::collections_identity(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/Collections;"
            && method.name == "binarySearch"
            && method
                .proto
                .descriptor_is("(Ljava/util/List;Ljava/lang/Object;)I")
        {
            let invocation = invocation?;
            let value = self.collections_binary_search(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/Arrays;"
            && method.name == "asList"
            && method
                .proto
                .descriptor_is("([Ljava/lang/Object;)Ljava/util/List;")
        {
            let invocation = invocation?;
            let value = self.arrays_as_list(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/Arrays;"
            && method.name == "fill"
            && method.proto.descriptor_is("([JIIJ)V")
        {
            let invocation = invocation?;
            self.arrays_fill_long_range(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/Arrays;"
            && method.name == "fill"
            && method.proto.descriptor_is("([IIII)V")
        {
            let invocation = invocation?;
            self.arrays_fill_int_range(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/Arrays;"
            && method.name == "fill"
            && method
                .proto
                .descriptor_is("([Ljava/lang/Object;IILjava/lang/Object;)V")
        {
            let invocation = invocation?;
            self.arrays_fill_object_range(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        // cite: libcore Arrays.java#fill
        if method.class_descriptor == "Ljava/util/Arrays;"
            && method.name == "fill"
            && matches!(
                method.proto.descriptor().as_str(),
                "([BB)V" | "([SS)V" | "([CC)V" | "([II)V" | "([ZZ)V" | "([JJ)V"
            )
        {
            let invocation = invocation?;
            let array = invocation.arguments.first()?;
            let fill_val = invocation.arguments.get(1)?.clone();
            let array_key = map_identity_key(array);
            let mut runtime = self.runtime.borrow_mut();
            let entries = runtime.array_entries.get_mut(&array_key)?;
            for slot in entries.iter_mut() {
                *slot = fill_val.clone();
            }

            return Some(DexExecutionResult::Void);
        }

        // cite: libcore Arrays.java#fill
        if method.class_descriptor == "Ljava/util/Arrays;"
            && method.name == "fill"
            && method
                .proto
                .descriptor_is("([Ljava/lang/Object;Ljava/lang/Object;)V")
        {
            let invocation = invocation?;
            let array = invocation.arguments.first()?;
            let fill_val = invocation.arguments.get(1)?.clone();
            let array_key = map_identity_key(array);
            let mut runtime = self.runtime.borrow_mut();
            let entries = runtime.array_entries.get_mut(&array_key)?;
            for slot in entries.iter_mut() {
                *slot = fill_val.clone();
            }

            return Some(DexExecutionResult::Void);
        }

        // cite: libcore AbstractMap.java#SimpleImmutableEntry
        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/AbstractMap$SimpleImmutableEntry;" | "Ljava/util/AbstractMap$SimpleEntry;"
        ) && method.name == "<init>"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;Ljava/lang/Object;)V")
        {
            let invocation = invocation?;
            let receiver = invocation.arguments.first()?;
            let key = invocation.arguments.get(1)?.clone();
            let value = invocation.arguments.get(2)?.clone();
            self.write_stored_instance_field_by_signature(
                receiver,
                "Ljava/util/AbstractMap$SimpleImmutableEntry;.key:Ljava/lang/Object;",
                key,
            );
            self.write_stored_instance_field_by_signature(
                receiver,
                "Ljava/util/AbstractMap$SimpleImmutableEntry;.value:Ljava/lang/Object;",
                value,
            );
            return Some(DexExecutionResult::Void);
        }

        // cite: libcore Map.java#getKey, getValue
        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/Map$Entry;"
                | "Ljava/util/AbstractMap$SimpleImmutableEntry;"
                | "Ljava/util/AbstractMap$SimpleEntry;"
        ) && method.name == "getKey"
            && method.proto.descriptor_is("()Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let receiver = invocation.arguments.first()?;
            if let Some(value) = self.read_stored_instance_field_by_signature(
                receiver,
                "Ljava/util/AbstractMap$SimpleImmutableEntry;.key:Ljava/lang/Object;",
            ) {
                return Some(DexExecutionResult::Value(value));
            }
        }

        // cite: libcore Map.java#getKey, getValue
        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/Map$Entry;"
                | "Ljava/util/AbstractMap$SimpleImmutableEntry;"
                | "Ljava/util/AbstractMap$SimpleEntry;"
        ) && method.name == "getValue"
            && method.proto.descriptor_is("()Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let receiver = invocation.arguments.first()?;
            if let Some(value) = self.read_stored_instance_field_by_signature(
                receiver,
                "Ljava/util/AbstractMap$SimpleImmutableEntry;.value:Ljava/lang/Object;",
            ) {
                return Some(DexExecutionResult::Value(value));
            }
        }

        if method.class_descriptor == "Ljava/util/Arrays;"
            && method.name == "equals"
            && method
                .proto
                .descriptor_is("([Ljava/lang/Object;[Ljava/lang/Object;)Z")
        {
            let invocation = invocation?;
            let value = self.arrays_equals_object(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/Arrays;"
            && method.name == "sort"
            && method.proto.descriptor_is("([III)V")
        {
            let invocation = invocation?;
            self.arrays_sort_int_range(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/Arrays;"
            && method.name == "copyOf"
            && method
                .proto
                .descriptor_is("([Ljava/lang/Object;I)[Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.arrays_copy_of_object(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/Arrays;"
            && method.name == "copyOfRange"
            && matches!(
                method.proto.descriptor().as_str(),
                "([Ljava/lang/Object;II)[Ljava/lang/Object;"
                    | "([Ljava/lang/Object;IILjava/lang/Class;)[Ljava/lang/Object;"
            )
        {
            let invocation = invocation?;
            let value = self.arrays_copy_of_range_object(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/Arrays;"
            && method.name == "copyOf"
            && method.proto.descriptor_is("([II)[I")
        {
            let invocation = invocation?;
            let value = self.arrays_copy_of_int(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/Arrays;"
            && method.name == "copyOf"
            && method.proto.descriptor_is("([JI)[J")
        {
            let invocation = invocation?;
            let value = self.arrays_copy_of_long(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: libcore Arrays.java#sort
        if method.class_descriptor == "Ljava/util/Arrays;"
            && method.name == "sort"
            && matches!(
                method.proto.descriptor().as_str(),
                "([Ljava/lang/Object;)V" | "([Ljava/lang/Object;II)V"
            )
        {
            let invocation = invocation?;
            let array_key = map_identity_key(invocation.arguments.first()?);
            let mut entries = self
                .runtime
                .borrow()
                .array_entries
                .get(&array_key)
                .cloned()?;
            let (from, to) = if invocation.arguments.len() == 3 {
                let DexValue::Int(f) = invocation.arguments[1] else {
                    return None;
                };
                let DexValue::Int(t) = invocation.arguments[2] else {
                    return None;
                };
                (
                    usize::try_from(f).ok()?.min(entries.len()),
                    usize::try_from(t).ok()?.min(entries.len()),
                )
            } else {
                (0, entries.len())
            };
            if from < to {
                entries[from..to].sort_by(|a, b| self.tree_map_compare_keys(a, b));
            }
            self.runtime
                .borrow_mut()
                .array_entries
                .insert(array_key, entries);
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/Collections;"
            && method.name == "singleton"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;)Ljava/util/Set;")
        {
            let invocation = invocation?;
            let value = self.collections_singleton_set(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/Collections;"
            && method.name == "newSetFromMap"
            && method
                .proto
                .descriptor_is("(Ljava/util/Map;)Ljava/util/Set;")
        {
            let invocation = invocation?;
            let value = self.collections_new_set_from_map(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/Collections;"
            && method.name == "emptyMap"
            && method.proto.descriptor_is("()Ljava/util/Map;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Ljava/util/Map;:collections:emptyMap"),
            )));
        }

        if method.class_descriptor == "Ljava/util/Collections;"
            && method.name == "unmodifiableMap"
            && method
                .proto
                .descriptor_is("(Ljava/util/Map;)Ljava/util/Map;")
        {
            let invocation = invocation?;
            let value = Self::collections_identity(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/Collections;"
            && method.name == "synchronizedMap"
            && method
                .proto
                .descriptor_is("(Ljava/util/Map;)Ljava/util/Map;")
        {
            let invocation = invocation?;
            let value = Self::collections_synchronized_map(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Iterable;"
            && method.name == "iterator"
            && method.proto.descriptor_is("()Ljava/util/Iterator;")
        {
            let value = self.iterable_iterator(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: libcore AbstractList.java#iterator, Itr
        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/List;" | "Ljava/util/AbstractList;"
        ) && method.name == "iterator"
            && method.proto.descriptor_is("()Ljava/util/Iterator;")
        {
            let invocation = invocation?;
            let value = self
                .list_iterator(invocation)
                .or_else(|| self.abstract_list_snapshot_iterator(invocation))?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/List;"
            && method.name == "listIterator"
            && method.proto.descriptor_is("(I)Ljava/util/ListIterator;")
        {
            let invocation = invocation?;
            let value = self.list_iterator_at(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/List;"
            && method.name == "size"
            && method.proto.descriptor_is("()I")
        {
            let value = self.list_size(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/List;"
            && method.name == "isEmpty"
            && method.proto.descriptor_is("()Z")
        {
            let value = self.list_is_empty(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/List;"
            && method.name == "get"
            && method.proto.descriptor_is("(I)Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.read_array_list_value(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/List;" | "Ljava/util/AbstractList;" | "Ljava/util/ArrayList;"
        ) && method.name == "set"
            && method
                .proto
                .descriptor_is("(ILjava/lang/Object;)Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.set_array_list_value(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/List;"
            && method.name == "contains"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let invocation = invocation?;
            let value = self.list_contains(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/List;"
            && method.name == "add"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let invocation = invocation?;
            self.add_array_list_value(invocation)?;

            return Some(DexExecutionResult::Int(1));
        }

        if method.class_descriptor == "Ljava/util/List;"
            && method.name == "add"
            && method.proto.descriptor_is("(ILjava/lang/Object;)V")
        {
            let invocation = invocation?;
            self.add_array_list_value_at(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/List;" | "Ljava/util/AbstractList;" | "Ljava/util/ArrayList;"
        ) && method.name == "subList"
            && method.proto.descriptor_is("(II)Ljava/util/List;")
        {
            let invocation = invocation?;
            let value = self.array_list_sub_list(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/List;" | "Ljava/util/AbstractList;"
        ) && method.name == "indexOf"
            && method.proto.descriptor_is("(Ljava/lang/Object;)I")
        {
            let invocation = invocation?;
            let value = self.list_index_of(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore AbstractList.java#lastIndexOf
        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/List;" | "Ljava/util/AbstractList;"
        ) && method.name == "lastIndexOf"
            && method.proto.descriptor_is("(Ljava/lang/Object;)I")
        {
            let invocation = invocation?;
            let value = self.list_last_index_of(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/List;"
            && method.name == "remove"
            && method.proto.descriptor_is("(I)Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.remove_array_list_value(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/List;"
            && method.name == "clear"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.clear_list(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/Iterator;"
            && method.name == "hasNext"
            && method.proto.descriptor_is("()Z")
        {
            let value = self.iterator_has_next(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/Iterator;"
            && method.name == "next"
            && method.proto.descriptor_is("()Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.iterator_next(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/Iterator;"
            && method.name == "remove"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.iterator_remove(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/ListIterator;"
            && method.name == "hasNext"
            && method.proto.descriptor_is("()Z")
        {
            let value = self.iterator_has_next(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/ListIterator;"
            && method.name == "next"
            && method.proto.descriptor_is("()Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.iterator_next(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/ListIterator;"
            && method.name == "hasPrevious"
            && method.proto.descriptor_is("()Z")
        {
            let value = self.iterator_has_previous(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/ListIterator;"
            && method.name == "previous"
            && method.proto.descriptor_is("()Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.iterator_previous(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/ListIterator;"
            && method.name == "remove"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.iterator_remove(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/Map$Entry;"
            && method.name == "getKey"
            && method.proto.descriptor_is("()Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.map_entry_key(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/Map$Entry;"
            && method.name == "getValue"
            && method.proto.descriptor_is("()Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.map_entry_value(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if is_supported_map_class(&method.class_descriptor)
            && method.name == "get"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;)Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.read_hash_map_value(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if is_supported_map_class(&method.class_descriptor)
            && method.name == "containsKey"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let invocation = invocation?;
            let value = self.hash_map_contains_key(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if is_supported_map_class(&method.class_descriptor)
            && method.name == "put"
            && method.proto.descriptor()
                == "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;"
        {
            let invocation = invocation?;
            let previous = self.write_hash_map_value(invocation)?;

            return Some(DexExecutionResult::Value(previous));
        }

        if is_supported_map_class(&method.class_descriptor)
            && method.name == "remove"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;)Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let previous = self.remove_hash_map_value(invocation)?;

            return Some(DexExecutionResult::Value(previous));
        }

        if is_supported_map_class(&method.class_descriptor)
            && method.name == "clear"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.clear_hash_map(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if is_supported_map_class(&method.class_descriptor)
            && method.name == "size"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.hash_map_size(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if is_supported_map_class(&method.class_descriptor)
            && method.name == "isEmpty"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = self.hash_map_is_empty(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if is_supported_map_class(&method.class_descriptor)
            && method.name == "entrySet"
            && method.proto.descriptor_is("()Ljava/util/Set;")
        {
            let invocation = invocation?;
            let value = self.map_entry_set(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if is_supported_map_class(&method.class_descriptor)
            && method.name == "values"
            && method.proto.descriptor_is("()Ljava/util/Collection;")
        {
            let invocation = invocation?;
            let value = self.map_values(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if is_supported_map_class(&method.class_descriptor)
            && method.name == "keySet"
            && method.proto.descriptor_is("()Ljava/util/Set;")
        {
            let invocation = invocation?;
            let value = self.map_key_set(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/HashMap;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/AbstractMap;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/AbstractCollection;"
                | "Ljava/util/AbstractList;"
                | "Ljava/util/AbstractSet;"
        ) && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/HashMap;" | "Ljava/util/LinkedHashMap;"
        ) && method.name == "<init>"
            && method.proto.descriptor_is("(Ljava/util/Map;)V")
        {
            let invocation = invocation?;
            self.initialize_hash_map_from_map(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/LinkedHashMap;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/IdentityHashMap;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/WeakHashMap;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/TreeMap;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: libcore TreeMap.java#ceilingEntry, floorEntry, higherEntry, lowerEntry, firstEntry, lastEntry, ceilingKey, floorKey, firstKey, lastKey
        if method.class_descriptor == "Ljava/util/TreeMap;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("ceilingEntry", "(Ljava/lang/Object;)Ljava/util/Map$Entry;")
                | ("floorEntry", "(Ljava/lang/Object;)Ljava/util/Map$Entry;")
                | ("higherEntry", "(Ljava/lang/Object;)Ljava/util/Map$Entry;")
                | ("lowerEntry", "(Ljava/lang/Object;)Ljava/util/Map$Entry;") => {
                    let invocation = invocation?;
                    let receiver = invocation.arguments.first()?;
                    let target = invocation.arguments.get(1)?;
                    let entry = self.tree_map_navigable_entry(receiver, Some(target), &method.name);
                    return Some(DexExecutionResult::Value(entry));
                }
                ("firstEntry", "()Ljava/util/Map$Entry;")
                | ("lastEntry", "()Ljava/util/Map$Entry;") => {
                    let invocation = invocation?;
                    let receiver = invocation.arguments.first()?;
                    let entry = self.tree_map_navigable_entry(receiver, None, &method.name);
                    return Some(DexExecutionResult::Value(entry));
                }
                ("ceilingKey", "(Ljava/lang/Object;)Ljava/lang/Object;")
                | ("floorKey", "(Ljava/lang/Object;)Ljava/lang/Object;") => {
                    let invocation = invocation?;
                    let receiver = invocation.arguments.first()?;
                    let target = invocation.arguments.get(1)?;
                    let op = if method.name == "ceilingKey" {
                        "ceilingEntry"
                    } else {
                        "floorEntry"
                    };
                    let key = self
                        .tree_map_navigable_pair(receiver, Some(target), op)
                        .map_or(DexValue::Null, |(k, _)| k);
                    return Some(DexExecutionResult::Value(key));
                }
                ("firstKey", "()Ljava/lang/Object;") | ("lastKey", "()Ljava/lang/Object;") => {
                    let invocation = invocation?;
                    let receiver = invocation.arguments.first()?;
                    let op = if method.name == "firstKey" {
                        "firstEntry"
                    } else {
                        "lastEntry"
                    };
                    let key = self
                        .tree_map_navigable_pair(receiver, None, op)
                        .map_or(DexValue::Null, |(k, _)| k);
                    return Some(DexExecutionResult::Value(key));
                }
                _ => {}
            }
        }

        if is_supported_map_implementation_class(&method.class_descriptor)
            && method.name == "<init>"
            && method.proto.descriptor_is("(I)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/HashMap;" | "Ljava/util/LinkedHashMap;"
        ) && method.name == "<init>"
            && matches!(method.proto.descriptor().as_str(), "(IF)V" | "(IFZ)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/concurrent/ConcurrentHashMap;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/HashSet;" | "Ljava/util/LinkedHashSet;"
        ) && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            if let Some(invocation) = invocation {
                self.initialize_empty_set(invocation)?;
            }

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/TreeSet;"
            && method.name == "<init>"
            && matches!(
                method.proto.descriptor().as_str(),
                "()V" | "(Ljava/util/Comparator;)V"
            )
        {
            if let Some(invocation) = invocation {
                self.initialize_tree_set(invocation)?;
            }

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/PriorityQueue;"
            && method.name == "<init>"
            && matches!(
                method.proto.descriptor().as_str(),
                "()V" | "(I)V" | "(ILjava/util/Comparator;)V" | "(Ljava/util/Collection;)V"
            )
        {
            if let Some(invocation) = invocation {
                self.initialize_priority_queue(invocation)?;
            }

            return Some(DexExecutionResult::Void);
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/HashSet;" | "Ljava/util/LinkedHashSet;"
        ) && method.name == "<init>"
            && method.proto.descriptor_is("(I)V")
        {
            if let Some(invocation) = invocation {
                self.initialize_empty_set_with_capacity(invocation)?;
            }

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/ArrayDeque;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/ArrayDeque;" {
            if method.name == "offer" && method.proto.descriptor_is("(Ljava/lang/Object;)Z") {
                // cite: libcore ArrayDeque.java#offer, offerLast, addLast
                let invocation = invocation?;
                self.array_deque_add_last(invocation).ok()??;
                return Some(DexExecutionResult::Int(1));
            }
            if method.name == "offerLast" && method.proto.descriptor_is("(Ljava/lang/Object;)Z") {
                // cite: libcore ArrayDeque.java#offerLast, addLast
                let invocation = invocation?;
                self.array_deque_add_last(invocation).ok()??;
                return Some(DexExecutionResult::Int(1));
            }
            if method.name == "add" && method.proto.descriptor_is("(Ljava/lang/Object;)Z") {
                // cite: libcore ArrayDeque.java#add, addLast
                let invocation = invocation?;
                self.array_deque_add_last(invocation).ok()??;
                return Some(DexExecutionResult::Int(1));
            }
            if method.name == "addLast" && method.proto.descriptor_is("(Ljava/lang/Object;)V") {
                // cite: libcore ArrayDeque.java#addLast
                let invocation = invocation?;
                self.array_deque_add_last(invocation).ok()??;
                return Some(DexExecutionResult::Void);
            }
            if matches!(method.name.as_str(), "poll" | "pollFirst")
                && method.proto.descriptor_is("()Ljava/lang/Object;")
            {
                // cite: libcore ArrayDeque.java#poll, pollFirst
                let invocation = invocation?;
                let key = map_identity_key(invocation.arguments.first()?);
                let value = self
                    .runtime
                    .borrow_mut()
                    .list_entries
                    .get_mut(&key)
                    .and_then(|entries| {
                        if entries.is_empty() {
                            None
                        } else {
                            Some(entries.remove(0))
                        }
                    })
                    .unwrap_or(DexValue::Null);
                return Some(DexExecutionResult::Value(value));
            }
            if matches!(method.name.as_str(), "peek" | "peekFirst")
                && method.proto.descriptor_is("()Ljava/lang/Object;")
            {
                // cite: libcore ArrayDeque.java#peek, peekFirst
                let invocation = invocation?;
                let key = map_identity_key(invocation.arguments.first()?);
                let value = self
                    .runtime
                    .borrow()
                    .list_entries
                    .get(&key)
                    .and_then(|entries| entries.first().cloned())
                    .unwrap_or(DexValue::Null);
                return Some(DexExecutionResult::Value(value));
            }
            if method.name == "isEmpty" && method.proto.descriptor_is("()Z") {
                // cite: libcore ArrayDeque.java#isEmpty
                let invocation = invocation?;
                let key = map_identity_key(invocation.arguments.first()?);
                let empty = self
                    .runtime
                    .borrow()
                    .list_entries
                    .get(&key)
                    .map_or(true, Vec::is_empty);
                return Some(DexExecutionResult::Int(i32::from(empty)));
            }
            if method.name == "size" && method.proto.descriptor_is("()I") {
                // cite: libcore ArrayDeque.java#size
                let invocation = invocation?;
                let key = map_identity_key(invocation.arguments.first()?);
                let size = self
                    .runtime
                    .borrow()
                    .list_entries
                    .get(&key)
                    .map_or(0, Vec::len);
                return Some(DexExecutionResult::Int(size as i32));
            }
        }

        if method.class_descriptor == "Ljava/util/concurrent/CopyOnWriteArraySet;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            if let Some(invocation) = invocation {
                self.initialize_empty_set(invocation)?;
            }

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/UUID;"
            && method.name == "randomUUID"
            && method.proto.descriptor_is("()Ljava/util/UUID;")
        {
            let value = random_uuid_string()?;

            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from(format!("Ljava/util/UUID;:{value}")),
            )));
        }

        if method.class_descriptor == "Ljava/util/UUID;"
            && method.name == "toString"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = uuid_to_string(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        None
    }

    fn tree_map_navigable_entry(
        &self,
        receiver: &DexValue,
        target: Option<&DexValue>,
        op: &str,
    ) -> DexValue {
        let Some((key, value)) = self.tree_map_navigable_pair(receiver, target, op) else {
            return DexValue::Null;
        };
        let mut runtime = self.runtime.borrow_mut();
        let entry_id = runtime.next_object_id;
        runtime.next_object_id += 1;
        let entry_ref = format!("Ljava/util/Map$Entry;:treemap:{entry_id}");
        runtime
            .map_entry_objects
            .insert(entry_ref.clone(), (key, value));
        DexValue::Reference(std::rc::Rc::from(entry_ref))
    }

    fn tree_map_navigable_pair(
        &self,
        receiver: &DexValue,
        target: Option<&DexValue>,
        op: &str,
    ) -> Option<(DexValue, DexValue)> {
        let source_map_key = map_identity_key(receiver);
        let mut entries = Vec::new();
        {
            let runtime = self.runtime.borrow();
            for ((map_key, entry_key), value) in &runtime.map_entries {
                if map_key != &source_map_key {
                    continue;
                }
                if let Some(key) = runtime
                    .map_entry_keys
                    .get(&(map_key.clone(), entry_key.clone()))
                {
                    entries.push((key.clone(), value.clone()));
                }
            }
        }
        entries.sort_by(|a, b| self.tree_map_compare_keys(&a.0, &b.0));
        match op {
            "firstEntry" => entries.into_iter().next(),
            "lastEntry" => entries.into_iter().next_back(),
            "ceilingEntry" => {
                let target = target?;
                entries.into_iter().find(|(k, _)| {
                    self.tree_map_compare_keys(k, target) != std::cmp::Ordering::Less
                })
            }
            "higherEntry" => {
                let target = target?;
                entries.into_iter().find(|(k, _)| {
                    self.tree_map_compare_keys(k, target) == std::cmp::Ordering::Greater
                })
            }
            "floorEntry" => {
                let target = target?;
                entries.into_iter().rev().find(|(k, _)| {
                    self.tree_map_compare_keys(k, target) != std::cmp::Ordering::Greater
                })
            }
            "lowerEntry" => {
                let target = target?;
                entries.into_iter().rev().find(|(k, _)| {
                    self.tree_map_compare_keys(k, target) == std::cmp::Ordering::Less
                })
            }
            _ => None,
        }
    }

    fn tree_map_compare_keys(&self, left: &DexValue, right: &DexValue) -> std::cmp::Ordering {
        if let (Some(l), Some(r)) = (
            self.tree_map_numeric_key(left),
            self.tree_map_numeric_key(right),
        ) {
            return l.cmp(&r);
        }
        if let (Some(l), Some(r)) = (self.string_text(left), self.string_text(right)) {
            return l.cmp(&r);
        }
        map_identity_key(left).cmp(&map_identity_key(right))
    }

    fn tree_map_numeric_key(&self, value: &DexValue) -> Option<i64> {
        match value {
            DexValue::Int(v) => Some(*v as i64),
            DexValue::Long(v) => Some(*v),
            DexValue::Reference(reference) => {
                if let Some(rest) = reference.strip_prefix("Ljava/lang/Integer;:") {
                    rest.parse::<i64>().ok()
                } else if let Some(rest) = reference.strip_prefix("Ljava/lang/Long;:") {
                    rest.parse::<i64>().ok()
                } else {
                    None
                }
            }
            DexValue::Object(object) => {
                if object.class_descriptor.as_ref() == "Ljava/lang/Integer;" {
                    let field = DexFieldId {
                        class_descriptor: "Ljava/lang/Integer;".to_owned(),
                        type_descriptor: "I".to_owned(),
                        name: "value".to_owned(),
                    };
                    if let Some(DexValue::Int(v)) = self.read_stored_instance_field(value, &field) {
                        return Some(v as i64);
                    }
                } else if object.class_descriptor.as_ref() == "Ljava/lang/Long;" {
                    let field = DexFieldId {
                        class_descriptor: "Ljava/lang/Long;".to_owned(),
                        type_descriptor: "J".to_owned(),
                        name: "value".to_owned(),
                    };
                    if let Some(DexValue::Long(v)) = self.read_stored_instance_field(value, &field)
                    {
                        return Some(v);
                    }
                }
                None
            }
            _ => None,
        }
    }
}

// cite: libcore UUID.java#randomUUID
fn random_uuid_string() -> Option<String> {
    use std::io::Read;

    let mut bytes = [0u8; 16];
    std::fs::File::open("/dev/urandom")
        .ok()?
        .read_exact(&mut bytes)
        .ok()?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    Some(format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    ))
}
