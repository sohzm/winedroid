use super::super::*;

impl DexProgram {
    pub(super) fn invoke_java_date_builtin(
        &self,
        method: &DexMethodId,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        let descriptor = method.proto.descriptor();

        if method.class_descriptor == "Ljava/util/Date;" {
            match (method.name.as_str(), descriptor.as_str()) {
                ("<init>", "()V") => {
                    let invocation = invocation?;
                    let millis = self.runtime.borrow().current_wall_clock_millis();
                    self.write_date_millis(invocation.arguments.first()?, millis)?;
                    return Some(DexExecutionResult::Void);
                }
                ("<init>", "(J)V") => {
                    let invocation = invocation?;
                    let DexValue::Long(millis) = invocation.arguments.get(1)? else {
                        return None;
                    };
                    self.write_date_millis(invocation.arguments.first()?, *millis)?;
                    return Some(DexExecutionResult::Void);
                }
                ("getTime", "()J") => {
                    let invocation = invocation?;
                    let millis = self.read_date_millis(invocation.arguments.first()?)?;
                    return Some(DexExecutionResult::Value(DexValue::Long(millis)));
                }
                ("setTime", "(J)V") => {
                    let invocation = invocation?;
                    let DexValue::Long(millis) = invocation.arguments.get(1)? else {
                        return None;
                    };
                    self.write_date_millis(invocation.arguments.first()?, *millis)?;
                    return Some(DexExecutionResult::Void);
                }
                _ => {}
            }
        }

        // cite: libcore TimeZone.java#getTimeZone, getDefault, getID, setID, getRawOffset, getOffset, clone
        if method.class_descriptor == "Ljava/util/TimeZone;" {
            match (method.name.as_str(), descriptor.as_str()) {
                ("getTimeZone", "(Ljava/lang/String;)Ljava/util/TimeZone;") => {
                    let invocation = invocation?;
                    let Some(id) = self.string_text(invocation.arguments.first()?) else {
                        return Some(DexExecutionResult::Throw(
                            self.null_pointer_exception("id == null"),
                        ));
                    };
                    let tz = self.new_time_zone(&id);
                    return Some(DexExecutionResult::Value(tz));
                }
                ("getDefault", "()Ljava/util/TimeZone;") => {
                    return Some(DexExecutionResult::Value(self.new_time_zone("UTC")));
                }
                ("getID", "()Ljava/lang/String;") => {
                    let invocation = invocation?;
                    let id = self
                        .read_time_zone_id(invocation.arguments.first()?)
                        .unwrap_or_else(|| "UTC".to_owned());
                    return Some(DexExecutionResult::Value(DexValue::String(
                        std::rc::Rc::from(id),
                    )));
                }
                ("setID", "(Ljava/lang/String;)V") => {
                    let invocation = invocation?;
                    let id = self.string_text(invocation.arguments.get(1)?)?;
                    self.write_time_zone_id(invocation.arguments.first()?, id)?;
                    return Some(DexExecutionResult::Void);
                }
                ("getRawOffset", "()I") | ("getOffset", "(J)I") => {
                    return Some(DexExecutionResult::Int(0));
                }
                ("clone", "()Ljava/lang/Object;") => {
                    let invocation = invocation?;
                    let id = self
                        .read_time_zone_id(invocation.arguments.first()?)
                        .unwrap_or_else(|| "UTC".to_owned());
                    return Some(DexExecutionResult::Value(self.new_time_zone(&id)));
                }
                _ => {}
            }
        }

        let is_date_format = matches!(
            method.class_descriptor.as_str(),
            "Ljava/text/DateFormat;" | "Ljava/text/SimpleDateFormat;"
        );
        if !is_date_format {
            return None;
        }

        match (method.name.as_str(), descriptor.as_str()) {
            ("<init>", "(Ljava/lang/String;)V")
            | ("<init>", "(Ljava/lang/String;Ljava/util/Locale;)V") => {
                let invocation = invocation?;
                let DexValue::String(pattern) = invocation.arguments.get(1)? else {
                    return None;
                };
                self.write_date_format_pattern(
                    invocation.arguments.first()?,
                    pattern.clone().to_string(),
                )?;
                Some(DexExecutionResult::Void)
            }
            ("applyPattern", "(Ljava/lang/String;)V") => {
                let invocation = invocation?;
                let DexValue::String(pattern) = invocation.arguments.get(1)? else {
                    return None;
                };
                self.write_date_format_pattern(
                    invocation.arguments.first()?,
                    pattern.clone().to_string(),
                )?;
                Some(DexExecutionResult::Void)
            }
            ("toPattern", "()Ljava/lang/String;") => {
                let invocation = invocation?;
                let pattern = self.read_date_format_pattern(invocation.arguments.first()?)?;
                Some(DexExecutionResult::Value(DexValue::String(
                    std::rc::Rc::from(pattern),
                )))
            }
            ("setTimeZone", "(Ljava/util/TimeZone;)V")
            | ("setLenient", "(Z)V")
            | ("setCalendar", "(Ljava/util/Calendar;)V")
            | ("setNumberFormat", "(Ljava/text/NumberFormat;)V") => Some(DexExecutionResult::Void),
            ("getInstance", "()Ljava/text/DateFormat;") => Some(DexExecutionResult::Value(
                self.new_date_format("M/d/yy, h:mm a"),
            )),
            ("getDateInstance", _) if descriptor.ends_with(")Ljava/text/DateFormat;") => {
                let style = style_argument(invocation, 0);
                Some(DexExecutionResult::Value(
                    self.new_date_format(date_style_pattern(style)),
                ))
            }
            ("getTimeInstance", _) if descriptor.ends_with(")Ljava/text/DateFormat;") => {
                let style = style_argument(invocation, 0);
                Some(DexExecutionResult::Value(
                    self.new_date_format(time_style_pattern(style)),
                ))
            }
            ("getDateTimeInstance", _) if descriptor.ends_with(")Ljava/text/DateFormat;") => {
                let date_style = style_argument(invocation, 0);
                let time_style = style_argument(invocation, 1);
                let pattern = format!(
                    "{}, {}",
                    date_style_pattern(date_style),
                    time_style_pattern(time_style)
                );
                Some(DexExecutionResult::Value(self.new_date_format(&pattern)))
            }
            ("format", "(Ljava/util/Date;)Ljava/lang/String;")
            | ("format", "(Ljava/lang/Object;)Ljava/lang/String;") => {
                let invocation = invocation?;
                let pattern = self.read_date_format_pattern(invocation.arguments.first()?)?;
                let millis = self.millis_from_formattable(invocation.arguments.get(1)?)?;
                Some(DexExecutionResult::Value(DexValue::String(
                    std::rc::Rc::from(format_date_pattern(&pattern, millis)),
                )))
            }
            ("parse", "(Ljava/lang/String;)Ljava/util/Date;") => {
                let invocation = invocation?;
                let pattern = self.read_date_format_pattern(invocation.arguments.first()?)?;
                let DexValue::String(text) = invocation.arguments.get(1)? else {
                    return None;
                };
                match parse_date_pattern(&pattern, text) {
                    Some(millis) => {
                        let date = self.allocate_object("Ljava/util/Date;".to_owned());
                        self.write_date_millis(&date, millis)?;
                        Some(DexExecutionResult::Value(date))
                    }
                    None => Some(DexExecutionResult::Throw(
                        self.parse_exception(&format!("Unparseable date: \"{text}\"")),
                    )),
                }
            }
            _ => None,
        }
    }

