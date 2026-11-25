use super::super::*;

impl DexProgram {
    pub(super) fn invoke_java_net_builtin(
        &self,
        method: &DexMethodId,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        if method.matches_builtin_signature("Ljava/net/URI;", "<init>", "(Ljava/lang/String;)V") {
            let invocation = invocation?;
            self.uri_store_string(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature("Ljava/net/URI;", "getHost", "()Ljava/lang/String;") {
            let invocation = invocation?;
            let value = self.uri_host(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature("Ljava/net/URL;", "<init>", "(Ljava/lang/String;)V") {
            let invocation = invocation?;
            self.url_store_spec(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/net/URL;",
            "openConnection",
            "()Ljava/net/URLConnection;",
        ) {
            let invocation = invocation?;
            let value = self.url_open_connection(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/net/URLConnection;"
                | "Ljava/net/HttpURLConnection;"
                | "Ljavax/net/ssl/HttpsURLConnection;"
        ) {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("setConnectTimeout", "(I)V") => {
                    let invocation = invocation?;
                    self.url_connection_store_int(invocation, &connection_connect_timeout_field())?;

                    return Some(DexExecutionResult::Void);
                }
                ("setReadTimeout", "(I)V") => {
                    let invocation = invocation?;
                    self.url_connection_store_int(invocation, &connection_read_timeout_field())?;

                    return Some(DexExecutionResult::Void);
                }
                ("setRequestProperty", "(Ljava/lang/String;Ljava/lang/String;)V") => {
                    let invocation = invocation?;
                    self.url_connection_set_request_property(invocation)?;

                    return Some(DexExecutionResult::Void);
                }
                ("connect", "()V") => {
                    let invocation = invocation?;
                    let result = self.http_connection_ensure_response(invocation)?;

                    return Some(match result {
                        Ok(_) => DexExecutionResult::Void,
                        Err(exception) => DexExecutionResult::Throw(exception),
                    });
                }
                ("getInputStream", "()Ljava/io/InputStream;") => {
                    let invocation = invocation?;
                    let result = self.http_connection_ensure_response(invocation)?;

                    return Some(match result {
                        Ok(stream) => DexExecutionResult::Value(stream),
                        Err(exception) => DexExecutionResult::Throw(exception),
                    });
                }
                ("disconnect", "()V") => {
                    return Some(DexExecutionResult::Void);
                }
                _ => {}
            }
        }

        if method.matches_builtin_signature(
            "Ljava/net/URLEncoder;",
            "encode",
            "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
        ) {
            let invocation = invocation?;
            let DexValue::String(value) = invocation.arguments.first()? else {
                return None;
            };
            let DexValue::String(charset) = invocation.arguments.get(1)? else {
                return None;
            };
            if !charset.eq_ignore_ascii_case("UTF-8") {
                return None;
            }

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(url_encode_form(value)),
            )));
        }

        // cite: libcore SocketFactory.java#getDefault
        if method.matches_builtin_signature(
            "Ljavax/net/SocketFactory;",
            "getDefault",
            "()Ljavax/net/SocketFactory;",
        ) {
            return Some(DexExecutionResult::Value(
                self.allocate_object("Ljavax/net/SocketFactory;".to_owned()),
            ));
        }

        // cite: libcore SSLSocketFactory.java#getDefault
        if method.matches_builtin_signature(
            "Ljavax/net/ssl/SSLSocketFactory;",
            "getDefault",
            "()Ljavax/net/SocketFactory;",
        ) {
            return Some(DexExecutionResult::Value(
                self.allocate_object("Ljavax/net/ssl/SSLSocketFactory;".to_owned()),
            ));
        }

        // cite: libcore ProxySelector.java#getDefault
        if method.matches_builtin_signature(
            "Ljava/net/ProxySelector;",
            "getDefault",
            "()Ljava/net/ProxySelector;",
        ) {
            return Some(DexExecutionResult::Value(
                self.allocate_object("Ljava/net/ProxySelector;".to_owned()),
            ));
        }

        // cite: libcore TrustManagerFactory.java#getDefaultAlgorithm, getInstance, init, getTrustManagers
        if method.class_descriptor == "Ljavax/net/ssl/TrustManagerFactory;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("getDefaultAlgorithm", "()Ljava/lang/String;") => {
                    return Some(DexExecutionResult::Value(DexValue::String(
                        std::rc::Rc::from("PKIX"),
                    )));
                }
                ("getInstance", "(Ljava/lang/String;)Ljavax/net/ssl/TrustManagerFactory;") => {
                    return Some(DexExecutionResult::Value(
                        self.allocate_object("Ljavax/net/ssl/TrustManagerFactory;".to_owned()),
                    ));
                }
                ("init", "(Ljava/security/KeyStore;)V") => {
                    return Some(DexExecutionResult::Void);
                }
                ("getTrustManagers", "()[Ljavax/net/ssl/TrustManager;") => {
                    let tm = self.allocate_object("Ljavax/net/ssl/X509TrustManager;".to_owned());
                    let array = self.allocate_filled_array(
                        "[Ljavax/net/ssl/TrustManager;".to_owned(),
                        vec![tm],
                    );
                    return Some(DexExecutionResult::Value(array));
                }
                _ => {}
            }
        }

