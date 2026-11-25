use super::super::*;

impl DexProgram {
    pub(super) fn invoke_android_net_builtin(
        &self,
        method: &DexMethodId,
        _invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        if method.class_descriptor == "Landroid/net/ConnectivityManager;"
            && method.name == "getActiveNetworkInfo"
            && method.proto.descriptor_is("()Landroid/net/NetworkInfo;")
        {
            return Some(DexExecutionResult::Value(DexValue::Null));
        }

        if method.class_descriptor == "Landroid/net/ConnectivityManager;"
            && method.name == "getActiveNetwork"
            && method.proto.descriptor_is("()Landroid/net/Network;")
        {
            return Some(DexExecutionResult::Value(DexValue::Null));
        }

        // cite: frameworks-base Context.java#CONNECTIVITY_SERVICE
        if method.class_descriptor == "Landroid/net/ConnectivityManager;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                (
                    "registerDefaultNetworkCallback",
                    "(Landroid/net/ConnectivityManager$NetworkCallback;)V"
                    | "(Landroid/net/ConnectivityManager$NetworkCallback;Landroid/os/Handler;)V",
                )
                | (
                    "registerNetworkCallback",
                    "(Landroid/net/NetworkRequest;Landroid/net/ConnectivityManager$NetworkCallback;)V"
                    | "(Landroid/net/NetworkRequest;Landroid/net/ConnectivityManager$NetworkCallback;Landroid/os/Handler;)V",
                )
                | (
                    "unregisterNetworkCallback",
                    "(Landroid/net/ConnectivityManager$NetworkCallback;)V",
                ) => {
                    return Some(DexExecutionResult::Void);
                }
                (
                    "getNetworkCapabilities",
                    "(Landroid/net/Network;)Landroid/net/NetworkCapabilities;",
                ) => {
                    return Some(DexExecutionResult::Value(DexValue::Null));
                }
                _ => {}
            }
        }

        // cite: frameworks-base Context.java#CONNECTIVITY_SERVICE
        if method.matches_builtin_signature(
            "Landroid/net/ConnectivityManager$NetworkCallback;",
            "<init>",
            "()V",
        ) {
            return Some(DexExecutionResult::Void);
        }

        None
    }
}