    fn new_date_format(&self, pattern: &str) -> DexValue {
        let formatter = self.allocate_object("Ljava/text/SimpleDateFormat;".to_owned());
        self.write_stored_instance_field(
            &formatter,
            &date_format_pattern_field(),
            DexValue::String(std::rc::Rc::from(pattern.to_owned())),
        );
        formatter
    }

    fn write_date_format_pattern(&self, formatter: &DexValue, pattern: String) -> Option<()> {
        self.write_stored_instance_field(
            formatter,
            &date_format_pattern_field(),
            DexValue::String(std::rc::Rc::from(pattern)),
        )
        .then_some(())
    }

    fn read_date_format_pattern(&self, formatter: &DexValue) -> Option<String> {
        match self.read_stored_instance_field(formatter, &date_format_pattern_field())? {
            DexValue::String(pattern) => Some(pattern.to_string()),
            _ => None,
        }
    }

    fn write_date_millis(&self, date: &DexValue, millis: i64) -> Option<()> {
        self.write_stored_instance_field(date, &date_millis_field(), DexValue::Long(millis))
            .then_some(())
    }

    fn read_date_millis(&self, date: &DexValue) -> Option<i64> {
        match self.read_stored_instance_field(date, &date_millis_field())? {
            DexValue::Long(millis) => Some(millis),
            _ => None,
        }
    }