        // cite: libcore X509TrustManager.java#getAcceptedIssuers
        if method.matches_builtin_signature(
            "Ljavax/net/ssl/X509TrustManager;",
            "getAcceptedIssuers",
            "()[Ljava/security/cert/X509Certificate;",
        ) {
            let array = self.allocate_filled_array(
                "[Ljava/security/cert/X509Certificate;".to_owned(),
                Vec::new(),
            );
            return Some(DexExecutionResult::Value(array));
        }

        // cite: libcore SSLContext.java#getInstance, init, getSocketFactory
        if method.class_descriptor == "Ljavax/net/ssl/SSLContext;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("getInstance", "(Ljava/lang/String;)Ljavax/net/ssl/SSLContext;")
                | (
                    "getInstance",
                    "(Ljava/lang/String;Ljava/lang/String;)Ljavax/net/ssl/SSLContext;",
                )
                | (
                    "getInstance",
                    "(Ljava/lang/String;Ljava/security/Provider;)Ljavax/net/ssl/SSLContext;",
                ) => {
                    return Some(DexExecutionResult::Value(
                        self.allocate_object("Ljavax/net/ssl/SSLContext;".to_owned()),
                    ));
                }
                (
                    "init",
                    "([Ljavax/net/ssl/KeyManager;[Ljavax/net/ssl/TrustManager;Ljava/security/SecureRandom;)V",
                ) => {
                    return Some(DexExecutionResult::Void);
                }
                ("getSocketFactory", "()Ljavax/net/ssl/SSLSocketFactory;") => {
                    return Some(DexExecutionResult::Value(
                        self.allocate_object("Ljavax/net/ssl/SSLSocketFactory;".to_owned()),
                    ));
                }
                _ => {}
            }
        }

        // cite: libcore Security.java#getProviders
        if method.matches_builtin_signature(
            "Ljava/security/Security;",
            "getProviders",
            "()[Ljava/security/Provider;",
        ) {
            let provider = self.allocate_object("Ljava/security/Provider;".to_owned());
            let array =
                self.allocate_filled_array("[Ljava/security/Provider;".to_owned(), vec![provider]);
            return Some(DexExecutionResult::Value(array));
        }

        // cite: libcore Provider.java#getName
        if method.matches_builtin_signature(
            "Ljava/security/Provider;",
            "getName",
            "()Ljava/lang/String;",
        ) {
            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from("AndroidOpenSSL"),
            )));
        }

        // cite: frameworks-base X509TrustManagerExtensions.java#X509TrustManagerExtensions
        if method.matches_builtin_signature(
            "Landroid/net/http/X509TrustManagerExtensions;",
            "<init>",
            "(Ljavax/net/ssl/X509TrustManager;)V",
        ) {
            return Some(DexExecutionResult::Void);
        }

        None
    }

    fn uri_store_string(&self, invocation: &DexInvocation) -> Option<()> {
        let receiver = invocation.arguments.first()?;
        let DexValue::String(value) = invocation.arguments.get(1)? else {
            return None;
        };

        self.write_stored_instance_field(
            receiver,
            &uri_string_field(),
            DexValue::String(value.clone()),
        )
        .then_some(())
    }

    fn uri_host(&self, invocation: &DexInvocation) -> Option<DexValue> {
        let receiver = invocation.arguments.first()?;
        let DexValue::String(value) =
            self.read_stored_instance_field(receiver, &uri_string_field())?
        else {
            return None;
        };

        Some(match uri_host_from_string(&value) {
            Some(host) => DexValue::String(std::rc::Rc::from(host)),
            None => DexValue::Null,
        })
    }

    fn url_store_spec(&self, invocation: &DexInvocation) -> Option<()> {
        let receiver = invocation.arguments.first()?;
        let DexValue::String(value) = invocation.arguments.get(1)? else {
            return None;
        };

        self.write_stored_instance_field(
            receiver,
            &url_spec_field(),
            DexValue::String(value.clone()),
        )
        .then_some(())
    }

    fn url_open_connection(&self, invocation: &DexInvocation) -> Option<DexValue> {
        let url = invocation.arguments.first()?;
        let DexValue::String(spec) = self.read_stored_instance_field(url, &url_spec_field())?
        else {
            return None;
        };

        // cite: libcore URL.java#openConnection
        let connection = self.allocate_object("Ljava/net/HttpURLConnection;".to_owned());
        self.write_stored_instance_field(
            &connection,
            &connection_url_field(),
            DexValue::String(spec),
        );

        Some(connection)
    }

    fn url_connection_store_int(
        &self,
        invocation: &DexInvocation,
        field: &DexFieldId,
    ) -> Option<()> {
        let connection = invocation.arguments.first()?;
        let DexValue::Int(value) = invocation.arguments.get(1)? else {
            return None;
        };

        self.write_stored_instance_field(connection, field, DexValue::Int(*value))
            .then_some(())
    }

    fn url_connection_set_request_property(&self, invocation: &DexInvocation) -> Option<()> {
        let connection = invocation.arguments.first()?;
        let DexValue::String(name) = invocation.arguments.get(1)? else {
            return None;
        };
        let DexValue::String(value) = invocation.arguments.get(2)? else {
            return None;
        };

        let mut properties = match self
            .read_stored_instance_field(connection, &connection_request_properties_field())
        {
            Some(DexValue::String(existing)) => existing
                .lines()
                .filter(|line| {
                    line.split_once(':')
                        .is_none_or(|(key, _)| !key.eq_ignore_ascii_case(name))
                })
                .map(str::to_owned)
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        properties.push(format!("{name}:{value}"));

        self.write_stored_instance_field(
            connection,
            &connection_request_properties_field(),
            DexValue::String(std::rc::Rc::from(properties.join("\n"))),
        )
        .then_some(())
    }

    fn http_connection_ensure_response(
        &self,
        invocation: &DexInvocation,
    ) -> Option<std::result::Result<DexValue, DexValue>> {
        let connection = invocation.arguments.first()?;
        if !dex_value_is_object_like(connection) {
            return None;
        }

        if let Some(stream) =
            self.read_stored_instance_field(connection, &connection_response_stream_field())
        {
            return Some(Ok(stream));
        }

        let DexValue::String(url) =
            self.read_stored_instance_field(connection, &connection_url_field())?
        else {
            return None;
        };

        let connect_timeout_millis = match self
            .read_stored_instance_field(connection, &connection_connect_timeout_field())
        {
            Some(DexValue::Int(value)) if value > 0 => Some(value),
            _ => None,
        };
        let read_timeout_millis =
            match self.read_stored_instance_field(connection, &connection_read_timeout_field()) {
                Some(DexValue::Int(value)) if value > 0 => Some(value),
                _ => None,
            };
        let request_properties = match self
            .read_stored_instance_field(connection, &connection_request_properties_field())
        {
            Some(DexValue::String(properties)) => properties
                .lines()
                .filter_map(|line| {
                    line.split_once(':')
                        .map(|(key, value)| (key.to_owned(), value.to_owned()))
                })
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };

        match host_http_get(
            &url,
            connect_timeout_millis,
            read_timeout_millis,
            &request_properties,
        ) {
            Ok(bytes) => {
                let stream = self.allocate_http_input_stream(bytes);
                self.write_stored_instance_field(
                    connection,
                    &connection_response_stream_field(),
                    stream.clone(),
                );

                Some(Ok(stream))
            }
            Err(message) => Some(Err(self.java_net_io_exception(&message))),
        }
    }

    fn allocate_http_input_stream(&self, bytes: Vec<u8>) -> DexValue {
        let id = {
            let mut runtime = self.runtime.borrow_mut();
            let id = runtime.next_object_id;
            runtime.next_object_id += 1;
            id
        };
        let reference = DexValue::Reference(std::rc::Rc::from(format!(
            "Ljava/io/InputStream;:http/{id}"
        )));
        self.runtime.borrow_mut().input_stream_values.insert(
            map_identity_key(&reference),
            InputStreamState { bytes, position: 0 },
        );

        reference
    }

    fn java_net_io_exception(&self, message: &str) -> DexValue {
        let exception = self.allocate_object("Ljava/io/IOException;".to_owned());
        self.write_stored_instance_field(
            &exception,
            &net_throwable_detail_message_field(),
            DexValue::String(std::rc::Rc::from(message.to_owned())),
        );

        exception
    }
}

