use super::super::*;

impl DexProgram {
    pub(super) fn invoke_android_widget_adapters_builtin(
        &self,
        method: &DexMethodId,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        // cite: frameworks-base View.java#View
        if is_bootstrap_view_constructor_class(&method.class_descriptor)
            && method.name == "<init>"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Landroid/content/Context;)V"
                    | "(Landroid/content/Context;Landroid/util/AttributeSet;)V"
                    | "(Landroid/content/Context;Landroid/util/AttributeSet;I)V"
                    | "(Landroid/content/Context;Landroid/util/AttributeSet;II)V"
            )
        {
            if let Some(invocation) = invocation {
                if let (Some(receiver), Some(attrs)) =
                    (invocation.arguments.first(), invocation.arguments.get(2))
                {
                    let attrs_key = map_identity_key(attrs);
                    let parsed_id =
                        self.runtime
                            .borrow()
                            .xml_parsers
                            .get(&attrs_key)
                            .and_then(|parser| match parser.position {
                                XmlParserPosition::Event(index) => match parser.events.get(index) {
                                    Some(crate::binary_xml::XmlEvent::Start(element)) => {
                                        xml_element_view_id(element)
                                    }
                                    _ => None,
                                },
                                _ => None,
                            });
                    if let Some(view_id) = parsed_id {
                        self.write_stored_instance_field(
                            receiver,
                            &view_id_field(),
                            DexValue::Int(view_id),
                        );
                    }
                }
            }
            return Some(DexExecutionResult::Void);
        }

        if is_bootstrap_view_constructor_class(&method.class_descriptor)
            && method.name == "onFinishInflate"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/BaseAdapter;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Landroid/widget/Adapter;" | "Landroid/widget/BaseAdapter;"
        ) && method.name == "getItemViewType"
            && method.proto.descriptor_is("(I)I")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Landroid/widget/Adapter;" | "Landroid/widget/BaseAdapter;"
        ) && method.name == "getViewTypeCount"
            && method.proto.descriptor_is("()I")
        {
            return Some(DexExecutionResult::Int(1));
        }

        if method.class_descriptor == "Landroid/widget/BaseAdapter;"
            && method.name == "hasStableIds"
            && method.proto.descriptor_is("()Z")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Landroid/widget/BaseAdapter;" | "Landroid/widget/Adapter;"
        ) && matches!(
            method.name.as_str(),
            "notifyDataSetChanged" | "notifyDataSetInvalidated"
        ) && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/database/DataSetObserver;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/database/DataSetObserver;"
            && matches!(method.name.as_str(), "onChanged" | "onInvalidated")
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Landroid/widget/Adapter;"
                | "Landroid/widget/ListAdapter;"
                | "Landroid/widget/BaseAdapter;"
        ) && matches!(
            method.name.as_str(),
            "registerDataSetObserver" | "unregisterDataSetObserver"
        ) && method
            .proto
            .descriptor_is("(Landroid/database/DataSetObserver;)V")
        {
            let invocation = invocation?;
            self.write_adapter_data_set_observer(invocation, &method.name)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/AbsListView;"
            && method.name == "setCacheColorHint"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.write_abs_list_view_cache_color_hint(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/AbsListView;"
            && method.name == "getCacheColorHint"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.read_abs_list_view_cache_color_hint(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/widget/AbsListView;"
            && method.name == "setOnScrollListener"
            && method
                .proto
                .descriptor_is("(Landroid/widget/AbsListView$OnScrollListener;)V")
        {
            let invocation = invocation?;
            self.write_abs_list_view_scroll_listener(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/AbsListView;"
            && matches!(
                method.name.as_str(),
                "getListPaddingLeft"
                    | "getListPaddingTop"
                    | "getListPaddingRight"
                    | "getListPaddingBottom"
            )
            && method.proto.descriptor_is("()I")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Landroid/widget/HorizontalScrollView;"
            && method.name == "fullScroll"
            && method.proto.descriptor_is("(I)Z")
        {
            let invocation = invocation?;
            let value = self.horizontal_scroll_view_full_scroll(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/widget/ListView;"
            && method.name == "setDividerHeight"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.write_list_view_divider_height(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/ListView;"
            && method.name == "getDividerHeight"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.read_list_view_divider_height(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/widget/ListView;"
            && method.name == "setDivider"
            && method
                .proto
                .descriptor_is("(Landroid/graphics/drawable/Drawable;)V")
        {
            let invocation = invocation?;
            self.write_list_view_divider(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/ListView;"
            && method.name == "getDivider"
            && method
                .proto
                .descriptor_is("()Landroid/graphics/drawable/Drawable;")
        {
            let invocation = invocation?;
            let value = self.read_list_view_divider(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Landroid/widget/AdapterView;"
                | "Landroid/widget/AbsListView;"
                | "Landroid/widget/ListView;"
        ) && method.name == "setAdapter"
            && method
                .proto
                .descriptor_is("(Landroid/widget/ListAdapter;)V")
        {
            let invocation = invocation?;
            self.write_adapter_view_adapter(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Landroid/widget/AdapterView;"
                | "Landroid/widget/AbsListView;"
                | "Landroid/widget/ListView;"
        ) && method.name == "getAdapter"
            && matches!(
                method.proto.descriptor().as_str(),
                "()Landroid/widget/Adapter;" | "()Landroid/widget/ListAdapter;"
            )
        {
            let invocation = invocation?;
            let value = self.read_adapter_view_adapter(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/widget/AdapterView;"
            && method.name == "setOnItemClickListener"
            && method
                .proto
                .descriptor_is("(Landroid/widget/AdapterView$OnItemClickListener;)V")
        {
            let invocation = invocation?;
            self.write_adapter_view_item_click_listener(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/AdapterView;"
            && method.name == "getOnItemClickListener"
            && method
                .proto
                .descriptor_is("()Landroid/widget/AdapterView$OnItemClickListener;")
        {
            let invocation = invocation?;
            let value = self.read_adapter_view_item_click_listener(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/widget/AdapterView;"
            && method.name == "setOnItemSelectedListener"
            && method
                .proto
                .descriptor_is("(Landroid/widget/AdapterView$OnItemSelectedListener;)V")
        {
            let invocation = invocation?;
            self.write_adapter_view_item_selected_listener(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/AdapterView;"
            && method.name == "getOnItemSelectedListener"
            && method
                .proto
                .descriptor_is("()Landroid/widget/AdapterView$OnItemSelectedListener;")
        {
            let invocation = invocation?;
            let value = self.read_adapter_view_item_selected_listener(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/widget/AdapterView;"
            && method.name == "setSelection"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.write_adapter_view_selected_position(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/AdapterView;"
            && method.name == "getSelectedItemPosition"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.read_adapter_view_selected_position(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/widget/PopupMenu;"
            && method.name == "<init>"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Landroid/content/Context;Landroid/view/View;)V"
                    | "(Landroid/content/Context;Landroid/view/View;I)V"
                    | "(Landroid/content/Context;Landroid/view/View;III)V"
            )
        {
            let invocation = invocation?;
            if let Some(anchor) = invocation.arguments.get(2) {
                self.write_stored_instance_field(
                    &invocation.arguments[0],
                    &popup_menu_anchor_field(),
                    anchor.clone(),
                );
            }
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupMenu;"
            && method.name == "getMenu"
            && method.proto.descriptor_is("()Landroid/view/Menu;")
        {
            let invocation = invocation?;
            return Some(DexExecutionResult::Value(popup_menu_menu_reference(
                &map_identity_key(&invocation.arguments[0]),
            )));
        }

        if method.class_descriptor == "Landroid/widget/PopupMenu;"
            && method.name == "getMenuInflater"
            && method.proto.descriptor_is("()Landroid/view/MenuInflater;")
        {
            return Some(DexExecutionResult::Value(
                self.allocate_object("Landroid/view/MenuInflater;".to_owned()),
            ));
        }

        if method.class_descriptor == "Landroid/widget/PopupMenu;"
            && method.name == "inflate"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.inflate_popup_menu(invocation)?;
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupMenu;"
            && method.name == "setOnMenuItemClickListener"
            && method
                .proto
                .descriptor_is("(Landroid/widget/PopupMenu$OnMenuItemClickListener;)V")
        {
            let invocation = invocation?;
            if let Some(listener) = invocation.arguments.get(1) {
                self.write_stored_instance_field(
                    &invocation.arguments[0],
                    &popup_menu_click_listener_field(),
                    listener.clone(),
                );
            }
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupMenu;"
            && method.name == "show"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.show_popup_menu(invocation)?;
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupMenu;"
            && matches!(
                (method.name.as_str(), method.proto.descriptor().as_str()),
                ("dismiss", "()V")
            )
        {
            let invocation = invocation?;
            self.dismiss_popup_menu(invocation)?;
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupMenu;"
            && matches!(
                (method.name.as_str(), method.proto.descriptor().as_str()),
                ("setForceShowIcon", "(Z)V")
                    | ("setGravity", "(I)V")
                    | (
                        "setOnDismissListener",
                        "(Landroid/widget/PopupMenu$OnDismissListener;)V"
                    )
            )
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "<init>"
            && method.proto.descriptor()
                == "(Landroid/content/Context;Landroid/util/AttributeSet;II)V"
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "setBackgroundDrawable"
            && method
                .proto
                .descriptor_is("(Landroid/graphics/drawable/Drawable;)V")
        {
            let invocation = invocation?;
            self.write_popup_window_background(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "getBackground"
            && method
                .proto
                .descriptor_is("()Landroid/graphics/drawable/Drawable;")
        {
            let invocation = invocation?;
            let value = self.read_popup_window_background(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "setInputMethodMode"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.write_popup_window_input_method_mode(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "getInputMethodMode"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.read_popup_window_input_method_mode(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "setWindowLayoutType"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.write_popup_window_window_layout_type(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "getWindowLayoutType"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.read_popup_window_window_layout_type(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "setIsClippedToScreen"
            && method.proto.descriptor_is("(Z)V")
        {
            let invocation = invocation?;
            self.write_popup_window_is_clipped_to_screen(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "setEpicenterBounds"
            && method.proto.descriptor_is("(Landroid/graphics/Rect;)V")
        {
            let invocation = invocation?;
            self.write_popup_window_epicenter_bounds(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "getEpicenterBounds"
            && method.proto.descriptor_is("()Landroid/graphics/Rect;")
        {
            let invocation = invocation?;
            let value = self.read_popup_window_epicenter_bounds(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "setContentView"
            && method.proto.descriptor_is("(Landroid/view/View;)V")
        {
            let invocation = invocation?;
            self.write_popup_window_content_view(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "getContentView"
            && method.proto.descriptor_is("()Landroid/view/View;")
        {
            let invocation = invocation?;
            let value = self.read_popup_window_content_view(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "getMaxAvailableHeight"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Landroid/view/View;)I" | "(Landroid/view/View;I)I" | "(Landroid/view/View;IZ)I"
            )
        {
            let invocation = invocation?;
            let value = self.popup_window_max_available_height(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && matches!(
                (method.name.as_str(), method.proto.descriptor().as_str()),
                ("showAsDropDown", "(Landroid/view/View;II)V")
                    | ("showAsDropDown", "(Landroid/view/View;III)V")
                    | ("showAtLocation", "(Landroid/view/View;III)V")
            )
        {
            let invocation = invocation?;
            self.show_popup_window(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "dismiss"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.dismiss_popup_window(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && matches!(
                (method.name.as_str(), method.proto.descriptor().as_str()),
                ("setFocusable", "(Z)V")
                    | ("setOutsideTouchable", "(Z)V")
                    | ("setClippingEnabled", "(Z)V")
                    | ("setSoftInputMode", "(I)V")
                    | ("setAnimationStyle", "(I)V")
                    | ("setWidth", "(I)V")
                    | ("setHeight", "(I)V")
                    | (
                        "setOnDismissListener",
                        "(Landroid/widget/PopupWindow$OnDismissListener;)V"
                    )
                    | (
                        "setTouchInterceptor",
                        "(Landroid/view/View$OnTouchListener;)V"
                    )
                    | ("update", "()V")
            )
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "setOverlapAnchor"
            && method.proto.descriptor_is("(Z)V")
        {
            let invocation = invocation?;
            self.write_popup_window_overlap_anchor(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/PopupWindow;"
            && method.name == "isShowing"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = self.popup_window_is_showing(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        None
    }
}
