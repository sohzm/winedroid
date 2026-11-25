use super::super::*;

impl DexProgram {
    pub(super) fn invoke_java_box_locale_builtin(
        &self,
        method: &DexMethodId,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        if method.matches_builtin_signature(
            "Ljava/lang/Boolean;",
            "valueOf",
            "(Z)Ljava/lang/Boolean;",
        ) {
            let invocation = invocation?;
            let value = boolean_value_of(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: libcore Boolean.java#parseBoolean
        if method.matches_builtin_signature(
            "Ljava/lang/Boolean;",
            "parseBoolean",
            "(Ljava/lang/String;)Z",
        ) {
            let invocation = invocation?;
            let result = self
                .string_text(invocation.arguments.first()?)
                .is_some_and(|s| s.eq_ignore_ascii_case("true"));
            return Some(DexExecutionResult::Int(i32::from(result)));
        }

        if method.matches_builtin_signature("Ljava/lang/Boolean;", "booleanValue", "()Z") {
            let invocation = invocation?;
            let value = boolean_boolean_value(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore Boolean.java#hashCode
        if method.matches_builtin_signature("Ljava/lang/Boolean;", "hashCode", "(Z)I") {
            let invocation = invocation?;
            let DexValue::Int(value) = invocation.arguments.first()? else {
                return None;
            };
            return Some(DexExecutionResult::Int(if *value != 0 {
                1231
            } else {
                1237
            }));
        }

        // cite: libcore Boolean.java#hashCode
        if method.matches_builtin_signature("Ljava/lang/Boolean;", "hashCode", "()I") {
            let invocation = invocation?;
            let value = boolean_boolean_value(invocation)?;
            return Some(DexExecutionResult::Int(if value != 0 {
                1231
            } else {
                1237
            }));
        }

        if method.matches_builtin_signature(
            "Ljava/lang/Integer;",
            "valueOf",
            "(I)Ljava/lang/Integer;",
        ) {
            let invocation = invocation?;
            let value = integer_value_of(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature(
            "Ljava/lang/Integer;",
            "valueOf",
            "(Ljava/lang/String;)Ljava/lang/Integer;",
        ) {
            let invocation = invocation?;
            // cite: libcore Integer.java#valueOf
            let value = integer_parse_int(invocation, 10)?;

            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from(format!("Ljava/lang/Integer;:{value}")),
            )));
        }

        if method.matches_builtin_signature("Ljava/lang/Integer;", "<init>", "(I)V") {
            let invocation = invocation?;
            self.initialize_integer_box(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/lang/Integer;",
            "parseInt",
            "(Ljava/lang/String;)I",
        ) {
            let invocation = invocation?;
            let value = integer_parse_int(invocation, 10)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/lang/Integer;",
            "parseInt",
            "(Ljava/lang/String;I)I",
        ) {
            let invocation = invocation?;
            let DexValue::Int(radix) = invocation.arguments.get(1)? else {
                return None;
            };
            let value = integer_parse_int(invocation, *radix)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore Integer.java#toString
        if method.matches_builtin_signature(
            "Ljava/lang/Integer;",
            "toString",
            "(II)Ljava/lang/String;",
        ) {
            let invocation = invocation?;
            let DexValue::Int(value) = invocation.arguments.first()? else {
                return None;
            };
            let DexValue::Int(radix) = invocation.arguments.get(1)? else {
                return None;
            };
            let radix = if !(2..=36).contains(radix) {
                10
            } else {
                *radix
            };

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(int_to_string_radix(i64::from(*value), radix as u32)),
            )));
        }

        // cite: libcore Integer.java#rotateLeft
        if method.matches_builtin_signature("Ljava/lang/Integer;", "rotateLeft", "(II)I") {
            let invocation = invocation?;
            let DexValue::Int(value) = invocation.arguments.first()? else {
                return None;
            };
            let DexValue::Int(distance) = invocation.arguments.get(1)? else {
                return None;
            };

            return Some(DexExecutionResult::Int(value.rotate_left(*distance as u32)));
        }

        // cite: libcore Integer.java#rotateRight
        if method.matches_builtin_signature("Ljava/lang/Integer;", "rotateRight", "(II)I") {
            let invocation = invocation?;
            let DexValue::Int(value) = invocation.arguments.first()? else {
                return None;
            };
            let DexValue::Int(distance) = invocation.arguments.get(1)? else {
                return None;
            };

            return Some(DexExecutionResult::Int(
                value.rotate_right(*distance as u32),
            ));
        }

        if method.matches_builtin_signature(
            "Ljava/lang/Integer;",
            "toHexString",
            "(I)Ljava/lang/String;",
        ) {
            let invocation = invocation?;
            let value = integer_to_hex_string(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.matches_builtin_signature("Ljava/lang/Integer;", "intValue", "()I") {
            let invocation = invocation?;
            let value = self.integer_int_value(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore Integer.java#longValue
        if method.matches_builtin_signature("Ljava/lang/Integer;", "longValue", "()J") {
            let invocation = invocation?;
            let value = self.integer_int_value(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(i64::from(value))));
        }

        // cite: libcore Byte.java#valueOf
        if method.matches_builtin_signature("Ljava/lang/Byte;", "valueOf", "(B)Ljava/lang/Byte;") {
            let invocation = invocation?;
            let DexValue::Int(raw) = invocation.arguments.first()? else {
                return None;
            };
            let byte_val = (*raw as i8) as i32;
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from(format!("Ljava/lang/Byte;:{byte_val}")),
            )));
        }

        // cite: libcore Byte.java#byteValue
        if method.matches_builtin_signature("Ljava/lang/Byte;", "byteValue", "()B") {
            let invocation = invocation?;
            let value = match invocation.arguments.first()? {
                DexValue::Reference(reference) => reference
                    .strip_prefix("Ljava/lang/Byte;:")?
                    .parse::<i32>()
                    .ok()?,
                DexValue::Int(raw) => (*raw as i8) as i32,
                _ => return None,
            };
            return Some(DexExecutionResult::Int((value as i8) as i32));
        }

        if method.matches_builtin_signature("Ljava/lang/Number;", "intValue", "()I") {
            let invocation = invocation?;
            let value = self.integer_int_value(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature("Ljava/lang/Character;", "digit", "(II)I") {
            let invocation = invocation?;
            let value = character_digit(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/lang/Character;",
            "valueOf",
            "(C)Ljava/lang/Character;",
        ) {
            let invocation = invocation?;
            let value = character_value_of(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature("Ljava/lang/Character;", "charValue", "()C") {
            let invocation = invocation?;
            let value = character_char_value(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/lang/Character;",
            "codePointAt",
            "(Ljava/lang/CharSequence;I)I",
        ) {
            let invocation = invocation?;
            let value = self.character_code_point_at_char_sequence(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature("Ljava/lang/Character;", "charCount", "(I)I") {
            let invocation = invocation?;
            let value = character_char_count(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature("Ljava/lang/Character;", "isDigit", "(C)Z") {
            let invocation = invocation?;
            let value = character_is_digit(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature("Ljava/lang/Character;", "isWhitespace", "(C)Z") {
            let invocation = invocation?;
            let value = character_is_whitespace(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature("Ljava/lang/Character;", "isSpaceChar", "(C)Z") {
            let invocation = invocation?;
            let value = character_is_space_char(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature("Ljava/lang/Long;", "valueOf", "(J)Ljava/lang/Long;") {
            let invocation = invocation?;
            let value = long_value_of(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature("Ljava/lang/Long;", "<init>", "(J)V") {
            let invocation = invocation?;
            self.initialize_long_box(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature("Ljava/lang/Long;", "toString", "(J)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = long_to_string(invocation, None)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.matches_builtin_signature(
            "Ljava/lang/Long;",
            "toString",
            "(JI)Ljava/lang/String;",
        ) {
            let invocation = invocation?;
            let value = long_to_string(invocation, Some(2))?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.matches_builtin_signature("Ljava/lang/Float;", "valueOf", "(F)Ljava/lang/Float;")
        {
            let invocation = invocation?;
            let value = float_value_of(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature("Ljava/lang/Float;", "<init>", "(F)V") {
            let invocation = invocation?;
            self.initialize_float_box(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if (method.class_descriptor == "Ljava/lang/Float;"
            || method.class_descriptor == "Ljava/lang/Number;")
            && method.matches_builtin_member("floatValue", "()F")
        {
            let invocation = invocation?;
            let value = self.float_float_value(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Float(value)));
        }

        if method.matches_builtin_signature(
            "Ljava/lang/Double;",
            "valueOf",
            "(D)Ljava/lang/Double;",
        ) {
            let invocation = invocation?;
            let value = double_value_of(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature("Ljava/lang/Double;", "<init>", "(D)V") {
            let invocation = invocation?;
            self.initialize_double_box(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if (method.class_descriptor == "Ljava/lang/Double;"
            || method.class_descriptor == "Ljava/lang/Number;")
            && method.matches_builtin_member("doubleValue", "()D")
        {
            let invocation = invocation?;
            let value = self.double_double_value(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.matches_builtin_signature(
            "Ljava/lang/Long;",
            "getLong",
            "(Ljava/lang/String;J)Ljava/lang/Long;",
        ) {
            let invocation = invocation?;
            let value = long_get_long(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if (method.class_descriptor == "Ljava/lang/Long;"
            || method.class_descriptor == "Ljava/lang/Number;")
            && method.matches_builtin_member("longValue", "()J")
        {
            let invocation = invocation?;
            let value = self.long_long_value(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        // cite: libcore Locale.java#Locale
        if method.class_descriptor == "Ljava/util/Locale;"
            && method.name == "<init>"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/lang/String;)V"
                    | "(Ljava/lang/String;Ljava/lang/String;)V"
                    | "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V"
            )
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: libcore Float.java#parseFloat
        if method.matches_builtin_signature(
            "Ljava/lang/Float;",
            "parseFloat",
            "(Ljava/lang/String;)F",
        ) {
            let invocation = invocation?;
            let text = self.string_text(invocation.arguments.first()?)?;
            let value = text.trim().parse::<f32>().ok()?;
            return Some(DexExecutionResult::Value(DexValue::Float(value)));
        }

        // cite: libcore Locale.java#getDefault
        if method.class_descriptor == "Ljava/util/Locale;"
            && method.name == "getDefault"
            && matches!(
                method.proto.descriptor().as_str(),
                "()Ljava/util/Locale;" | "(Ljava/util/Locale$Category;)Ljava/util/Locale;"
            )
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Ljava/util/Locale;:default"),
            )));
        }

        if method.matches_builtin_signature(
            "Ljava/util/Locale;",
            "forLanguageTag",
            "(Ljava/lang/String;)Ljava/util/Locale;",
        ) {
            let invocation = invocation?;
            let value = locale_for_language_tag(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/Locale;",
            "getCountry",
            "()Ljava/lang/String;",
        ) {
            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from("US"),
            )));
        }

        if method.matches_builtin_signature(
            "Ljava/util/Locale;",
            "getDisplayLanguage",
            "()Ljava/lang/String;",
        ) {
            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from("English"),
            )));
        }

        None
    }
}

impl DexProgram {
    fn initialize_integer_box(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }
        let DexValue::Int(value) = invocation.arguments[1] else {
            return None;
        };

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &integer_value_field(),
            DexValue::Int(value),
        )
        .then_some(())
    }

    fn integer_int_value(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        match &invocation.arguments[0] {
            DexValue::Reference(reference) => {
                reference.strip_prefix("Ljava/lang/Integer;:")?.parse().ok()
            }
            value if dex_value_is_object_like(value) => {
                match self.read_stored_instance_field(value, &integer_value_field())? {
                    DexValue::Int(value) => Some(value),
                    _ => None,
                }
            }
            // cite: libcore Integer.java#intValue
            DexValue::Int(value) => Some(*value),
            _ => None,
        }
    }

    fn initialize_float_box(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let value = float_box_argument(&invocation.arguments[1])?;

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &float_value_field(),
            DexValue::Float(value),
        )
        .then_some(())
    }

    fn float_float_value(&self, invocation: &DexInvocation) -> Option<f32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        match &invocation.arguments[0] {
            DexValue::Reference(reference) => boxed_float_reference_value(reference),
            value if dex_value_is_object_like(value) => {
                match self.read_stored_instance_field(value, &float_value_field())? {
                    DexValue::Float(value) => Some(value),
                    DexValue::Int(bits) => Some(f32::from_bits(bits as u32)),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn initialize_long_box(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Long(value) = invocation.arguments[1] else {
            return None;
        };
        let DexValue::Long(high_slot_value) = invocation.arguments[2] else {
            return None;
        };
        if high_slot_value != value {
            return None;
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &long_value_field(),
            DexValue::Long(value),
        )
        .then_some(())
    }

    fn initialize_double_box(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Long(value) = invocation.arguments[1] else {
            return None;
        };
        let DexValue::Long(high_slot_value) = invocation.arguments[2] else {
            return None;
        };
        if high_slot_value != value {
            return None;
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &double_value_field(),
            DexValue::Long(value),
        )
        .then_some(())
    }

    fn double_double_value(&self, invocation: &DexInvocation) -> Option<i64> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        match &invocation.arguments[0] {
            DexValue::Reference(reference) => boxed_double_reference_value(reference),
            value if dex_value_is_object_like(value) => {
                match self.read_stored_instance_field(value, &double_value_field())? {
                    DexValue::Long(value) => Some(value),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn long_long_value(&self, invocation: &DexInvocation) -> Option<i64> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        match &invocation.arguments[0] {
            DexValue::Reference(reference) => boxed_long_reference_value(reference),
            value if dex_value_is_object_like(value) => {
                match self.read_stored_instance_field(value, &long_value_field())? {
                    DexValue::Long(value) => Some(value),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

fn integer_value_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Integer;".to_owned(),
        type_descriptor: "I".to_owned(),
        name: "value".to_owned(),
    }
}

fn float_box_argument(value: &DexValue) -> Option<f32> {
    match value {
        DexValue::Float(value) => Some(*value),
        DexValue::Int(bits) => Some(f32::from_bits(*bits as u32)),
        DexValue::Uninitialized
        | DexValue::Null
        | DexValue::Long(_)
        | DexValue::String(_)
        | DexValue::Object(_)
        | DexValue::Reference(_) => None,
    }
}

fn float_value_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Float;".to_owned(),
        type_descriptor: "F".to_owned(),
        name: "value".to_owned(),
    }
}

fn long_value_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Long;".to_owned(),
        type_descriptor: "J".to_owned(),
        name: "value".to_owned(),
    }
}

fn double_value_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Double;".to_owned(),
        type_descriptor: "D".to_owned(),
        name: "value".to_owned(),
    }
}

// cite: libcore Integer.java#toString
fn int_to_string_radix(value: i64, radix: u32) -> String {
    if value == 0 {
        return "0".to_owned();
    }

    let negative = value < 0;
    let mut magnitude = value.unsigned_abs();
    let mut digits = Vec::new();
    while magnitude > 0 {
        let digit = (magnitude % u64::from(radix)) as u32;
        digits.push(std::char::from_digit(digit, radix).expect("digit within radix"));
        magnitude /= u64::from(radix);
    }
    if negative {
        digits.push('-');
    }

    digits.iter().rev().collect()
}