fn host_http_get(
    url: &str,
    connect_timeout_millis: Option<i32>,
    read_timeout_millis: Option<i32>,
    request_properties: &[(String, String)],
) -> std::result::Result<Vec<u8>, String> {
    let scheme_is_http = url.split_once(':').is_some_and(|(scheme, _)| {
        scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
    });
    if !scheme_is_http {
        return Err(format!("unsupported URL scheme: {url}"));
    }

    let connect_timeout_seconds = timeout_seconds(connect_timeout_millis, 10);
    let total_timeout_seconds = connect_timeout_seconds + timeout_seconds(read_timeout_millis, 20);

    let mut command = std::process::Command::new("curl");
    command
        .arg("--silent")
        .arg("--show-error")
        .arg("--fail")
        .arg("--location")
        .arg("--connect-timeout")
        .arg(connect_timeout_seconds.to_string())
        .arg("--max-time")
        .arg(total_timeout_seconds.to_string());

    for (name, value) in request_properties {
        command.arg("--header").arg(format!("{name}: {value}"));
    }

    let output = command
        .arg("--")
        .arg(url)
        .output()
        .map_err(|error| format!("host HTTP transport unavailable: {error}"))?;

    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        let detail = detail.trim();
        if detail.is_empty() {
            return Err(format!("HTTP request failed: {url}"));
        }

        return Err(format!("HTTP request failed: {detail}"));
    }

    Ok(output.stdout)
}