    fn millis_from_formattable(&self, value: &DexValue) -> Option<i64> {
        match value {
            DexValue::Long(millis) => Some(*millis),
            DexValue::Int(millis) => Some(*millis as i64),
            _ => self.read_date_millis(value),
        }
    }

    fn parse_exception(&self, message: &str) -> DexValue {
        let exception = self.allocate_object("Ljava/text/ParseException;".to_owned());
        self.write_stored_instance_field(
            &exception,
            &throwable_detail_message_field(),
            DexValue::String(std::rc::Rc::from(message.to_owned())),
        );
        exception
    }

    fn null_pointer_exception(&self, message: &str) -> DexValue {
        let exception = self.allocate_object("Ljava/lang/NullPointerException;".to_owned());
        self.write_stored_instance_field(
            &exception,
            &throwable_detail_message_field(),
            DexValue::String(std::rc::Rc::from(message.to_owned())),
        );
        exception
    }

    fn new_time_zone(&self, id: &str) -> DexValue {
        let tz = self.allocate_object("Ljava/util/SimpleTimeZone;".to_owned());
        self.write_stored_instance_field(
            &tz,
            &time_zone_id_field(),
            DexValue::String(std::rc::Rc::from(id.to_owned())),
        );
        tz
    }

    fn write_time_zone_id(&self, tz: &DexValue, id: String) -> Option<()> {
        self.write_stored_instance_field(
            tz,
            &time_zone_id_field(),
            DexValue::String(std::rc::Rc::from(id)),
        )
        .then_some(())
    }

    fn read_time_zone_id(&self, tz: &DexValue) -> Option<String> {
        match self.read_stored_instance_field(tz, &time_zone_id_field())? {
            DexValue::String(id) => Some(id.to_string()),
            _ => None,
        }
    }
}

fn time_zone_id_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/util/TimeZone;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "ID".to_owned(),
    }
}

fn date_format_pattern_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/text/DateFormat;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "pattern".to_owned(),
    }
}

fn date_millis_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/util/Date;".to_owned(),
        type_descriptor: "J".to_owned(),
        name: "fastTime".to_owned(),
    }
}

fn throwable_detail_message_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Throwable;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "detailMessage".to_owned(),
    }
}

fn style_argument(invocation: Option<&DexInvocation>, index: usize) -> i32 {
    match invocation.and_then(|invocation| invocation.arguments.get(index + 1)) {
        Some(DexValue::Int(style)) => *style,
        _ => DATE_FORMAT_MEDIUM,
    }
}

const DATE_FORMAT_FULL: i32 = 0;
const DATE_FORMAT_LONG: i32 = 1;
const DATE_FORMAT_MEDIUM: i32 = 2;
const DATE_FORMAT_SHORT: i32 = 3;

fn date_style_pattern(style: i32) -> &'static str {
    match style {
        DATE_FORMAT_FULL => "EEEE, MMMM d, yyyy",
        DATE_FORMAT_LONG => "MMMM d, yyyy",
        DATE_FORMAT_SHORT => "M/d/yy",
        _ => "MMM d, yyyy",
    }
}

fn time_style_pattern(style: i32) -> &'static str {
    match style {
        DATE_FORMAT_FULL | DATE_FORMAT_LONG => "h:mm:ss a",
        DATE_FORMAT_SHORT | DATE_FORMAT_MEDIUM => "h:mm:ss a",
        _ => "h:mm:ss a",
    }
}

#[derive(Clone, Copy)]
struct CivilDateTime {
    year: i64,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    milli: u32,
    weekday: u32,
    day_of_year: u32,
}

