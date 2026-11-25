use super::super::*;

impl DexProgram {
    pub(super) fn invoke_android_text_image_builtin(
        &self,
        method: &DexMethodId,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        // cite: frameworks-base ImageView.java#setAdjustViewBounds
        if method.class_descriptor == "Landroid/widget/ImageView;"
            && method.name == "setAdjustViewBounds"
            && method.proto.descriptor_is("(Z)V")
        {
            let invocation = invocation?;
            if invocation.arguments.len() != 2
                || !dex_value_is_object_like(&invocation.arguments[0])
            {
                return None;
            }
            let DexValue::Int(value) = invocation.arguments[1] else {
                return None;
            };
            self.write_stored_instance_field(
                &invocation.arguments[0],
                &image_view_adjust_view_bounds_field(),
                DexValue::Int(value),
            );

            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base ImageView.java#setMaxWidth
        if method.class_descriptor == "Landroid/widget/ImageView;"
            && method.name == "setMaxWidth"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            if invocation.arguments.len() != 2
                || !dex_value_is_object_like(&invocation.arguments[0])
            {
                return None;
            }
            let DexValue::Int(value) = invocation.arguments[1] else {
                return None;
            };
            self.write_stored_instance_field(
                &invocation.arguments[0],
                &image_view_max_width_field(),
                DexValue::Int(value),
            );

            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base ImageView.java#setMaxHeight
        if method.class_descriptor == "Landroid/widget/ImageView;"
            && method.name == "setMaxHeight"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            if invocation.arguments.len() != 2
                || !dex_value_is_object_like(&invocation.arguments[0])
            {
                return None;
            }
            let DexValue::Int(value) = invocation.arguments[1] else {
                return None;
            };
            self.write_stored_instance_field(
                &invocation.arguments[0],
                &image_view_max_height_field(),
                DexValue::Int(value),
            );

            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base ImageView.java#setScaleType
        if method.class_descriptor == "Landroid/widget/ImageView;"
            && method.name == "setScaleType"
            && method
                .proto
                .descriptor_is("(Landroid/widget/ImageView$ScaleType;)V")
        {
            let invocation = invocation?;
            if invocation.arguments.len() != 2
                || !dex_value_is_object_like(&invocation.arguments[0])
                || !dex_value_is_object_like(&invocation.arguments[1])
            {
                return None;
            }
            let field = image_view_scale_type_field();
            let changed = self.read_stored_instance_field(&invocation.arguments[0], &field)
                != Some(invocation.arguments[1].clone());
            self.write_stored_instance_field(
                &invocation.arguments[0],
                &field,
                invocation.arguments[1].clone(),
            );
            if changed {
                self.runtime.borrow_mut().request_layout_traversal();
            }

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/TextView;"
            && method.name == "setText"
            && method.proto.descriptor_is("(Ljava/lang/CharSequence;)V")
        {
            let invocation = invocation?;
            self.write_text_view_text(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Landroid/text/method/LinkMovementMethod;"
                | "Landroid/text/method/ScrollingMovementMethod;"
                | "Landroid/text/method/ArrowKeyMovementMethod;"
        ) && method.name == "getInstance"
            && method
                .proto
                .descriptor_is("()Landroid/text/method/MovementMethod;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from(format!("{}.instance", method.class_descriptor)),
            )));
        }

        if method.class_descriptor == "Landroid/widget/TextView;"
            && method.name == "setMovementMethod"
            && method
                .proto
                .descriptor_is("(Landroid/text/method/MovementMethod;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/text/Html;"
            && method.name == "fromHtml"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/lang/String;)Landroid/text/Spanned;"
                    | "(Ljava/lang/String;I)Landroid/text/Spanned;"
            )
        {
            let invocation = invocation?;
            let html = match invocation.arguments.first() {
                Some(DexValue::String(value)) => value.as_ref(),
                Some(DexValue::Null) | None => "",
                _ => return None,
            };

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(html_to_text(html)),
            )));
        }

        if method.class_descriptor == "Landroid/widget/TextView;"
            && method.name == "getText"
            && method.proto.descriptor_is("()Ljava/lang/CharSequence;")
        {
            let invocation = invocation?;
            let value = self.read_text_view_text(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/widget/TextView;"
            && method.name == "getPaint"
            && method.proto.descriptor_is("()Landroid/text/TextPaint;")
        {
            let invocation = invocation?;
            let value = text_view_get_paint(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/TextPaint;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.paint_init_with_flags(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/text/style/CharacterStyle;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Landroid/text/style/MetricAffectingSpan;"
                | "Landroid/text/style/CharacterStyle;"
                | "Landroid/text/style/ReplacementSpan;"
        ) && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/text/style/ForegroundColorSpan;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.initialize_foreground_color_span(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/text/style/ForegroundColorSpan;"
            && method.name == "getForegroundColor"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let color = self.foreground_color_span_color(invocation)?;

            return Some(DexExecutionResult::Int(color));
        }

        if method.class_descriptor == "Landroid/text/style/ForegroundColorSpan;"
            && method.name == "updateDrawState"
            && method.proto.descriptor_is("(Landroid/text/TextPaint;)V")
        {
            let invocation = invocation?;
            self.foreground_color_span_update_draw_state(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/text/style/BackgroundColorSpan;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.initialize_background_color_span(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/text/style/BackgroundColorSpan;"
            && method.name == "getBackgroundColor"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let color = self.background_color_span_color(invocation)?;

            return Some(DexExecutionResult::Int(color));
        }

        if method.class_descriptor == "Landroid/text/style/BackgroundColorSpan;"
            && method.name == "updateDrawState"
            && method.proto.descriptor_is("(Landroid/text/TextPaint;)V")
        {
            let invocation = invocation?;
            self.background_color_span_update_draw_state(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/text/SpannableString;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(Ljava/lang/CharSequence;)V")
        {
            let invocation = invocation?;
            self.initialize_spannable_string(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/text/Spannable;"
            && method.name == "setSpan"
            && method.proto.descriptor_is("(Ljava/lang/Object;III)V")
        {
            let invocation = invocation?;
            self.spannable_set_span(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/text/BoringLayout;"
            && method.name == "isBoring"
            && method.proto.descriptor()
                == "(Ljava/lang/CharSequence;Landroid/text/TextPaint;Landroid/text/TextDirectionHeuristic;ZLandroid/text/BoringLayout$Metrics;)Landroid/text/BoringLayout$Metrics;"
        {
            let invocation = invocation?;
            let value = self.boring_layout_is_boring(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/BoringLayout;"
            && method.name == "<init>"
            && method.proto.descriptor()
                == "(Ljava/lang/CharSequence;Landroid/text/TextPaint;ILandroid/text/Layout$Alignment;FFLandroid/text/BoringLayout$Metrics;ZLandroid/text/TextUtils$TruncateAt;IZ)V"
        {
            let invocation = invocation?;
            self.initialize_boring_layout(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/text/BoringLayout;"
            && method.name == "isFallbackLineSpacingEnabled"
            && method.proto.descriptor_is("()Z")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout;"
            && method.name == "isFallbackLineSpacingEnabled"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = self.text_layout_is_fallback_line_spacing_enabled(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "obtain"
            && method.proto.descriptor()
                == "(Ljava/lang/CharSequence;IILandroid/text/TextPaint;I)Landroid/text/StaticLayout$Builder;"
        {
            let invocation = invocation?;
            let value = self.static_layout_builder_obtain(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "setTextDirection"
            && method.proto.descriptor()
                == "(Landroid/text/TextDirectionHeuristic;)Landroid/text/StaticLayout$Builder;"
        {
            let invocation = invocation?;
            let value = self.static_layout_builder_set_object(
                invocation,
                "textDirection",
                "Landroid/text/TextDirectionHeuristic;",
            )?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "setAlignment"
            && method.proto.descriptor()
                == "(Landroid/text/Layout$Alignment;)Landroid/text/StaticLayout$Builder;"
        {
            let invocation = invocation?;
            let value = self.static_layout_builder_set_object(
                invocation,
                "alignment",
                "Landroid/text/Layout$Alignment;",
            )?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "setMaxLines"
            && method
                .proto
                .descriptor_is("(I)Landroid/text/StaticLayout$Builder;")
        {
            let invocation = invocation?;
            let value = self.static_layout_builder_set_int(invocation, "maxLines")?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "setEllipsize"
            && method.proto.descriptor()
                == "(Landroid/text/TextUtils$TruncateAt;)Landroid/text/StaticLayout$Builder;"
        {
            let invocation = invocation?;
            let value = self.static_layout_builder_set_nullable_object(
                invocation,
                "ellipsize",
                "Landroid/text/TextUtils$TruncateAt;",
            )?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "setEllipsizedWidth"
            && method
                .proto
                .descriptor_is("(I)Landroid/text/StaticLayout$Builder;")
        {
            let invocation = invocation?;
            let value = self.static_layout_builder_set_int(invocation, "ellipsizedWidth")?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "setLineSpacing"
            && method
                .proto
                .descriptor_is("(FF)Landroid/text/StaticLayout$Builder;")
        {
            let invocation = invocation?;
            let value = self.static_layout_builder_set_line_spacing(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "setIncludePad"
            && method
                .proto
                .descriptor_is("(Z)Landroid/text/StaticLayout$Builder;")
        {
            let invocation = invocation?;
            let value = self.static_layout_builder_set_bool(invocation, "includePad")?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "setBreakStrategy"
            && method
                .proto
                .descriptor_is("(I)Landroid/text/StaticLayout$Builder;")
        {
            let invocation = invocation?;
            let value = self.static_layout_builder_set_int(invocation, "breakStrategy")?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "setHyphenationFrequency"
            && method
                .proto
                .descriptor_is("(I)Landroid/text/StaticLayout$Builder;")
        {
            let invocation = invocation?;
            let value = self.static_layout_builder_set_int(invocation, "hyphenationFrequency")?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "setIndents"
            && method
                .proto
                .descriptor_is("([I[I)Landroid/text/StaticLayout$Builder;")
        {
            let invocation = invocation?;
            let value = self.static_layout_builder_set_indents(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "setJustificationMode"
            && method
                .proto
                .descriptor_is("(I)Landroid/text/StaticLayout$Builder;")
        {
            let invocation = invocation?;
            let value = self.static_layout_builder_set_int(invocation, "justificationMode")?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "setUseLineSpacingFromFallbacks"
            && method
                .proto
                .descriptor_is("(Z)Landroid/text/StaticLayout$Builder;")
        {
            let invocation = invocation?;
            let value =
                self.static_layout_builder_set_bool(invocation, "useLineSpacingFromFallbacks")?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "setLineBreakConfig"
            && method.proto.descriptor()
                == "(Landroid/graphics/text/LineBreakConfig;)Landroid/text/StaticLayout$Builder;"
        {
            let invocation = invocation?;
            let value = self.static_layout_builder_set_object(
                invocation,
                "lineBreakConfig",
                "Landroid/graphics/text/LineBreakConfig;",
            )?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "setUseBoundsForWidth"
            && method
                .proto
                .descriptor_is("(Z)Landroid/text/StaticLayout$Builder;")
        {
            let invocation = invocation?;
            let value = self.static_layout_builder_set_bool(invocation, "useBoundsForWidth")?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/StaticLayout$Builder;"
            && method.name == "build"
            && method.proto.descriptor_is("()Landroid/text/StaticLayout;")
        {
            let invocation = invocation?;
            let value = self.static_layout_builder_build(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/graphics/text/LineBreakConfig$Builder;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.initialize_line_break_config_builder(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/graphics/text/LineBreakConfig$Builder;"
            && method.name == "setLineBreakStyle"
            && method
                .proto
                .descriptor_is("(I)Landroid/graphics/text/LineBreakConfig$Builder;")
        {
            let invocation = invocation?;
            let value = self.line_break_config_builder_set_int(invocation, "lineBreakStyle")?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/graphics/text/LineBreakConfig$Builder;"
            && method.name == "setLineBreakWordStyle"
            && method
                .proto
                .descriptor_is("(I)Landroid/graphics/text/LineBreakConfig$Builder;")
        {
            let invocation = invocation?;
            let value = self.line_break_config_builder_set_int(invocation, "lineBreakWordStyle")?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/graphics/text/LineBreakConfig$Builder;"
            && method.name == "build"
            && method
                .proto
                .descriptor_is("()Landroid/graphics/text/LineBreakConfig;")
        {
            let invocation = invocation?;
            let value = self.line_break_config_builder_build(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/Layout$Alignment;"
            && method.name == "values"
            && method
                .proto
                .descriptor_is("()[Landroid/text/Layout$Alignment;")
        {
            let value = self.layout_alignment_values();

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getLineCount"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_line_count(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: frameworks-base Layout.java#DIR_LEFT_TO_RIGHT
        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getParagraphDirection"
            && method.proto.descriptor_is("(I)I")
        {
            return Some(DexExecutionResult::Int(1));
        }

        // cite: frameworks-base Layout.java#isRtlCharAt
        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "isRtlCharAt"
            && method.proto.descriptor_is("(I)Z")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getDesiredWidth"
            && method
                .proto
                .descriptor_is("(Ljava/lang/CharSequence;IILandroid/text/TextPaint;)F")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_desired_width(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Float(value)));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getHeight"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_height(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getLineStart"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_line_start(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getLineEnd"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_line_end(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getLineTop"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_line_top(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getLineForOffset"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_line_for_offset(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getLineForVertical"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_line_for_vertical(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getLineAscent"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_line_metric(invocation, "ascent")?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getLineDescent"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_line_metric(invocation, "descent")?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getLineBaseline"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_line_baseline(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getLineBottom"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_line_bottom(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getLineLeft"
            && method.proto.descriptor_is("(I)F")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_line_left(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Float(value)));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getPrimaryHorizontal"
            && method.proto.descriptor_is("(I)F")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_primary_horizontal(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Float(value)));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getOffsetForHorizontal"
            && method.proto.descriptor_is("(IF)I")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_offset_for_horizontal(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getEllipsisCount"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_ellipsis_count(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getTopPadding"
            && method.proto.descriptor_is("()I")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getBottomPadding"
            && method.proto.descriptor_is("()I")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getText"
            && method.proto.descriptor_is("()Ljava/lang/CharSequence;")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_text(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "draw"
            && method.proto.descriptor_is("(Landroid/graphics/Canvas;)V")
        {
            let invocation = invocation?;
            self.text_layout_draw(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/text/Layout;"
            && method.name == "getPaint"
            && method.proto.descriptor_is("()Landroid/text/TextPaint;")
        {
            let invocation = invocation?;
            let value = self.text_layout_get_paint(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/widget/EditText;"
            && method.name == "getText"
            && method.proto.descriptor_is("()Landroid/text/Editable;")
        {
            let invocation = invocation?;
            let value = self.read_text_view_text(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if is_text_view_class(&method.class_descriptor)
            && method.name == "length"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let text = self.read_text_view_text(invocation)?;
            let DexValue::String(text) = text else {
                return None;
            };
            let value = i32::try_from(text.encode_utf16().count()).ok()?;

            return Some(DexExecutionResult::Int(value));
        }

        if is_text_view_class(&method.class_descriptor)
            && method.name == "getSelectionStart"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.read_text_view_selection_start(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if is_text_view_class(&method.class_descriptor)
            && method.name == "getSelectionEnd"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.read_text_view_selection_end(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if is_text_view_class(&method.class_descriptor)
            && method.name == "setSelection"
            && matches!(method.proto.descriptor().as_str(), "(I)V" | "(II)V")
        {
            let invocation = invocation?;
            self.write_text_view_selection_from_invocation(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base TextView.java#getTransformationMethod
        if is_text_view_class(&method.class_descriptor)
            && method.name == "getTransformationMethod"
            && method
                .proto
                .descriptor_is("()Landroid/text/method/TransformationMethod;")
        {
            return Some(DexExecutionResult::Value(DexValue::Null));
        }

        // cite: frameworks-base TextView.java#setCompoundDrawablePadding
        if is_text_view_class(&method.class_descriptor)
            && method.name == "setCompoundDrawablePadding"
            && method.proto.descriptor_is("(I)V")
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base TextView.java#getCompoundDrawablesRelative, getCompoundDrawables
        if is_text_view_class(&method.class_descriptor)
            && matches!(
                method.name.as_str(),
                "getCompoundDrawablesRelative" | "getCompoundDrawables"
            )
            && method
                .proto
                .descriptor_is("()[Landroid/graphics/drawable/Drawable;")
        {
            let array = self.allocate_filled_array(
                "[Landroid/graphics/drawable/Drawable;".to_owned(),
                vec![
                    DexValue::Null,
                    DexValue::Null,
                    DexValue::Null,
                    DexValue::Null,
                ],
            );
            return Some(DexExecutionResult::Value(array));
        }

        // cite: frameworks-base TextView.java#getLayout
        if is_text_view_class(&method.class_descriptor)
            && method.name == "getLayout"
            && method.proto.descriptor_is("()Landroid/text/Layout;")
        {
            return Some(DexExecutionResult::Value(DexValue::Null));
        }

        // cite: frameworks-base TextView.java#getMaxLines
        if is_text_view_class(&method.class_descriptor)
            && method.name == "getMaxLines"
            && method.proto.descriptor_is("()I")
        {
            return Some(DexExecutionResult::Int(i32::MAX));
        }

        // cite: frameworks-base TextView.java#setTextColor
        if is_text_view_class(&method.class_descriptor)
            && method.name == "setTextColor"
            && method
                .proto
                .descriptor_is("(Landroid/content/res/ColorStateList;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/TextView;"
            && method.name == "setCursorVisible"
            && method.proto.descriptor_is("(Z)V")
        {
            let invocation = invocation?;
            self.write_text_view_cursor_visible(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/TextView;"
            && method.name == "isCursorVisible"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = self.read_text_view_cursor_visible(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/widget/TextView;"
            && method.name == "setTypeface"
            && method.proto.descriptor_is("(Landroid/graphics/Typeface;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base Typeface.java#defaultFromStyle
        if method.class_descriptor == "Landroid/graphics/Typeface;"
            && method.name == "defaultFromStyle"
            && method.proto.descriptor_is("(I)Landroid/graphics/Typeface;")
        {
            let invocation = invocation?;
            let DexValue::Int(style) = invocation.arguments.first()? else {
                return None;
            };

            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from(format!(
                    "Landroid/graphics/Typeface;:family/sans-serif:style/{style}"
                )),
            )));
        }

        if method.class_descriptor == "Landroid/graphics/Typeface;"
            && method.name == "create"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;I)Landroid/graphics/Typeface;")
        {
            let invocation = invocation?;
            let value = typeface_create_from_family(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/graphics/Typeface;"
            && method.name == "create"
            && method.proto.descriptor()
                == "(Landroid/graphics/Typeface;IZ)Landroid/graphics/Typeface;"
        {
            let invocation = invocation?;
            let value = typeface_create_from_typeface(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/widget/TextView;"
            && method.name == "setEnabled"
            && method.proto.descriptor_is("(Z)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/TextView;"
            && method.name == "setMinWidth"
            && method.proto.descriptor_is("(I)V")
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base TextView.java#setWidth
        if method.class_descriptor == "Landroid/widget/TextView;"
            && method.name == "setWidth"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.write_view_int_field_and_request_layout(invocation, &text_view_width_field())?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/TextView;"
            && method.name == "setTextSize"
            && method.proto.descriptor_is("(IF)V")
        {
            let invocation = invocation?;
            self.write_text_view_text_size(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base TextView.java#setTextSize
        if method.class_descriptor == "Landroid/widget/TextView;"
            && method.name == "setTextSize"
            && method.proto.descriptor_is("(F)V")
        {
            let invocation = invocation?;
            if invocation.arguments.len() != 2 {
                return None;
            }
            const COMPLEX_UNIT_SP: i32 = 2;
            let sp_invocation = DexInvocation {
                arguments: smallvec::smallvec![
                    invocation.arguments[0].clone(),
                    DexValue::Int(COMPLEX_UNIT_SP),
                    invocation.arguments[1].clone(),
                ],
                ..invocation.clone()
            };
            self.write_text_view_text_size(&sp_invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if is_text_view_class(&method.class_descriptor)
            && method.name == "setTextColor"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.write_text_view_text_color(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/TextView;"
            && method.name == "setTextAppearance"
            && matches!(
                method.proto.descriptor().as_str(),
                "(I)V" | "(Landroid/content/Context;I)V"
            )
        {
            let invocation = invocation?;
            self.write_text_view_text_appearance(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/TextView;"
            && method.name == "setShowSoftInputOnFocus"
            && method.proto.descriptor_is("(Z)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/widget/TextView;"
            && method.name == "addTextChangedListener"
            && method.proto.descriptor_is("(Landroid/text/TextWatcher;)V")
        {
            let invocation = invocation?;
            self.write_text_view_text_watcher(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/graphics/Bitmap;"
            && method.name == "createBitmap"
            && method.proto.descriptor()
                == "(Landroid/util/DisplayMetrics;IILandroid/graphics/Bitmap$Config;ZLandroid/graphics/ColorSpace;)Landroid/graphics/Bitmap;"
        {
            let invocation = invocation?;
            let value = self.bitmap_create_bitmap(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature("Landroid/graphics/Bitmap;", "prepareToDraw", "()V") {
            let invocation = invocation?;
            self.bitmap_prepare_to_draw(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/graphics/Bitmap;"
            && matches!(method.name.as_str(), "getWidth" | "getHeight")
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.bitmap_get_dimension(invocation, &method.name)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Landroid/graphics/Bitmap;",
            "getConfig",
            "()Landroid/graphics/Bitmap$Config;",
        ) {
            let invocation = invocation?;
            let value = self.bitmap_get_config(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature(
            "Landroid/graphics/drawable/BitmapDrawable;",
            "getBitmap",
            "()Landroid/graphics/Bitmap;",
        ) {
            let invocation = invocation?;
            let value = self.bitmap_drawable_get_bitmap(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if is_image_view_compatible_class(&method.class_descriptor)
            && method.name == "setVisibility"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.write_view_visibility(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if is_image_view_compatible_class(&method.class_descriptor)
            && method.name == "setImageResource"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.write_image_view_resource(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if is_image_view_compatible_class(&method.class_descriptor)
            && method.name == "setImageDrawable"
            && method
                .proto
                .descriptor_is("(Landroid/graphics/drawable/Drawable;)V")
        {
            let invocation = invocation?;
            self.write_image_view_drawable(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if is_image_view_compatible_class(&method.class_descriptor)
            && method.name == "getDrawable"
            && method
                .proto
                .descriptor_is("()Landroid/graphics/drawable/Drawable;")
        {
            let invocation = invocation?;
            let value = self.read_image_view_drawable(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if is_image_view_compatible_class(&method.class_descriptor)
            && method.name == "setImageTintList"
            && method
                .proto
                .descriptor_is("(Landroid/content/res/ColorStateList;)V")
        {
            let invocation = invocation?;
            self.write_image_view_tint_list(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if is_image_view_compatible_class(&method.class_descriptor)
            && method.name == "getImageTintList"
            && method
                .proto
                .descriptor_is("()Landroid/content/res/ColorStateList;")
        {
            let invocation = invocation?;
            let value = self.read_image_view_tint_list(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if is_image_view_compatible_class(&method.class_descriptor)
            && method.name == "setImageTintMode"
            && method
                .proto
                .descriptor_is("(Landroid/graphics/PorterDuff$Mode;)V")
        {
            let invocation = invocation?;
            self.write_image_view_tint_mode(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if is_image_view_compatible_class(&method.class_descriptor)
            && method.name == "getImageTintMode"
            && method
                .proto
                .descriptor_is("()Landroid/graphics/PorterDuff$Mode;")
        {
            let invocation = invocation?;
            let value = self.read_image_view_tint_mode(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        None
    }
}

fn typeface_create_from_family(invocation: &DexInvocation) -> Option<DexValue> {
    let [DexValue::String(family), DexValue::Int(style)] = invocation.arguments.as_slice() else {
        return None;
    };

    Some(DexValue::Reference(std::rc::Rc::from(format!(
        "Landroid/graphics/Typeface;:family/{family}:style/{style}"
    ))))
}

fn typeface_create_from_typeface(invocation: &DexInvocation) -> Option<DexValue> {
    let [base, DexValue::Int(weight), DexValue::Int(italic)] = invocation.arguments.as_slice()
    else {
        return None;
    };

    if !dex_value_is_object_like(base) {
        return None;
    }

    Some(DexValue::Reference(std::rc::Rc::from(format!(
        "Landroid/graphics/Typeface;:base/{}:weight/{weight}:italic/{}",
        map_identity_key(base),
        i32::from(*italic != 0)
    ))))
}

fn html_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut chars = html.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                let mut tag = String::new();
                for tag_ch in chars.by_ref() {
                    if tag_ch == '>' {
                        break;
                    }
                    tag.push(tag_ch);
                }
                let is_closing = tag.starts_with('/');
                let name: String = tag
                    .trim_start_matches('/')
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric())
                    .flat_map(char::to_lowercase)
                    .collect();
                match name.as_str() {
                    "br" | "p" | "div" | "tr" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "ul"
                    | "ol" | "table" | "blockquote" => out.push('\n'),
                    "li" if !is_closing => out.push_str("\n• "),
                    _ => {}
                }
            }
            '&' => {
                let mut entity = String::new();
                while let Some(&next) = chars.peek() {
                    if next == ';' {
                        chars.next();
                        break;
                    }
                    if entity.len() > 32 || next == '&' || next == '<' {
                        break;
                    }
                    entity.push(next);
                    chars.next();
                }
                out.push_str(&decode_html_entity(&entity));
            }
            _ => out.push(ch),
        }
    }

    let trimmed: Vec<&str> = out.lines().map(str::trim_end).collect();
    let mut result = String::with_capacity(out.len());
    let mut blank_run = 0;
    for line in trimmed {
        if line.is_empty() {
            blank_run += 1;
            if blank_run > 1 {
                continue;
            }
        } else {
            blank_run = 0;
        }
        result.push_str(line);
        result.push('\n');
    }

    result.trim().to_owned()
}

fn decode_html_entity(entity: &str) -> String {
    match entity {
        "amp" => "&".to_owned(),
        "lt" => "<".to_owned(),
        "gt" => ">".to_owned(),
        "quot" => "\"".to_owned(),
        "apos" | "#39" => "'".to_owned(),
        "nbsp" | "#160" => " ".to_owned(),
        other => {
            if let Some(rest) = other.strip_prefix('#') {
                let code = if let Some(hex) = rest.strip_prefix(['x', 'X']) {
                    u32::from_str_radix(hex, 16).ok()
                } else {
                    rest.parse::<u32>().ok()
                };
                if let Some(ch) = code.and_then(char::from_u32) {
                    return ch.to_string();
                }
            }
            format!("&{other};")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::html_to_text;

    #[test]
    fn html_to_text_decodes_entities_and_breaks_blocks() {
        let rendered = html_to_text(
            "<h1>Title</h1><p>First &amp; second.</p>line<br>next<ul><li>one</li><li>two</li></ul>",
        );

        assert_eq!(
            rendered,
            "Title\n\nFirst & second.\nline\nnext\n\n• one\n• two"
        );
    }

    #[test]
    fn html_to_text_decodes_numeric_entities() {
        assert_eq!(html_to_text("a&#8226;b&#x2022;c"), "a•b•c");
    }

    #[test]
    fn html_to_text_preserves_unknown_entities_verbatim() {
        assert_eq!(html_to_text("x&unknownentity;y"), "x&unknownentity;y");
    }
}