fn timeout_seconds(timeout_millis: Option<i32>, default_seconds: u32) -> u32 {
    match timeout_millis {
        Some(millis) if millis > 0 => ((millis as u32).div_ceil(1000)).max(1),
        _ => default_seconds,
    }
}

fn uri_string_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/net/URI;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "string".to_owned(),
    }
}

fn url_spec_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/net/URL;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "spec".to_owned(),
    }
}

fn connection_url_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/net/URLConnection;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "url".to_owned(),
    }
}

fn connection_connect_timeout_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/net/URLConnection;".to_owned(),
        type_descriptor: "I".to_owned(),
        name: "connectTimeout".to_owned(),
    }
}

fn connection_read_timeout_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/net/URLConnection;".to_owned(),
        type_descriptor: "I".to_owned(),
        name: "readTimeout".to_owned(),
    }
}

fn connection_response_stream_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/net/URLConnection;".to_owned(),
        type_descriptor: "Ljava/io/InputStream;".to_owned(),
        name: "responseStream".to_owned(),
    }
}

fn connection_request_properties_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/net/URLConnection;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "requestProperties".to_owned(),
    }
}

fn net_throwable_detail_message_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Throwable;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "detailMessage".to_owned(),
    }
}

fn uri_host_from_string(uri: &str) -> Option<String> {
    // cite: libcore URI.java#getHost, parseAuthority, parseServer
    let (_scheme, rest) = uri.split_once(':')?;
    let rest = rest.strip_prefix("//")?;
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    if authority.is_empty() {
        return None;
    }

    let host_port = match authority.split_once('@') {
        Some((_userinfo, host_port)) => host_port,
        None => authority,
    };

    if let Some(bracketed) = host_port.strip_prefix('[') {
        let end = bracketed.find(']')?;
        return Some(format!("[{}]", &bracketed[..end]));
    }

    let host = match host_port.split_once(':') {
        Some((host, _port)) => host,
        None => host_port,
    };
    if host.is_empty() {
        return None;
    }

    Some(host.to_owned())
}

fn url_encode_form(value: &str) -> String {
    // cite: libcore URLEncoder.java#encode, dontNeedEncoding
    let mut encoded = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '*' => encoded.push(character),
            ' ' => encoded.push('+'),
            _ => {
                let mut bytes = [0u8; 4];
                for byte in character.encode_utf8(&mut bytes).as_bytes() {
                    encoded.push_str(&format!("%{byte:02X}"));
                }
            }
        }
    }

    encoded
}
