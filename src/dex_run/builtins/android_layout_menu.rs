use super::super::*;

impl DexProgram {
    pub(super) fn invoke_android_layout_menu_builtin(
        &self,
        method: &DexMethodId,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        if method.class_descriptor == "Landroid/view/LayoutInflater;"
            && method.name == "from"
            && method.proto.descriptor()
                == "(Landroid/content/Context;)Landroid/view/LayoutInflater;"
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroid/view/LayoutInflater;.bootstrap"),
            )));
        }

        if method.class_descriptor == "Landroid/view/LayoutInflater;"
            && method.name == "getFactory"
            && method
                .proto
                .descriptor_is("()Landroid/view/LayoutInflater$Factory;")
        {
            return Some(DexExecutionResult::Value(DexValue::Null));
        }

        if method.class_descriptor == "Landroid/view/LayoutInflater;"
            && method.name == "cloneInContext"
            && method.proto.descriptor()
                == "(Landroid/content/Context;)Landroid/view/LayoutInflater;"
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroid/view/LayoutInflater;.bootstrap"),
            )));
        }

        // cite: frameworks-base LayoutInflater.java#setFactory2
        if method.class_descriptor == "Landroid/view/LayoutInflater;"
            && method.name == "setFactory2"
            && method
                .proto
                .descriptor_is("(Landroid/view/LayoutInflater$Factory2;)V")
        {
            if let Some(invocation) = invocation {
                if let (Some(receiver), Some(factory)) =
                    (invocation.arguments.first(), invocation.arguments.get(1))
                {
                    self.write_stored_instance_field(
                        receiver,
                        &DexFieldId {
                            class_descriptor: "Landroid/view/LayoutInflater;".to_owned(),
                            type_descriptor: "Landroid/view/LayoutInflater$Factory2;".to_owned(),
                            name: "mFactory2".to_owned(),
                        },
                        factory.clone(),
                    );
                }
            }
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base LayoutInflater.java#getFactory2
        if method.class_descriptor == "Landroid/view/LayoutInflater;"
            && method.name == "getFactory2"
            && method
                .proto
                .descriptor_is("()Landroid/view/LayoutInflater$Factory2;")
        {
            let invocation = invocation?;
            let factory = self
                .read_stored_instance_field(
                    invocation.arguments.first()?,
                    &DexFieldId {
                        class_descriptor: "Landroid/view/LayoutInflater;".to_owned(),
                        type_descriptor: "Landroid/view/LayoutInflater$Factory2;".to_owned(),
                        name: "mFactory2".to_owned(),
                    },
                )
                .unwrap_or(DexValue::Null);
            return Some(DexExecutionResult::Value(factory));
        }

        // cite: frameworks-base LayoutInflater.java#getContext
        if method.class_descriptor == "Landroid/view/LayoutInflater;"
            && method.name == "getContext"
            && method.proto.descriptor_is("()Landroid/content/Context;")
        {
            return Some(DexExecutionResult::Value(self.active_context_value()));
        }

        if method.class_descriptor == "Landroid/view/LayoutInflater;"
            && method.name == "inflate"
            && method
                .proto
                .descriptor_is("(ILandroid/view/ViewGroup;Z)Landroid/view/View;")
        {
            let invocation = invocation?;
            let value = self.layout_inflater_inflate(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/view/LayoutInflater;"
            && method.name == "inflate"
            && method
                .proto
                .descriptor_is("(ILandroid/view/ViewGroup;)Landroid/view/View;")
        {
            let invocation = invocation?;
            let value = self.layout_inflater_inflate_without_attach(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/view/MenuInflater;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(Landroid/content/Context;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/view/MenuItem;"
            && method.name == "getItemId"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            if let DexValue::Reference(reference) = &invocation.arguments[0] {
                if let Some(item_id) = reference
                    .strip_prefix("Landroid/view/MenuItem;:option/")
                    .and_then(|id| id.parse::<i32>().ok())
                {
                    return Some(DexExecutionResult::Int(item_id));
                }
            }
        }

        if method.class_descriptor == "Ln/n0;"
            && method.name == "setWindowCallback"
            && method
                .proto
                .descriptor_is("(Landroid/view/Window$Callback;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ln/n0;"
            && method.name == "setWindowTitle"
            && method.proto.descriptor_is("(Ljava/lang/CharSequence;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        None
    }
}
