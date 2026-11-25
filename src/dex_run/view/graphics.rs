use super::helpers::*;
use super::*;

fn render_node_float_property_field(name: &str) -> Option<(DexFieldId, f32)> {
    let value = match name {
        "alpha" => (render_node_alpha_field(), 1.0),
        "scaleX" => (render_node_scale_x_field(), 1.0),
        "scaleY" => (render_node_scale_y_field(), 1.0),
        "elevation" => (render_node_elevation_field(), 0.0),
        "pivotX" => (render_node_pivot_x_field(), 0.0),
        "pivotY" => (render_node_pivot_y_field(), 0.0),
        "translationX" => (render_node_float_field("translationX"), 0.0),
        "translationY" => (render_node_float_field("translationY"), 0.0),
        "rotationZ" => (render_node_float_field("rotationZ"), 0.0),
        "rotationX" => (render_node_float_field("rotationX"), 0.0),
        "rotationY" => (render_node_float_field("rotationY"), 0.0),
        "cameraDistance" => (render_node_float_field("cameraDistance"), 0.0),
        _ => return None,
    };

    Some(value)
}

fn render_node_int_property_field(name: &str) -> Option<(DexFieldId, i32)> {
    let value = match name {
        "ambientShadowColor" => (
            render_node_int_field("ambientShadowColor"),
            0xff00_0000u32 as i32,
        ),
        "spotShadowColor" => (
            render_node_int_field("spotShadowColor"),
            0xff00_0000u32 as i32,
        ),
        _ => return None,
    };

    Some(value)
}

fn render_node_bool_property_field(name: &str) -> Option<(DexFieldId, i32)> {
    let value = match name {
        "clipToBounds" => (render_node_clip_to_bounds_field(), 1),
        "clipToOutline" => (render_node_clip_to_outline_field(), 0),
        "useCompositingLayer" => (render_node_use_compositing_layer_field(), 0),
        "hasOverlappingRendering" => (render_node_has_overlapping_rendering_field(), 1),
        _ => return None,
    };

    Some(value)
}

impl DexProgram {
    pub(in crate::dex_run) fn rect_set(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 5 {
            return None;
        }

        let DexValue::Int(left) = invocation.arguments[1] else {
            return None;
        };
        let DexValue::Int(top) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Int(right) = invocation.arguments[3] else {
            return None;
        };
        let DexValue::Int(bottom) = invocation.arguments[4] else {
            return None;
        };

        for (name, value) in [
            ("left", left),
            ("top", top),
            ("right", right),
            ("bottom", bottom),
        ] {
            if !self.write_stored_instance_field(
                &invocation.arguments[0],
                &rect_int_field(name),
                DexValue::Int(value),
            ) {
                return None;
            }
        }

        Some(())
    }

    pub(in crate::dex_run) fn rect_set_from_rect(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2
            || !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        for name in ["left", "top", "right", "bottom"] {
            let value = self
                .read_stored_instance_field(&invocation.arguments[1], &rect_int_field(name))
                .unwrap_or(DexValue::Int(0));
            let DexValue::Int(value) = value else {
                return None;
            };
            if !self.write_stored_instance_field(
                &invocation.arguments[0],
                &rect_int_field(name),
                DexValue::Int(value),
            ) {
                return None;
            }
        }

        Some(())
    }

    pub(in crate::dex_run) fn rect_set_empty(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        match &invocation.arguments[0] {
            DexValue::Object(_) | DexValue::Reference(_) => {}
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        }

        for name in ["left", "top", "right", "bottom"] {
            if !self.write_stored_instance_field(
                &invocation.arguments[0],
                &rect_int_field(name),
                DexValue::Int(0),
            ) {
                return None;
            }
        }

        Some(())
    }

    pub(in crate::dex_run) fn drawable_set_bounds_from_rect(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2
            || !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        for name in ["left", "top", "right", "bottom"] {
            let DexValue::Int(value) =
                self.read_stored_instance_field(&invocation.arguments[1], &rect_int_field(name))?
            else {
                return None;
            };
            if !self.write_stored_instance_field(
                &invocation.arguments[0],
                &drawable_bounds_int_field(name),
                DexValue::Int(value),
            ) {
                return None;
            }
        }

        Some(())
    }

    pub(in crate::dex_run) fn drawable_set_hotspot(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 3 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Float(x) = invocation.arguments[1] else {
            return None;
        };
        let DexValue::Float(y) = invocation.arguments[2] else {
            return None;
        };

        let wrote_x = self.write_stored_instance_field(
            &invocation.arguments[0],
            &drawable_hotspot_float_field("x"),
            DexValue::Float(x),
        );
        let wrote_y = self.write_stored_instance_field(
            &invocation.arguments[0],
            &drawable_hotspot_float_field("y"),
            DexValue::Float(y),
        );

        (wrote_x && wrote_y).then_some(())
    }

    pub(in crate::dex_run) fn drawable_set_state(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let new_values = match &invocation.arguments[1] {
            DexValue::Null | DexValue::Int(0) => Vec::new(),
            DexValue::Object(_) | DexValue::Reference(_) => {
                let array_key = map_identity_key(&invocation.arguments[1]);
                self.runtime.borrow().array_entries.get(&array_key)?.clone()
            }
            DexValue::Uninitialized
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        };

        if !new_values
            .iter()
            .all(|value| matches!(value, DexValue::Int(_)))
        {
            return None;
        }

        let old_values = self
            .read_stored_instance_field(&invocation.arguments[0], &drawable_state_field())
            .and_then(|state_array| {
                let state_key = map_identity_key(&state_array);
                self.runtime.borrow().array_entries.get(&state_key).cloned()
            })
            .unwrap_or_default();
        let changed = old_values != new_values;
        let state_array = self.allocate_filled_array("[I".to_owned(), new_values);
        if !self.write_stored_instance_field(
            &invocation.arguments[0],
            &drawable_state_field(),
            state_array,
        ) {
            return None;
        }

        Some(i32::from(changed))
    }

    pub(in crate::dex_run) fn drawable_set_visible(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 3 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(visible) = invocation.arguments[1] else {
            return None;
        };
        let DexValue::Int(restart) = invocation.arguments[2] else {
            return None;
        };
        if !matches!(visible, 0 | 1) || !matches!(restart, 0 | 1) {
            return None;
        }

        let old_visible = self
            .read_stored_instance_field(&invocation.arguments[0], &drawable_visible_field())
            .and_then(|value| match value {
                DexValue::Int(value) => Some(value),
                _ => None,
            })
            .unwrap_or(1);
        let changed = old_visible != visible;
        if !self.write_stored_instance_field(
            &invocation.arguments[0],
            &drawable_visible_field(),
            DexValue::Int(visible),
        ) {
            return None;
        }

        Some(i32::from(changed))
    }

    pub(in crate::dex_run) fn display_write_size(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        match &invocation.arguments[1] {
            DexValue::Object(_) | DexValue::Reference(_) => {}
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        }

        let display_size = self.display_size();
        // cite: frameworks-base Display.java#getSize, getRealSize
        for (name, value) in [
            ("x", display_size.width_i32()),
            ("y", display_size.height_i32()),
        ] {
            if !self.write_stored_instance_field(
                &invocation.arguments[1],
                &point_int_field(name),
                DexValue::Int(value),
            ) {
                return None;
            }
        }

        Some(())
    }

    pub(in crate::dex_run) fn initialize_matrix(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let matrix_key = map_identity_key(&invocation.arguments[0]);
        self.runtime
            .borrow_mut()
            .matrix_values
            .insert(matrix_key, AndroidMatrixState::identity());

        Some(())
    }

    pub(in crate::dex_run) fn initialize_rectf(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        for name in ["left", "top", "right", "bottom"] {
            if !self.write_stored_instance_field(
                &invocation.arguments[0],
                &rectf_float_field(name),
                DexValue::Float(0.0),
            ) {
                return None;
            }
        }

        Some(())
    }

    // cite: frameworks-base RectF.java#width, height
    pub(in crate::dex_run) fn rectf_dimension(
        &self,
        invocation: &DexInvocation,
        name: &str,
    ) -> Option<f32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let (max_name, min_name) = match name {
            "width" => ("right", "left"),
            "height" => ("bottom", "top"),
            _ => return None,
        };
        let mut edges = [0.0f32; 2];
        for (index, field_name) in [max_name, min_name].iter().enumerate() {
            let value = self
                .read_stored_instance_field(
                    &invocation.arguments[0],
                    &rectf_float_field(field_name),
                )
                .unwrap_or(DexValue::Float(0.0));
            let DexValue::Float(value) = value else {
                return None;
            };
            edges[index] = value;
        }

