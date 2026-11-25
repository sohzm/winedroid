use super::super::*;

impl DexProgram {
    pub(super) fn invoke_java_regex_util_builtin(
        &self,
        method: &DexMethodId,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        if method.class_descriptor == "Ljava/util/regex/Pattern;"
            && method.name == "compile"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Ljava/util/regex/Pattern;")
        {
            let invocation = invocation?;
            let value = self.regex_pattern_compile(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: libcore Pattern.java#quote
        if method.matches_builtin_signature(
            "Ljava/util/regex/Pattern;",
            "quote",
            "(Ljava/lang/String;)Ljava/lang/String;",
        ) {
            let invocation = invocation?;
            let DexValue::String(value) = invocation.arguments.first()? else {
                return None;
            };

            let quoted = if value.contains("\\E") {
                let mut quoted = String::with_capacity(value.len() * 2);
                quoted.push_str("\\Q");
                let mut rest: &str = value.as_ref();
                while let Some(index) = rest.find("\\E") {
                    quoted.push_str(&rest[..index]);
                    quoted.push_str("\\E\\\\E\\Q");
                    rest = &rest[index + 2..];
                }
                quoted.push_str(rest);
                quoted.push_str("\\E");
                quoted
            } else {
                format!("\\Q{value}\\E")
            };

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(quoted),
            )));
        }

