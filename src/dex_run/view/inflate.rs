use super::helpers::*;
use super::*;

impl DexProgram {
    pub(in crate::dex_run) fn record_content_layout(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let DexValue::Int(layout_id) = invocation.arguments[1] else {
            return None;
        };

        self.runtime.borrow_mut().content_layouts.push(layout_id);
        let root = self.layout_reference_for_id(layout_id);
        self.register_inflated_layout_tree(layout_id, &root).ok()?;

        Some(())
    }

    pub(in crate::dex_run) fn record_content_view_argument(
        &self,
        invocation: &DexInvocation,
        view_argument_index: usize,
    ) -> bool {
        let Some(argument) = invocation.arguments.get(view_argument_index) else {
            return false;
        };
        let (layout_id, is_layout_resource) = match argument {
            DexValue::Int(layout_id) => (*layout_id, true),
            _ => match layout_id_from_view_value(argument) {
                Some(layout_id) => (layout_id, false),
                None => return false,
            },
        };

        {
            let mut runtime = self.runtime.borrow_mut();
            if runtime.content_layouts.last() != Some(&layout_id) {
                runtime.content_layouts.push(layout_id);
            }
        }

        // cite: frameworks-base PhoneWindow.java#setContentView
        if is_layout_resource {
            let root = self.layout_reference_for_id(layout_id);
            let _ = self.register_inflated_layout_tree(layout_id, &root);

            // cite: frameworks-base PhoneWindow.java#generateLayout
            let content = view_reference(ANDROID_CONTENT_VIEW_ID);
            let content_key = map_identity_key(&content);
            let root_key = map_identity_key(&root);
            let mut runtime = self.runtime.borrow_mut();
            runtime.view_parents.insert(root_key, content);
            let siblings = runtime.view_group_children.entry(content_key).or_default();
            if !siblings
                .iter()
                .any(|child| map_identity_key(child) == map_identity_key(&root))
            {
                siblings.push(root);
            }
        }

        true
    }

