use super::super::*;

impl DexProgram {
    pub(super) fn invoke_java_string_text_builtin(
        &self,
        method: &DexMethodId,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "<init>"
            && method.proto.descriptor_is("([CII)V")
        {
            let invocation = invocation?;
            self.initialize_string_from_char_array(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "<init>"
            && method.proto.descriptor_is("([C)V")
        {
            let invocation = invocation?;
            self.initialize_string_from_full_char_array(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        // cite: libcore String.java#trim
        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "trim"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let text = self.string_text(invocation.arguments.first()?)?;
            let trimmed = text.trim_matches(|ch: char| ch <= ' ');
            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(trimmed.to_owned()),
            )));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "<init>"
            && matches!(
                method.proto.descriptor().as_str(),
                "([BIILjava/nio/charset/Charset;)V"
                    | "([BLjava/nio/charset/Charset;)V"
                    | "([BII)V"
                    | "([B)V"
            )
        {
            // cite: libcore String.java#String
            let invocation = invocation?;
            let (offset, length) = if invocation.arguments.len() >= 4 {
                let DexValue::Int(off) = invocation.arguments[2] else {
                    return None;
                };
                let DexValue::Int(len) = invocation.arguments[3] else {
                    return None;
                };
                (usize::try_from(off).ok()?, Some(usize::try_from(len).ok()?))
            } else {
                (0, None)
            };
            let array_key = map_identity_key(&invocation.arguments[1]);
            let bytes: Vec<u8> = {
                let runtime = self.runtime.borrow();
                let entries = runtime.array_entries.get(&array_key)?;
                let len = length.unwrap_or(entries.len().saturating_sub(offset));
                let end = offset.checked_add(len)?;
                if end > entries.len() {
                    return None;
                }
                entries[offset..end]
                    .iter()
                    .map(|v| match v {
                        DexValue::Int(b) => *b as u8,
                        _ => 0,
                    })
                    .collect()
            };
            let text = String::from_utf8_lossy(&bytes).into_owned();
            let key = map_identity_key(&invocation.arguments[0]);
            self.runtime.borrow_mut().string_values.insert(key, text);

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/text/Normalizer;",
            "normalize",
            "(Ljava/lang/CharSequence;Ljava/text/Normalizer$Form;)Ljava/lang/String;",
        ) {
            // cite: libcore Normalizer.java#normalize
            let invocation = invocation?;
            let text = self.char_sequence_text(invocation.arguments.first()?)?;
            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(text),
            )));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "getBytes"
            && matches!(
                method.proto.descriptor().as_str(),
                "()[B" | "(Ljava/nio/charset/Charset;)[B" | "(Ljava/lang/String;)[B"
            )
        {
            // cite: libcore String.java#getBytes
            let invocation = invocation?;
            let text = self.string_text(invocation.arguments.first()?)?;
            let elements: Vec<DexValue> = text
                .into_bytes()
                .into_iter()
                .map(|b| DexValue::Int((b as i8) as i32))
                .collect();
            return Some(DexExecutionResult::Value(
                self.allocate_filled_array("[B".to_owned(), elements),
            ));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "regionMatches"
            && matches!(
                method.proto.descriptor().as_str(),
                "(ZILjava/lang/String;II)Z" | "(ILjava/lang/String;II)Z"
            )
        {
            // cite: libcore String.java#regionMatches
            let invocation = invocation?;
            let (ignore_case, toffset, other_arg, ooffset, len) = if invocation.arguments.len() == 6
            {
                let DexValue::Int(ic) = invocation.arguments[1] else {
                    return None;
                };
                let DexValue::Int(to) = invocation.arguments[2] else {
                    return None;
                };
                let DexValue::Int(oo) = invocation.arguments[4] else {
                    return None;
                };
                let DexValue::Int(ln) = invocation.arguments[5] else {
                    return None;
                };
                (ic != 0, to, &invocation.arguments[3], oo, ln)
            } else if invocation.arguments.len() == 5 {
                let DexValue::Int(to) = invocation.arguments[1] else {
                    return None;
                };
                let DexValue::Int(oo) = invocation.arguments[3] else {
                    return None;
                };
                let DexValue::Int(ln) = invocation.arguments[4] else {
                    return None;
                };
                (false, to, &invocation.arguments[2], oo, ln)
            } else {
                return None;
            };
            let this_text = self.string_text(&invocation.arguments[0])?;
            let other_text = self.string_text(other_arg)?;
            if toffset < 0 || ooffset < 0 || len < 0 {
                return Some(DexExecutionResult::Int(0));
            }
            let this_units: Vec<u16> = this_text.encode_utf16().collect();
            let other_units: Vec<u16> = other_text.encode_utf16().collect();
            let (to, oo, ln) = (toffset as usize, ooffset as usize, len as usize);
            if to.saturating_add(ln) > this_units.len() || oo.saturating_add(ln) > other_units.len()
            {
                return Some(DexExecutionResult::Int(0));
            }
            let matched = if ignore_case {
                this_units[to..to + ln]
                    .iter()
                    .zip(&other_units[oo..oo + ln])
                    .all(|(&a, &b)| {
                        let ca = char::from_u32(u32::from(a));
                        let cb = char::from_u32(u32::from(b));
                        match (ca, cb) {
                            (Some(ca), Some(cb)) => {
                                ca.to_ascii_lowercase() == cb.to_ascii_lowercase()
                            }
                            _ => a == b,
                        }
                    })
            } else {
                this_units[to..to + ln] == other_units[oo..oo + ln]
            };
            return Some(DexExecutionResult::Int(i32::from(matched)));
        }

        if method.matches_builtin_signature(
            "Ljava/text/BreakIterator;",
            "getLineInstance",
            "(Ljava/util/Locale;)Ljava/text/BreakIterator;",
        ) {
            let invocation = invocation?;
            let value = self.break_iterator_get_line_instance(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature(
            "Ljava/text/BreakIterator;",
            "setText",
            "(Ljava/text/CharacterIterator;)V",
        ) {
            let invocation = invocation?;
            self.break_iterator_set_character_iterator_text(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature("Ljava/text/BreakIterator;", "next", "()I") {
            let invocation = invocation?;
            let value = self.break_iterator_next(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "toString"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.string_text(&invocation.arguments[0])?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "valueOf"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.objects_to_string(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "valueOf"
            && method.proto.descriptor_is("(C)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = string_value_of_char(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "valueOf"
            && method.proto.descriptor_is("(I)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = string_value_of_int(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "valueOf"
            && method.proto.descriptor_is("(J)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = string_value_of_long(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "valueOf"
            && method.proto.descriptor_is("(D)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = string_value_of_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "format"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/lang/String;[Ljava/lang/Object;)Ljava/lang/String;"
                    | "(Ljava/util/Locale;Ljava/lang/String;[Ljava/lang/Object;)Ljava/lang/String;"
            )
        {
            let invocation = invocation?;
            let value = self.string_format(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "equals"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let invocation = invocation?;
            let value = self.string_equals(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore String.java#contentEquals
        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "contentEquals"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/lang/CharSequence;)Z" | "(Ljava/lang/StringBuffer;)Z"
            )
        {
            let invocation = invocation?;
            let this = self.char_sequence_full_text(&invocation.arguments[0])?;
            let other = match self.char_sequence_full_text(&invocation.arguments[1]) {
                Some(other) => other,
                None => return Some(DexExecutionResult::Int(0)),
            };

            return Some(DexExecutionResult::Int(i32::from(this == other)));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "equalsIgnoreCase"
            && method.proto.descriptor_is("(Ljava/lang/String;)Z")
        {
            let invocation = invocation?;
            let value = self.string_equals_ignore_case(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "contains"
            && method.proto.descriptor_is("(Ljava/lang/CharSequence;)Z")
        {
            let invocation = invocation?;
            let value = self.string_contains(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "compareToIgnoreCase"
            && method.proto.descriptor_is("(Ljava/lang/String;)I")
        {
            let invocation = invocation?;
            let value = self.string_compare_to_ignore_case(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "startsWith"
            && method.proto.descriptor_is("(Ljava/lang/String;)Z")
        {
            let invocation = invocation?;
            let value = self.string_starts_with(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "endsWith"
            && method.proto.descriptor_is("(Ljava/lang/String;)Z")
        {
            let invocation = invocation?;
            let value = self.string_ends_with(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "length"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.string_length(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore String.java#getChars
        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "getChars"
            && method.proto.descriptor_is("(II[CI)V")
        {
            let invocation = invocation?;
            self.string_get_chars(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "hashCode"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.string_hash_code(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "charAt"
            && method.proto.descriptor_is("(I)C")
        {
            let invocation = invocation?;
            let value = self.string_char_at(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "isEmpty"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = self.string_is_empty(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: frameworks-base TextUtils.java#getLayoutDirectionFromLocale
        if method.class_descriptor == "Landroid/text/TextUtils;"
            && method.name == "getLayoutDirectionFromLocale"
            && method.proto.descriptor_is("(Ljava/util/Locale;)I")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Landroid/text/TextUtils;"
            && method.name == "isEmpty"
            && method.proto.descriptor_is("(Ljava/lang/CharSequence;)Z")
        {
            let invocation = invocation?;
            let value = text_utils_is_empty(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/text/TextUtils;"
            && method.name == "equals"
            && method
                .proto
                .descriptor_is("(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Z")
        {
            let invocation = invocation?;
            let value = text_utils_equals(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "substring"
            && method.proto.descriptor_is("(I)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.string_substring(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "substring"
            && method.proto.descriptor_is("(II)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.string_substring_range(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "indexOf"
            && method.proto.descriptor_is("(Ljava/lang/String;)I")
        {
            let invocation = invocation?;
            let value = self.string_index_of(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "indexOf"
            && method.proto.descriptor_is("(Ljava/lang/String;I)I")
        {
            let invocation = invocation?;
            let value = self.string_index_of(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "indexOf"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.string_index_of_code_point(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "indexOf"
            && method.proto.descriptor_is("(II)I")
        {
            let invocation = invocation?;
            let value = self.string_index_of_code_point(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "lastIndexOf"
            && matches!(method.proto.descriptor().as_str(), "(I)I" | "(II)I")
        {
            let invocation = invocation?;
            let value = self.string_last_index_of_code_point(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "lastIndexOf"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/lang/String;)I" | "(Ljava/lang/String;I)I"
            )
        {
            let invocation = invocation?;
            let value = self.string_last_index_of(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "split"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/lang/String;)[Ljava/lang/String;"
                    | "(Ljava/lang/String;I)[Ljava/lang/String;"
            )
        {
            let invocation = invocation?;
            let value = self.string_split(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "replace"
            && method.proto.descriptor_is("(CC)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.string_replace_chars(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "concat"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.string_concat(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "toLowerCase"
            && method
                .proto
                .descriptor_is("(Ljava/util/Locale;)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.string_to_lower_case_locale(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "toUpperCase"
            && method
                .proto
                .descriptor_is("(Ljava/util/Locale;)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.string_to_upper_case_locale(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        // cite: libcore String.java#toLowerCase
        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "toLowerCase"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.string_text(&invocation.arguments[0])?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value.to_lowercase()),
            )));
        }

        // cite: libcore String.java#toUpperCase
        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "toUpperCase"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.string_text(&invocation.arguments[0])?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value.to_uppercase()),
            )));
        }

        if method.class_descriptor == "Ljava/lang/CharSequence;"
            && method.name == "toString"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.string_text(&invocation.arguments[0])?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/CharSequence;"
            && method.name == "subSequence"
            && method.proto.descriptor_is("(II)Ljava/lang/CharSequence;")
        {
            let invocation = invocation?;
            let value = self.char_sequence_sub_sequence(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/String;"
            && method.name == "subSequence"
            && method.proto.descriptor_is("(II)Ljava/lang/CharSequence;")
        {
            let invocation = invocation?;
            let value = self.char_sequence_sub_sequence(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: libcore CharSequence.java#length
        if method.class_descriptor == "Ljava/lang/CharSequence;"
            && method.name == "length"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let text = self.char_sequence_text(invocation.arguments.first()?)?;
            let value = i32::try_from(text.encode_utf16().count()).ok()?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore CharSequence.java#charAt
        if method.class_descriptor == "Ljava/lang/CharSequence;"
            && method.name == "charAt"
            && method.proto.descriptor_is("(I)C")
        {
            let invocation = invocation?;
            let text = self.char_sequence_text(invocation.arguments.first()?)?;
            let DexValue::Int(index) = invocation.arguments.get(1)? else {
                return None;
            };
            let index = usize::try_from(*index).ok()?;
            let value = text.encode_utf16().nth(index)?;

            return Some(DexExecutionResult::Int(i32::from(value)));
        }

        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(Ljava/lang/String;)V")
        {
            let invocation = invocation?;
            self.initialize_string_builder(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(Ljava/lang/CharSequence;)V")
        {
            let invocation = invocation?;
            self.initialize_string_builder(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.initialize_string_builder(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            string_builder_capacity_constructor(invocation)?;
            self.initialize_string_builder(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "append"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/lang/String;)Ljava/lang/StringBuilder;"
                    | "(Ljava/lang/Object;)Ljava/lang/StringBuilder;"
                    | "(I)Ljava/lang/StringBuilder;"
            )
        {
            let invocation = invocation?;
            let value = self.append_string_builder(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "append"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/lang/CharSequence;)Ljava/lang/Appendable;"
                    | "(Ljava/lang/CharSequence;)Ljava/lang/StringBuilder;"
            )
        {
            let invocation = invocation?;
            let value = self.append_string_builder_char_sequence(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "append"
            && method.proto.descriptor_is("(F)Ljava/lang/StringBuilder;")
        {
            let invocation = invocation?;
            let value = self.append_string_builder_float(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: libcore AbstractStringBuilder.java#append
        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "append"
            && method.proto.descriptor_is("(J)Ljava/lang/StringBuilder;")
        {
            let invocation = invocation?;
            let value = self.append_string_builder_long(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "append"
            && method
                .proto
                .descriptor_is("(Ljava/lang/CharSequence;II)Ljava/lang/StringBuilder;")
        {
            let invocation = invocation?;
            let value = self.append_string_builder_char_sequence_range(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "append"
            && matches!(
                method.proto.descriptor().as_str(),
                "(C)Ljava/lang/Appendable;" | "(C)Ljava/lang/StringBuilder;"
            )
        {
            let invocation = invocation?;
            let value = self.append_string_builder_char(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: libcore AbstractStringBuilder.java#append
        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "append"
            && method
                .proto
                .descriptor_is("([CII)Ljava/lang/StringBuilder;")
        {
            let invocation = invocation?;
            let value = self.append_string_builder_char_array_range(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "appendCodePoint"
            && method.proto.descriptor_is("(I)Ljava/lang/StringBuilder;")
        {
            let invocation = invocation?;
            let value = self.append_string_builder_code_point(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "append"
            && method.proto.descriptor_is("(Z)Ljava/lang/StringBuilder;")
        {
            let invocation = invocation?;
            let value = self.append_string_builder_boolean(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "insert"
            && method.proto.descriptor_is("(IC)Ljava/lang/StringBuilder;")
        {
            let invocation = invocation?;
            let value = self.insert_string_builder_char(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "reverse"
            && method.proto.descriptor_is("()Ljava/lang/StringBuilder;")
        {
            let invocation = invocation?;
            let value = self.reverse_string_builder(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "toString"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.string_builder_to_string(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/StringBuilder;"
            && method.name == "length"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.string_builder_length(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Enum;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(Ljava/lang/String;I)V")
        {
            let invocation = invocation?;
            self.initialize_enum(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/Enum;"
            && method.name == "valueOf"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Class;Ljava/lang/String;)Ljava/lang/Enum;")
        {
            let invocation = invocation?;
            let result = self.enum_value_of(invocation)?;

            return Some(result);
        }

        if method.class_descriptor == "Ljava/lang/Enum;"
            && method.name == "compareTo"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/lang/Enum;)I" | "(Ljava/lang/Object;)I"
            )
        {
            let invocation = invocation?;
            let value = self.enum_compare_to(&invocation.arguments)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Enum;"
            && method.name == "ordinal"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.enum_ordinal(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Enum;"
            && method.name == "name"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.enum_name(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/Enum;"
            && method.name == "toString"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.enum_name(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        None
    }
}

impl DexProgram {
    fn enum_value_of(&self, invocation: &DexInvocation) -> Option<DexExecutionResult> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let class_descriptor = class_reference_descriptor_from_value(&invocation.arguments[0])?;
        let DexValue::String(name) = &invocation.arguments[1] else {
            return None;
        };
        let field = DexFieldId {
            class_descriptor: class_descriptor.to_owned(),
            type_descriptor: class_descriptor.to_owned(),
            name: name.clone().to_string(),
        };

        if let Some(value) = self
            .read_stored_static_field(&field)
            .or_else(|| bootstrap_static_field(&field))
            .or_else(|| self.enum_value_by_name(class_descriptor, name))
        {
            return Some(DexExecutionResult::Value(value));
        }

        Some(DexExecutionResult::Throw(
            self.illegal_argument_exception_for_missing_enum(class_descriptor, name),
        ))
    }

    fn illegal_argument_exception_for_missing_enum(
        &self,
        class_descriptor: &str,
        name: &str,
    ) -> DexValue {
        let exception = self.allocate_object("Ljava/lang/IllegalArgumentException;".to_owned());
        self.write_stored_instance_field(
            &exception,
            &enum_throwable_detail_message_field(),
            DexValue::String(std::rc::Rc::from(format!(
                "No enum constant {class_descriptor}.{name}"
            ))),
        );

        exception
    }
}

fn enum_throwable_detail_message_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Throwable;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "detailMessage".to_owned(),
    }
}
