use super::super::*;

impl DexProgram {
    pub(super) fn invoke_java_core_runtime_builtin(
        &self,
        method: &DexMethodId,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        if method.name == "getClass" && method.proto.descriptor_is("()Ljava/lang/Class;") {
            let invocation = invocation?;
            let value = object_get_class(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: libcore String.java#hashCode
        if method.class_descriptor == "Ljava/lang/Object;"
            && method.name == "hashCode"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self
                .string_hash_code(invocation)
                .or_else(|| object_hash_code(invocation))?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore AccessController.java#doPrivileged
        if method.class_descriptor == "Ljava/security/AccessController;"
            && method.name == "doPrivileged"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/security/PrivilegedAction;)Ljava/lang/Object;"
                    | "(Ljava/security/PrivilegedExceptionAction;)Ljava/lang/Object;"
            )
        {
            let invocation = invocation?;
            let action = invocation.arguments.first()?.clone();
            let class_descriptor = value_class_descriptor(&action)?.to_owned();
            let run_method = DexMethodId {
                class_descriptor,
                name: "run".to_owned(),
                proto: crate::dex::DexProto {
                    shorty: "L".to_owned(),
                    return_type: "Ljava/lang/Object;".to_owned(),
                    parameters: Vec::new(),
                },
            };
            if let Ok(Some(resolved)) = self.resolve_exact_method(&run_method) {
                if let Ok(result) =
                    self.execute_method(None, &resolved, Some(std::slice::from_ref(&action)), 1)
                {
                    return Some(result);
                }
            }
            return None;
        }

        // cite: libcore ByteOrder.java#nativeOrder
        if method.class_descriptor == "Ljava/nio/ByteOrder;"
            && method.name == "nativeOrder"
            && method.proto.descriptor_is("()Ljava/nio/ByteOrder;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Ljava/nio/ByteOrder;:LITTLE_ENDIAN"),
            )));
        }

        if method.name == "clone"
            && method.proto.descriptor_is("()Ljava/lang/Object;")
            && (method.class_descriptor == "Ljava/lang/Object;"
                || method.class_descriptor.starts_with('['))
        {
            let invocation = invocation?;
            let value = self.clone_array_object(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Object;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/Object;"
            && method.name == "notifyAll"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            object_notify_all(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if is_supported_throwable_class(&method.class_descriptor)
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if is_supported_throwable_class(&method.class_descriptor)
            && method.name == "<init>"
            && method.proto.descriptor_is("(Ljava/lang/String;)V")
        {
            if let Some(invocation) = invocation {
                self.initialize_throwable_message(invocation)?;
            }

            return Some(DexExecutionResult::Void);
        }

        if is_supported_throwable_class(&method.class_descriptor)
            && method.name == "<init>"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;Ljava/lang/Throwable;)V")
        {
            if let Some(invocation) = invocation {
                self.initialize_throwable_message_and_cause(invocation)?;
            }

            return Some(DexExecutionResult::Void);
        }

        if is_supported_throwable_class(&method.class_descriptor)
            && method.name == "<init>"
            && method.proto.descriptor_is("(Ljava/lang/Throwable;)V")
        {
            if let Some(invocation) = invocation {
                self.initialize_throwable_cause(invocation)?;
            }

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/Throwable;"
            && method.name == "printStackTrace"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.throwable_print_stack_trace(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/Throwable;"
            && method.name == "initCause"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Throwable;)Ljava/lang/Throwable;")
        {
            let invocation = invocation?;
            let value = self.throwable_init_cause(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Throwable;"
            && method.name == "addSuppressed"
            && method.proto.descriptor_is("(Ljava/lang/Throwable;)V")
        {
            let invocation = invocation?;
            self.throwable_add_suppressed(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/Throwable;"
            && method.name == "getSuppressed"
            && method.proto.descriptor_is("()[Ljava/lang/Throwable;")
        {
            let invocation = invocation?;
            let value = self.throwable_get_suppressed(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/Object;"
            && method.name == "equals"
            && method.proto.descriptor_is("(Ljava/lang/Object;)Z")
        {
            let invocation = invocation?;
            if let Some(left) = invocation
                .arguments
                .first()
                .and_then(|v| self.string_text(v))
            {
                // cite: libcore String.java#equals
                let matched = invocation
                    .arguments
                    .get(1)
                    .and_then(|v| self.string_text(v))
                    .is_some_and(|right| left == right);
                return Some(DexExecutionResult::Int(i32::from(matched)));
            }
            let value = object_equals(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/lang/Object;" | "Ljava/lang/CharSequence;"
        ) && method.name == "toString"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.object_to_string(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/Comparable;"
            && method.name == "compareTo"
            && method.proto.descriptor_is("(Ljava/lang/Object;)I")
        {
            let invocation = invocation?;
            let value = self.comparable_compare_to(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/Objects;"
            && method.name == "toString"
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

        if method.class_descriptor == "Ljava/util/Objects;"
            && method.name == "requireNonNull"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/lang/Object;)Ljava/lang/Object;"
                    | "(Ljava/lang/Object;Ljava/lang/String;)Ljava/lang/Object;"
            )
        {
            let invocation = invocation?;
            let value = objects_require_non_null(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: libcore Objects.java#equals
        if method.matches_builtin_signature(
            "Ljava/util/Objects;",
            "equals",
            "(Ljava/lang/Object;Ljava/lang/Object;)Z",
        ) {
            let invocation = invocation?;
            let left = invocation.arguments.first()?;
            let right = invocation.arguments.get(1)?;
            let equal = self.runtime_values_equal(left, right, 0, true).ok()?;
            return Some(DexExecutionResult::Int(i32::from(equal)));
        }

        if method.class_descriptor == "Ljava/lang/System;"
            && method.name == "currentTimeMillis"
            && method.proto.descriptor_is("()J")
        {
            let value = self.runtime.borrow().current_wall_clock_millis();

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/System;"
            && method.name == "nanoTime"
            && method.proto.descriptor_is("()J")
        {
            let value = self.runtime.borrow().current_monotonic_time_nanos();

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/time/Instant;"
            && method.name == "now"
            && method.proto.descriptor_is("()Ljava/time/Instant;")
        {
            let value = self.instant_now();

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/time/Instant;"
            && method.name == "ofEpochMilli"
            && method.proto.descriptor_is("(J)Ljava/time/Instant;")
        {
            let invocation = invocation?;
            let value = self.instant_of_epoch_milli(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/time/Instant;"
            && method.name == "ofEpochSecond"
            && matches!(
                method.proto.descriptor().as_str(),
                "(J)Ljava/time/Instant;" | "(JJ)Ljava/time/Instant;"
            )
        {
            let invocation = invocation?;
            let has_nano_adjustment = method.proto.descriptor_is("(JJ)Ljava/time/Instant;");
            let value = self.instant_of_epoch_second(invocation, has_nano_adjustment)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/time/Instant;"
            && method.name == "toEpochMilli"
            && method.proto.descriptor_is("()J")
        {
            let invocation = invocation?;
            let value = self.instant_to_epoch_milli(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/time/Instant;"
            && method.name == "getEpochSecond"
            && method.proto.descriptor_is("()J")
        {
            let invocation = invocation?;
            let value = self.instant_epoch_second(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/time/Instant;"
            && method.name == "getNano"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.instant_nano(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/time/ZoneId;"
            && method.name == "of"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Ljava/time/ZoneId;")
        {
            // cite: libcore ZoneId.java#of
            let invocation = invocation?;
            let value = self.zone_id_of(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/time/ZoneId;"
            && method.name == "systemDefault"
            && method.proto.descriptor_is("()Ljava/time/ZoneId;")
        {
            // cite: libcore ZoneId.java#systemDefault
            let value = self.zone_id_system_default();

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/time/ZoneId;"
            && method.name == "getId"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            // cite: libcore ZoneId.java#getId
            let invocation = invocation?;
            let value = self.zone_id_get_id(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/time/ZoneId;"
            && method.name == "getRules"
            && method.proto.descriptor_is("()Ljava/time/zone/ZoneRules;")
        {
            // cite: libcore ZoneId.java#getRules
            let invocation = invocation?;
            let value = self.zone_id_get_rules(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/time/zone/ZoneRules;"
            && method.name == "isFixedOffset"
            && method.proto.descriptor_is("()Z")
        {
            // cite: libcore ZoneRules.java#isFixedOffset
            let invocation = invocation?;
            let rules = self.zone_rules_data(invocation)?;

            return Some(DexExecutionResult::Int(i32::from(rules.is_fixed())));
        }

        if method.class_descriptor == "Ljava/time/LocalDateTime;"
            && method.name == "ofInstant"
            && method.proto.descriptor()
                == "(Ljava/time/Instant;Ljava/time/ZoneId;)Ljava/time/LocalDateTime;"
        {
            // cite: libcore LocalDateTime.java#ofInstant, ofEpochSecond
            let invocation = invocation?;
            let value = self.local_date_time_of_instant(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/time/LocalDateTime;"
            && method.name == "atZone"
            && method
                .proto
                .descriptor_is("(Ljava/time/ZoneId;)Ljava/time/ZonedDateTime;")
        {
            // cite: libcore LocalDateTime.java#atZone
            let invocation = invocation?;
            let value = self.local_date_time_at_zone(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/time/Instant;"
            && method.name == "atZone"
            && method
                .proto
                .descriptor_is("(Ljava/time/ZoneId;)Ljava/time/ZonedDateTime;")
        {
            // cite: libcore Instant.java#atZone
            let invocation = invocation?;
            let value = self.instant_at_zone(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/time/ZonedDateTime;"
            && method.proto.descriptor_is("()I")
        {
            if let Some(component) = zoned_component_index(&method.name) {
                let invocation = invocation?;
                let components = self.zoned_date_time_components(invocation)?;

                return Some(DexExecutionResult::Int(components[component]));
            }
        }

        if method.class_descriptor == "Ljava/time/ZonedDateTime;"
            && matches!(
                method.name.as_str(),
                "withDayOfMonth"
                    | "withMonth"
                    | "withYear"
                    | "withHour"
                    | "withMinute"
                    | "withSecond"
            )
            && method.proto.descriptor_is("(I)Ljava/time/ZonedDateTime;")
        {
            // cite: libcore ZonedDateTime.java#resolveLocal
            let invocation = invocation?;
            let value = self.zoned_date_time_adjusted(invocation, &method.name)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/time/ZonedDateTime;"
            && matches!(
                method.name.as_str(),
                "plusDays"
                    | "minusDays"
                    | "plusWeeks"
                    | "minusWeeks"
                    | "plusMonths"
                    | "minusMonths"
                    | "plusYears"
                    | "minusYears"
            )
            && method.proto.descriptor_is("(J)Ljava/time/ZonedDateTime;")
        {
            // cite: libcore ZonedDateTime.java#resolveLocal
            let invocation = invocation?;
            let value = self.zoned_date_time_adjusted(invocation, &method.name)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/time/ZonedDateTime;"
            && method.name == "getDayOfWeek"
            && method.proto.descriptor_is("()Ljava/time/DayOfWeek;")
        {
            // cite: libcore LocalDate.java#getDayOfWeek
            let invocation = invocation?;
            let value = self.zoned_date_time_day_of_week(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/time/ZonedDateTime;"
            && method.name == "getMonth"
            && method.proto.descriptor_is("()Ljava/time/Month;")
        {
            // cite: libcore Month.java#of
            let invocation = invocation?;
            let components = self.zoned_date_time_components(invocation)?;
            let name = MONTH_NAMES.get(usize::try_from(components[1]).ok()?.checked_sub(1)?)?;

            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from(format!("Ljava/time/Month;.{name}")),
            )));
        }

        if method.class_descriptor == "Ljava/time/ZonedDateTime;"
            && method.name == "toLocalDate"
            && method.proto.descriptor_is("()Ljava/time/LocalDate;")
        {
            // cite: libcore ZonedDateTime.java#toLocalDate
            let invocation = invocation?;
            let components = self.zoned_date_time_components(invocation)?;

            return Some(DexExecutionResult::Value(local_date_reference(
                components[0],
                components[1],
                components[2],
            )));
        }

        if method.class_descriptor == "Ljava/time/LocalDateTime;"
            && method.name == "toLocalDate"
            && method.proto.descriptor_is("()Ljava/time/LocalDate;")
        {
            // cite: libcore LocalDateTime.java#toLocalDate
            let invocation = invocation?;
            if invocation.arguments.len() != 1 {
                return None;
            }
            let mut components = [0i32; 3];
            for (index, getter) in ["getYear", "getMonthValue", "getDayOfMonth"]
                .iter()
                .enumerate()
            {
                let DexValue::Int(value) = self.read_stored_instance_field(
                    &invocation.arguments[0],
                    &local_date_time_component_field(getter)?,
                )?
                else {
                    return None;
                };
                components[index] = value;
            }

            return Some(DexExecutionResult::Value(local_date_reference(
                components[0],
                components[1],
                components[2],
            )));
        }

        if method.class_descriptor == "Ljava/time/LocalDate;"
            && method.name == "of"
            && method.proto.descriptor_is("(III)Ljava/time/LocalDate;")
        {
            // cite: libcore LocalDate.java#of
            let invocation = invocation?;
            let [DexValue::Int(year), DexValue::Int(month), DexValue::Int(day)] =
                invocation.arguments[..]
            else {
                return None;
            };

            return Some(DexExecutionResult::Value(local_date_reference(
                year, month, day,
            )));
        }

        if method.class_descriptor == "Ljava/time/LocalDate;" {
            if let Some(result) = self.invoke_local_date_builtin(&method.name, invocation) {
                return Some(result);
            }
        }

        if method.class_descriptor == "Ljava/time/LocalDate;"
            && method.name == "atTime"
            && method
                .proto
                .descriptor_is("(Ljava/time/LocalTime;)Ljava/time/LocalDateTime;")
        {
            // cite: libcore LocalDate.java#atTime
            let invocation = invocation?;
            let (year, month, day) = local_date_parts(invocation.arguments.first()?)?;
            let (second_of_day, nano) = local_time_parts(invocation.arguments.get(1)?)?;
            let value = self.allocate_local_date_time([
                year,
                month,
                day,
                second_of_day / 3600,
                second_of_day / 60 % 60,
                second_of_day % 60,
                nano,
            ])?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/time/LocalDate;"
            && method.name == "atStartOfDay"
            && method.proto.descriptor_is("()Ljava/time/LocalDateTime;")
        {
            // cite: libcore LocalDate.java#atStartOfDay
            let invocation = invocation?;
            let (year, month, day) = local_date_parts(invocation.arguments.first()?)?;
            let value = self.allocate_local_date_time([year, month, day, 0, 0, 0, 0])?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/time/LocalTime;"
            && method.name == "of"
            && matches!(
                method.proto.descriptor().as_str(),
                "(II)Ljava/time/LocalTime;" | "(III)Ljava/time/LocalTime;"
            )
        {
            // cite: libcore LocalTime.java#of
            let invocation = invocation?;
            let hour = match invocation.arguments.first() {
                Some(DexValue::Int(value)) => *value,
                _ => return None,
            };
            let minute = match invocation.arguments.get(1) {
                Some(DexValue::Int(value)) => *value,
                _ => return None,
            };
            let second = match invocation.arguments.get(2) {
                Some(DexValue::Int(value)) => *value,
                None => 0,
                _ => return None,
            };

            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from(format!(
                    "Ljava/time/LocalTime;:{}:0",
                    hour * 3600 + minute * 60 + second
                )),
            )));
        }

        if method.class_descriptor == "Ljava/time/LocalTime;" && method.proto.descriptor_is("()I") {
            // cite: libcore LocalTime.java#getHour, getMinute, getSecond, toSecondOfDay
            let invocation = invocation?;
            let (second_of_day, nano) = local_time_parts(invocation.arguments.first()?)?;
            let value = match method.name.as_str() {
                "getHour" => second_of_day / 3600,
                "getMinute" => second_of_day / 60 % 60,
                "getSecond" => second_of_day % 60,
                "getNano" => nano,
                "toSecondOfDay" => second_of_day,
                _ => return None,
            };

            return Some(DexExecutionResult::Int(value));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/time/LocalDateTime;" | "Ljava/time/chrono/ChronoLocalDateTime;"
        ) && method.name == "toEpochSecond"
            && method.proto.descriptor_is("(Ljava/time/ZoneOffset;)J")
        {
            // cite: libcore ChronoLocalDateTime.java#toEpochSecond
            let invocation = invocation?;
            if invocation.arguments.len() != 2 {
                return None;
            }
            let mut components = [0i32; 7];
            for (index, getter) in [
                "getYear",
                "getMonthValue",
                "getDayOfMonth",
                "getHour",
                "getMinute",
                "getSecond",
                "getNano",
            ]
            .iter()
            .enumerate()
            {
                let DexValue::Int(value) = self.read_stored_instance_field(
                    &invocation.arguments[0],
                    &local_date_time_component_field(getter)?,
                )?
                else {
                    return None;
                };
                components[index] = value;
            }
            let offset_seconds = zone_offset_total_seconds(invocation.arguments.get(1)?)?;
            let epoch_day = epoch_day_from_civil_date(components[0], components[1], components[2])?;
            let value = epoch_day
                .checked_mul(86_400)?
                .checked_add(i64::from(
                    components[3] * 3600 + components[4] * 60 + components[5],
                ))?
                .checked_sub(i64::from(offset_seconds))?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/time/ZoneOffset;"
            && method.name == "getTotalSeconds"
            && method.proto.descriptor_is("()I")
        {
            // cite: libcore ZoneOffset.java#getTotalSeconds
            let invocation = invocation?;
            let value = zone_offset_total_seconds(invocation.arguments.first()?)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/time/ZoneOffset;"
            && method.name == "ofTotalSeconds"
            && method.proto.descriptor_is("(I)Ljava/time/ZoneOffset;")
        {
            // cite: libcore ZoneOffset.java#ofTotalSeconds
            let invocation = invocation?;
            let DexValue::Int(total_seconds) = *invocation.arguments.first()? else {
                return None;
            };

            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from(format!("Ljava/time/ZoneOffset;:{total_seconds}")),
            )));
        }

        if method.class_descriptor == "Ljava/time/zone/ZoneRules;"
            && method.name == "getOffset"
            && method
                .proto
                .descriptor_is("(Ljava/time/Instant;)Ljava/time/ZoneOffset;")
        {
            // cite: libcore ZoneRules.java#getOffset
            let invocation = invocation?;
            let rules = self.zone_rules_data(invocation)?;
            let (epoch_second, _nano) = self.instant_parts(invocation.arguments.get(1)?)?;
            let total_seconds = rules.offset_seconds_at(epoch_second);

            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from(format!("Ljava/time/ZoneOffset;:{total_seconds}")),
            )));
        }

        if method.class_descriptor == "Ljava/time/LocalDateTime;"
            && method.name == "toLocalTime"
            && method.proto.descriptor_is("()Ljava/time/LocalTime;")
        {
            // cite: libcore LocalDateTime.java#toLocalTime
            let invocation = invocation?;
            if invocation.arguments.len() != 1 {
                return None;
            }
            let mut parts = [0i32; 4];
            for (index, getter) in ["getHour", "getMinute", "getSecond", "getNano"]
                .iter()
                .enumerate()
            {
                let DexValue::Int(value) = self.read_stored_instance_field(
                    &invocation.arguments[0],
                    &local_date_time_component_field(getter)?,
                )?
                else {
                    return None;
                };
                parts[index] = value;
            }

            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from(format!(
                    "Ljava/time/LocalTime;:{}:{}",
                    parts[0] * 3600 + parts[1] * 60 + parts[2],
                    parts[3]
                )),
            )));
        }

        if method.class_descriptor == "Ljava/time/ZonedDateTime;"
            && method.name == "toLocalDateTime"
            && method.proto.descriptor_is("()Ljava/time/LocalDateTime;")
        {
            // cite: libcore ZonedDateTime.java#toLocalDateTime
            let invocation = invocation?;
            let components = self.zoned_date_time_components(invocation)?;
            let value = self.allocate_local_date_time(components)?;

            return Some(DexExecutionResult::Value(value));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/time/ZonedDateTime;" | "Ljava/time/chrono/ChronoZonedDateTime;"
        ) && method.name == "toInstant"
            && method.proto.descriptor_is("()Ljava/time/Instant;")
        {
            // cite: libcore ChronoZonedDateTime.java#toInstant
            let invocation = invocation?;
            let value = self.zoned_date_time_to_instant(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/time/ZonedDateTime;" | "Ljava/time/chrono/ChronoZonedDateTime;"
        ) && method.name == "toEpochSecond"
            && method.proto.descriptor_is("()J")
        {
            // cite: libcore ChronoZonedDateTime.java#toEpochSecond
            let invocation = invocation?;
            let value = self.zoned_date_time_epoch_second(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/time/LocalDateTime;"
            && method.proto.descriptor_is("()I")
        {
            if let Some(field) = local_date_time_component_field(&method.name) {
                let invocation = invocation?;
                if invocation.arguments.len() != 1 {
                    return None;
                }
                let DexValue::Int(value) =
                    self.read_stored_instance_field(&invocation.arguments[0], &field)?
                else {
                    return None;
                };

                return Some(DexExecutionResult::Int(value));
            }
        }

        if method.class_descriptor == "Ljava/time/temporal/WeekFields;"
            && method.name == "of"
            && method
                .proto
                .descriptor_is("(Ljava/util/Locale;)Ljava/time/temporal/WeekFields;")
        {
            // cite: libcore WeekFields.java#of
            let invocation = invocation?;
            let value = week_fields_of_locale(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/time/temporal/WeekFields;"
            && method.name == "getFirstDayOfWeek"
            && method.proto.descriptor_is("()Ljava/time/DayOfWeek;")
        {
            // cite: libcore WeekFields.java#getFirstDayOfWeek
            let invocation = invocation?;
            let (first_day, _min_days) = week_fields_parts(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from(format!("Ljava/time/DayOfWeek;.{first_day}")),
            )));
        }

        if method.class_descriptor == "Ljava/time/temporal/WeekFields;"
            && method.name == "getMinimalDaysInFirstWeek"
            && method.proto.descriptor_is("()I")
        {
            // cite: libcore WeekFields.java#getMinimalDaysInFirstWeek
            let invocation = invocation?;
            let (_first_day, min_days) = week_fields_parts(invocation)?;

            return Some(DexExecutionResult::Int(min_days));
        }

        if method.class_descriptor == "Ljava/time/DayOfWeek;"
            && method.name == "values"
            && method.proto.descriptor_is("()[Ljava/time/DayOfWeek;")
        {
            // cite: libcore DayOfWeek.java#ENUMS
            let values = DAY_OF_WEEK_NAMES
                .iter()
                .map(|name| {
                    DexValue::Reference(std::rc::Rc::from(format!("Ljava/time/DayOfWeek;.{name}")))
                })
                .collect();

            return Some(DexExecutionResult::Value(
                self.allocate_filled_array("[Ljava/time/DayOfWeek;".to_owned(), values),
            ));
        }

        if method.class_descriptor == "Ljava/time/DayOfWeek;"
            && method.name == "getDisplayName"
            && method.proto.descriptor()
                == "(Ljava/time/format/TextStyle;Ljava/util/Locale;)Ljava/lang/String;"
        {
            // cite: libcore DayOfWeek.java#getDisplayName
            let invocation = invocation?;
            let value = day_of_week_display_name(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/time/Month;"
            && method.name == "values"
            && method.proto.descriptor_is("()[Ljava/time/Month;")
        {
            // cite: libcore Month.java#ENUMS
            let values = MONTH_NAMES
                .iter()
                .map(|name| {
                    DexValue::Reference(std::rc::Rc::from(format!("Ljava/time/Month;.{name}")))
                })
                .collect();

            return Some(DexExecutionResult::Value(
                self.allocate_filled_array("[Ljava/time/Month;".to_owned(), values),
            ));
        }

        if method.class_descriptor == "Ljava/time/Month;"
            && method.name == "getValue"
            && method.proto.descriptor_is("()I")
        {
            // cite: libcore Month.java#getValue
            let invocation = invocation?;
            let value = month_value(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/time/Month;"
            && method.name == "getDisplayName"
            && method.proto.descriptor()
                == "(Ljava/time/format/TextStyle;Ljava/util/Locale;)Ljava/lang/String;"
        {
            // cite: libcore Month.java#getDisplayName
            let invocation = invocation?;
            let value = month_display_name(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/time/DayOfWeek;"
            && method.name == "getValue"
            && method.proto.descriptor_is("()I")
        {
            // cite: libcore DayOfWeek.java#getValue
            let invocation = invocation?;
            let value = day_of_week_value(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/System;"
            && method.name == "getProperty"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/lang/String;)Ljava/lang/String;"
                    | "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;"
            )
        {
            let invocation = invocation?;
            let value = system_get_property(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/lang/System;"
            && method.name == "arraycopy"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;ILjava/lang/Object;II)V")
        {
            let invocation = invocation?;
            self.system_arraycopy(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/System;"
            && method.name == "identityHashCode"
            && method.proto.descriptor_is("(Ljava/lang/Object;)I")
        {
            let invocation = invocation?;
            let value = object_hash_code(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/io/PrintStream;"
            && matches!(method.name.as_str(), "print" | "println")
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/lang/Object;)V" | "(Ljava/lang/String;)V" | "(I)V" | "(Z)V"
            )
        {
            let invocation = invocation?;
            print_stream_write(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/Runtime;"
            && method.name == "getRuntime"
            && method.proto.descriptor_is("()Ljava/lang/Runtime;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Ljava/lang/Runtime;.bootstrap"),
            )));
        }

        if method.class_descriptor == "Ljava/lang/Runtime;"
            && method.name == "availableProcessors"
            && method.proto.descriptor_is("()I")
        {
            return Some(DexExecutionResult::Int(runtime_available_processors()));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "min"
            && method.proto.descriptor_is("(II)I")
        {
            let invocation = invocation?;
            let value = math_min_int(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "min"
            && method.proto.descriptor_is("(JJ)J")
        {
            let invocation = invocation?;
            let value = math_min_long(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "min"
            && method.proto.descriptor_is("(FF)F")
        {
            let invocation = invocation?;
            let value = math_min_float(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Float(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "max"
            && method.proto.descriptor_is("(II)I")
        {
            let invocation = invocation?;
            let value = math_max_int(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "max"
            && method.proto.descriptor_is("(JJ)J")
        {
            let invocation = invocation?;
            let value = math_max_long(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "max"
            && method.proto.descriptor_is("(FF)F")
        {
            let invocation = invocation?;
            let value = math_max_float(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Float(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "atan2"
            && method.proto.descriptor_is("(DD)D")
        {
            let invocation = invocation?;
            let value = math_atan2_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "max"
            && method.proto.descriptor_is("(DD)D")
        {
            let invocation = invocation?;
            let value = math_max_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "round"
            && method.proto.descriptor_is("(F)I")
        {
            let invocation = invocation?;
            let value = math_round_float(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "round"
            && method.proto.descriptor_is("(D)J")
        {
            let invocation = invocation?;
            let value = math_round_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "signum"
            && method.proto.descriptor_is("(F)F")
        {
            let invocation = invocation?;
            let value = math_signum_float(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Float(value)));
        }

        if method.class_descriptor == "Ljava/lang/Integer;"
            && method.name == "numberOfLeadingZeros"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = integer_number_of_leading_zeros(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Integer;"
            && method.name == "bitCount"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = integer_bit_count(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Integer;"
            && method.name == "lowestOneBit"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = integer_lowest_one_bit(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Integer;"
            && method.name == "highestOneBit"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = integer_highest_one_bit(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Integer;"
            && method.name == "hashCode"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = integer_hash_code(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Integer;"
            && method.name == "signum"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = integer_signum(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Long;"
            && method.name == "numberOfLeadingZeros"
            && method.proto.descriptor_is("(J)I")
        {
            let invocation = invocation?;
            let value = long_number_of_leading_zeros(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Long;"
            && method.name == "numberOfTrailingZeros"
            && method.proto.descriptor_is("(J)I")
        {
            let invocation = invocation?;
            let value = long_number_of_trailing_zeros(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Long;"
            && method.name == "hashCode"
            && method.proto.descriptor_is("(J)I")
        {
            let invocation = invocation?;
            let value = long_hash_code(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Long;"
            && method.name == "signum"
            && method.proto.descriptor_is("(J)I")
        {
            let invocation = invocation?;
            let value = long_signum(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Long;"
            && method.name == "compareUnsigned"
            && method.proto.descriptor_is("(JJ)I")
        {
            let invocation = invocation?;
            let value = long_compare_unsigned(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Long;"
            && method.name == "rotateLeft"
            && method.proto.descriptor_is("(JI)J")
        {
            let invocation = invocation?;
            let value = long_rotate_left(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Long;"
            && method.name == "rotateRight"
            && method.proto.descriptor_is("(JI)J")
        {
            let invocation = invocation?;
            let value = long_rotate_right(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Float;"
            && method.name == "intBitsToFloat"
            && method.proto.descriptor_is("(I)F")
        {
            let invocation = invocation?;
            let value = float_int_bits_to_float(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Float(value)));
        }

        if method.class_descriptor == "Ljava/lang/Float;"
            && method.name == "floatToRawIntBits"
            && method.proto.descriptor_is("(F)I")
        {
            let invocation = invocation?;
            let value = float_to_raw_int_bits(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Float;"
            && method.name == "floatToIntBits"
            && method.proto.descriptor_is("(F)I")
        {
            let invocation = invocation?;
            let value = float_to_int_bits(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Float;"
            && method.name == "hashCode"
            && method.proto.descriptor_is("(F)I")
        {
            let invocation = invocation?;
            let value = float_to_int_bits(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Float;"
            && method.name == "compare"
            && method.proto.descriptor_is("(FF)I")
        {
            let invocation = invocation?;
            let value = float_compare(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore Integer.java#max, min
        if method.class_descriptor == "Ljava/lang/Integer;"
            && method.name == "max"
            && method.proto.descriptor_is("(II)I")
        {
            let invocation = invocation?;
            let value = math_max_int(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Integer;"
            && method.name == "min"
            && method.proto.descriptor_is("(II)I")
        {
            let invocation = invocation?;
            let value = math_min_int(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore Long.java#compare
        if method.class_descriptor == "Ljava/lang/Long;"
            && method.name == "compare"
            && method.proto.descriptor_is("(JJ)I")
        {
            let invocation = invocation?;
            let (first, second) = math_two_longs(invocation)?;
            let value = match first.cmp(&second) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            };

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Float;"
            && method.name == "isNaN"
            && method.proto.descriptor_is("(F)Z")
        {
            let invocation = invocation?;
            let value = float_is_nan(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Float;"
            && method.name == "isInfinite"
            && method.proto.descriptor_is("(F)Z")
        {
            let invocation = invocation?;
            let value = float_is_infinite(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Double;"
            && method.name == "isNaN"
            && method.proto.descriptor_is("(D)Z")
        {
            let invocation = invocation?;
            let value = double_is_nan(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/lang/Double;"
            && method.name == "doubleToRawLongBits"
            && method.proto.descriptor_is("(D)J")
        {
            let invocation = invocation?;
            let value = double_to_raw_long_bits(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Double;"
            && method.name == "parseDouble"
            && method.proto.descriptor_is("(Ljava/lang/String;)D")
        {
            let invocation = invocation?;
            let bits = double_parse_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(bits)));
        }

        if method.class_descriptor == "Ljava/lang/Double;"
            && method.name == "toString"
            && method.proto.descriptor_is("(D)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = string_value_of_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "abs"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = math_abs_int(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore Math.java#abs
        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "abs"
            && method.proto.descriptor_is("(J)J")
        {
            let invocation = invocation?;
            let value = single_long_argument(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(
                value.wrapping_abs(),
            )));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "abs"
            && method.proto.descriptor_is("(F)F")
        {
            let invocation = invocation?;
            let value = math_abs_float(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Float(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "abs"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_abs_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "ceil"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_ceil_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "floor"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_floor_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "rint"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_rint_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "log"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_log_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "exp"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_exp_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "log10"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_log10_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "ulp"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_ulp_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "sqrt"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_sqrt_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "toRadians"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_to_radians_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "toDegrees"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_to_degrees_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "sin"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_sin_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "asin"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_asin_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "cos"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_cos_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "acos"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_acos_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "tan"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_tan_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "atan"
            && method.proto.descriptor_is("(D)D")
        {
            let invocation = invocation?;
            let value = math_atan_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "pow"
            && method.proto.descriptor_is("(DD)D")
        {
            let invocation = invocation?;
            let value = math_pow_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/lang/Math;"
            && method.name == "hypot"
            && method.proto.descriptor_is("(DD)D")
        {
            let invocation = invocation?;
            let value = math_hypot_double(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/text/DecimalFormatSymbols;"
            && method.name == "getInstance"
            && method
                .proto
                .descriptor_is("()Ljava/text/DecimalFormatSymbols;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Ljava/text/DecimalFormatSymbols;:root"),
            )));
        }

        if method.class_descriptor == "Ljava/text/DecimalFormatSymbols;"
            && method.name == "getDecimalSeparator"
            && method.proto.descriptor_is("()C")
        {
            return Some(DexExecutionResult::Int('.' as i32));
        }

        if method.class_descriptor == "Ljava/text/DecimalFormatSymbols;"
            && method.name == "getGroupingSeparator"
            && method.proto.descriptor_is("()C")
        {
            return Some(DexExecutionResult::Int(',' as i32));
        }

        if method.class_descriptor == "Landroid/os/SystemClock;"
            && method.name == "elapsedRealtime"
            && method.proto.descriptor_is("()J")
        {
            let value = self.runtime.borrow().main_thread_uptime_millis;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Landroid/os/SystemClock;"
            && method.name == "uptimeMillis"
            && method.proto.descriptor_is("()J")
        {
            let value = self.runtime.borrow().main_thread_uptime_millis;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Landroid/os/ext/SdkExtensions;"
            && method.name == "getExtensionVersion"
            && method.proto.descriptor_is("(I)I")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Landroid/os/Looper;"
            && method.name == "getMainLooper"
            && method.proto.descriptor_is("()Landroid/os/Looper;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroid/os/Looper;:main"),
            )));
        }

        if method.class_descriptor == "Landroid/os/Looper;"
            && method.name == "myLooper"
            && method.proto.descriptor_is("()Landroid/os/Looper;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroid/os/Looper;:main"),
            )));
        }

        if method.class_descriptor == "Landroid/os/Looper;"
            && method.name == "getThread"
            && method.proto.descriptor_is("()Ljava/lang/Thread;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Ljava/lang/Thread;:main"),
            )));
        }

        if method.class_descriptor == "Ljava/lang/Thread;"
            && method.name == "currentThread"
            && method.proto.descriptor_is("()Ljava/lang/Thread;")
        {
            // cite: libcore Thread.java#currentThread
            return Some(DexExecutionResult::Value(
                self.runtime.borrow().current_guest_thread.clone(),
            ));
        }

        if method.class_descriptor == "Ljava/lang/Thread;"
            && method.name == "interrupted"
            && method.proto.descriptor_is("()Z")
        {
            return Some(DexExecutionResult::Int(0));
        }

        // cite: libcore Thread.java#isInterrupted
        if method.class_descriptor == "Ljava/lang/Thread;"
            && method.name == "isInterrupted"
            && method.proto.descriptor_is("()Z")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Ljava/lang/Thread;"
            && method.name == "getUncaughtExceptionHandler"
            && method
                .proto
                .descriptor_is("()Ljava/lang/Thread$UncaughtExceptionHandler;")
        {
            let invocation = invocation?;
            thread_reference(invocation)?;

            return Some(DexExecutionResult::Value(
                thread_uncaught_exception_handler(),
            ));
        }

        if method.class_descriptor == "Ljava/lang/Thread;"
            && method.name == "getDefaultUncaughtExceptionHandler"
            && method
                .proto
                .descriptor_is("()Ljava/lang/Thread$UncaughtExceptionHandler;")
        {
            return Some(DexExecutionResult::Value(
                thread_uncaught_exception_handler(),
            ));
        }

        // cite: libcore Throwable.java#getStackTrace
        if method.class_descriptor == "Ljava/lang/Throwable;"
            && method.name == "getStackTrace"
            && method
                .proto
                .descriptor_is("()[Ljava/lang/StackTraceElement;")
        {
            return Some(DexExecutionResult::Value(self.allocate_filled_array(
                "[Ljava/lang/StackTraceElement;".to_owned(),
                Vec::new(),
            )));
        }

        // cite: libcore Throwable.java#setStackTrace
        if method.class_descriptor == "Ljava/lang/Throwable;"
            && method.name == "setStackTrace"
            && method
                .proto
                .descriptor_is("([Ljava/lang/StackTraceElement;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/Thread$UncaughtExceptionHandler;"
            && method.name == "uncaughtException"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Thread;Ljava/lang/Throwable;)V")
        {
            let invocation = invocation?;
            thread_uncaught_exception(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/Thread;"
            && method.name == "<init>"
            && matches!(
                method.proto.descriptor().as_str(),
                "()V"
                    | "(Ljava/lang/Runnable;)V"
                    | "(Ljava/lang/String;)V"
                    | "(Ljava/lang/Runnable;Ljava/lang/String;)V"
            )
        {
            if let Some(invocation) = invocation {
                self.initialize_thread(invocation, &method.proto.descriptor())?;
            }

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/Thread;"
            && method.name == "start"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.thread_start(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/Thread;"
            && method.name == "setDaemon"
            && method.proto.descriptor_is("(Z)V")
        {
            let invocation = invocation?;
            self.thread_set_daemon(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/Thread;"
            && method.name == "setContextClassLoader"
            && method.proto.descriptor_is("(Ljava/lang/ClassLoader;)V")
        {
            let invocation = invocation?;
            self.thread_set_context_class_loader(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/Thread;"
            && method.name == "setName"
            && method.proto.descriptor_is("(Ljava/lang/String;)V")
        {
            let invocation = invocation?;
            self.thread_set_name(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/lang/Thread;"
            && method.name == "getId"
            && method.proto.descriptor_is("()J")
        {
            let invocation = invocation?;
            thread_get_id(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(1)));
        }

        // cite: libcore Thread.java#getStackTrace
        if method.class_descriptor == "Ljava/lang/Thread;"
            && method.name == "getStackTrace"
            && method
                .proto
                .descriptor_is("()[Ljava/lang/StackTraceElement;")
        {
            let array =
                self.allocate_filled_array("[Ljava/lang/StackTraceElement;".to_owned(), Vec::new());
            return Some(DexExecutionResult::Value(array));
        }

        // cite: libcore Thread.java#sleep
        if method.class_descriptor == "Ljava/lang/Thread;"
            && method.name == "sleep"
            && matches!(method.proto.descriptor().as_str(), "(J)V" | "(JI)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/os/Handler;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(Landroid/os/Looper;)V")
        {
            let invocation = invocation?;
            self.initialize_handler(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/os/Handler;"
            && method.name == "createAsync"
            && method
                .proto
                .descriptor_is("(Landroid/os/Looper;)Landroid/os/Handler;")
        {
            let invocation = invocation?;
            let value = self.create_async_handler(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/os/Handler;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.initialize_handler(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/os/Handler;"
            && method.name == "postDelayed"
            && method.proto.descriptor_is("(Ljava/lang/Runnable;J)Z")
        {
            let invocation = invocation?;
            self.post_delayed_handler_runnable(invocation)?;

            return Some(DexExecutionResult::Int(1));
        }

        if method.class_descriptor == "Landroid/os/Handler;"
            && method.name == "post"
            && method.proto.descriptor_is("(Ljava/lang/Runnable;)Z")
        {
            let invocation = invocation?;
            self.post_handler_runnable(invocation)?;

            return Some(DexExecutionResult::Int(1));
        }

        if method.class_descriptor == "Landroid/os/Handler;"
            && method.name == "postAtFrontOfQueue"
            && method.proto.descriptor_is("(Ljava/lang/Runnable;)Z")
        {
            let invocation = invocation?;
            self.post_handler_runnable_at_front(invocation)?;

            return Some(DexExecutionResult::Int(1));
        }

        if method.class_descriptor == "Landroid/os/Handler;"
            && method.name == "removeCallbacks"
            && method.proto.descriptor_is("(Ljava/lang/Runnable;)V")
        {
            let invocation = invocation?;
            self.remove_handler_callbacks(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/os/Handler;"
            && method.name == "obtainMessage"
            && method
                .proto
                .descriptor_is("(ILjava/lang/Object;)Landroid/os/Message;")
        {
            let invocation = invocation?;
            let value = self.handler_obtain_message(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/os/Handler;"
            && method.name == "getLooper"
            && method.proto.descriptor_is("()Landroid/os/Looper;")
        {
            let invocation = invocation?;
            let value = self.handler_looper(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        None
    }
}

impl DexProgram {
    fn initialize_thread(&self, invocation: &DexInvocation, descriptor: &str) -> Option<()> {
        match descriptor {
            "()V" if invocation.arguments.len() == 1 => Some(()),
            "(Ljava/lang/Runnable;)V" if invocation.arguments.len() == 2 => {
                self.thread_store_target(invocation)?;
                Some(())
            }
            "(Ljava/lang/String;)V" if invocation.arguments.len() == 2 => {
                self.thread_store_name(invocation)?;
                Some(())
            }
            "(Ljava/lang/Runnable;Ljava/lang/String;)V" if invocation.arguments.len() == 3 => {
                self.thread_store_target(invocation)?;
                self.thread_store_name_from_argument(invocation, 2)?;
                Some(())
            }
            _ => None,
        }
    }

    pub(in crate::dex_run) fn thread_start(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &thread_started_field(),
            DexValue::Int(1),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn thread_target(&self, thread: &DexValue) -> Option<DexValue> {
        if !dex_value_is_object_like(thread) {
            return None;
        }

        self.read_stored_instance_field(thread, &thread_target_field())
    }

    fn thread_set_daemon(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(value) = invocation.arguments[1] else {
            return None;
        };
        let daemon = if value == 0 { 0 } else { 1 };

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &thread_daemon_field(),
            DexValue::Int(daemon),
        )
        .then_some(())
    }

    fn thread_set_context_class_loader(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        match &invocation.arguments[1] {
            DexValue::Null => {}
            value if dex_value_is_object_like(value) => {}
            _ => return None,
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &thread_context_class_loader_field(),
            invocation.arguments[1].clone(),
        )
        .then_some(())
    }

    fn thread_set_name(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        self.thread_store_name(invocation)
    }

    fn thread_store_target(&self, invocation: &DexInvocation) -> Option<()> {
        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        match &invocation.arguments[1] {
            DexValue::Null => {}
            value if dex_value_is_object_like(value) => {}
            _ => return None,
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &thread_target_field(),
            invocation.arguments[1].clone(),
        )
        .then_some(())
    }

    fn thread_store_name(&self, invocation: &DexInvocation) -> Option<()> {
        self.thread_store_name_from_argument(invocation, 1)
    }

    fn thread_store_name_from_argument(
        &self,
        invocation: &DexInvocation,
        index: usize,
    ) -> Option<()> {
        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let name = invocation.arguments.get(index)?;

        match name {
            DexValue::String(_) => {}
            _ => return None,
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &thread_name_field(),
            name.clone(),
        )
        .then_some(())
    }

    fn instant_now(&self) -> DexValue {
        let epoch_millis = self.runtime.borrow().current_wall_clock_millis();

        self.instant_from_epoch_milli(epoch_millis)
    }

    fn instant_of_epoch_milli(&self, invocation: &DexInvocation) -> Option<DexValue> {
        let epoch_millis = single_long_argument(invocation)?;

        Some(self.instant_from_epoch_milli(epoch_millis))
    }

    fn instant_of_epoch_second(
        &self,
        invocation: &DexInvocation,
        has_nano_adjustment: bool,
    ) -> Option<DexValue> {
        let (epoch_second, nano_adjustment) = if has_nano_adjustment {
            math_two_longs(invocation)?
        } else {
            (single_long_argument(invocation)?, 0)
        };
        let carry_seconds = nano_adjustment.div_euclid(1_000_000_000);
        let nano = nano_adjustment.rem_euclid(1_000_000_000);
        let second = epoch_second.checked_add(carry_seconds)?;
        let nano = i32::try_from(nano).ok()?;

        Some(self.instant_from_parts(second, nano))
    }

    fn instant_from_epoch_milli(&self, epoch_millis: i64) -> DexValue {
        let second = epoch_millis.div_euclid(1000);
        let milli = epoch_millis.rem_euclid(1000);
        let nano = i32::try_from(milli.saturating_mul(1_000_000)).unwrap_or(0);

        self.instant_from_parts(second, nano)
    }

    fn instant_from_parts(&self, epoch_second: i64, nano: i32) -> DexValue {
        let instant = self.allocate_object("Ljava/time/Instant;".to_owned());
        self.write_stored_instance_field(
            &instant,
            &instant_seconds_field(),
            DexValue::Long(epoch_second),
        );
        self.write_stored_instance_field(&instant, &instant_nanos_field(), DexValue::Int(nano));

        instant
    }

    fn zone_id_of(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }
        let DexValue::String(id) = &invocation.arguments[0] else {
            return None;
        };

        Some(self.zone_id_from_string(id.clone().to_string()))
    }

    fn zone_id_system_default(&self) -> DexValue {
        self.zone_id_from_string(host_default_zone_id())
    }

    fn zone_id_from_string(&self, id: String) -> DexValue {
        let zone = self.allocate_object("Ljava/time/ZoneId;".to_owned());
        self.write_stored_instance_field(
            &zone,
            &zone_id_id_field(),
            DexValue::String(std::rc::Rc::from(id)),
        );

        zone
    }

    fn zone_id_get_id(&self, invocation: &DexInvocation) -> Option<String> {
        if invocation.arguments.len() != 1 {
            return None;
        }
        match self.read_stored_instance_field(&invocation.arguments[0], &zone_id_id_field())? {
            DexValue::String(id) => Some(id.to_string()),
            _ => None,
        }
    }

    fn zone_id_get_rules(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }
        let id = self.zone_id_get_id(invocation)?;
        let rules = self.allocate_object("Ljava/time/zone/ZoneRules;".to_owned());
        self.write_stored_instance_field(
            &rules,
            &zone_rules_zone_id_field(),
            DexValue::String(std::rc::Rc::from(id)),
        );

        Some(rules)
    }

    fn zone_rules_data(&self, invocation: &DexInvocation) -> Option<crate::tzdata::ZoneRulesData> {
        if invocation.arguments.is_empty() {
            return None;
        }
        let DexValue::String(zone_id) =
            self.read_stored_instance_field(&invocation.arguments[0], &zone_rules_zone_id_field())?
        else {
            return None;
        };

        crate::tzdata::zone_rules_for_id(&zone_id)
    }

    fn local_date_time_of_instant(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }
        let (epoch_second, nano) = self.instant_parts(&invocation.arguments[0])?;
        let DexValue::String(zone_id) =
            self.read_stored_instance_field(&invocation.arguments[1], &zone_id_id_field())?
        else {
            return None;
        };
        let rules = crate::tzdata::zone_rules_for_id(&zone_id)?;
        let offset = i64::from(rules.offset_seconds_at(epoch_second));
        let components = local_components_from_epoch(epoch_second, nano, offset)?;

        self.allocate_local_date_time(components)
    }

    fn instant_parts(&self, instant: &DexValue) -> Option<(i64, i32)> {
        let DexValue::Long(epoch_second) =
            self.read_stored_instance_field(instant, &instant_seconds_field())?
        else {
            return None;
        };
        let DexValue::Int(nano) =
            self.read_stored_instance_field(instant, &instant_nanos_field())?
        else {
            return None;
        };

        Some((epoch_second, nano))
    }

    fn allocate_local_date_time(&self, components: [i32; 7]) -> Option<DexValue> {
        let value = self.allocate_object("Ljava/time/LocalDateTime;".to_owned());
        for (getter, component) in [
            "getYear",
            "getMonthValue",
            "getDayOfMonth",
            "getHour",
            "getMinute",
            "getSecond",
            "getNano",
        ]
        .iter()
        .zip(components)
        {
            self.write_stored_instance_field(
                &value,
                &local_date_time_component_field(getter)?,
                DexValue::Int(component),
            );
        }

        Some(value)
    }

    fn instant_at_zone(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }
        let (epoch_second, nano) = self.instant_parts(&invocation.arguments[0])?;
        let DexValue::String(zone_id) =
            self.read_stored_instance_field(&invocation.arguments[1], &zone_id_id_field())?
        else {
            return None;
        };
        crate::tzdata::zone_rules_for_id(&zone_id)?;

        let value = self.allocate_object("Ljava/time/ZonedDateTime;".to_owned());
        self.write_stored_instance_field(
            &value,
            &zoned_date_time_seconds_field(),
            DexValue::Long(epoch_second),
        );
        self.write_stored_instance_field(
            &value,
            &zoned_date_time_nanos_field(),
            DexValue::Int(nano),
        );
        self.write_stored_instance_field(&value, &zone_id_id_field(), DexValue::String(zone_id));

        Some(value)
    }

    fn zoned_date_time_components(&self, invocation: &DexInvocation) -> Option<[i32; 7]> {
        let epoch_second = self.zoned_date_time_epoch_second(invocation)?;
        let DexValue::Int(nano) = self
            .read_stored_instance_field(&invocation.arguments[0], &zoned_date_time_nanos_field())?
        else {
            return None;
        };
        let DexValue::String(zone_id) =
            self.read_stored_instance_field(&invocation.arguments[0], &zone_id_id_field())?
        else {
            return None;
        };
        let rules = crate::tzdata::zone_rules_for_id(&zone_id)?;
        let offset = i64::from(rules.offset_seconds_at(epoch_second));

        local_components_from_epoch(epoch_second, nano, offset)
    }

    fn zoned_date_time_adjusted(
        &self,
        invocation: &DexInvocation,
        method_name: &str,
    ) -> Option<DexValue> {
        let amount = match invocation.arguments.get(1..) {
            Some([DexValue::Int(value)]) => i64::from(*value),
            Some([DexValue::Long(value), DexValue::Long(high)]) if value == high => *value,
            _ => return None,
        };
        let DexValue::String(zone_id) =
            self.read_stored_instance_field(&invocation.arguments[0], &zone_id_id_field())?
        else {
            return None;
        };
        let rules = crate::tzdata::zone_rules_for_id(&zone_id)?;
        let mut components = self.zoned_date_time_components(invocation)?;

        // cite: libcore LocalDate.java#withMonth, plusMonths, resolvePreviousValid
        match method_name {
            "withYear" => components[0] = i32::try_from(amount).ok()?,
            "withMonth" => components[1] = i32::try_from(amount).ok()?,
            "withDayOfMonth" => components[2] = i32::try_from(amount).ok()?,
            "withHour" => components[3] = i32::try_from(amount).ok()?,
            "withMinute" => components[4] = i32::try_from(amount).ok()?,
            "withSecond" => components[5] = i32::try_from(amount).ok()?,
            "plusMonths" | "minusMonths" | "plusYears" | "minusYears" => {
                let months = match method_name {
                    "plusMonths" => amount,
                    "minusMonths" => amount.checked_neg()?,
                    "plusYears" => amount.checked_mul(12)?,
                    _ => amount.checked_neg()?.checked_mul(12)?,
                };
                let total = i64::from(components[0])
                    .checked_mul(12)?
                    .checked_add(i64::from(components[1]) - 1)?
                    .checked_add(months)?;
                components[0] = i32::try_from(total.div_euclid(12)).ok()?;
                components[1] = i32::try_from(total.rem_euclid(12) + 1).ok()?;
            }
            "plusDays" | "minusDays" | "plusWeeks" | "minusWeeks" => {
                let days = match method_name {
                    "plusDays" => amount,
                    "minusDays" => amount.checked_neg()?,
                    "plusWeeks" => amount.checked_mul(7)?,
                    _ => amount.checked_neg()?.checked_mul(7)?,
                };
                let epoch_day =
                    epoch_day_from_civil_date(components[0], components[1], components[2])?
                        .checked_add(days)?;
                let (year, month, day) = civil_date_from_epoch_day(epoch_day)?;
                components[0] = year;
                components[1] = month;
                components[2] = day;
            }
            _ => return None,
        }
        components[2] = components[2].min(month_length(components[0], components[1]));

        let epoch_day = epoch_day_from_civil_date(components[0], components[1], components[2])?;
        let local_second = epoch_day.checked_mul(86_400)?.checked_add(i64::from(
            components[3] * 3600 + components[4] * 60 + components[5],
        ))?;
        let offset = i64::from(rules.offset_seconds_for_local(local_second));
        let epoch_second = local_second.checked_sub(offset)?;

        let value = self.allocate_object("Ljava/time/ZonedDateTime;".to_owned());
        self.write_stored_instance_field(
            &value,
            &zoned_date_time_seconds_field(),
            DexValue::Long(epoch_second),
        );
        self.write_stored_instance_field(
            &value,
            &zoned_date_time_nanos_field(),
            DexValue::Int(components[6]),
        );
        self.write_stored_instance_field(&value, &zone_id_id_field(), DexValue::String(zone_id));

        Some(value)
    }

    fn invoke_local_date_builtin(
        &self,
        method_name: &str,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        let invocation = invocation?;
        let (year, month, day) = local_date_parts(invocation.arguments.first()?)?;

        match method_name {
            // cite: libcore LocalDate.java#getYear, getMonthValue, getDayOfMonth
            "getYear" => Some(DexExecutionResult::Int(year)),
            "getMonthValue" => Some(DexExecutionResult::Int(month)),
            "getDayOfMonth" => Some(DexExecutionResult::Int(day)),
            // cite: libcore LocalDate.java#lengthOfMonth
            "lengthOfMonth" => Some(DexExecutionResult::Int(month_length(year, month))),
            // cite: libcore LocalDate.java#toEpochDay
            "toEpochDay" => Some(DexExecutionResult::Value(DexValue::Long(
                epoch_day_from_civil_date(year, month, day)?,
            ))),
            // cite: libcore LocalDate.java#getDayOfWeek
            "getDayOfWeek" => {
                let epoch_day = epoch_day_from_civil_date(year, month, day)?;
                let dow0 = usize::try_from((epoch_day + 3).rem_euclid(7)).ok()?;
                let name = DAY_OF_WEEK_NAMES.get(dow0)?;

                Some(DexExecutionResult::Value(DexValue::Reference(
                    std::rc::Rc::from(format!("Ljava/time/DayOfWeek;.{name}")),
                )))
            }
            // cite: libcore Month.java#of
            "getMonth" => {
                let name = MONTH_NAMES.get(usize::try_from(month).ok()?.checked_sub(1)?)?;

                Some(DexExecutionResult::Value(DexValue::Reference(
                    std::rc::Rc::from(format!("Ljava/time/Month;.{name}")),
                )))
            }
            "toString" => {
                let DexValue::Reference(reference) = invocation.arguments.first()? else {
                    return None;
                };

                Some(DexExecutionResult::Value(DexValue::String(
                    std::rc::Rc::from(reference.strip_prefix("Ljava/time/LocalDate;:")?.to_owned()),
                )))
            }
            // cite: libcore LocalDate.java#plusDays, plusMonths, resolvePreviousValid
            "plusDays" | "minusDays" | "plusWeeks" | "minusWeeks" | "plusMonths"
            | "minusMonths" | "plusYears" | "minusYears" | "withDayOfMonth" | "withMonth"
            | "withYear" => {
                let amount = match invocation.arguments.get(1..) {
                    Some([DexValue::Int(value)]) => i64::from(*value),
                    Some([DexValue::Long(value), DexValue::Long(high)]) if value == high => *value,
                    _ => return None,
                };
                let (year, month, day) =
                    adjusted_civil_date(year, month, day, method_name, amount)?;

                Some(DexExecutionResult::Value(local_date_reference(
                    year, month, day,
                )))
            }
            // cite: libcore ChronoLocalDate.java#isBefore, isAfter, isEqual, compareTo
            "isBefore" | "isAfter" | "isEqual" | "compareTo" | "equals" => {
                let other = local_date_parts(invocation.arguments.get(1)?)?;
                let this_day = epoch_day_from_civil_date(year, month, day)?;
                let other_day = epoch_day_from_civil_date(other.0, other.1, other.2)?;

                Some(match method_name {
                    "isBefore" => DexExecutionResult::Int(i32::from(this_day < other_day)),
                    "isAfter" => DexExecutionResult::Int(i32::from(this_day > other_day)),
                    "compareTo" => DexExecutionResult::Int(match this_day.cmp(&other_day) {
                        std::cmp::Ordering::Less => -1,
                        std::cmp::Ordering::Equal => 0,
                        std::cmp::Ordering::Greater => 1,
                    }),
                    _ => DexExecutionResult::Int(i32::from(this_day == other_day)),
                })
            }
            _ => None,
        }
    }

    // cite: libcore LocalDate.java#getDayOfWeek
    fn zoned_date_time_day_of_week(&self, invocation: &DexInvocation) -> Option<DexValue> {
        let epoch_second = self.zoned_date_time_epoch_second(invocation)?;
        let DexValue::String(zone_id) =
            self.read_stored_instance_field(&invocation.arguments[0], &zone_id_id_field())?
        else {
            return None;
        };
        let rules = crate::tzdata::zone_rules_for_id(&zone_id)?;
        let offset = i64::from(rules.offset_seconds_at(epoch_second));
        let local_epoch_day = epoch_second.checked_add(offset)?.div_euclid(86_400);
        let dow0 = usize::try_from((local_epoch_day + 3).rem_euclid(7)).ok()?;
        let name = DAY_OF_WEEK_NAMES.get(dow0)?;

        Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Ljava/time/DayOfWeek;.{name}"
        ))))
    }

    fn local_date_time_at_zone(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }
        let local = &invocation.arguments[0];
        let mut components = [0i32; 7];
        for (index, getter) in [
            "getYear",
            "getMonthValue",
            "getDayOfMonth",
            "getHour",
            "getMinute",
            "getSecond",
            "getNano",
        ]
        .iter()
        .enumerate()
        {
            let DexValue::Int(value) =
                self.read_stored_instance_field(local, &local_date_time_component_field(getter)?)?
            else {
                return None;
            };
            components[index] = value;
        }
        let DexValue::String(zone_id) =
            self.read_stored_instance_field(&invocation.arguments[1], &zone_id_id_field())?
        else {
            return None;
        };
        let rules = crate::tzdata::zone_rules_for_id(&zone_id)?;

        let [year, month, day, hour, minute, second, nano] = components;
        let epoch_day = epoch_day_from_civil_date(year, month, day)?;
        let local_second = epoch_day
            .checked_mul(86_400)?
            .checked_add(i64::from(hour * 3600 + minute * 60 + second))?;
        let offset = i64::from(rules.offset_seconds_for_local(local_second));
        let epoch_second = local_second.checked_sub(offset)?;

        let value = self.allocate_object("Ljava/time/ZonedDateTime;".to_owned());
        self.write_stored_instance_field(
            &value,
            &zoned_date_time_seconds_field(),
            DexValue::Long(epoch_second),
        );
        self.write_stored_instance_field(
            &value,
            &zoned_date_time_nanos_field(),
            DexValue::Int(nano),
        );
        self.write_stored_instance_field(&value, &zone_id_id_field(), DexValue::String(zone_id));

        Some(value)
    }

    fn zoned_date_time_to_instant(&self, invocation: &DexInvocation) -> Option<DexValue> {
        let epoch_second = self.zoned_date_time_epoch_second(invocation)?;
        let DexValue::Int(nano) = self
            .read_stored_instance_field(&invocation.arguments[0], &zoned_date_time_nanos_field())?
        else {
            return None;
        };

        Some(self.instant_from_parts(epoch_second, nano))
    }

    fn zoned_date_time_epoch_second(&self, invocation: &DexInvocation) -> Option<i64> {
        if invocation.arguments.is_empty() {
            return None;
        }
        match self.read_stored_instance_field(
            &invocation.arguments[0],
            &zoned_date_time_seconds_field(),
        )? {
            DexValue::Long(value) => Some(value),
            _ => None,
        }
    }

    fn instant_to_epoch_milli(&self, invocation: &DexInvocation) -> Option<i64> {
        if invocation.arguments.len() != 1 {
            return None;
        }
        let second = self.instant_epoch_second(invocation)?;
        let nano = self.instant_nano(invocation)?;
        let millis_from_second = second.checked_mul(1000)?;
        let millis_from_nano = i64::from(nano / 1_000_000);

        millis_from_second.checked_add(millis_from_nano)
    }

    fn instant_epoch_second(&self, invocation: &DexInvocation) -> Option<i64> {
        if invocation.arguments.len() != 1 {
            return None;
        }
        match self.read_stored_instance_field(&invocation.arguments[0], &instant_seconds_field())? {
            DexValue::Long(value) => Some(value),
            _ => None,
        }
    }

    fn instant_nano(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }
        match self.read_stored_instance_field(&invocation.arguments[0], &instant_nanos_field())? {
            DexValue::Int(value) => Some(value),
            _ => None,
        }
    }

    fn initialize_throwable_message(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let message = self.throwable_message_argument(&invocation.arguments[1])?;

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &throwable_detail_message_field(),
            message,
        )
        .then_some(())
    }

    fn initialize_throwable_message_and_cause(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let message = self.throwable_message_argument(&invocation.arguments[1])?;
        match &invocation.arguments[2] {
            DexValue::Object(_) | DexValue::Reference(_) | DexValue::Null | DexValue::Int(0) => {}
            _ => return None,
        }

        let cause = match &invocation.arguments[2] {
            DexValue::Int(0) => DexValue::Null,
            value => value.clone(),
        };

        if !self.write_stored_instance_field(
            &invocation.arguments[0],
            &throwable_detail_message_field(),
            message,
        ) {
            return None;
        }

        self.write_stored_instance_field(&invocation.arguments[0], &throwable_cause_field(), cause)
            .then_some(())
    }

    fn initialize_throwable_cause(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        match &invocation.arguments[1] {
            DexValue::Object(_) | DexValue::Reference(_) | DexValue::Null | DexValue::Int(0) => {}
            _ => return None,
        }

        let cause = match &invocation.arguments[1] {
            DexValue::Int(0) => DexValue::Null,
            value => value.clone(),
        };

        self.write_stored_instance_field(&invocation.arguments[0], &throwable_cause_field(), cause)
            .then_some(())
    }

    fn throwable_message_argument(&self, value: &DexValue) -> Option<DexValue> {
        match value {
            DexValue::String(_) => Some(value.clone()),
            DexValue::Null | DexValue::Int(0) => Some(DexValue::Null),
            DexValue::Object(_) | DexValue::Reference(_)
                if self.value_matches_type(value, "Ljava/lang/String;") =>
            {
                Some(value.clone())
            }
            DexValue::Object(_)
            | DexValue::Reference(_)
            | DexValue::Uninitialized
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_) => None,
        }
    }

    fn throwable_print_stack_trace(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        // cite: libcore Throwable.java#printStackTrace, toString
        let mut current = invocation.arguments[0].clone();
        let mut printed = 0;
        while dex_value_is_object_like(&current) {
            let class_descriptor = match &current {
                DexValue::Object(object) => object.class_descriptor.clone(),
                DexValue::Reference(reference) => {
                    std::rc::Rc::from(reference_descriptor(reference)?.to_owned())
                }
                _ => return None,
            };
            let class_name = descriptor_to_java_name(&class_descriptor)?;
            let header = match self
                .read_stored_instance_field(&current, &throwable_detail_message_field())
            {
                Some(DexValue::String(message)) => format!("{class_name}: {message}"),
                _ => class_name,
            };
            if printed == 0 {
                eprintln!("{header}");
            } else {
                eprintln!("Caused by: {header}");
            }

            printed += 1;
            if printed > 8 {
                break;
            }
            current = self
                .read_stored_instance_field(&current, &throwable_cause_field())
                .unwrap_or(DexValue::Null);
        }

        Some(())
    }

    fn throwable_init_cause(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        match &invocation.arguments[1] {
            DexValue::Object(_) | DexValue::Reference(_) | DexValue::Null | DexValue::Int(0) => {}
            _ => return None,
        }

        let cause = match &invocation.arguments[1] {
            DexValue::Int(0) => DexValue::Null,
            value => value.clone(),
        };
        let throwable = invocation.arguments[0].clone();

        self.write_stored_instance_field(&throwable, &throwable_cause_field(), cause)
            .then_some(throwable)
    }

    fn throwable_add_suppressed(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }
        if !dex_value_is_object_like(&invocation.arguments[1]) {
            return None;
        }

        let mut suppressed = match self
            .read_stored_instance_field(&invocation.arguments[0], &throwable_suppressed_field())
        {
            Some(array) => {
                let array_key = map_identity_key(&array);
                self.runtime
                    .borrow()
                    .array_entries
                    .get(&array_key)
                    .cloned()
                    .unwrap_or_default()
            }
            None => Vec::new(),
        };
        suppressed.push(invocation.arguments[1].clone());

        let array = self.allocate_filled_array("[Ljava/lang/Throwable;".to_owned(), suppressed);

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &throwable_suppressed_field(),
            array,
        )
        .then_some(())
    }

    fn throwable_get_suppressed(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let suppressed = match self
            .read_stored_instance_field(&invocation.arguments[0], &throwable_suppressed_field())
        {
            Some(array) => {
                let array_key = map_identity_key(&array);
                self.runtime
                    .borrow()
                    .array_entries
                    .get(&array_key)
                    .cloned()
                    .unwrap_or_default()
            }
            None => Vec::new(),
        };

        Some(self.allocate_filled_array("[Ljava/lang/Throwable;".to_owned(), suppressed))
    }
}

fn is_supported_throwable_class(class_descriptor: &str) -> bool {
    matches!(
        class_descriptor,
        "Ljava/lang/Throwable;"
            | "Ljava/lang/Exception;"
            | "Ljava/lang/RuntimeException;"
            | "Ljava/lang/Error;"
            | "Ljava/lang/IllegalArgumentException;"
            | "Ljava/lang/IllegalStateException;"
            // cite: frameworks-base SQLiteException.java#SQLiteException
            | "Landroid/database/sqlite/SQLiteException;"
            // cite: frameworks-base SQLException.java#SQLException
            | "Landroid/database/SQLException;"
            | "Ljava/lang/NullPointerException;"
            | "Ljava/lang/UnsupportedOperationException;"
            | "Ljava/lang/IndexOutOfBoundsException;"
            | "Ljava/lang/ClassCastException;"
            | "Ljava/util/NoSuchElementException;"
            | "Ljava/util/ConcurrentModificationException;"
            | "Ljava/util/concurrent/CancellationException;"
    )
}

fn zone_id_id_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/time/ZoneId;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "id".to_owned(),
    }
}

fn host_default_zone_id() -> String {
    if let Ok(tz) = std::env::var("TZ") {
        if !tz.is_empty() {
            return tz;
        }
    }
    if let Ok(target) = std::fs::read_link("/etc/localtime") {
        let target = target.to_string_lossy();
        if let Some((_, zone)) = target.split_once("zoneinfo/") {
            return zone.to_owned();
        }
    }

    "UTC".to_owned()
}

fn zone_rules_zone_id_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/time/zone/ZoneRules;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "zoneId".to_owned(),
    }
}

fn local_date_time_component_field(getter_name: &str) -> Option<DexFieldId> {
    let field_name = match getter_name {
        "getYear" => "year",
        "getMonthValue" => "month",
        "getDayOfMonth" => "day",
        "getHour" => "hour",
        "getMinute" => "minute",
        "getSecond" => "second",
        "getNano" => "nano",
        _ => return None,
    };

    Some(DexFieldId {
        class_descriptor: "Ljava/time/LocalDateTime;".to_owned(),
        type_descriptor: "I".to_owned(),
        name: field_name.to_owned(),
    })
}

fn zoned_component_index(method_name: &str) -> Option<usize> {
    match method_name {
        "getYear" => Some(0),
        "getMonthValue" => Some(1),
        "getDayOfMonth" => Some(2),
        "getHour" => Some(3),
        "getMinute" => Some(4),
        "getSecond" => Some(5),
        "getNano" => Some(6),
        _ => None,
    }
}

// cite: libcore LocalDateTime.java#ofEpochSecond
fn local_components_from_epoch(epoch_second: i64, nano: i32, offset: i64) -> Option<[i32; 7]> {
    let local_second = epoch_second.checked_add(offset)?;
    let local_epoch_day = local_second.div_euclid(86_400);
    let secs_of_day = local_second.rem_euclid(86_400);
    let (year, month, day) = civil_date_from_epoch_day(local_epoch_day)?;

    Some([
        year,
        month,
        day,
        i32::try_from(secs_of_day / 3600).ok()?,
        i32::try_from(secs_of_day / 60 % 60).ok()?,
        i32::try_from(secs_of_day % 60).ok()?,
        nano,
    ])
}

fn zone_offset_total_seconds(value: &DexValue) -> Option<i32> {
    let DexValue::Reference(reference) = value else {
        return None;
    };

    reference
        .strip_prefix("Ljava/time/ZoneOffset;:")?
        .parse()
        .ok()
}

fn local_time_parts(value: &DexValue) -> Option<(i32, i32)> {
    let DexValue::Reference(reference) = value else {
        return None;
    };
    let payload = reference.strip_prefix("Ljava/time/LocalTime;:")?;
    let (second_of_day, nano) = payload.split_once(':')?;

    Some((second_of_day.parse().ok()?, nano.parse().ok()?))
}

fn local_date_reference(year: i32, month: i32, day: i32) -> DexValue {
    DexValue::Reference(std::rc::Rc::from(format!(
        "Ljava/time/LocalDate;:{year:04}-{month:02}-{day:02}"
    )))
}

fn local_date_parts(value: &DexValue) -> Option<(i32, i32, i32)> {
    let DexValue::Reference(reference) = value else {
        return None;
    };
    let payload = reference.strip_prefix("Ljava/time/LocalDate;:")?;
    let mut parts = payload.splitn(3, '-');
    let year = parts.next()?.parse().ok()?;
    let month = parts.next()?.parse().ok()?;
    let day = parts.next()?.parse().ok()?;

    Some((year, month, day))
}

// cite: libcore LocalDate.java#plusDays, plusMonths, resolvePreviousValid
fn adjusted_civil_date(
    year: i32,
    month: i32,
    day: i32,
    method_name: &str,
    amount: i64,
) -> Option<(i32, i32, i32)> {
    match method_name {
        "withYear" => {
            let year = i32::try_from(amount).ok()?;
            Some((year, month, day.min(month_length(year, month))))
        }
        "withMonth" => {
            let month = i32::try_from(amount).ok()?;
            Some((year, month, day.min(month_length(year, month))))
        }
        "withDayOfMonth" => Some((year, month, i32::try_from(amount).ok()?)),
        "plusMonths" | "minusMonths" | "plusYears" | "minusYears" => {
            let months = match method_name {
                "plusMonths" => amount,
                "minusMonths" => amount.checked_neg()?,
                "plusYears" => amount.checked_mul(12)?,
                _ => amount.checked_neg()?.checked_mul(12)?,
            };
            let total = i64::from(year)
                .checked_mul(12)?
                .checked_add(i64::from(month) - 1)?
                .checked_add(months)?;
            let year = i32::try_from(total.div_euclid(12)).ok()?;
            let month = i32::try_from(total.rem_euclid(12) + 1).ok()?;
            Some((year, month, day.min(month_length(year, month))))
        }
        "plusDays" | "minusDays" | "plusWeeks" | "minusWeeks" => {
            let days = match method_name {
                "plusDays" => amount,
                "minusDays" => amount.checked_neg()?,
                "plusWeeks" => amount.checked_mul(7)?,
                _ => amount.checked_neg()?.checked_mul(7)?,
            };
            let epoch_day = epoch_day_from_civil_date(year, month, day)?.checked_add(days)?;
            civil_date_from_epoch_day(epoch_day)
        }
        _ => None,
    }
}

// cite: libcore LocalDate.java#resolvePreviousValid
fn month_length(year: i32, month: i32) -> i32 {
    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    match month {
        2 => {
            if leap_year {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

// cite: libcore LocalDate.java#ofEpochDay
fn civil_date_from_epoch_day(epoch_day: i64) -> Option<(i32, i32, i32)> {
    let mut zero_day = epoch_day.checked_add(719_528)? - 60;
    let mut adjust = 0;
    if zero_day < 0 {
        let adjust_cycles = (zero_day + 1) / 146_097 - 1;
        adjust = adjust_cycles * 400;
        zero_day += -adjust_cycles * 146_097;
    }
    let mut year_est = (400 * zero_day + 591) / 146_097;
    let mut doy_est = zero_day - (365 * year_est + year_est / 4 - year_est / 100 + year_est / 400);
    if doy_est < 0 {
        year_est -= 1;
        doy_est = zero_day - (365 * year_est + year_est / 4 - year_est / 100 + year_est / 400);
    }
    year_est += adjust;
    let march_doy0 = doy_est;
    let march_month0 = (march_doy0 * 5 + 2) / 153;
    let month = (march_month0 + 2) % 12 + 1;
    let day = march_doy0 - (march_month0 * 306 + 5) / 10 + 1;
    year_est += march_month0 / 10;

    Some((
        i32::try_from(year_est).ok()?,
        i32::try_from(month).ok()?,
        i32::try_from(day).ok()?,
    ))
}

const DAY_OF_WEEK_NAMES: [&str; 7] = [
    "MONDAY",
    "TUESDAY",
    "WEDNESDAY",
    "THURSDAY",
    "FRIDAY",
    "SATURDAY",
    "SUNDAY",
];

// cite: libcore WeekFields.java#of
fn week_fields_of_locale(invocation: &DexInvocation) -> Option<DexValue> {
    if invocation.arguments.len() != 1 {
        return None;
    }
    let country = match &invocation.arguments[0] {
        DexValue::Reference(reference) => reference
            .strip_prefix("Ljava/util/Locale;:")
            .and_then(|tag| tag.rsplit(['-', '_']).next())
            .unwrap_or("US")
            .to_ascii_uppercase(),
        DexValue::Object(_) => "US".to_owned(),
        _ => return None,
    };

    let sunday_first = matches!(
        country.as_str(),
        "US" | "CA"
            | "JP"
            | "BR"
            | "MX"
            | "IN"
            | "KR"
            | "TW"
            | "PH"
            | "CO"
            | "PE"
            | "VE"
            | "ZA"
            | "IL"
            | "HK"
            | "TH"
            | "SG"
            | "PK"
            | "BD"
            | "NP"
            | "ID"
            | "KE"
            | "NI"
            | "PA"
            | "PR"
            | "GT"
            | "HN"
            | "SV"
            | "DO"
            | "BO"
            | "EC"
            | "PY"
    );
    let minimal_days = if matches!(
        country.as_str(),
        "AT" | "BE"
            | "BG"
            | "CH"
            | "CZ"
            | "DE"
            | "DK"
            | "EE"
            | "ES"
            | "FI"
            | "FR"
            | "GB"
            | "GR"
            | "HR"
            | "HU"
            | "IE"
            | "IS"
            | "IT"
            | "LT"
            | "LU"
            | "LV"
            | "NL"
            | "NO"
            | "PL"
            | "PT"
            | "RO"
            | "SE"
            | "SI"
            | "SK"
    ) {
        4
    } else {
        1
    };
    let first_day = if sunday_first { "SUNDAY" } else { "MONDAY" };

    Some(DexValue::Reference(std::rc::Rc::from(format!(
        "Ljava/time/temporal/WeekFields;:{first_day}:{minimal_days}"
    ))))
}

fn week_fields_parts(invocation: &DexInvocation) -> Option<(String, i32)> {
    if invocation.arguments.is_empty() {
        return None;
    }
    let DexValue::Reference(reference) = &invocation.arguments[0] else {
        return None;
    };
    let payload = reference.strip_prefix("Ljava/time/temporal/WeekFields;:")?;
    let (first_day, min_days) = payload.split_once(':')?;

    Some((first_day.to_owned(), min_days.parse().ok()?))
}

// cite: libcore DayOfWeek.java#getValue
fn day_of_week_value(invocation: &DexInvocation) -> Option<i32> {
    if invocation.arguments.is_empty() {
        return None;
    }
    let DexValue::Reference(reference) = &invocation.arguments[0] else {
        return None;
    };
    let name = reference.strip_prefix("Ljava/time/DayOfWeek;.")?;
    let index = DAY_OF_WEEK_NAMES.iter().position(|&entry| entry == name)?;

    i32::try_from(index + 1).ok()
}

const MONTH_NAMES: [&str; 12] = [
    "JANUARY",
    "FEBRUARY",
    "MARCH",
    "APRIL",
    "MAY",
    "JUNE",
    "JULY",
    "AUGUST",
    "SEPTEMBER",
    "OCTOBER",
    "NOVEMBER",
    "DECEMBER",
];

fn text_style_name(value: &DexValue) -> Option<String> {
    let DexValue::Reference(reference) = value else {
        return None;
    };

    Some(
        reference
            .strip_prefix("Ljava/time/format/TextStyle;.")?
            .to_owned(),
    )
}

fn english_style_name(full: &str, style: &str) -> String {
    let mut lowercase = full.to_ascii_lowercase();
    let mut chars = lowercase.chars();
    let first = chars.next().unwrap_or_default().to_ascii_uppercase();
    lowercase = format!("{first}{}", chars.as_str());
    match style {
        "SHORT" | "SHORT_STANDALONE" => lowercase.chars().take(3).collect(),
        "NARROW" | "NARROW_STANDALONE" => lowercase.chars().take(1).collect(),
        _ => lowercase,
    }
}

// cite: libcore DayOfWeek.java#getDisplayName
fn day_of_week_display_name(invocation: &DexInvocation) -> Option<String> {
    if invocation.arguments.len() != 3 {
        return None;
    }
    let DexValue::Reference(reference) = &invocation.arguments[0] else {
        return None;
    };
    let name = reference.strip_prefix("Ljava/time/DayOfWeek;.")?;
    let style = text_style_name(&invocation.arguments[1])?;

    Some(english_style_name(name, &style))
}

// cite: libcore Month.java#getValue
fn month_value(invocation: &DexInvocation) -> Option<i32> {
    if invocation.arguments.is_empty() {
        return None;
    }
    let DexValue::Reference(reference) = &invocation.arguments[0] else {
        return None;
    };
    let name = reference.strip_prefix("Ljava/time/Month;.")?;
    let index = MONTH_NAMES.iter().position(|&entry| entry == name)?;

    i32::try_from(index + 1).ok()
}

// cite: libcore Month.java#getDisplayName
fn month_display_name(invocation: &DexInvocation) -> Option<String> {
    if invocation.arguments.len() != 3 {
        return None;
    }
    let DexValue::Reference(reference) = &invocation.arguments[0] else {
        return None;
    };
    let name = reference.strip_prefix("Ljava/time/Month;.")?;
    let style = text_style_name(&invocation.arguments[1])?;

    Some(english_style_name(name, &style))
}

fn zoned_date_time_seconds_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/time/ZonedDateTime;".to_owned(),
        type_descriptor: "J".to_owned(),
        name: "seconds".to_owned(),
    }
}

fn zoned_date_time_nanos_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/time/ZonedDateTime;".to_owned(),
        type_descriptor: "I".to_owned(),
        name: "nanos".to_owned(),
    }
}

// cite: libcore LocalDate.java#toEpochDay
fn epoch_day_from_civil_date(year: i32, month: i32, day: i32) -> Option<i64> {
    let y = i64::from(year);
    let m = i64::from(month);
    let mut total = 365 * y;
    if y >= 0 {
        total += (y + 3) / 4 - (y + 99) / 100 + (y + 399) / 400;
    } else {
        total -= y / -4 - y / -100 + y / -400;
    }
    total += (367 * m - 362) / 12;
    total += i64::from(day) - 1;
    if m > 2 {
        total -= 1;
        let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        if !leap_year {
            total -= 1;
        }
    }

    total.checked_sub(719_528)
}

fn instant_seconds_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/time/Instant;".to_owned(),
        type_descriptor: "J".to_owned(),
        name: "seconds".to_owned(),
    }
}

fn instant_nanos_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/time/Instant;".to_owned(),
        type_descriptor: "I".to_owned(),
        name: "nanos".to_owned(),
    }
}

fn throwable_detail_message_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Throwable;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "detailMessage".to_owned(),
    }
}

fn throwable_cause_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Throwable;".to_owned(),
        type_descriptor: "Ljava/lang/Throwable;".to_owned(),
        name: "cause".to_owned(),
    }
}

fn throwable_suppressed_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Throwable;".to_owned(),
        type_descriptor: "[Ljava/lang/Throwable;".to_owned(),
        name: "suppressedExceptions".to_owned(),
    }
}

fn object_notify_all(invocation: &DexInvocation) -> Option<()> {
    if invocation.arguments.len() != 1 || !dex_value_is_object_like(&invocation.arguments[0]) {
        return None;
    }

    Some(())
}

fn thread_reference(invocation: &DexInvocation) -> Option<()> {
    (invocation.arguments.len() == 1 && dex_value_is_object_like(&invocation.arguments[0]))
        .then_some(())
}

fn thread_uncaught_exception_handler() -> DexValue {
    DexValue::Reference(std::rc::Rc::from(
        "Ljava/lang/Thread$UncaughtExceptionHandler;:bootstrap",
    ))
}

fn thread_uncaught_exception(invocation: &DexInvocation) -> Option<()> {
    if invocation.arguments.len() != 3 || !dex_value_is_object_like(&invocation.arguments[0]) {
        return None;
    }
    if !dex_value_is_object_like(&invocation.arguments[1])
        || !dex_value_is_object_like(&invocation.arguments[2])
    {
        return None;
    }

    Some(())
}

fn thread_daemon_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Thread;".to_owned(),
        type_descriptor: "Z".to_owned(),
        name: "daemon".to_owned(),
    }
}

fn thread_started_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Thread;".to_owned(),
        type_descriptor: "Z".to_owned(),
        name: "started".to_owned(),
    }
}

fn thread_target_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Thread;".to_owned(),
        type_descriptor: "Ljava/lang/Runnable;".to_owned(),
        name: "target".to_owned(),
    }
}

fn thread_context_class_loader_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Thread;".to_owned(),
        type_descriptor: "Ljava/lang/ClassLoader;".to_owned(),
        name: "contextClassLoader".to_owned(),
    }
}

fn thread_name_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Thread;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "name".to_owned(),
    }
}