    pub(in crate::dex_run) fn activity_set_content_view(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if !matches!(invocation.arguments.len(), 2 | 3) {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        let content = view_reference(ANDROID_CONTENT_VIEW_ID);
        let content_key = map_identity_key(&content);
        let view = invocation.arguments[1].clone();
        let view_key = map_identity_key(&view);

        if let Some(layout_params) = invocation.arguments.get(2) {
            if dex_value_is_object_like(layout_params) {
                self.write_stored_instance_field(
                    &view,
                    &view_layout_params_field(),
                    layout_params.clone(),
                );
            }
        }

        // cite: frameworks-base Activity.java#setContentView
        // cite: frameworks-base PhoneWindow.java#setContentView
        // cite: frameworks-base ViewGroup.java#removeAllViews, addView
        let (removed_children, previous_parent) = {
            let mut runtime = self.runtime.borrow_mut();
            let removed_children = runtime
                .view_group_children
                .remove(&content_key)
                .unwrap_or_default();
            for child in &removed_children {
                runtime.view_parents.remove(&map_identity_key(&child));
            }
            let previous_parent = runtime.view_parents.get(&view_key).cloned();
            if let Some(previous_parent) = &previous_parent {
                let previous_parent_key = map_identity_key(previous_parent);
                if let Some(previous_siblings) =
                    runtime.view_group_children.get_mut(&previous_parent_key)
                {
                    previous_siblings.retain(|child| map_identity_key(child) != view_key);
                }
            }
            runtime
                .view_parents
                .insert(view_key.clone(), content.clone());
            runtime
                .view_group_children
                .entry(content_key.clone())
                .or_default()
                .push(view.clone());

            let root_start = runtime
                .activity_stack
                .last()
                .map(|activity| activity.runtime_view_root_start)
                .unwrap_or(0);
            runtime.runtime_view_roots.truncate(root_start);

            if layout_id_from_view_value(&view).is_none() {
                if let Some(class_descriptor) = value_class_descriptor(&view) {
                    runtime.runtime_view_roots.push(DexRuntimeViewRoot {
                        view_key,
                        class_descriptor: class_descriptor.to_owned(),
                    });
                }
            }
            runtime.request_layout_traversal();
            (removed_children, previous_parent)
        };

        for child in &removed_children {
            self.discard_view_render_node_subtree(child);
        }
        self.mark_view_render_node_dirty(&content);
        if let Some(previous_parent) = previous_parent {
            self.mark_view_render_node_dirty(&previous_parent);
        }

        Some(())
    }

    pub(in crate::dex_run) fn appcompat_delegate_find_view_by_id(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let DexValue::Int(view_id) = invocation.arguments[1] else {
            return None;
        };

        self.runtime.borrow_mut().view_lookup_ids.push(view_id);
        if let Some(value) = self.find_registered_view_by_id(&invocation.arguments[0], view_id) {
            return Some(value);
        }

        // cite: androidx AppCompatDelegateImpl.java#findViewById
        // cite: frameworks-base View.java#findViewById, findViewTraversal
        Some(DexValue::Null)
    }

    pub(in crate::dex_run) fn layout_inflater_inflate(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        self.layout_inflater_inflate_value(invocation, 4).ok()
    }

    pub(in crate::dex_run) fn layout_inflater_inflate_without_attach(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        self.layout_inflater_inflate_value(invocation, 3).ok()
    }

    pub(in crate::dex_run) fn layout_inflater_inflate_value(
        &self,
        invocation: &DexInvocation,
        expected_argument_count: usize,
    ) -> Result<DexValue> {
        self.layout_inflater_inflate_value_with_initialization(
            invocation,
            expected_argument_count,
            None,
        )
    }

    fn layout_inflater_inflate_value_with_initialization(
        &self,
        invocation: &DexInvocation,
        expected_argument_count: usize,
        initialization_depth: Option<usize>,
    ) -> Result<DexValue> {
        if invocation.arguments.len() != expected_argument_count {
            return Err(WinedroidError::new(format!(
                "LayoutInflater.inflate expected {expected_argument_count} arguments, got {}",
                invocation.arguments.len()
            )));
        }
        let DexValue::Int(layout_id) = invocation.arguments[1] else {
            return Err(WinedroidError::new(format!(
                "LayoutInflater.inflate requires an integer layout ID\nargument: {}",
                invocation.arguments[1]
            )));
        };

        self.runtime.borrow_mut().inflated_layouts.push(layout_id);
        let root = self.layout_reference_for_id(layout_id);
        match initialization_depth {
            Some(depth) => {
                self.register_initialized_layout_tree(layout_id, &root, depth)?;
            }
            None => self.register_inflated_layout_tree(layout_id, &root)?,
        }
        let attached_parent =
            self.record_attached_content_layout(layout_id, invocation, expected_argument_count);

        Ok(attached_parent.unwrap_or(root))
    }

    pub(in crate::dex_run) fn record_attached_content_layout(
        &self,
        layout_id: i32,
        invocation: &DexInvocation,
        expected_argument_count: usize,
    ) -> Option<DexValue> {
        let parent = match expected_argument_count {
            3 => invocation.arguments.get(2)?,
            4 => {
                let DexValue::Int(attach_to_root) = invocation.arguments.get(3)? else {
                    return None;
                };
                if *attach_to_root == 0 {
                    return None;
                }
                invocation.arguments.get(2)?
            }
            _ => return None,
        };

        if self.view_id_from_value(parent)? != ANDROID_CONTENT_VIEW_ID {
            return None;
        }

        {
            let mut runtime = self.runtime.borrow_mut();
            if runtime.content_layouts.last() != Some(&layout_id) {
                runtime.content_layouts.push(layout_id);
            }
        }

        Some(parent.clone())
    }

    pub(in crate::dex_run) fn layout_inflater_inflate_and_finish(
        &self,
        invocation: &DexInvocation,
        expected_argument_count: usize,
        depth: usize,
    ) -> Result<DexValue> {
        self.layout_inflater_inflate_value_with_initialization(
            invocation,
            expected_argument_count,
            Some(depth),
        )
    }

    pub(in crate::dex_run) fn row_view_layout_id(&self, view: &DexValue) -> Option<i32> {
        layout_id_from_view_value(view)
    }

    pub(in crate::dex_run) fn layout_reference_for_id(&self, layout_id: i32) -> DexValue {
        let Ok(Some(class_descriptor)) = self.layout_root_class_descriptor(layout_id) else {
            return layout_reference(layout_id);
        };

        DexValue::Reference(std::rc::Rc::from(format!(
            "{class_descriptor}:layout/{layout_id}"
        )))
    }

    pub(in crate::dex_run) fn layout_root_class_descriptor(
        &self,
        layout_id: i32,
    ) -> Result<Option<String>> {
        if let Some(descriptor) = self
            .runtime
            .borrow()
            .layout_root_descriptors
            .get(&layout_id)
        {
            return Ok(descriptor.clone());
        }

        let descriptor = match self.layout_xml_events(layout_id)? {
            Some(events) => events.into_iter().find_map(|event| match event {
                XmlEvent::Start(element) => layout_element_class_descriptor(&element.name),
                XmlEvent::End(_) => None,
            }),
            None => None,
        };
        self.runtime
            .borrow_mut()
            .layout_root_descriptors
            .insert(layout_id, descriptor.clone());

        Ok(descriptor)
    }

    pub(in crate::dex_run) fn layout_xml_events(
        &self,
        layout_id: i32,
    ) -> Result<Option<Vec<XmlEvent>>> {
        if layout_id <= 0 {
            return Ok(None);
        }

        let Some(apk_path) = self.apk_path.as_ref() else {
            return Ok(None);
        };

        let resource_id = layout_id as u32;
        let Some(resource) = self.resolve_resource_file(resource_id)? else {
            return Ok(None);
        };
        if resource.type_name != "layout" {
            return Ok(None);
        }

        let mut apk = ApkArchive::open(apk_path)?;
        let data = apk.read_required_file(&resource.path)?;
        let events = binary_xml::parse_binary_xml(&data).map_err(|error| {
            WinedroidError::with_source(
                format!(
                    "failed to parse layout XML resource 0x{resource_id:08x} at `{}`",
                    resource.path
                ),
                error,
            )
        })?;

        Ok(Some(events))
    }

    // cite: frameworks-base LayoutInflater.java#rInflate, rInflateChildren
    pub(in crate::dex_run) fn register_inflated_layout_tree(
        &self,
        layout_id: i32,
        root: &DexValue,
    ) -> Result<()> {
        let root_key = map_identity_key(root);
        self.register_layout_file_views(layout_id, Some(root), &root_key, None, None, 0, None)
    }

    fn register_initialized_layout_tree(
        &self,
        layout_id: i32,
        root: &DexValue,
        initialization_depth: usize,
    ) -> Result<()> {
        let root_key = map_identity_key(root);
        self.register_layout_file_views(
            layout_id,
            Some(root),
            &root_key,
            None,
            None,
            0,
            Some(initialization_depth),
        )
    }

    fn register_layout_file_views(
        &self,
        layout_id: i32,
        file_root: Option<&DexValue>,
        flat_root_key: &str,
        attach_parent: Option<&DexValue>,
        root_id_override: Option<i32>,
        depth: usize,
        initialization_depth: Option<usize>,
    ) -> Result<()> {
        if depth > 16 {
            return Err(WinedroidError::new(format!(
                "layout include depth exceeded for layout 0x{layout_id:08x}"
            )));
        }
        let Some(events) = self.layout_xml_events(layout_id)? else {
            return Ok(());
        };

        self.register_layout_events(
            layout_id,
            events,
            file_root,
            flat_root_key,
            attach_parent,
            root_id_override,
            depth,
            initialization_depth,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::dex_run) fn register_layout_events(
        &self,
        layout_id: i32,
        events: Vec<XmlEvent>,
        file_root: Option<&DexValue>,
        flat_root_key: &str,
        attach_parent: Option<&DexValue>,
        root_id_override: Option<i32>,
        depth: usize,
        initialization_depth: Option<usize>,
    ) -> Result<()> {
        let attrs = self.fresh_xml_resource_parser_reference(layout_id);
        let attrs_key = map_identity_key(&attrs);
        // cite: frameworks-base LayoutInflater.java#inflate, createViewFromTag, rInflate
        self.runtime.borrow_mut().xml_parsers.insert(
            attrs_key.clone(),
            XmlParserState {
                resource_id: layout_id,
                events: events.clone(),
                position: XmlParserPosition::StartDocument,
                closed: false,
            },
        );
        let result = self.register_layout_events_with_attribute_set(
            layout_id,
            events,
            file_root,
            flat_root_key,
            attach_parent,
            root_id_override,
            depth,
            initialization_depth,
            &attrs,
        );
        self.runtime.borrow_mut().xml_parsers.remove(&attrs_key);

        result
    }

    #[allow(clippy::too_many_arguments)]
    fn register_layout_events_with_attribute_set(
        &self,
        layout_id: i32,
        events: Vec<XmlEvent>,
        file_root: Option<&DexValue>,
        flat_root_key: &str,
        attach_parent: Option<&DexValue>,
        root_id_override: Option<i32>,
        depth: usize,
        initialization_depth: Option<usize>,
        attrs: &DexValue,
    ) -> Result<()> {
        let mut stack: Vec<(DexValue, bool)> = Vec::new();
        let mut ordinal = 0usize;
        let mut seen_first = false;
        let attrs_key = map_identity_key(attrs);

        for (event_index, event) in events.into_iter().enumerate() {
            let positioned = self
                .runtime
                .borrow_mut()
                .xml_parsers
                .get_mut(&attrs_key)
                .map(|parser| parser.position = XmlParserPosition::Event(event_index))
                .is_some();
            if !positioned {
                return Err(WinedroidError::new(format!(
                    "inflated layout lost its active XML parser\nlayout: 0x{layout_id:08x}"
                )));
            }

            match event {
                XmlEvent::Start(element) => {
                    let is_first = !seen_first;
                    seen_first = true;

                    if element.name == "include" && !is_first {
                        // cite: frameworks-base LayoutInflater.java#parseInclude
                        let parent = stack.last().map(|(view, _)| view.clone());
                        if let Some(included_layout_id) = xml_element_layout_reference(&element) {
                            self.register_layout_file_views(
                                included_layout_id,
                                None,
                                flat_root_key,
                                parent.as_ref(),
                                xml_element_view_id(&element),
                                depth + 1,
                                initialization_depth,
                            )?;
                        }
                        stack.push((parent.unwrap_or(DexValue::Null), false));
                        ordinal += 1;
                        continue;
                    }

                    if element.name == "merge" && is_first {
                        // cite: frameworks-base LayoutInflater.java#parseInclude, rInflate
                        let merge_owner = file_root
                            .cloned()
                            .or_else(|| attach_parent.cloned())
                            .unwrap_or(DexValue::Null);
                        stack.push((merge_owner, false));
                        ordinal += 1;
                        continue;
                    }

                    // cite: frameworks-base LayoutInflater.java#parseInclude
                    let view_id = if is_first {
                        root_id_override.or_else(|| xml_element_view_id(&element))
                    } else {
                        xml_element_view_id(&element)
                    };
                    let descriptor = layout_element_class_descriptor(&element.name)
                        .unwrap_or_else(|| "Landroid/view/View;".to_owned());
                    let parent = stack
                        .last()
                        .map(|(view, _)| view.clone())
                        .or_else(|| attach_parent.cloned())
                        .filter(|parent| !matches!(parent, DexValue::Null));

                    let mut factory_created = false;
                    let view = if let (true, Some(file_root)) = (is_first, file_root) {
                        file_root.clone()
                    } else if let Some(initialization_depth) = initialization_depth {
                        if matches!(
                            element.name.as_str(),
                            "androidx.fragment.app.FragmentContainerView" | "fragment"
                        ) {
                            if let Some(created) = self.try_create_view_via_factory2(
                                parent.as_ref(),
                                &element.name,
                                attrs,
                                initialization_depth,
                            )? {
                                factory_created = true;
                                created
                            } else if let Some(view_id) = view_id {
                                layout_child_reference(layout_id, view_id, &descriptor)
                            } else {
                                layout_node_reference(layout_id, ordinal, &descriptor)
                            }
                        } else if let Some(view_id) = view_id {
                            layout_child_reference(layout_id, view_id, &descriptor)
                        } else {
                            layout_node_reference(layout_id, ordinal, &descriptor)
                        }
                    } else if let Some(view_id) = view_id {
                        layout_child_reference(layout_id, view_id, &descriptor)
                    } else {
                        layout_node_reference(layout_id, ordinal, &descriptor)
                    };
                    ordinal += 1;

                    if let Some(view_id) = view_id {
                        self.write_stored_instance_field(
                            &view,
                            &view_id_field(),
                            DexValue::Int(view_id),
                        );
                        self.runtime
                            .borrow_mut()
                            .inflated_view_children
                            .insert((flat_root_key.to_owned(), view_id), view.clone());
                    }
                    if !factory_created {
                        if let Some(initialization_depth) = initialization_depth {
                            // cite: frameworks-base LayoutInflater.java#createViewFromTag, createView, mConstructorSignature
                            self.initialize_inflated_layout_root(
                                &view,
                                attrs,
                                initialization_depth,
                            )?;
                        }
                    }
                    if let Some(parent) = parent {
                        if map_identity_key(&parent) != map_identity_key(&view) {
                            self.append_view_child(&parent, &view);
                        }
                    }
                    stack.push((view, true));
                }
                XmlEvent::End(_) => {
                    let Some((view, finish_inflate)) = stack.pop() else {
                        continue;
                    };
                    if let (true, Some(initialization_depth)) =
                        (finish_inflate, initialization_depth)
                    {
                        // cite: frameworks-base LayoutInflater.java#rInflate, rInflateChildren
                        self.finish_inflated_layout_root(&view, initialization_depth)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub(in crate::dex_run) fn finish_inflated_layout_root(
        &self,
        root: &DexValue,
        depth: usize,
    ) -> Result<()> {
        let Some(class_descriptor) = value_class_descriptor(root).map(str::to_owned) else {
            return Ok(());
        };
        let method = DexMethodId {
            class_descriptor: class_descriptor.clone(),
            name: "onFinishInflate".to_owned(),
            proto: crate::dex::DexProto {
                shorty: "V".to_owned(),
                return_type: "V".to_owned(),
                parameters: Vec::new(),
            },
        };
        let Some(resolved) =
            self.resolve_exact_method_in_class_hierarchy(&class_descriptor, &method)?
        else {
            return Ok(());
        };
        let result = self.execute_method(None, &resolved, Some(&[root.clone()]), depth + 1)?;

        match result {
            DexExecutionResult::Void => Ok(()),
            DexExecutionResult::Int(_)
            | DexExecutionResult::Value(_)
            | DexExecutionResult::Throw(_) => Err(WinedroidError::new(format!(
                "View.onFinishInflate did not complete normally\nclass: {class_descriptor}"
            ))),
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                Err(WinedroidError::new(format!(
                    "View.onFinishInflate did not complete normally\nclass: {class_descriptor}"
                )))
            }
        }
    }

    // cite: frameworks-base LayoutInflater.java#tryCreateView
    // cite: frameworks-base PhoneWindow.java#PhoneWindow
    fn try_create_view_via_factory2(
        &self,
        parent: Option<&DexValue>,
        tag_name: &str,
        attrs: &DexValue,
        depth: usize,
    ) -> Result<Option<DexValue>> {
        let context = self.active_context_value();
        let Some(context_descriptor) = value_class_descriptor(&context).map(str::to_owned) else {
            return Ok(None);
        };
        let on_create_view = DexMethodId {
            class_descriptor: context_descriptor.clone(),
            name: "onCreateView".to_owned(),
            proto: crate::dex::DexProto {
                shorty: "LLLLL".to_owned(),
                return_type: "Landroid/view/View;".to_owned(),
                parameters: vec![
                    "Landroid/view/View;".to_owned(),
                    "Ljava/lang/String;".to_owned(),
                    "Landroid/content/Context;".to_owned(),
                    "Landroid/util/AttributeSet;".to_owned(),
                ],
            },
        };
        let Some(resolved) =
            self.resolve_exact_method_in_class_hierarchy(&context_descriptor, &on_create_view)?
        else {
            return Ok(None);
        };
        let args = vec![
            context.clone(),
            parent.cloned().unwrap_or(DexValue::Null),
            DexValue::String(std::rc::Rc::from(tag_name.to_owned())),
            context,
            attrs.clone(),
        ];
        match self.execute_method(None, &resolved, Some(&args), depth + 1)? {
            DexExecutionResult::Value(view)
                if !matches!(view, DexValue::Null | DexValue::Uninitialized) =>
            {
                Ok(Some(view))
            }
            DexExecutionResult::Value(_) | DexExecutionResult::Void => Ok(None),
            DexExecutionResult::Throw(exception) => {
                let detail_field = DexFieldId {
                    class_descriptor: "Ljava/lang/Throwable;".to_owned(),
                    type_descriptor: "Ljava/lang/String;".to_owned(),
                    name: "detailMessage".to_owned(),
                };
                let cause_field = DexFieldId {
                    class_descriptor: "Ljava/lang/Throwable;".to_owned(),
                    type_descriptor: "Ljava/lang/Throwable;".to_owned(),
                    name: "cause".to_owned(),
                };
                let mut lines = vec![format!(
                    "LayoutInflater Factory2.onCreateView threw an exception\ntag: {tag_name}\nexception: {exception}"
                )];
                let mut current = exception;
                for depth_idx in 0..8 {
                    if let Some(detail) = self
                        .read_stored_instance_field(&current, &detail_field)
                        .and_then(|v| self.string_text(&v))
                    {
                        lines.push(format!("message[{depth_idx}]: {detail}"));
                    }
                    let Some(next) = self.read_stored_instance_field(&current, &cause_field) else {
                        break;
                    };
                    if matches!(next, DexValue::Null | DexValue::Uninitialized)
                        || map_identity_key(&next) == map_identity_key(&current)
                    {
                        break;
                    }
                    lines.push(format!("caused by: {next}"));
                    current = next;
                }
                Err(WinedroidError::new(lines.join("\n")))
            }
            DexExecutionResult::Int(_) | DexExecutionResult::Parked(_) => Ok(None),
        }
    }

    pub(in crate::dex_run) fn initialize_inflated_layout_root(
        &self,
        root: &DexValue,
        attrs: &DexValue,
        depth: usize,
    ) -> Result<()> {
        let Some(class_descriptor) = value_class_descriptor(root).map(str::to_owned) else {
            return Ok(());
        };
        let context = self.active_context_value();
        let constructors =
            inflated_view_constructor_candidates(&class_descriptor, root, &context, attrs);

        for (method, arguments) in constructors {
            let Some(resolved) = self.resolve_exact_method(&method)? else {
                continue;
            };
            let result = self.execute_method(None, &resolved, Some(&arguments), depth + 1)?;

            return match result {
                DexExecutionResult::Void => Ok(()),
                DexExecutionResult::Int(_)
                | DexExecutionResult::Value(_)
                | DexExecutionResult::Throw(_) => {
                    Err(WinedroidError::new(format!(
                        "inflated view constructor did not complete normally\nclass: {class_descriptor}\nmethod: {}",
                        resolved.method.display_signature()
                    )))
                }
                DexExecutionResult::Parked(execution) => {
                    self.discard_parked_execution(&execution);
                    Err(WinedroidError::new(format!(
                        "inflated view constructor did not complete normally\nclass: {class_descriptor}\nmethod: {}",
                        resolved.method.display_signature()
                    )))
                }
            };
        }

        // cite: frameworks-base LayoutInflater.java#createView, mConstructorSignature
        if self.load_class(&class_descriptor).is_some() {
            return Err(WinedroidError::new(format!(
                "inflated DEX view lacks required constructor\nclass: {class_descriptor}\nconstructor: (Landroid/content/Context;Landroid/util/AttributeSet;)V"
            )));
        }

        Ok(())
    }

    pub(in crate::dex_run) fn view_find_view_by_id(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let DexValue::Int(view_id) = invocation.arguments[1] else {
            return None;
        };

        self.runtime.borrow_mut().view_lookup_ids.push(view_id);
        if let Some(value) = self.find_registered_view_by_id(&invocation.arguments[0], view_id) {
            return Some(value);
        }
        if let Some(value) = self.find_view_in_subtree(&invocation.arguments[0], view_id, 0) {
            return Some(value);
        }
        if let Some(content_view) = self.dialog_stored_content_view(&invocation.arguments[0]) {
            if let Some(value) = self.find_registered_view_by_id(&content_view, view_id) {
                return Some(value);
            }
            if let Some(value) = self.find_view_in_subtree(&content_view, view_id, 0) {
                return Some(value);
            }
        }
        self.ensure_window_decor_tree();
        if let Some(value) = self.find_view_in_subtree(&decor_view_reference(), view_id, 0) {
            return Some(value);
        }

        // cite: frameworks-base View.java#findViewById, findViewTraversal
        Some(DexValue::Null)
    }

    // cite: frameworks-base View.java#findViewTraversal
    // cite: frameworks-base ViewGroup.java#findViewTraversal
    pub(in crate::dex_run) fn find_view_in_subtree(
        &self,
        view: &DexValue,
        view_id: i32,
        depth: usize,
    ) -> Option<DexValue> {
        if depth > 256 {
            return None;
        }
        if self.view_id_from_value(view) == Some(view_id) {
            return Some(view.clone());
        }
        let children = self
            .runtime
            .borrow()
            .view_group_children
            .get(&map_identity_key(view))
            .cloned()
            .unwrap_or_default();
        for child in children {
            if let Some(found) = self.find_view_in_subtree(&child, view_id, depth + 1) {
                return Some(found);
            }
        }
        None
    }

    // cite: frameworks-base PhoneWindow.java#generateLayout
    pub(in crate::dex_run) fn reset_window_decor_tree(&self) {
        let decor = decor_view_reference();
        let content = view_reference(ANDROID_CONTENT_VIEW_ID);
        self.clear_view_group_children(&decor);
        self.clear_view_group_children(&content);
        self.write_stored_instance_field(
            &content,
            &view_id_field(),
            DexValue::Int(ANDROID_CONTENT_VIEW_ID),
        );
        self.runtime.borrow_mut().decor_action_bar = None;
        self.ensure_window_decor_tree();
    }

    // cite: frameworks-base PhoneWindow.java#generateLayout
    // cite: frameworks-base themes_material.xml#windowActionBarFullscreenDecorLayout
    // cite: frameworks-base screen_toolbar.xml#action_bar
    pub(in crate::dex_run) fn ensure_window_decor_tree(&self) {
        let decor = decor_view_reference();
        self.ensure_decor_child(&decor, &view_reference(ANDROID_CONTENT_VIEW_ID));
        if self.window_has_decor_action_bar() {
            let toolbar = decor_action_bar_reference();
            self.write_stored_instance_field(
                &toolbar,
                &view_id_field(),
                DexValue::Int(ANDROID_ACTION_BAR_VIEW_ID),
            );
            self.ensure_decor_child(&decor, &toolbar);
        }
    }

    fn ensure_decor_child(&self, decor: &DexValue, child: &DexValue) {
        let decor_key = map_identity_key(decor);
        let child_key = map_identity_key(child);
        let already_attached = self
            .runtime
            .borrow()
            .view_group_children
            .get(&decor_key)
            .is_some_and(|children| {
                children
                    .iter()
                    .any(|existing| map_identity_key(existing) == child_key)
            });
        if already_attached {
            return;
        }
        self.append_view_child(decor, child);
    }

    fn window_has_decor_action_bar(&self) -> bool {
        if let Some(cached) = self.runtime.borrow().decor_action_bar {
            return cached;
        }
        let Some(apk_path) = self.apk_path.clone() else {
            return false;
        };
        let snapshot = self.view_snapshot();
        let value = crate::layout_render::activity_has_decor_action_bar(&apk_path, &snapshot);
        self.runtime.borrow_mut().decor_action_bar = Some(value);
        value
    }
}