        if method.matches_builtin_signature(
            "Ljava/util/regex/Pattern;",
            "compile",
            "(Ljava/lang/String;I)Ljava/util/regex/Pattern;",
        ) {
            let invocation = invocation?;
            let value = self.regex_pattern_compile_with_flags(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: libcore Pattern.java#quote
        if method.matches_builtin_signature(
            "Ljava/util/regex/Pattern;",
            "quote",
            "(Ljava/lang/String;)Ljava/lang/String;",
        ) {
            let invocation = invocation?;
            if invocation.arguments.len() != 1 {
                return None;
            }
            let DexValue::String(text) = &invocation.arguments[0] else {
                return None;
            };
            let quoted = format!("\\Q{}\\E", text.replace("\\E", "\\E\\\\E\\Q"));

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(quoted),
            )));
        }

        if method.matches_builtin_signature(
            "Ljava/util/regex/Pattern;",
            "matcher",
            "(Ljava/lang/CharSequence;)Ljava/util/regex/Matcher;",
        ) {
            let invocation = invocation?;
            let value = self.regex_pattern_matcher(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: libcore Pattern.java#pattern
        if method.matches_builtin_signature(
            "Ljava/util/regex/Pattern;",
            "pattern",
            "()Ljava/lang/String;",
        ) {
            let invocation = invocation?;
            let key = map_identity_key(invocation.arguments.first()?);
            let pattern = self
                .runtime
                .borrow()
                .regex_patterns
                .get(&key)
                .cloned()
                .unwrap_or_default();
            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(pattern),
            )));
        }

        if method.matches_builtin_signature("Ljava/util/regex/Matcher;", "find", "()Z") {
            let invocation = invocation?;
            let value = self.regex_matcher_find(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature("Ljava/util/regex/Matcher;", "find", "(I)Z") {
            let invocation = invocation?;
            let value = self.regex_matcher_find_from(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/regex/Matcher;",
            "pattern",
            "()Ljava/util/regex/Pattern;",
        ) {
            let invocation = invocation?;
            let value = self.regex_matcher_pattern(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature("Ljava/util/regex/Matcher;", "matches", "()Z") {
            let invocation = invocation?;
            let value = self.regex_matcher_matches(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_any_builtin_signature(
            "Ljava/util/regex/Matcher;",
            "group",
            &["()Ljava/lang/String;", "(I)Ljava/lang/String;"],
        ) {
            let invocation = invocation?;
            let value = self.regex_matcher_group(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_any_builtin_signature(
            "Ljava/util/regex/Matcher;",
            "start",
            &["()I", "(I)I"],
        ) {
            let invocation = invocation?;
            let value = self.regex_matcher_start(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_any_builtin_signature(
            "Ljava/util/regex/Matcher;",
            "end",
            &["()I", "(I)I"],
        ) {
            let invocation = invocation?;
            let value = self.regex_matcher_end(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/regex/Matcher;",
            "replaceAll",
            "(Ljava/lang/String;)Ljava/lang/String;",
        ) {
            let invocation = invocation?;
            let value = self.regex_matcher_replace_all(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.matches_builtin_signature(
            "Ljava/util/Calendar;",
            "getInstance",
            "()Ljava/util/Calendar;",
        ) {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Ljava/util/Calendar;.bootstrap"),
            )));
        }

        if method.matches_builtin_signature("Ljava/util/Calendar;", "get", "(I)I") {
            let invocation = invocation?;
            let value = calendar_get_field(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature("Lo8/a;", "a", "()Ljava/util/Random;") {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Ljava/util/Random;.bootstrap"),
            )));
        }

        if method.matches_builtin_signature("LB1/a;", "a", "()Ljava/util/Random;") {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Ljava/util/Random;.bootstrap"),
            )));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/ThreadLocalRandom;",
            "current",
            "()Ljava/util/concurrent/ThreadLocalRandom;",
        ) {
            // cite: libcore ThreadLocalRandom.java#current
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Ljava/util/Random;.bootstrap"),
            )));
        }

        if method.class_descriptor == "Ljava/util/Random;"
            && method.name == "<init>"
            && matches!(method.proto.descriptor().as_str(), "()V" | "(J)V")
        {
            // cite: libcore Random.java#Random
            if let Some(invocation) = invocation {
                self.initialize_random(invocation).ok()??;
            }
            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature("Ljava/util/Random;", "nextInt", "()I") {
            let invocation = invocation?;
            // cite: libcore Random.java#nextInt, next
            let value = self.random_next_unbounded_int(invocation).ok()??;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature("Ljava/util/Random;", "nextInt", "(I)I") {
            let invocation = invocation?;
            // cite: libcore Random.java#nextInt, next
            let value = self.random_next_int(invocation).ok()??;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore Logger.java#getLogger, setUseParentHandlers, setLevel, getLevel, addHandler, isLoggable, log, warning, info, fine, finer, finest
        if method.class_descriptor == "Ljava/util/logging/Logger;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("getLogger", "(Ljava/lang/String;)Ljava/util/logging/Logger;")
                | (
                    "getLogger",
                    "(Ljava/lang/String;Ljava/lang/String;)Ljava/util/logging/Logger;",
                ) => {
                    let logger = self.allocate_object("Ljava/util/logging/Logger;".to_owned());
                    return Some(DexExecutionResult::Value(logger));
                }
                ("setUseParentHandlers", "(Z)V")
                | ("setLevel", "(Ljava/util/logging/Level;)V")
                | ("addHandler", "(Ljava/util/logging/Handler;)V")
                | ("log", "(Ljava/util/logging/LogRecord;)V")
                | ("log", "(Ljava/util/logging/Level;Ljava/lang/String;)V")
                | ("log", "(Ljava/util/logging/Level;Ljava/lang/String;Ljava/lang/Throwable;)V")
                | ("log", "(Ljava/util/logging/Level;Ljava/lang/String;Ljava/lang/Object;)V")
                | ("log", "(Ljava/util/logging/Level;Ljava/lang/String;[Ljava/lang/Object;)V")
                | ("warning", "(Ljava/lang/String;)V")
                | ("info", "(Ljava/lang/String;)V")
                | ("fine", "(Ljava/lang/String;)V")
                | ("finer", "(Ljava/lang/String;)V")
                | ("finest", "(Ljava/lang/String;)V") => {
                    return Some(DexExecutionResult::Void);
                }
                ("isLoggable", "(Ljava/util/logging/Level;)Z") => {
                    return Some(DexExecutionResult::Int(0));
                }
                ("getLevel", "()Ljava/util/logging/Level;") => {
                    return Some(DexExecutionResult::Value(DexValue::Null));
                }
                _ => {}
            }
        }

        // cite: libcore Handler.java#Handler
        if method.matches_builtin_signature("Ljava/util/logging/Handler;", "<init>", "()V") {
            return Some(DexExecutionResult::Void);
        }

        // cite: libcore Level.java#intValue
        if method.matches_builtin_signature("Ljava/util/logging/Level;", "intValue", "()I") {
            return Some(DexExecutionResult::Int(800));
        }

        // cite: libcore BitSet.java#BitSet, set, clear, get, cardinality, isEmpty, length, nextSetBit
        if method.class_descriptor == "Ljava/util/BitSet;" {
            let invocation = invocation?;
            let receiver = invocation.arguments.first()?;
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("<init>", "()V") | ("<init>", "(I)V") => {
                    self.write_bit_set(receiver, &std::collections::BTreeSet::new());
                    return Some(DexExecutionResult::Void);
                }
                ("set", "(I)V") => {
                    let DexValue::Int(index) = invocation.arguments.get(1)? else {
                        return None;
                    };
                    if *index >= 0 {
                        let mut bits = self.read_bit_set(receiver);
                        bits.insert(*index);
                        self.write_bit_set(receiver, &bits);
                    }
                    return Some(DexExecutionResult::Void);
                }
                ("set", "(IZ)V") => {
                    let DexValue::Int(index) = invocation.arguments.get(1)? else {
                        return None;
                    };
                    let DexValue::Int(value) = invocation.arguments.get(2)? else {
                        return None;
                    };
                    if *index >= 0 {
                        let mut bits = self.read_bit_set(receiver);
                        if *value != 0 {
                            bits.insert(*index);
                        } else {
                            bits.remove(index);
                        }
                        self.write_bit_set(receiver, &bits);
                    }
                    return Some(DexExecutionResult::Void);
                }
                ("clear", "(I)V") => {
                    let DexValue::Int(index) = invocation.arguments.get(1)? else {
                        return None;
                    };
                    let mut bits = self.read_bit_set(receiver);
                    bits.remove(index);
                    self.write_bit_set(receiver, &bits);
                    return Some(DexExecutionResult::Void);
                }
                ("clear", "()V") => {
                    self.write_bit_set(receiver, &std::collections::BTreeSet::new());
                    return Some(DexExecutionResult::Void);
                }
                ("get", "(I)Z") => {
                    let DexValue::Int(index) = invocation.arguments.get(1)? else {
                        return None;
                    };
                    let bits = self.read_bit_set(receiver);
                    return Some(DexExecutionResult::Int(i32::from(bits.contains(index))));
                }
                ("cardinality", "()I") => {
                    let bits = self.read_bit_set(receiver);
                    return Some(DexExecutionResult::Int(bits.len() as i32));
                }
                ("isEmpty", "()Z") => {
                    let bits = self.read_bit_set(receiver);
                    return Some(DexExecutionResult::Int(i32::from(bits.is_empty())));
                }
                ("length", "()I") => {
                    let bits = self.read_bit_set(receiver);
                    let len = bits.iter().next_back().map_or(0, |max| max + 1);
                    return Some(DexExecutionResult::Int(len));
                }
                ("nextSetBit", "(I)I") => {
                    let DexValue::Int(from) = invocation.arguments.get(1)? else {
                        return None;
                    };
                    let bits = self.read_bit_set(receiver);
                    let next = bits.range(*from..).next().copied().unwrap_or(-1);
                    return Some(DexExecutionResult::Int(next));
                }
                _ => {}
            }
        }

        None
    }

    fn read_bit_set(&self, receiver: &DexValue) -> std::collections::BTreeSet<i32> {
        let mut bits = std::collections::BTreeSet::new();
        if let Some(DexValue::String(encoded)) =
            self.read_stored_instance_field(receiver, &bit_set_words_field())
        {
            for token in encoded.split_whitespace() {
                if let Ok(index) = token.parse::<i32>() {
                    bits.insert(index);
                }
            }
        }
        bits
    }

    fn write_bit_set(&self, receiver: &DexValue, bits: &std::collections::BTreeSet<i32>) {
        let encoded = bits
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        self.write_stored_instance_field(
            receiver,
            &bit_set_words_field(),
            DexValue::String(std::rc::Rc::from(encoded)),
        );
    }
}

fn bit_set_words_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/util/BitSet;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "words".to_owned(),
    }
}