const MONTHS_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const MONTHS_LONG: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const WEEKDAYS_SHORT: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const WEEKDAYS_LONG: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let mp = if month > 2 { month - 3 } else { month + 9 } as i64;
    let doy = (153 * mp + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_millis(millis: i64) -> CivilDateTime {
    let seconds = millis.div_euclid(1000);
    let milli = millis.rem_euclid(1000) as u32;
    let days = seconds.div_euclid(86_400);
    let time_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let weekday = ((days % 7 + 7) % 7 + 4) as u32 % 7;
    let day_of_year = (days - days_from_civil(year, 1, 1) + 1) as u32;
    CivilDateTime {
        year,
        month,
        day,
        hour: (time_of_day / 3600) as u32,
        minute: (time_of_day % 3600 / 60) as u32,
        second: (time_of_day % 60) as u32,
        milli,
        weekday,
        day_of_year,
    }
}

fn pad(value: i64, width: usize) -> String {
    format!("{:0>width$}", value, width = width)
}

fn format_date_pattern(pattern: &str, millis: i64) -> String {
    let civil = civil_from_millis(millis);
    let chars: Vec<char> = pattern.chars().collect();
    let mut output = String::new();
    let mut index = 0;

    while index < chars.len() {
        let current = chars[index];
        if current == '\'' {
            index += 1;
            if index < chars.len() && chars[index] == '\'' {
                output.push('\'');
                index += 1;
                continue;
            }
            while index < chars.len() && chars[index] != '\'' {
                output.push(chars[index]);
                index += 1;
            }
            index += 1;
            continue;
        }

        if !current.is_ascii_alphabetic() {
            output.push(current);
            index += 1;
            continue;
        }

        let mut count = 0;
        while index < chars.len() && chars[index] == current {
            count += 1;
            index += 1;
        }
        output.push_str(&format_field(current, count, &civil));
    }

    output
}

fn format_field(letter: char, count: usize, civil: &CivilDateTime) -> String {
    let month_index = (civil.month - 1) as usize;
    let weekday_index = civil.weekday as usize;
    match letter {
        'y' | 'Y' => {
            if count == 2 {
                pad(civil.year.rem_euclid(100), 2)
            } else {
                pad(civil.year, count.max(1))
            }
        }
        'M' | 'L' => match count {
            1 => civil.month.to_string(),
            2 => pad(civil.month as i64, 2),
            3 => MONTHS_SHORT[month_index].to_owned(),
            _ => MONTHS_LONG[month_index].to_owned(),
        },
        'd' => pad(civil.day as i64, count),
        'D' => pad(civil.day_of_year as i64, count),
        'E' | 'c' => {
            if count >= 4 {
                WEEKDAYS_LONG[weekday_index].to_owned()
            } else {
                WEEKDAYS_SHORT[weekday_index].to_owned()
            }
        }
        'u' => (if civil.weekday == 0 { 7 } else { civil.weekday }).to_string(),
        'H' => pad(civil.hour as i64, count),
        'k' => pad(if civil.hour == 0 { 24 } else { civil.hour } as i64, count),
        'K' => pad((civil.hour % 12) as i64, count),
        'h' => {
            let hour = civil.hour % 12;
            pad(if hour == 0 { 12 } else { hour } as i64, count)
        }
        'm' => pad(civil.minute as i64, count),
        's' => pad(civil.second as i64, count),
        'S' => {
            let millis = pad(civil.milli as i64, 3);
            if count <= 3 {
                millis[..count].to_owned()
            } else {
                format!("{millis}{}", "0".repeat(count - 3))
            }
        }
        'a' => if civil.hour < 12 { "AM" } else { "PM" }.to_owned(),
        'G' => "AD".to_owned(),
        'z' => "UTC".to_owned(),
        'Z' => "+0000".to_owned(),
        'X' => "Z".to_owned(),
        other => other.to_string().repeat(count),
    }
}

#[derive(Default)]
struct ParsedFields {
    year: Option<i64>,
    month: Option<u32>,
    day: Option<u32>,
    hour24: Option<u32>,
    hour12: Option<u32>,
    minute: Option<u32>,
    second: Option<u32>,
    milli: Option<u32>,
    pm: Option<bool>,
}

fn parse_date_pattern(pattern: &str, text: &str) -> Option<i64> {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let input: Vec<char> = text.chars().collect();
    let mut fields = ParsedFields::default();
    let mut pattern_index = 0;
    let mut input_index = 0;

    while pattern_index < pattern_chars.len() {
        let current = pattern_chars[pattern_index];

        if current == '\'' {
            pattern_index += 1;
            if pattern_index < pattern_chars.len() && pattern_chars[pattern_index] == '\'' {
                match_literal('\'', &input, &mut input_index)?;
                pattern_index += 1;
                continue;
            }
            while pattern_index < pattern_chars.len() && pattern_chars[pattern_index] != '\'' {
                match_literal(pattern_chars[pattern_index], &input, &mut input_index)?;
                pattern_index += 1;
            }
            pattern_index += 1;
            continue;
        }

        if !current.is_ascii_alphabetic() {
            match_literal(current, &input, &mut input_index)?;
            pattern_index += 1;
            continue;
        }

        let mut count = 0;
        while pattern_index < pattern_chars.len() && pattern_chars[pattern_index] == current {
            count += 1;
            pattern_index += 1;
        }
        parse_field(current, count, &input, &mut input_index, &mut fields)?;
    }

    build_millis(&fields)
}

fn match_literal(expected: char, input: &[char], input_index: &mut usize) -> Option<()> {
    if expected.is_whitespace() {
        while input.get(*input_index).is_some_and(|c| c.is_whitespace()) {
            *input_index += 1;
        }
        return Some(());
    }
    let actual = input.get(*input_index)?;
    if actual.eq_ignore_ascii_case(&expected) {
        *input_index += 1;
        Some(())
    } else {
        None
    }
}

fn read_number(input: &[char], input_index: &mut usize, max_digits: usize) -> Option<i64> {
    while input.get(*input_index).is_some_and(|c| c.is_whitespace()) {
        *input_index += 1;
    }
    let start = *input_index;
    let mut digits = 0;
    let mut value: i64 = 0;
    while digits < max_digits {
        let Some(digit) = input.get(*input_index).and_then(|c| c.to_digit(10)) else {
            break;
        };
        value = value * 10 + digit as i64;
        *input_index += 1;
        digits += 1;
    }
    if *input_index == start {
        None
    } else {
        Some(value)
    }
}

fn parse_field(
    letter: char,
    count: usize,
    input: &[char],
    input_index: &mut usize,
    fields: &mut ParsedFields,
) -> Option<()> {
    match letter {
        'y' | 'Y' => {
            let max = if count <= 2 { 2 } else { count.max(4) };
            let value = read_number(input, input_index, max)?;
            let year = if count <= 2 && value < 100 {
                2000 + value
            } else {
                value
            };
            fields.year = Some(year);
        }
        'M' | 'L' => {
            if count >= 3 {
                fields.month = Some(read_month_name(input, input_index)?);
            } else {
                fields.month = Some(read_number(input, input_index, 2)? as u32);
            }
        }
        'd' => fields.day = Some(read_number(input, input_index, 2)? as u32),
        'D' => {
            read_number(input, input_index, 3)?;
        }
        'H' => fields.hour24 = Some(read_number(input, input_index, 2)? as u32),
        'k' => {
            let hour = read_number(input, input_index, 2)? as u32;
            fields.hour24 = Some(if hour == 24 { 0 } else { hour });
        }
        'K' => fields.hour12 = Some(read_number(input, input_index, 2)? as u32),
        'h' => {
            let hour = read_number(input, input_index, 2)? as u32;
            fields.hour12 = Some(if hour == 12 { 0 } else { hour });
        }
        'm' => fields.minute = Some(read_number(input, input_index, 2)? as u32),
        's' => fields.second = Some(read_number(input, input_index, 2)? as u32),
        'S' => fields.milli = Some(read_number(input, input_index, count.max(3))? as u32),
        'a' => fields.pm = Some(read_meridiem(input, input_index)?),
        'E' | 'c' => {
            read_word(input, input_index)?;
        }
        'G' | 'z' | 'Z' | 'X' | 'u' | 'w' | 'W' | 'F' => {
            read_word(input, input_index)?;
        }
        _ => return None,
    }
    Some(())
}

fn read_word(input: &[char], input_index: &mut usize) -> Option<()> {
    while input.get(*input_index).is_some_and(|c| c.is_whitespace()) {
        *input_index += 1;
    }
    let start = *input_index;
    while input
        .get(*input_index)
        .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '+' || *c == '-' || *c == ':')
    {
        *input_index += 1;
    }
    if *input_index == start {
        None
    } else {
        Some(())
    }
}