        Some(edges[0] - edges[1])
    }

    pub(in crate::dex_run) fn rectf_set(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 5 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let values = [
            ("left", canvas_float_argument(&invocation.arguments[1])?),
            ("top", canvas_float_argument(&invocation.arguments[2])?),
            ("right", canvas_float_argument(&invocation.arguments[3])?),
            ("bottom", canvas_float_argument(&invocation.arguments[4])?),
        ];
        for (name, value) in values {
            if !self.write_stored_instance_field(
                &invocation.arguments[0],
                &rectf_float_field(name),
                DexValue::Float(value),
            ) {
                return None;
            }
        }

        Some(())
    }

    pub(in crate::dex_run) fn matrix_translate(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let dx = canvas_float_argument(&invocation.arguments[1])?;
        let dy = canvas_float_argument(&invocation.arguments[2])?;
        self.translate_matrix(&invocation.arguments[0], dx, dy)?;

        Some(1)
    }

    // cite: frameworks-base Matrix.java#setTranslate
    pub(in crate::dex_run) fn matrix_set_translate(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let dx = canvas_float_argument(&invocation.arguments[1])?;
        let dy = canvas_float_argument(&invocation.arguments[2])?;
        let matrix_key = map_identity_key(&invocation.arguments[0]);
        self.runtime.borrow_mut().matrix_values.insert(
            matrix_key,
            AndroidMatrixState {
                values: [1.0, 0.0, dx, 0.0, 1.0, dy, 0.0, 0.0, 1.0],
            },
        );

        Some(())
    }

    pub(in crate::dex_run) fn matrix_reset(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let matrix_key = map_identity_key(&invocation.arguments[0]);
        self.runtime
            .borrow_mut()
            .matrix_values
            .insert(matrix_key, AndroidMatrixState::identity());

        Some(())
    }

    // cite: frameworks-base Matrix.java#setValues
    pub(in crate::dex_run) fn matrix_set_values(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        let array_key = map_identity_key(&invocation.arguments[1]);
        let source = self.runtime.borrow().array_entries.get(&array_key)?.clone();
        if source.len() < 9 {
            return None;
        }

        let mut values = [0.0f32; 9];
        for (index, slot) in values.iter_mut().enumerate() {
            *slot = match source[index] {
                DexValue::Float(value) => value,
                DexValue::Int(value) => value as f32,
                _ => return None,
            };
        }

        let matrix_key = map_identity_key(&invocation.arguments[0]);
        self.runtime
            .borrow_mut()
            .matrix_values
            .insert(matrix_key, AndroidMatrixState { values });

        Some(())
    }

    // cite: frameworks-base Path.java#transform
    pub(in crate::dex_run) fn path_transform(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        let matrix_key = map_identity_key(&invocation.arguments[1]);
        let matrix = self
            .runtime
            .borrow()
            .matrix_values
            .get(&matrix_key)
            .cloned()
            .unwrap_or_else(AndroidMatrixState::identity)
            .values;

        let path_key = map_identity_key(&invocation.arguments[0]);
        let mut runtime = self.runtime.borrow_mut();
        let Some(ops) = runtime.path_ops.get_mut(&path_key) else {
            return Some(());
        };
        for op in ops.iter_mut() {
            if !transform_path_op(op, &matrix) {
                return None;
            }
        }

        Some(())
    }

    pub(in crate::dex_run) fn matrix_get_values(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        let matrix_key = map_identity_key(&invocation.arguments[0]);
        let values = self
            .runtime
            .borrow()
            .matrix_values
            .get(&matrix_key)
            .cloned()
            .unwrap_or_else(AndroidMatrixState::identity)
            .values;
        let array_key = map_identity_key(&invocation.arguments[1]);
        let mut runtime = self.runtime.borrow_mut();
        let target = runtime.array_entries.get_mut(&array_key)?;
        if target.len() < values.len() {
            return None;
        }

        for (index, value) in values.iter().copied().enumerate() {
            target[index] = DexValue::Float(value);
        }

        Some(())
    }

    pub(in crate::dex_run) fn view_transform_matrix(
        &self,
        invocation: &DexInvocation,
        method_name: &str,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        let (dx, dy) = self.view_global_offset(&invocation.arguments[0]);
        let (dx, dy) = match method_name {
            "transformMatrixToGlobal" => (dx, dy),
            "transformMatrixToLocal" => (-dx, -dy),
            _ => return None,
        };

        self.translate_matrix(&invocation.arguments[1], dx as f32, dy as f32)
    }

    fn view_global_offset(&self, view: &DexValue) -> (i32, i32) {
        let mut current = Some(view.clone());
        let mut visited = Vec::new();
        let mut dx = 0_i32;
        let mut dy = 0_i32;

        while let Some(value) = current {
            let key = map_identity_key(&value);
            if visited.contains(&key) {
                break;
            }
            visited.push(key.clone());
            dx = dx.saturating_add(self.read_view_bound(&value, "left"));
            dy = dy.saturating_add(self.read_view_bound(&value, "top"));
            current = self.runtime.borrow().view_parents.get(&key).cloned();
        }

        (dx, dy)
    }

    fn translate_matrix(&self, matrix: &DexValue, dx: f32, dy: f32) -> Option<()> {
        let matrix_key = map_identity_key(matrix);
        let mut runtime = self.runtime.borrow_mut();
        let matrix = runtime
            .matrix_values
            .entry(matrix_key)
            .or_insert_with(AndroidMatrixState::identity);
        matrix.values[2] += dx;
        matrix.values[5] += dy;

        Some(())
    }

    pub(in crate::dex_run) fn path_set_fill_type(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[1]) {
            return None;
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &path_fill_type_field(),
            invocation.arguments[1].clone(),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn path_get_fill_type(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        Some(
            self.read_stored_instance_field(&invocation.arguments[0], &path_fill_type_field())
                .unwrap_or_else(|| {
                    DexValue::Reference(std::rc::Rc::from(
                        "Landroid/graphics/Path$FillType;.WINDING",
                    ))
                }),
        )
    }

    // cite: frameworks-base Path.java#computeBounds
    pub(in crate::dex_run) fn path_compute_bounds(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        let ops = self
            .runtime
            .borrow()
            .path_ops
            .get(&map_identity_key(&invocation.arguments[0]))
            .cloned()
            .unwrap_or_default();
        let mut points: Vec<(f32, f32)> = Vec::new();
        if !self.collect_path_op_points(&ops, &mut points) {
            return None;
        }

        let bounds = if points.is_empty() {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            let mut bounds = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
            for (x, y) in points {
                bounds.0 = bounds.0.min(x);
                bounds.1 = bounds.1.min(y);
                bounds.2 = bounds.2.max(x);
                bounds.3 = bounds.3.max(y);
            }
            bounds
        };

        for (name, value) in [
            ("left", bounds.0),
            ("top", bounds.1),
            ("right", bounds.2),
            ("bottom", bounds.3),
        ] {
            if !self.write_stored_instance_field(
                &invocation.arguments[1],
                &rectf_float_field(name),
                DexValue::Float(value),
            ) {
                return None;
            }
        }

        Some(())
    }

    fn collect_path_op_points(&self, ops: &[PathOp], points: &mut Vec<(f32, f32)>) -> bool {
        let mut current = (0.0f32, 0.0f32);
        for op in ops {
            match op {
                PathOp::MoveTo { x, y } | PathOp::LineTo { x, y } => {
                    current = (*x, *y);
                    points.push(current);
                }
                PathOp::RLineTo { dx, dy } => {
                    current = (current.0 + dx, current.1 + dy);
                    points.push(current);
                }
                PathOp::QuadTo { x1, y1, x2, y2 } => {
                    points.push((*x1, *y1));
                    current = (*x2, *y2);
                    points.push(current);
                }
                PathOp::CubicTo {
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                } => {
                    points.push((*x1, *y1));
                    points.push((*x2, *y2));
                    current = (*x3, *y3);
                    points.push(current);
                }
                PathOp::RCubicTo {
                    dx1,
                    dy1,
                    dx2,
                    dy2,
                    dx3,
                    dy3,
                } => {
                    points.push((current.0 + dx1, current.1 + dy1));
                    points.push((current.0 + dx2, current.1 + dy2));
                    current = (current.0 + dx3, current.1 + dy3);
                    points.push(current);
                }
                PathOp::Rect {
                    left,
                    top,
                    right,
                    bottom,
                    ..
                }
                | PathOp::RoundRect {
                    left,
                    top,
                    right,
                    bottom,
                    ..
                } => {
                    points.push((*left, *top));
                    points.push((*right, *bottom));
                    current = (*right, *bottom);
                }
                PathOp::BooleanOp { .. } => return false,
                PathOp::Close => {}
            }
        }

        true
    }

    // cite: frameworks-base Path.java#isEmpty
    pub(in crate::dex_run) fn path_is_empty(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let is_empty = self
            .runtime
            .borrow()
            .path_ops
            .get(&map_identity_key(&invocation.arguments[0]))
            .is_none_or(Vec::is_empty);

        Some(i32::from(is_empty))
    }

    pub(in crate::dex_run) fn path_rewind(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        self.runtime
            .borrow_mut()
            .path_ops
            .remove(&map_identity_key(&invocation.arguments[0]));

        Some(())
    }

    pub(in crate::dex_run) fn path_move_to(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let x = canvas_float_argument(&invocation.arguments[1])?;
        let y = canvas_float_argument(&invocation.arguments[2])?;
        self.runtime
            .borrow_mut()
            .path_ops
            .entry(map_identity_key(&invocation.arguments[0]))
            .or_default()
            .push(PathOp::MoveTo { x, y });

        Some(())
    }

    pub(in crate::dex_run) fn path_line_to(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let x = canvas_float_argument(&invocation.arguments[1])?;
        let y = canvas_float_argument(&invocation.arguments[2])?;
        self.runtime
            .borrow_mut()
            .path_ops
            .entry(map_identity_key(&invocation.arguments[0]))
            .or_default()
            .push(PathOp::LineTo { x, y });

        Some(())
    }

    pub(in crate::dex_run) fn path_r_line_to(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let dx = canvas_float_argument(&invocation.arguments[1])?;
        let dy = canvas_float_argument(&invocation.arguments[2])?;
        self.runtime
            .borrow_mut()
            .path_ops
            .entry(map_identity_key(&invocation.arguments[0]))
            .or_default()
            .push(PathOp::RLineTo { dx, dy });

        Some(())
    }

    pub(in crate::dex_run) fn path_quad_to(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 5 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let x1 = canvas_float_argument(&invocation.arguments[1])?;
        let y1 = canvas_float_argument(&invocation.arguments[2])?;
        let x2 = canvas_float_argument(&invocation.arguments[3])?;
        let y2 = canvas_float_argument(&invocation.arguments[4])?;
        self.runtime
            .borrow_mut()
            .path_ops
            .entry(map_identity_key(&invocation.arguments[0]))
            .or_default()
            .push(PathOp::QuadTo { x1, y1, x2, y2 });

        Some(())
    }

    // cite: frameworks-base Path.java#cubicTo, rCubicTo
    pub(in crate::dex_run) fn path_cubic_to(
        &self,
        invocation: &DexInvocation,
        relative: bool,
    ) -> Option<()> {
        if invocation.arguments.len() != 7 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let mut coordinates = [0.0f32; 6];
        for (index, coordinate) in coordinates.iter_mut().enumerate() {
            *coordinate = canvas_float_argument(&invocation.arguments[index + 1])?;
        }
        let op = if relative {
            PathOp::RCubicTo {
                dx1: coordinates[0],
                dy1: coordinates[1],
                dx2: coordinates[2],
                dy2: coordinates[3],
                dx3: coordinates[4],
                dy3: coordinates[5],
            }
        } else {
            PathOp::CubicTo {
                x1: coordinates[0],
                y1: coordinates[1],
                x2: coordinates[2],
                y2: coordinates[3],
                x3: coordinates[4],
                y3: coordinates[5],
            }
        };
        self.runtime
            .borrow_mut()
            .path_ops
            .entry(map_identity_key(&invocation.arguments[0]))
            .or_default()
            .push(op);

        Some(())
    }

    pub(in crate::dex_run) fn path_add_rect(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
            || !dex_value_is_object_like(&invocation.arguments[2])
        {
            return None;
        }

        // cite: frameworks-base Path.java#addRect
        let [left, top, right, bottom] = self.rectf_components(&invocation.arguments[1])?;
        self.runtime
            .borrow_mut()
            .path_ops
            .entry(map_identity_key(&invocation.arguments[0]))
            .or_default()
            .push(PathOp::Rect {
                left,
                top,
                right,
                bottom,
                clockwise: path_direction_is_clockwise(&invocation.arguments[2]),
            });

        Some(())
    }

    fn rectf_components(&self, rect: &DexValue) -> Option<[f32; 4]> {
        let mut components = [0.0f32; 4];
        for (index, name) in ["left", "top", "right", "bottom"].iter().enumerate() {
            let value = self
                .read_stored_instance_field(rect, &rectf_float_field(name))
                .unwrap_or(DexValue::Float(0.0));
            let DexValue::Float(value) = value else {
                return None;
            };
            components[index] = value;
        }

        Some(components)
    }

    pub(in crate::dex_run) fn path_op(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 4 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
            || !dex_value_is_object_like(&invocation.arguments[2])
            || !dex_value_is_object_like(&invocation.arguments[3])
        {
            return None;
        }

        let op = path_boolean_op_from_value(&invocation.arguments[3])?;
        let left_key = map_identity_key(&invocation.arguments[1]);
        let right_key = map_identity_key(&invocation.arguments[2]);
        let (left, right) = {
            let runtime = self.runtime.borrow();
            (
                runtime.path_ops.get(&left_key).cloned().unwrap_or_default(),
                runtime
                    .path_ops
                    .get(&right_key)
                    .cloned()
                    .unwrap_or_default(),
            )
        };

        self.runtime.borrow_mut().path_ops.insert(
            map_identity_key(&invocation.arguments[0]),
            vec![PathOp::BooleanOp { left, right, op }],
        );

        Some(1)
    }

    pub(in crate::dex_run) fn path_add_round_rect(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 4 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
            || !dex_value_is_object_like(&invocation.arguments[2])
            || !dex_value_is_object_like(&invocation.arguments[3])
        {
            return None;
        }

        let radii_key = map_identity_key(&invocation.arguments[2]);
        let radii_values = self.runtime.borrow().array_entries.get(&radii_key)?.clone();
        if radii_values.len() != 8 {
            return None;
        }

        let mut radii = Vec::with_capacity(8);
        for value in radii_values {
            radii.push(canvas_float_argument(&value)?);
        }

        // cite: frameworks-base Path.java#addRoundRect
        let [left, top, right, bottom] = self.rectf_components(&invocation.arguments[1])?;
        self.runtime
            .borrow_mut()
            .path_ops
            .entry(map_identity_key(&invocation.arguments[0]))
            .or_default()
            .push(PathOp::RoundRect {
                left,
                top,
                right,
                bottom,
                radii,
                clockwise: path_direction_is_clockwise(&invocation.arguments[3]),
            });

        Some(())
    }

    pub(in crate::dex_run) fn path_close(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        self.runtime
            .borrow_mut()
            .path_ops
            .entry(map_identity_key(&invocation.arguments[0]))
            .or_default()
            .push(PathOp::Close);

        Some(())
    }

    pub(in crate::dex_run) fn blend_mode_color_filter_init(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(color) = invocation.arguments[1] else {
            return None;
        };

        if !dex_value_is_object_like(&invocation.arguments[2]) {
            return None;
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &blend_mode_color_filter_color_field(),
            DexValue::Int(color),
        )
        .then_some(())?;
        self.write_stored_instance_field(
            &invocation.arguments[0],
            &blend_mode_color_filter_mode_field(),
            invocation.arguments[2].clone(),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn render_node_init(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let name = self.char_sequence_text(&invocation.arguments[1])?;
        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_name_field(),
            DexValue::String(std::rc::Rc::from(name)),
        )
        .then_some(())?;
        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_clip_to_bounds_field(),
            DexValue::Int(1),
        )
        .then_some(())?;
        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_has_overlapping_rendering_field(),
            DexValue::Int(1),
        )
        .then_some(())
    }

    // cite: frameworks-base RenderNode.java#beginRecording
    pub(in crate::dex_run) fn render_node_begin_recording(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 1 && invocation.arguments.len() != 3 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        if self
            .read_stored_instance_field(
                &invocation.arguments[0],
                &render_node_recording_canvas_field(),
            )
            .is_some_and(|value| dex_value_is_object_like(&value))
        {
            return None;
        }

        let (width, height) = if invocation.arguments.len() == 3 {
            let DexValue::Int(width) = invocation.arguments[1] else {
                return None;
            };
            let DexValue::Int(height) = invocation.arguments[2] else {
                return None;
            };
            (width.max(0), height.max(0))
        } else {
            (
                self.render_node_dimension(&invocation.arguments[0], "getWidth"),
                self.render_node_dimension(&invocation.arguments[0], "getHeight"),
            )
        };
        let canvas = self.allocate_object("Landroid/graphics/RecordingCanvas;".to_owned());
        self.write_stored_instance_field(
            &canvas,
            &recording_canvas_render_node_field(),
            invocation.arguments[0].clone(),
        )
        .then_some(())?;
        self.write_stored_instance_field(
            &canvas,
            &recording_canvas_int_field("width"),
            DexValue::Int(width),
        )
        .then_some(())?;
        self.write_stored_instance_field(
            &canvas,
            &recording_canvas_int_field("height"),
            DexValue::Int(height),
        )
        .then_some(())?;
        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_recording_canvas_field(),
            canvas.clone(),
        )
        .then_some(canvas)
    }

    // cite: frameworks-base RenderNode.java#endRecording
    pub(in crate::dex_run) fn render_node_end_recording(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let canvas = self.read_stored_instance_field(
            &invocation.arguments[0],
            &render_node_recording_canvas_field(),
        )?;
        if !dex_value_is_object_like(&canvas) {
            return None;
        }
        let canvas_key = map_identity_key(&canvas);
        let render_node_key = map_identity_key(&invocation.arguments[0]);
        let draw_ops = self
            .runtime
            .borrow()
            .canvas_draw_ops
            .get(&canvas_key)
            .cloned()
            .unwrap_or_default();

        {
            let mut runtime = self.runtime.borrow_mut();
            runtime
                .render_node_draw_ops
                .insert(render_node_key.clone(), draw_ops);
            *runtime
                .render_node_generations
                .entry(render_node_key)
                .or_insert(0) += 1;
        }
        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_recording_canvas_field(),
            DexValue::Null,
        )
        .then_some(())
    }

    // cite: frameworks-base RenderNode.java#discardDisplayList
    pub(in crate::dex_run) fn render_node_discard_display_list(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let render_node_key = map_identity_key(&invocation.arguments[0]);
        self.runtime
            .borrow_mut()
            .render_node_draw_ops
            .remove(&render_node_key);
        self.runtime
            .borrow_mut()
            .render_node_generations
            .remove(&render_node_key);

        Some(())
    }

    // cite: frameworks-base RenderNode.java#hasDisplayList
    pub(in crate::dex_run) fn render_node_has_display_list(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let render_node_key = map_identity_key(&invocation.arguments[0]);
        Some(i32::from(
            self.runtime
                .borrow()
                .render_node_draw_ops
                .contains_key(&render_node_key),
        ))
    }

    // cite: frameworks-base RenderNode.java#getLeft, getTop, getRight, getBottom, getWidth, getHeight
    pub(in crate::dex_run) fn render_node_get_position_dimension(
        &self,
        invocation: &DexInvocation,
        getter: &str,
    ) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        Some(self.render_node_dimension(&invocation.arguments[0], getter))
    }

    // cite: frameworks-base RenderNode.java#getUniqueId
    pub(in crate::dex_run) fn render_node_get_unique_id(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i64> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let key = map_identity_key(&invocation.arguments[0]);
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for byte in key.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }

        Some((hash & 0x7fff_ffff_ffff_ffff) as i64)
    }

    // cite: frameworks-base RenderNode.java#offsetLeftAndRight, offsetTopAndBottom
    pub(in crate::dex_run) fn render_node_offset_position(
        &self,
        invocation: &DexInvocation,
        horizontal: bool,
    ) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(offset) = invocation.arguments[1] else {
            return None;
        };
        if offset == 0 {
            return Some(0);
        }

        let fields = if horizontal {
            ["positionLeft", "positionRight"]
        } else {
            ["positionTop", "positionBottom"]
        };
        for field in fields {
            let value = self
                .render_node_position_value(&invocation.arguments[0], field)
                .saturating_add(offset);
            if !self.write_stored_instance_field(
                &invocation.arguments[0],
                &render_node_position_field(field),
                DexValue::Int(value),
            ) {
                return None;
            }
        }

        Some(1)
    }

    // cite: frameworks-base RenderNode.java#getMatrix, getInverseMatrix
    pub(in crate::dex_run) fn render_node_get_matrix(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        let matrix_key = map_identity_key(&invocation.arguments[1]);
        self.runtime
            .borrow_mut()
            .matrix_values
            .insert(matrix_key, AndroidMatrixState::identity());

        Some(())
    }

    // cite: frameworks-base RenderNode.java#getAlpha, getScaleX, getScaleY, getElevation, getPivotX, getPivotY, getTranslationX, getTranslationY, getRotationZ, getRotationX, getRotationY, getCameraDistance
    pub(in crate::dex_run) fn render_node_get_float_property(
        &self,
        invocation: &DexInvocation,
        name: &str,
    ) -> Option<f32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let (_, default) = render_node_float_property_field(name)?;
        Some(self.render_node_float_property_value(&invocation.arguments[0], name, default))
    }

    // cite: frameworks-base RenderNode.java#setTranslationX, setTranslationY, setRotationZ, setRotationX, setRotationY, setCameraDistance
    pub(in crate::dex_run) fn render_node_set_float_property(
        &self,
        invocation: &DexInvocation,
        name: &str,
    ) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let value = canvas_float_argument(&invocation.arguments[1])?;
        if name == "cameraDistance" && (!value.is_finite() || value < 0.0) {
            return None;
        }
        let (field, default) = render_node_float_property_field(name)?;
        let previous =
            self.render_node_float_property_value(&invocation.arguments[0], name, default);

        self.write_stored_instance_field(&invocation.arguments[0], &field, DexValue::Float(value))
            .then_some(i32::from(previous != value))
    }

    // cite: frameworks-base RenderNode.java#getAmbientShadowColor, getSpotShadowColor
    pub(in crate::dex_run) fn render_node_get_int_property(
        &self,
        invocation: &DexInvocation,
        name: &str,
    ) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let (field, default) = render_node_int_property_field(name)?;
        let value = self
            .read_stored_instance_field(&invocation.arguments[0], &field)
            .and_then(|field_value| match field_value {
                DexValue::Int(value) => Some(value),
                _ => None,
            })
            .unwrap_or(default);

        Some(value)
    }

    // cite: frameworks-base RenderNode.java#setAmbientShadowColor, setSpotShadowColor
    pub(in crate::dex_run) fn render_node_set_int_property(
        &self,
        invocation: &DexInvocation,
        name: &str,
    ) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(value) = invocation.arguments[1] else {
            return None;
        };
        let (field, default) = render_node_int_property_field(name)?;
        let previous = self
            .read_stored_instance_field(&invocation.arguments[0], &field)
            .and_then(|field_value| match field_value {
                DexValue::Int(value) => Some(value),
                _ => None,
            })
            .unwrap_or(default);

        self.write_stored_instance_field(&invocation.arguments[0], &field, DexValue::Int(value))
            .then_some(i32::from(previous != value))
    }

    // cite: frameworks-base RenderNode.java#getClipToBounds, getClipToOutline, getUseCompositingLayer, hasOverlappingRendering
    pub(in crate::dex_run) fn render_node_get_bool_property(
        &self,
        invocation: &DexInvocation,
        name: &str,
    ) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let (field, default) = render_node_bool_property_field(name)?;
        let value = self
            .read_stored_instance_field(&invocation.arguments[0], &field)
            .and_then(|field_value| match field_value {
                DexValue::Int(value) => Some(i32::from(value != 0)),
                _ => None,
            })
            .unwrap_or(default);

        Some(value)
    }

    pub(in crate::dex_run) fn render_node_set_clip_to_bounds(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(value) = invocation.arguments[1] else {
            return None;
        };
        let value = i32::from(value != 0);
        let previous = self
            .read_stored_instance_field(
                &invocation.arguments[0],
                &render_node_clip_to_bounds_field(),
            )
            .and_then(|field_value| match field_value {
                DexValue::Int(previous) => Some(previous),
                _ => None,
            })
            .unwrap_or(1);

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_clip_to_bounds_field(),
            DexValue::Int(value),
        )
        .then_some(i32::from(previous != value))
    }

    pub(in crate::dex_run) fn render_node_set_compositing_layer(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(value) = invocation.arguments[1] else {
            return None;
        };
        let value = i32::from(value != 0);
        let paint = nullable_object_argument(&invocation.arguments[2])?;
        let previous_value = self
            .read_stored_instance_field(
                &invocation.arguments[0],
                &render_node_use_compositing_layer_field(),
            )
            .and_then(|field_value| match field_value {
                DexValue::Int(previous) => Some(previous),
                _ => None,
            })
            .unwrap_or(0);
        let previous_paint = self
            .read_stored_instance_field(
                &invocation.arguments[0],
                &render_node_compositing_layer_paint_field(),
            )
            .unwrap_or(DexValue::Null);
        let changed = previous_value != value || previous_paint != paint;

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_use_compositing_layer_field(),
            DexValue::Int(value),
        )
        .then_some(())?;
        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_compositing_layer_paint_field(),
            paint,
        )
        .then_some(i32::from(changed))
    }

    pub(in crate::dex_run) fn render_node_set_clip_to_outline(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(value) = invocation.arguments[1] else {
            return None;
        };
        let value = i32::from(value != 0);
        let previous = self
            .read_stored_instance_field(
                &invocation.arguments[0],
                &render_node_clip_to_outline_field(),
            )
            .and_then(|field_value| match field_value {
                DexValue::Int(previous) => Some(previous),
                _ => None,
            })
            .unwrap_or(0);

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_clip_to_outline_field(),
            DexValue::Int(value),
        )
        .then_some(i32::from(previous != value))
    }

    pub(in crate::dex_run) fn render_node_set_elevation(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let elevation = match invocation.arguments[1] {
            DexValue::Float(value) => value,
            DexValue::Int(value) => f32::from_bits(value as u32),
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Long(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };
        let previous = self
            .read_stored_instance_field(&invocation.arguments[0], &render_node_elevation_field())
            .and_then(|field_value| match field_value {
                DexValue::Float(previous) => Some(previous),
                _ => None,
            })
            .unwrap_or(0.0);

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_elevation_field(),
            DexValue::Float(elevation),
        )
        .then_some(i32::from(previous != elevation))
    }

    pub(in crate::dex_run) fn render_node_set_alpha(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let alpha = match invocation.arguments[1] {
            DexValue::Float(value) => value,
            DexValue::Int(value) => f32::from_bits(value as u32),
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Long(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };
        let previous = self
            .read_stored_instance_field(&invocation.arguments[0], &render_node_alpha_field())
            .and_then(|field_value| match field_value {
                DexValue::Float(previous) => Some(previous),
                _ => None,
            })
            .unwrap_or(1.0);

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_alpha_field(),
            DexValue::Float(alpha),
        )
        .then_some(i32::from(previous != alpha))
    }

    pub(in crate::dex_run) fn render_node_set_scale_x(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let scale_x = match invocation.arguments[1] {
            DexValue::Float(value) => value,
            DexValue::Int(value) => f32::from_bits(value as u32),
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Long(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };
        let previous = self
            .read_stored_instance_field(&invocation.arguments[0], &render_node_scale_x_field())
            .and_then(|field_value| match field_value {
                DexValue::Float(previous) => Some(previous),
                _ => None,
            })
            .unwrap_or(1.0);

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_scale_x_field(),
            DexValue::Float(scale_x),
        )
        .then_some(i32::from(previous != scale_x))
    }

    pub(in crate::dex_run) fn render_node_set_scale_y(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let scale_y = match invocation.arguments[1] {
            DexValue::Float(value) => value,
            DexValue::Int(value) => f32::from_bits(value as u32),
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Long(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };
        let previous = self
            .read_stored_instance_field(&invocation.arguments[0], &render_node_scale_y_field())
            .and_then(|field_value| match field_value {
                DexValue::Float(previous) => Some(previous),
                _ => None,
            })
            .unwrap_or(1.0);

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_scale_y_field(),
            DexValue::Float(scale_y),
        )
        .then_some(i32::from(previous != scale_y))
    }

    pub(in crate::dex_run) fn render_node_set_position(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 5 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(left) = invocation.arguments[1] else {
            return None;
        };
        let DexValue::Int(top) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Int(right) = invocation.arguments[3] else {
            return None;
        };
        let DexValue::Int(bottom) = invocation.arguments[4] else {
            return None;
        };
        let mut changed = false;
        for (name, value) in [
            ("positionLeft", left),
            ("positionTop", top),
            ("positionRight", right),
            ("positionBottom", bottom),
        ] {
            let previous = self
                .read_stored_instance_field(
                    &invocation.arguments[0],
                    &render_node_position_field(name),
                )
                .and_then(|field_value| match field_value {
                    DexValue::Int(previous) => Some(previous),
                    _ => None,
                })
                .unwrap_or(0);
            if previous != value {
                changed = true;
            }
        }

        for (name, value) in [
            ("positionLeft", left),
            ("positionTop", top),
            ("positionRight", right),
            ("positionBottom", bottom),
        ] {
            if !self.write_stored_instance_field(
                &invocation.arguments[0],
                &render_node_position_field(name),
                DexValue::Int(value),
            ) {
                return None;
            }
        }

        Some(i32::from(changed))
    }

    pub(in crate::dex_run) fn render_node_set_pivot_x(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let pivot_x = match invocation.arguments[1] {
            DexValue::Float(value) => value,
            DexValue::Int(value) => f32::from_bits(value as u32),
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Long(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };
        let previous = self
            .read_stored_instance_field(&invocation.arguments[0], &render_node_pivot_x_field())
            .and_then(|field_value| match field_value {
                DexValue::Float(previous) => Some(previous),
                _ => None,
            })
            .unwrap_or(0.0);

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_pivot_x_field(),
            DexValue::Float(pivot_x),
        )
        .then_some(i32::from(previous != pivot_x))
    }

    pub(in crate::dex_run) fn render_node_set_pivot_y(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let pivot_y = match invocation.arguments[1] {
            DexValue::Float(value) => value,
            DexValue::Int(value) => f32::from_bits(value as u32),
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Long(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };
        let previous = self
            .read_stored_instance_field(&invocation.arguments[0], &render_node_pivot_y_field())
            .and_then(|field_value| match field_value {
                DexValue::Float(previous) => Some(previous),
                _ => None,
            })
            .unwrap_or(0.0);

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_pivot_y_field(),
            DexValue::Float(pivot_y),
        )
        .then_some(i32::from(previous != pivot_y))
    }

    pub(in crate::dex_run) fn render_node_set_has_overlapping_rendering(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(value) = invocation.arguments[1] else {
            return None;
        };
        let value = i32::from(value != 0);
        let previous = self
            .read_stored_instance_field(
                &invocation.arguments[0],
                &render_node_has_overlapping_rendering_field(),
            )
            .and_then(|field_value| match field_value {
                DexValue::Int(previous) => Some(previous),
                _ => None,
            })
            .unwrap_or(1);

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_has_overlapping_rendering_field(),
            DexValue::Int(value),
        )
        .then_some(i32::from(previous != value))
    }

    pub(in crate::dex_run) fn render_node_set_outline(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let outline = nullable_object_argument(&invocation.arguments[1])?;
        let previous = self
            .read_stored_instance_field(&invocation.arguments[0], &render_node_outline_field())
            .unwrap_or(DexValue::Null);
        let changed = previous != outline;

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &render_node_outline_field(),
            outline,
        )
        .then_some(i32::from(changed))
    }

    pub(in crate::dex_run) fn rect_dimension(
        &self,
        invocation: &DexInvocation,
        dimension: &str,
    ) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        match &invocation.arguments[0] {
            DexValue::Object(_) | DexValue::Reference(_) => {}
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        }

        let start_field = if dimension == "width" { "left" } else { "top" };
        let end_field = if dimension == "width" {
            "right"
        } else {
            "bottom"
        };
        let start = self
            .read_stored_instance_field(&invocation.arguments[0], &rect_int_field(start_field))
            .unwrap_or(DexValue::Int(0));
        let end = self
            .read_stored_instance_field(&invocation.arguments[0], &rect_int_field(end_field))
            .unwrap_or(DexValue::Int(0));
        let DexValue::Int(start) = start else {
            return None;
        };
        let DexValue::Int(end) = end else {
            return None;
        };

        Some(end.wrapping_sub(start))
    }

    pub(in crate::dex_run) fn canvas_save(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        self.canvas_push_save(&invocation.arguments[0])
    }

    pub(in crate::dex_run) fn canvas_save_layer(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 6 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        for argument in &invocation.arguments[1..5] {
            canvas_float_argument(argument)?;
        }
        if !dex_value_is_nullable_object_like(&invocation.arguments[5]) {
            return None;
        }

        self.canvas_push_save(&invocation.arguments[0])
    }

    fn canvas_push_save(&self, canvas: &DexValue) -> Option<i32> {
        let count = self
            .read_stored_instance_field(canvas, &canvas_save_count_field())
            .and_then(|field_value| match field_value {
                DexValue::Int(value) => Some(value),
                _ => None,
            })
            .unwrap_or(1);
        self.write_stored_instance_field(
            canvas,
            &canvas_save_count_field(),
            DexValue::Int(count.saturating_add(1)),
        );
        self.record_canvas_draw_op(canvas, CanvasDrawOp::Save);

        Some(count)
    }

    pub(in crate::dex_run) fn canvas_restore(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let count = self
            .read_stored_instance_field(&invocation.arguments[0], &canvas_save_count_field())
            .and_then(|field_value| match field_value {
                DexValue::Int(value) => Some(value),
                _ => None,
            })
            .unwrap_or(1);
        self.write_stored_instance_field(
            &invocation.arguments[0],
            &canvas_save_count_field(),
            DexValue::Int(count.saturating_sub(1).max(1)),
        );
        self.record_canvas_draw_op(&invocation.arguments[0], CanvasDrawOp::Restore);

        Some(())
    }

    pub(in crate::dex_run) fn canvas_init_with_bitmap(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &canvas_bitmap_field(),
            invocation.arguments[1].clone(),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn canvas_scale(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let scale_x = canvas_float_argument(&invocation.arguments[1])?;
        let scale_y = canvas_float_argument(&invocation.arguments[2])?;
        let current_scale_x =
            self.canvas_float_field_value(&invocation.arguments[0], "scaleX", 1.0);
        let current_scale_y =
            self.canvas_float_field_value(&invocation.arguments[0], "scaleY", 1.0);

        for (name, value) in [
            ("scaleX", current_scale_x * scale_x),
            ("scaleY", current_scale_y * scale_y),
        ] {
            if !self.write_stored_instance_field(
                &invocation.arguments[0],
                &canvas_float_field(name),
                DexValue::Float(value),
            ) {
                return None;
            }
        }

        self.record_canvas_draw_op(
            &invocation.arguments[0],
            CanvasDrawOp::Scale {
                sx: scale_x,
                sy: scale_y,
            },
        );

        Some(())
    }

    pub(in crate::dex_run) fn canvas_translate(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let dx = canvas_float_argument(&invocation.arguments[1])?;
        let dy = canvas_float_argument(&invocation.arguments[2])?;
        self.record_canvas_draw_op(&invocation.arguments[0], CanvasDrawOp::Translate { dx, dy });

        Some(())
    }

    pub(in crate::dex_run) fn canvas_rotate(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let degrees = canvas_float_argument(&invocation.arguments[1])?;
        self.record_canvas_draw_op(&invocation.arguments[0], CanvasDrawOp::Rotate { degrees });

        Some(())
    }

    pub(in crate::dex_run) fn canvas_is_hardware_accelerated(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let accelerated = self
            .read_stored_instance_field(
                &invocation.arguments[0],
                &recording_canvas_render_node_field(),
            )
            .is_some()
            || value_class_descriptor(&invocation.arguments[0])
                == Some("Landroid/graphics/RecordingCanvas;");

        Some(i32::from(accelerated))
    }

    pub(in crate::dex_run) fn canvas_get_clip_bounds(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        let (right, bottom) = self.canvas_clip_size(&invocation.arguments[0]);
        for (name, value) in [
            ("left", 0),
            ("top", 0),
            ("right", right),
            ("bottom", bottom),
        ] {
            if !self.write_stored_instance_field(
                &invocation.arguments[1],
                &rect_int_field(name),
                DexValue::Int(value),
            ) {
                return None;
            }
        }

        Some(i32::from(right > 0 && bottom > 0))
    }

    pub(in crate::dex_run) fn canvas_set_z_enabled(
        &self,
        invocation: &DexInvocation,
        enabled: bool,
    ) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &canvas_bool_field("zEnabled"),
            DexValue::Int(i32::from(enabled)),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn canvas_concat_matrix(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        self.record_canvas_draw_op(
            &invocation.arguments[0],
            CanvasDrawOp::ConcatMatrix {
                matrix: invocation.arguments[1].clone(),
            },
        );

        Some(())
    }

    pub(in crate::dex_run) fn canvas_clip_rect_region_op(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 6 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let left = canvas_float_argument(&invocation.arguments[1])?;
        let top = canvas_float_argument(&invocation.arguments[2])?;
        let right = canvas_float_argument(&invocation.arguments[3])?;
        let bottom = canvas_float_argument(&invocation.arguments[4])?;
        if !dex_value_is_object_like(&invocation.arguments[5]) {
            return None;
        }

        self.record_canvas_draw_op(
            &invocation.arguments[0],
            CanvasDrawOp::ClipRect {
                left,
                top,
                right,
                bottom,
                op: invocation.arguments[5].clone(),
            },
        );

        Some(i32::from(right > left && bottom > top))
    }

    pub(in crate::dex_run) fn canvas_clip_path_region_op(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
            || !dex_value_is_object_like(&invocation.arguments[2])
        {
            return None;
        }

        // cite: frameworks-base Canvas.java#clipPath
        let ops = self.current_path_ops(&invocation.arguments[1]);
        self.record_canvas_draw_op(
            &invocation.arguments[0],
            CanvasDrawOp::ClipPath {
                ops,
                op: invocation.arguments[2].clone(),
            },
        );

        Some(1)
    }

    fn current_path_ops(&self, path: &DexValue) -> Vec<PathOp> {
        self.runtime
            .borrow()
            .path_ops
            .get(&map_identity_key(path))
            .cloned()
            .unwrap_or_default()
    }

    pub(in crate::dex_run) fn canvas_draw_rect(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 6 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let left = canvas_float_argument(&invocation.arguments[1])?;
        let top = canvas_float_argument(&invocation.arguments[2])?;
        let right = canvas_float_argument(&invocation.arguments[3])?;
        let bottom = canvas_float_argument(&invocation.arguments[4])?;
        if !dex_value_is_object_like(&invocation.arguments[5]) {
            return None;
        }

        let paint = invocation.arguments[5].clone();
        self.record_canvas_draw_op(
            &invocation.arguments[0],
            CanvasDrawOp::Rect {
                left,
                top,
                right,
                bottom,
                paint,
            },
        );

        Some(())
    }

    pub(in crate::dex_run) fn canvas_draw_line(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 6 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let x1 = canvas_float_argument(&invocation.arguments[1])?;
        let y1 = canvas_float_argument(&invocation.arguments[2])?;
        let x2 = canvas_float_argument(&invocation.arguments[3])?;
        let y2 = canvas_float_argument(&invocation.arguments[4])?;
        if !dex_value_is_object_like(&invocation.arguments[5]) {
            return None;
        }

        let paint = invocation.arguments[5].clone();
        self.record_canvas_draw_op(
            &invocation.arguments[0],
            CanvasDrawOp::Line {
                x1,
                y1,
                x2,
                y2,
                paint,
            },
        );

        Some(())
    }

    pub(in crate::dex_run) fn canvas_draw_circle(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 5 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let cx = canvas_float_argument(&invocation.arguments[1])?;
        let cy = canvas_float_argument(&invocation.arguments[2])?;
        let radius = canvas_float_argument(&invocation.arguments[3])?;
        if !dex_value_is_object_like(&invocation.arguments[4]) {
            return None;
        }

        let paint = invocation.arguments[4].clone();
        self.record_canvas_draw_op(
            &invocation.arguments[0],
            CanvasDrawOp::Circle {
                cx,
                cy,
                radius,
                paint,
            },
        );

        Some(())
    }

    pub(in crate::dex_run) fn canvas_draw_round_rect(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 8 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let left = canvas_float_argument(&invocation.arguments[1])?;
        let top = canvas_float_argument(&invocation.arguments[2])?;
        let right = canvas_float_argument(&invocation.arguments[3])?;
        let bottom = canvas_float_argument(&invocation.arguments[4])?;
        let rx = canvas_float_argument(&invocation.arguments[5])?;
        let ry = canvas_float_argument(&invocation.arguments[6])?;
        if !dex_value_is_object_like(&invocation.arguments[7]) {
            return None;
        }

        let paint = invocation.arguments[7].clone();
        self.record_canvas_draw_op(
            &invocation.arguments[0],
            CanvasDrawOp::RoundRect {
                left,
                top,
                right,
                bottom,
                rx,
                ry,
                paint,
            },
        );

        Some(())
    }

    pub(in crate::dex_run) fn canvas_draw_path(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
            || !dex_value_is_object_like(&invocation.arguments[2])
        {
            return None;
        }

        // cite: frameworks-base Canvas.java#drawPath
        let ops = self.current_path_ops(&invocation.arguments[1]);
        self.record_canvas_draw_op(
            &invocation.arguments[0],
            CanvasDrawOp::Path {
                ops,
                paint: invocation.arguments[2].clone(),
            },
        );

        Some(())
    }

    pub(in crate::dex_run) fn canvas_draw_bitmap_rect(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 5 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
            || !dex_value_is_object_like(&invocation.arguments[3])
        {
            return None;
        }

        let source = nullable_object_argument(&invocation.arguments[2])?;
        let paint = nullable_object_argument(&invocation.arguments[4])?;
        self.record_canvas_draw_op(
            &invocation.arguments[0],
            CanvasDrawOp::Bitmap {
                bitmap: invocation.arguments[1].clone(),
                source,
                destination: invocation.arguments[3].clone(),
                paint,
            },
        );

        Some(())
    }

    pub(in crate::dex_run) fn canvas_draw_bitmap_at(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 5 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        let left = canvas_float_argument(&invocation.arguments[2])?;
        let top = canvas_float_argument(&invocation.arguments[3])?;
        let paint = nullable_object_argument(&invocation.arguments[4])?;
        let width = self.bitmap_int_value(&invocation.arguments[1], "width") as f32;
        let height = self.bitmap_int_value(&invocation.arguments[1], "height") as f32;
        let destination = self.allocate_object("Landroid/graphics/RectF;".to_owned());
        for (name, value) in [
            ("left", left),
            ("top", top),
            ("right", left + width),
            ("bottom", top + height),
        ] {
            if !self.write_stored_instance_field(
                &destination,
                &rectf_float_field(name),
                DexValue::Float(value),
            ) {
                return None;
            }
        }

        self.record_canvas_draw_op(
            &invocation.arguments[0],
            CanvasDrawOp::Bitmap {
                bitmap: invocation.arguments[1].clone(),
                source: DexValue::Null,
                destination,
                paint,
            },
        );

        Some(())
    }

    pub(in crate::dex_run) fn canvas_draw_render_node(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        self.record_canvas_draw_op(
            &invocation.arguments[0],
            CanvasDrawOp::RenderNode {
                render_node: invocation.arguments[1].clone(),
            },
        );

        Some(())
    }

    pub(in crate::dex_run::view) fn record_canvas_draw_op(
        &self,
        canvas: &DexValue,
        op: CanvasDrawOp,
    ) {
        let op = self.snapshot_canvas_draw_op_paint(op);
        let canvas_key = map_identity_key(canvas);
        let mut runtime = self.runtime.borrow_mut();
        runtime.active_canvas_key = Some(canvas_key.clone());
        runtime
            .canvas_draw_ops
            .entry(canvas_key)
            .or_default()
            .push(op);
    }

    fn snapshot_canvas_draw_op_paint(&self, op: CanvasDrawOp) -> CanvasDrawOp {
        match op {
            CanvasDrawOp::Line {
                x1,
                y1,
                x2,
                y2,
                paint,
            } => CanvasDrawOp::Line {
                x1,
                y1,
                x2,
                y2,
                paint: self.snapshot_draw_paint(paint),
            },
            CanvasDrawOp::Rect {
                left,
                top,
                right,
                bottom,
                paint,
            } => CanvasDrawOp::Rect {
                left,
                top,
                right,
                bottom,
                paint: self.snapshot_draw_paint(paint),
            },
            CanvasDrawOp::Circle {
                cx,
                cy,
                radius,
                paint,
            } => CanvasDrawOp::Circle {
                cx,
                cy,
                radius,
                paint: self.snapshot_draw_paint(paint),
            },
            CanvasDrawOp::RoundRect {
                left,
                top,
                right,
                bottom,
                rx,
                ry,
                paint,
            } => CanvasDrawOp::RoundRect {
                left,
                top,
                right,
                bottom,
                rx,
                ry,
                paint: self.snapshot_draw_paint(paint),
            },
            CanvasDrawOp::Path { ops, paint } => CanvasDrawOp::Path {
                ops,
                paint: self.snapshot_draw_paint(paint),
            },
            CanvasDrawOp::Bitmap {
                bitmap,
                source,
                destination,
                paint,
            } => CanvasDrawOp::Bitmap {
                bitmap,
                source,
                destination,
                paint: self.snapshot_nullable_draw_paint(paint),
            },
            CanvasDrawOp::Save
            | CanvasDrawOp::Restore
            | CanvasDrawOp::Translate { .. }
            | CanvasDrawOp::Scale { .. }
            | CanvasDrawOp::Rotate { .. }
            | CanvasDrawOp::ConcatMatrix { .. }
            | CanvasDrawOp::ClipRect { .. }
            | CanvasDrawOp::ClipPath { .. }
            | CanvasDrawOp::TextLayout { .. }
            | CanvasDrawOp::RenderNode { .. } => op,
        }
    }

    fn snapshot_nullable_draw_paint(&self, paint: DexValue) -> DexValue {
        if matches!(paint, DexValue::Null | DexValue::Int(0)) {
            return paint;
        }

        self.snapshot_draw_paint(paint)
    }

    fn snapshot_draw_paint(&self, paint: DexValue) -> DexValue {
        let fields = vec![
            paint_flags_field(),
            paint_hinting_field(),
            paint_color_field(),
            paint_color_long_field(),
            paint_alpha_field(),
            paint_text_size_field(),
            paint_text_scale_x_field(),
            paint_letter_spacing_field(),
            paint_stroke_width_field(),
            paint_stroke_miter_field(),
            paint_stroke_cap_field(),
            paint_stroke_join_field(),
            paint_shader_field(),
            paint_style_field(),
            paint_blend_mode_field(),
            paint_color_filter_field(),
            paint_typeface_field(),
        ];
        let values = fields
            .into_iter()
            .filter_map(|field| {
                self.read_stored_instance_field(&paint, &field)
                    .map(|value| (field, value))
            })
            .collect::<Vec<_>>();

        if values.is_empty() {
            return paint;
        }

        let snapshot = self.allocate_object("Landroid/graphics/Paint;".to_owned());
        for (field, value) in values {
            let wrote = self.write_stored_instance_field(&snapshot, &field, value);
            debug_assert!(wrote);
        }

        snapshot
    }

    pub(in crate::dex_run::view) fn canvas_clip_size(&self, canvas: &DexValue) -> (i32, i32) {
        if let Some(bitmap) = self.read_stored_instance_field(canvas, &canvas_bitmap_field()) {
            return (
                self.bitmap_int_value(&bitmap, "width"),
                self.bitmap_int_value(&bitmap, "height"),
            );
        }

        if self
            .read_stored_instance_field(canvas, &recording_canvas_render_node_field())
            .is_some()
        {
            let width = self.recording_canvas_int_value(canvas, "width");
            let height = self.recording_canvas_int_value(canvas, "height");
            if width > 0 || height > 0 {
                return (width, height);
            }
        }

        let Some(render_node) =
            self.read_stored_instance_field(canvas, &recording_canvas_render_node_field())
        else {
            // cite: frameworks-base Canvas.java#Canvas, getWidth, getHeight, getClipBounds
            return (0, 0);
        };

        let left = self.render_node_position_value(&render_node, "positionLeft");
        let top = self.render_node_position_value(&render_node, "positionTop");
        let right = self.render_node_position_value(&render_node, "positionRight");
        let bottom = self.render_node_position_value(&render_node, "positionBottom");

        (
            right.saturating_sub(left).max(0),
            bottom.saturating_sub(top).max(0),
        )
    }

    pub(in crate::dex_run::view) fn bitmap_int_value(&self, bitmap: &DexValue, name: &str) -> i32 {
        let value = self
            .read_stored_instance_field(bitmap, &bitmap_int_field(name))
            .unwrap_or(DexValue::Int(0));
        let DexValue::Int(value) = value else {
            return 0;
        };

        value.max(0)
    }

    fn recording_canvas_int_value(&self, canvas: &DexValue, name: &str) -> i32 {
        let value = self
            .read_stored_instance_field(canvas, &recording_canvas_int_field(name))
            .unwrap_or(DexValue::Int(0));
        let DexValue::Int(value) = value else {
            return 0;
        };

        value.max(0)
    }

    pub(in crate::dex_run::view) fn canvas_float_field_value(
        &self,
        canvas: &DexValue,
        name: &str,
        default: f32,
    ) -> f32 {
        let value = self
            .read_stored_instance_field(canvas, &canvas_float_field(name))
            .unwrap_or(DexValue::Float(default));
        let DexValue::Float(value) = value else {
            return default;
        };

        value
    }

    fn render_node_dimension(&self, render_node: &DexValue, getter: &str) -> i32 {
        let left = self.render_node_position_value(render_node, "positionLeft");
        let top = self.render_node_position_value(render_node, "positionTop");
        let right = self.render_node_position_value(render_node, "positionRight");
        let bottom = self.render_node_position_value(render_node, "positionBottom");
        match getter {
            "getLeft" => left,
            "getTop" => top,
            "getRight" => right,
            "getBottom" => bottom,
            "getWidth" => right.saturating_sub(left).max(0),
            "getHeight" => bottom.saturating_sub(top).max(0),
            _ => 0,
        }
    }

    pub(in crate::dex_run::view) fn render_node_position_value(
        &self,
        render_node: &DexValue,
        name: &str,
    ) -> i32 {
        let value = self
            .read_stored_instance_field(render_node, &render_node_position_field(name))
            .unwrap_or(DexValue::Int(0));
        let DexValue::Int(value) = value else {
            return 0;
        };

        value
    }

    fn render_node_float_property_value(
        &self,
        render_node: &DexValue,
        name: &str,
        default: f32,
    ) -> f32 {
        let Some((field, _)) = render_node_float_property_field(name) else {
            return default;
        };
        let value = self
            .read_stored_instance_field(render_node, &field)
            .unwrap_or(DexValue::Float(default));
        let DexValue::Float(value) = value else {
            return default;
        };

        value
    }

    pub(in crate::dex_run) fn paint_get_text_bounds(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 5 {
            return None;
        }

        match &invocation.arguments[0] {
            DexValue::Object(_) | DexValue::Reference(_) => {}
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        }

        if matches!(invocation.arguments[1], DexValue::Null | DexValue::Int(0)) {
            return None;
        }
        let text = self.char_sequence_text(&invocation.arguments[1])?;
        let DexValue::Int(start) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Int(end) = invocation.arguments[3] else {
            return None;
        };
        match &invocation.arguments[4] {
            DexValue::Object(_) | DexValue::Reference(_) => {}
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        }

        let length = i32::try_from(text.encode_utf16().count()).ok()?;
        if start < 0 || end < start || end > length {
            return None;
        }

        let text_units = end - start;
        let text_size = self
            .paint_text_size(&invocation.arguments[0])
            .unwrap_or(16.0)
            .abs();
        let text_scale_x = self
            .read_stored_instance_field(&invocation.arguments[0], &paint_text_scale_x_field())
            .unwrap_or(DexValue::Float(1.0));
        let DexValue::Float(text_scale_x) = text_scale_x else {
            return None;
        };
        let text_scale_x = text_scale_x.abs();

        let right = if text_units == 0 || text_size == 0.0 || text_scale_x == 0.0 {
            0
        } else {
            ((text_units as f32) * text_size * text_scale_x * 0.5).ceil() as i32
        };
        let top = if right == 0 {
            0
        } else {
            -(text_size.ceil() as i32)
        };

        for (name, value) in [("left", 0), ("top", top), ("right", right), ("bottom", 0)] {
            self.write_stored_instance_field(
                &invocation.arguments[4],
                &rect_int_field(name),
                DexValue::Int(value),
            )
            .then_some(())?;
        }

        Some(())
    }

    pub(in crate::dex_run) fn outline_set_round_rect(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 6 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(left) = invocation.arguments[1] else {
            return None;
        };
        let DexValue::Int(top) = invocation.arguments[2] else {
            return None;
        };
        let DexValue::Int(right) = invocation.arguments[3] else {
            return None;
        };
        let DexValue::Int(bottom) = invocation.arguments[4] else {
            return None;
        };
        let radius = match invocation.arguments[5] {
            DexValue::Float(value) => value,
            DexValue::Int(value) => f32::from_bits(value as u32),
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Long(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };

        for (name, value) in [
            ("roundRectLeft", DexValue::Int(left)),
            ("roundRectTop", DexValue::Int(top)),
            ("roundRectRight", DexValue::Int(right)),
            ("roundRectBottom", DexValue::Int(bottom)),
            ("roundRectRadius", DexValue::Float(radius)),
        ] {
            if !self.write_stored_instance_field(
                &invocation.arguments[0],
                &outline_field(name, &value),
                value,
            ) {
                return None;
            }
        }

        Some(())
    }

    // cite: frameworks-base Outline.java#setPath, setEmpty
    pub(in crate::dex_run) fn outline_set_path(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        let ops = self
            .runtime
            .borrow()
            .path_ops
            .get(&map_identity_key(&invocation.arguments[1]))
            .cloned()
            .unwrap_or_default();
        if ops.is_empty() {
            self.remove_stored_instance_field(&invocation.arguments[0], &outline_path_field());

            return Some(());
        }

        let path_copy = self.allocate_object("Landroid/graphics/Path;".to_owned());
        self.runtime
            .borrow_mut()
            .path_ops
            .insert(map_identity_key(&path_copy), ops);

        self.write_stored_instance_field(&invocation.arguments[0], &outline_path_field(), path_copy)
            .then_some(())
    }

    pub(in crate::dex_run) fn outline_set_alpha(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let alpha = match invocation.arguments[1] {
            DexValue::Float(value) => value,
            DexValue::Int(value) => f32::from_bits(value as u32),
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Long(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &outline_field("alpha", &DexValue::Float(alpha)),
            DexValue::Float(alpha),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn initialize_paint(&self, paint: &DexValue, flags: i32) -> Option<()> {
        if !dex_value_is_object_like(paint) {
            return None;
        }

        // cite: frameworks-base Paint.java#Paint, HIDDEN_DEFAULT_PAINT_FLAGS
        let flags = flags | 0x502;

        self.write_stored_instance_field(paint, &paint_flags_field(), DexValue::Int(flags))
            .then_some(())?;
        self.write_stored_instance_field(paint, &paint_text_size_field(), DexValue::Float(16.0))
            .then_some(())?;
        self.write_stored_instance_field(
            paint,
            &paint_color_field(),
            DexValue::Int(0xff00_0000u32 as i32),
        )
        .then_some(())?;
        self.write_stored_instance_field(paint, &paint_alpha_field(), DexValue::Int(255))
            .then_some(())?;
        self.write_stored_instance_field(paint, &paint_stroke_width_field(), DexValue::Float(0.0))
            .then_some(())?;
        self.write_stored_instance_field(paint, &paint_stroke_miter_field(), DexValue::Float(4.0))
            .then_some(())?;
        self.write_stored_instance_field(
            paint,
            &paint_stroke_cap_field(),
            default_paint_stroke_cap(),
        )
        .then_some(())?;
        self.write_stored_instance_field(
            paint,
            &paint_stroke_join_field(),
            default_paint_stroke_join(),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn paint_init_default(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        // cite: frameworks-base Paint.java#Paint
        self.initialize_paint(&invocation.arguments[0], 0x01)
    }

    pub(in crate::dex_run) fn paint_init_with_flags(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        let DexValue::Int(flags) = invocation.arguments[1] else {
            return None;
        };

        self.initialize_paint(&invocation.arguments[0], flags)
    }

    pub(in crate::dex_run) fn paint_get_flags(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let value = self
            .read_stored_instance_field(&invocation.arguments[0], &paint_flags_field())
            .unwrap_or(DexValue::Int(0));
        let DexValue::Int(flags) = value else {
            return None;
        };

        Some(flags)
    }

    pub(in crate::dex_run) fn paint_get_color_long(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i64> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let value = self
            .read_stored_instance_field(&invocation.arguments[0], &paint_color_long_field())
            .unwrap_or(DexValue::Int(0xff00_0000u32 as i32));
        if let DexValue::Long(color_long) = value {
            return Some(color_long);
        }

        let value = self
            .read_stored_instance_field(&invocation.arguments[0], &paint_color_field())
            .unwrap_or(value);
        let DexValue::Int(color) = value else {
            return None;
        };

        Some(((color as u32 as u64) << 32) as i64)
    }

    // cite: frameworks-base Paint.java#getColor
    // cite: frameworks-base Color.java#toArgb
    pub(in crate::dex_run) fn paint_get_color(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        if let Some(DexValue::Int(color)) =
            self.read_stored_instance_field(&invocation.arguments[0], &paint_color_field())
        {
            return Some(color);
        }

        if let Some(DexValue::Long(color_long)) =
            self.read_stored_instance_field(&invocation.arguments[0], &paint_color_long_field())
        {
            if color_long & 0x3f == 0 {
                return Some((color_long >> 32) as i32);
            }

            return None;
        }

        Some(0xff00_0000u32 as i32)
    }

    pub(in crate::dex_run) fn paint_get_alpha(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        if let Some(DexValue::Int(alpha)) =
            self.read_stored_instance_field(&invocation.arguments[0], &paint_alpha_field())
        {
            return Some(alpha);
        }

        if let Some(DexValue::Int(color)) =
            self.read_stored_instance_field(&invocation.arguments[0], &paint_color_field())
        {
            return Some(((color as u32) >> 24) as i32);
        }

        if let Some(DexValue::Long(color_long)) =
            self.read_stored_instance_field(&invocation.arguments[0], &paint_color_long_field())
        {
            return Some((((color_long as u64) >> 56) & 0xff) as i32);
        }

        Some(255)
    }

    pub(in crate::dex_run) fn paint_is_filter_bitmap(
        &self,
        invocation: &DexInvocation,
    ) -> Option<i32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let value = self
            .read_stored_instance_field(&invocation.arguments[0], &paint_flags_field())
            .unwrap_or(DexValue::Int(0));
        let DexValue::Int(flags) = value else {
            return None;
        };

        Some(i32::from(flags & PAINT_FILTER_BITMAP_FLAG != 0))
    }

    pub(in crate::dex_run) fn paint_get_text_size(
        &self,
        invocation: &DexInvocation,
    ) -> Option<f32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        Some(
            self.paint_text_size(&invocation.arguments[0])
                .unwrap_or(16.0),
        )
    }

    pub(in crate::dex_run) fn paint_get_text_scale_x(
        &self,
        invocation: &DexInvocation,
    ) -> Option<f32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let value = self
            .read_stored_instance_field(&invocation.arguments[0], &paint_text_scale_x_field())
            .unwrap_or(DexValue::Float(1.0));
        let DexValue::Float(scale) = value else {
            return None;
        };

        Some(scale)
    }

    pub(in crate::dex_run) fn paint_get_letter_spacing(
        &self,
        invocation: &DexInvocation,
    ) -> Option<f32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let value = self
            .read_stored_instance_field(&invocation.arguments[0], &paint_letter_spacing_field())
            .unwrap_or(DexValue::Float(0.0));
        let DexValue::Float(letter_spacing) = value else {
            return None;
        };

        Some(letter_spacing)
    }

    pub(in crate::dex_run) fn paint_get_text_locale(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        Some(
            self.read_stored_instance_field(&invocation.arguments[0], &paint_text_locale_field())
                .unwrap_or_else(default_locale),
        )
    }

    pub(in crate::dex_run) fn paint_get_stroke_width(
        &self,
        invocation: &DexInvocation,
    ) -> Option<f32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let value = self
            .read_stored_instance_field(&invocation.arguments[0], &paint_stroke_width_field())
            .unwrap_or(DexValue::Float(0.0));
        let DexValue::Float(stroke_width) = value else {
            return None;
        };

        Some(stroke_width)
    }

    pub(in crate::dex_run) fn paint_get_stroke_miter(
        &self,
        invocation: &DexInvocation,
    ) -> Option<f32> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let value = self
            .read_stored_instance_field(&invocation.arguments[0], &paint_stroke_miter_field())
            .unwrap_or(DexValue::Float(4.0));
        let DexValue::Float(stroke_miter) = value else {
            return None;
        };

        Some(stroke_miter)
    }

    pub(in crate::dex_run) fn paint_get_stroke_cap(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        Some(
            self.read_stored_instance_field(&invocation.arguments[0], &paint_stroke_cap_field())
                .unwrap_or_else(default_paint_stroke_cap),
        )
    }

    pub(in crate::dex_run) fn paint_get_stroke_join(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        Some(
            self.read_stored_instance_field(&invocation.arguments[0], &paint_stroke_join_field())
                .unwrap_or_else(default_paint_stroke_join),
        )
    }

    pub(in crate::dex_run) fn paint_set_flags(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(flags) = invocation.arguments[1] else {
            return None;
        };

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_flags_field(),
            DexValue::Int(flags),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn paint_set_hinting(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(hinting) = invocation.arguments[1] else {
            return None;
        };

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_hinting_field(),
            DexValue::Int(hinting),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn paint_set_color(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(color) = invocation.arguments[1] else {
            return None;
        };

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_color_field(),
            DexValue::Int(color),
        )
        .then_some(())?;

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_alpha_field(),
            DexValue::Int(((color as u32) >> 24) as i32),
        )
        .then_some(())?;
        self.remove_stored_instance_field(&invocation.arguments[0], &paint_color_long_field());

        Some(())
    }

    pub(in crate::dex_run) fn paint_set_alpha(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(alpha) = invocation.arguments[1] else {
            return None;
        };
        if !(0..=255).contains(&alpha) {
            return None;
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_alpha_field(),
            DexValue::Int(alpha),
        )
        .then_some(())?;

        let color = self
            .read_stored_instance_field(&invocation.arguments[0], &paint_color_field())
            .unwrap_or(DexValue::Int(0xff00_0000u32 as i32));
        let DexValue::Int(color) = color else {
            return None;
        };
        let color = ((alpha as u32) << 24) | (color as u32 & 0x00ff_ffff);

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_color_field(),
            DexValue::Int(color as i32),
        )
        .then_some(())?;
        self.remove_stored_instance_field(&invocation.arguments[0], &paint_color_long_field());

        Some(())
    }

    pub(in crate::dex_run) fn paint_set_color_long(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 3 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Long(color_long) = invocation.arguments[1] else {
            return None;
        };
        let DexValue::Long(color_long_high_slot) = invocation.arguments[2] else {
            return None;
        };
        if color_long != color_long_high_slot {
            return None;
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_color_long_field(),
            DexValue::Long(color_long),
        )
        .then_some(())?;

        let color_long_bits = color_long as u64;
        if color_long_bits & 0xffff_ffff == 0 {
            let color = (color_long_bits >> 32) as u32 as i32;
            self.write_stored_instance_field(
                &invocation.arguments[0],
                &paint_color_field(),
                DexValue::Int(color),
            )
            .then_some(())?;
            self.write_stored_instance_field(
                &invocation.arguments[0],
                &paint_alpha_field(),
                DexValue::Int(((color as u32) >> 24) as i32),
            )
            .then_some(())?;
        }

        Some(())
    }

    pub(in crate::dex_run) fn paint_set_text_size(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let size = match invocation.arguments[1] {
            DexValue::Float(value) => value,
            DexValue::Int(value) => f32::from_bits(value as u32),
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Long(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_text_size_field(),
            DexValue::Float(size),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn paint_set_text_scale_x(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let scale = match invocation.arguments[1] {
            DexValue::Float(value) => value,
            DexValue::Int(value) => f32::from_bits(value as u32),
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Long(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_text_scale_x_field(),
            DexValue::Float(scale),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn paint_set_letter_spacing(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let letter_spacing = match invocation.arguments[1] {
            DexValue::Float(value) => value,
            DexValue::Int(value) => f32::from_bits(value as u32),
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Long(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_letter_spacing_field(),
            DexValue::Float(letter_spacing),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn paint_set_stroke_width(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let stroke_width = match invocation.arguments[1] {
            DexValue::Float(value) => value,
            DexValue::Int(value) => f32::from_bits(value as u32),
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Long(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_stroke_width_field(),
            DexValue::Float(stroke_width),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn paint_set_stroke_miter(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let stroke_miter = match invocation.arguments[1] {
            DexValue::Float(value) => value,
            DexValue::Int(value) => f32::from_bits(value as u32),
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Long(_)
            | DexValue::String(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_stroke_miter_field(),
            DexValue::Float(stroke_miter),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn paint_set_stroke_cap(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_stroke_cap_field(),
            invocation.arguments[1].clone(),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn paint_set_stroke_join(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_stroke_join_field(),
            invocation.arguments[1].clone(),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn paint_set_shader(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        if !dex_value_is_nullable_object_like(&invocation.arguments[1]) {
            return None;
        }

        let shader = match &invocation.arguments[1] {
            DexValue::Null | DexValue::Int(0) => DexValue::Null,
            value => value.clone(),
        };
        let previous = self
            .read_stored_instance_field(&invocation.arguments[0], &paint_shader_field())
            .unwrap_or(DexValue::Null);
        self.write_stored_instance_field(&invocation.arguments[0], &paint_shader_field(), shader)
            .then_some(previous)
    }

    pub(in crate::dex_run) fn paint_set_style(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_object_like(&invocation.arguments[1])
        {
            return None;
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_style_field(),
            invocation.arguments[1].clone(),
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn paint_set_blend_mode(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0])
            || !dex_value_is_nullable_object_like(&invocation.arguments[1])
        {
            return None;
        }

        let blend_mode = match &invocation.arguments[1] {
            DexValue::Null | DexValue::Int(0) => DexValue::Null,
            value => value.clone(),
        };

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_blend_mode_field(),
            blend_mode,
        )
        .then_some(())
    }

    pub(in crate::dex_run) fn paint_set_color_filter(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let color_filter = nullable_object_argument(&invocation.arguments[1])?;
        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_color_filter_field(),
            color_filter.clone(),
        )
        .then_some(())?;

        Some(color_filter)
    }

    pub(in crate::dex_run) fn paint_set_typeface(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }

        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        if !dex_value_is_nullable_object_like(&invocation.arguments[1]) {
            return None;
        }

        let typeface = match &invocation.arguments[1] {
            DexValue::Null | DexValue::Int(0) => DexValue::Null,
            value => value.clone(),
        };
        let previous = self
            .read_stored_instance_field(&invocation.arguments[0], &paint_typeface_field())
            .unwrap_or(DexValue::Null);
        self.write_stored_instance_field(
            &invocation.arguments[0],
            &paint_typeface_field(),
            typeface,
        )
        .then_some(previous)
    }

    pub(in crate::dex_run) fn paint_text_size(&self, paint: &DexValue) -> Option<f32> {
        if let Some(value) = self.read_stored_instance_field(paint, &paint_text_size_field()) {
            let DexValue::Float(size) = value else {
                return None;
            };

            return Some(size);
        }

        let DexValue::Reference(reference) = paint else {
            return None;
        };
        let owner_key = reference.strip_prefix("Landroid/text/TextPaint;:")?;
        let key = (
            owner_key.to_owned(),
            text_view_text_size_field().display_signature(),
        );
        let value = self.runtime.borrow().field(&key.0, &key.1).cloned()?;
        let DexValue::Float(size) = value else {
            return None;
        };

        Some(size)
    }
}

fn default_paint_stroke_cap() -> DexValue {
    DexValue::Reference(std::rc::Rc::from("Landroid/graphics/Paint$Cap;.BUTT"))
}

fn default_paint_stroke_join() -> DexValue {
    DexValue::Reference(std::rc::Rc::from("Landroid/graphics/Paint$Join;.MITER"))
}

fn default_locale() -> DexValue {
    DexValue::Reference(std::rc::Rc::from("Ljava/util/Locale;:default"))
}

fn path_boolean_op_from_value(value: &DexValue) -> Option<PathBooleanOp> {
    let DexValue::Reference(reference) = value else {
        return None;
    };

    match reference.rsplit('.').next()? {
        "DIFFERENCE" => Some(PathBooleanOp::Difference),
        "INTERSECT" => Some(PathBooleanOp::Intersect),
        "UNION" => Some(PathBooleanOp::Union),
        "XOR" => Some(PathBooleanOp::Xor),
        "REVERSE_DIFFERENCE" => Some(PathBooleanOp::ReverseDifference),
        _ => None,
    }
}

impl DexProgram {
    pub(in crate::dex_run) fn paint_style_values(&self) -> DexValue {
        self.allocate_filled_array(
            "[Landroid/graphics/Paint$Style;".to_owned(),
            paint_style_names()
                .iter()
                .map(|name| {
                    DexValue::Reference(std::rc::Rc::from(format!(
                        "Landroid/graphics/Paint$Style;.{name}"
                    )))
                })
                .collect(),
        )
    }

    pub(in crate::dex_run) fn paint_cap_values(&self) -> DexValue {
        self.allocate_filled_array(
            "[Landroid/graphics/Paint$Cap;".to_owned(),
            paint_cap_names()
                .iter()
                .map(|name| {
                    DexValue::Reference(std::rc::Rc::from(format!(
                        "Landroid/graphics/Paint$Cap;.{name}"
                    )))
                })
                .collect(),
        )
    }

    pub(in crate::dex_run) fn paint_join_values(&self) -> DexValue {
        self.allocate_filled_array(
            "[Landroid/graphics/Paint$Join;".to_owned(),
            paint_join_names()
                .iter()
                .map(|name| {
                    DexValue::Reference(std::rc::Rc::from(format!(
                        "Landroid/graphics/Paint$Join;.{name}"
                    )))
                })
                .collect(),
        )
    }
}

fn paint_style_names() -> &'static [&'static str] {
    &["FILL", "STROKE", "FILL_AND_STROKE"]
}

fn paint_cap_names() -> &'static [&'static str] {
    &["BUTT", "ROUND", "SQUARE"]
}

fn paint_join_names() -> &'static [&'static str] {
    &["MITER", "ROUND", "BEVEL"]
}

// cite: frameworks-base Path.java#Direction
fn path_direction_is_clockwise(direction: &DexValue) -> bool {
    match direction {
        DexValue::Reference(value) => !value.ends_with(".CCW"),
        _ => true,
    }
}

// cite: frameworks-base Matrix.java#mapPoints
fn matrix_map_point(matrix: &[f32; 9], x: f32, y: f32) -> (f32, f32) {
    let mapped_x = matrix[0] * x + matrix[1] * y + matrix[2];
    let mapped_y = matrix[3] * x + matrix[4] * y + matrix[5];
    let w = matrix[6] * x + matrix[7] * y + matrix[8];
    if w != 0.0 && w != 1.0 {
        return (mapped_x / w, mapped_y / w);
    }

    (mapped_x, mapped_y)
}

// cite: frameworks-base Matrix.java#mapVectors
fn matrix_map_vector(matrix: &[f32; 9], dx: f32, dy: f32) -> (f32, f32) {
    (
        matrix[0] * dx + matrix[1] * dy,
        matrix[3] * dx + matrix[4] * dy,
    )
}

fn transform_path_op(op: &mut PathOp, matrix: &[f32; 9]) -> bool {
    match op {
        PathOp::MoveTo { x, y } | PathOp::LineTo { x, y } => {
            (*x, *y) = matrix_map_point(matrix, *x, *y);
            true
        }
        PathOp::RLineTo { dx, dy } => {
            (*dx, *dy) = matrix_map_vector(matrix, *dx, *dy);
            true
        }
        PathOp::QuadTo { x1, y1, x2, y2 } => {
            (*x1, *y1) = matrix_map_point(matrix, *x1, *y1);
            (*x2, *y2) = matrix_map_point(matrix, *x2, *y2);
            true
        }
        PathOp::CubicTo {
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
        } => {
            (*x1, *y1) = matrix_map_point(matrix, *x1, *y1);
            (*x2, *y2) = matrix_map_point(matrix, *x2, *y2);
            (*x3, *y3) = matrix_map_point(matrix, *x3, *y3);
            true
        }
        PathOp::RCubicTo {
            dx1,
            dy1,
            dx2,
            dy2,
            dx3,
            dy3,
        } => {
            (*dx1, *dy1) = matrix_map_vector(matrix, *dx1, *dy1);
            (*dx2, *dy2) = matrix_map_vector(matrix, *dx2, *dy2);
            (*dx3, *dy3) = matrix_map_vector(matrix, *dx3, *dy3);
            true
        }
        PathOp::Close => true,
        PathOp::BooleanOp { left, right, .. } => {
            left.iter_mut().all(|op| transform_path_op(op, matrix))
                && right.iter_mut().all(|op| transform_path_op(op, matrix))
        }
        PathOp::Rect { .. } | PathOp::RoundRect { .. } => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // cite: frameworks-base Canvas.java#Canvas, getWidth, getHeight, getClipBounds
    #[test]
    fn unbacked_canvas_has_empty_native_drawing_layer_for_non_default_display() {
        let program =
            DexProgram::with_config(Vec::new(), WinedroidConfig::with_display_size(1000, 600));
        let canvas = program.allocate_object("Landroid/graphics/Canvas;".to_owned());

        assert_eq!(program.canvas_clip_size(&canvas), (0, 0));
    }
}