fn read_month_name(input: &[char], input_index: &mut usize) -> Option<u32> {
    while input.get(*input_index).is_some_and(|c| c.is_whitespace()) {
        *input_index += 1;
    }
    let remaining: String = input[*input_index..].iter().collect();
    let lowered = remaining.to_ascii_lowercase();
    for month in 0..12 {
        let long = MONTHS_LONG[month].to_ascii_lowercase();
        if lowered.starts_with(&long) {
            *input_index += long.chars().count();
            return Some(month as u32 + 1);
        }
    }
    for month in 0..12 {
        let short = MONTHS_SHORT[month].to_ascii_lowercase();
        if lowered.starts_with(&short) {
            *input_index += short.chars().count();
            return Some(month as u32 + 1);
        }
    }
    None
}

fn read_meridiem(input: &[char], input_index: &mut usize) -> Option<bool> {
    while input.get(*input_index).is_some_and(|c| c.is_whitespace()) {
        *input_index += 1;
    }
    let first = input.get(*input_index)?.to_ascii_uppercase();
    let second = input.get(*input_index + 1)?.to_ascii_uppercase();
    if second != 'M' {
        return None;
    }
    *input_index += 2;
    match first {
        'A' => Some(false),
        'P' => Some(true),
        _ => None,
    }
}

fn build_millis(fields: &ParsedFields) -> Option<i64> {
    let year = fields.year.unwrap_or(1970);
    let month = fields.month.unwrap_or(1);
    let day = fields.day.unwrap_or(1);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let hour = match (fields.hour24, fields.hour12, fields.pm) {
        (Some(hour), _, _) => hour,
        (None, Some(hour), pm) => hour % 12 + if pm.unwrap_or(false) { 12 } else { 0 },
        (None, None, _) => 0,
    };
    let minute = fields.minute.unwrap_or(0);
    let second = fields.second.unwrap_or(0);
    let milli = fields.milli.unwrap_or(0);

    let days = days_from_civil(year, month, day);
    let seconds = days * 86_400 + hour as i64 * 3600 + minute as i64 * 60 + second as i64;
    Some(seconds * 1000 + milli as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_math_round_trips_known_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 3, 1), 11017);
        assert_eq!(civil_from_days(11017), (2000, 3, 1));
        let civil = civil_from_millis(0);
        assert_eq!(civil.weekday, 4);
    }

    #[test]
    fn formats_iso_and_medium_patterns() {
        let millis = 1_719_580_009_123;
        assert_eq!(format_date_pattern("yyyy-MM-dd", millis), "2024-06-28");
        assert_eq!(format_date_pattern("MMM d, yyyy", millis), "Jun 28, 2024");
        assert_eq!(format_date_pattern("EEEE", millis), "Friday");
        assert_eq!(format_date_pattern("HH:mm:ss", millis), "13:06:49");
        assert_eq!(format_date_pattern("h:mm a", millis), "1:06 PM");
        assert_eq!(format_date_pattern("'on' yyyy", millis), "on 2024");
    }

    #[test]
    fn parses_then_formats_back() {
        let millis = parse_date_pattern("yyyy-MM-dd", "2024-06-28").expect("parse");
        assert_eq!(format_date_pattern("yyyy-MM-dd", millis), "2024-06-28");
        assert_eq!(format_date_pattern("MMM d, yyyy", millis), "Jun 28, 2024");
    }

    #[test]
    fn parses_month_name_and_meridiem() {
        let millis =
            parse_date_pattern("MMM d, yyyy h:mm a", "Jun 28, 2024 1:05 PM").expect("parse");
        assert_eq!(
            format_date_pattern("yyyy-MM-dd HH:mm", millis),
            "2024-06-28 13:05"
        );
    }

    #[test]
    fn rejects_unparseable_input() {
        assert_eq!(parse_date_pattern("yyyy-MM-dd", "not-a-date"), None);
    }
}
