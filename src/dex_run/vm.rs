use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use crate::dex::{DexClassDef, DexCodeItem, DexEncodedMethod, DexFieldId, DexMethodId};
use crate::dex_interpreter::DexValue;
use crate::resource_table::{ResourceFile, ResourceValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResolvedMethod {
    pub(super) dex_file_index: usize,
    pub(super) dex_file_name: String,
    pub(super) class_descriptor: String,
    pub(super) encoded_method: DexEncodedMethod,
    pub(super) method: DexMethodId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MethodIntrinsic {
    InstanceFieldGetter {
        receiver_arg: usize,
        field_index: u32,
        value_type: IntrinsicFieldValueType,
    },
    FloatPack {
        high_arg: usize,
        low_arg: usize,
    },
    AffineFloatPack {
        receiver_arg: usize,
        x_arg: usize,
        y_arg: usize,
        x_offset_field_index: u32,
        scale_field_index: u32,
        y_offset_field_index: u32,
    },
    MatrixPointTransform {
        receiver_arg: usize,
        x_arg: usize,
        y_arg: usize,
        matrix_field_index: u32,
    },
    LongHighFloat {
        value_arg: usize,
    },
    LongLowFloat {
        value_arg: usize,
    },
    FloatArrayGetter {
        receiver_arg: usize,
        field_index: u32,
        element_index: i32,
    },
    FloatArrayPairTransform {
        receiver_arg: usize,
        transformer_arg: usize,
        index_arg: usize,
        array_field_index: u32,
        transform_method_index: u32,
        transform_is_interface: bool,
    },
    FloatRemAddRem {
        left_arg: usize,
        right_arg: usize,
    },
    FloatLerp {
        start_arg: usize,
        end_arg: usize,
        fraction_arg: usize,
    },
    FloatAtan2Normalize {
        x_arg: usize,
        y_arg: usize,
        limit_field_index: u32,
    },
    CubicFloatArrayPack {
        receiver_arg: usize,
        t_arg: usize,
        field_index: u32,
    },
    PathAngleDelta {
        receiver_arg: usize,
        t_arg: usize,
        curve_field_index: u32,
        curve_array_field_index: u32,
        anchor_field_index: u32,
        anchor_x_field_index: u32,
        anchor_y_field_index: u32,
        start_angle_field_index: u32,
        offset_field_index: u32,
        limit_field_index: u32,
    },
    CubicTransform {
        receiver_arg: usize,
        transformer_arg: usize,
        points_field_index: u32,
        mutable_cubic_type_index: u32,
        transform_method_index: u32,
    },
    FeatureEdgeTransform {
        receiver_arg: usize,
        transformer_arg: usize,
        cubics_field_index: u32,
        edge_type_index: u32,
        cubic_transform_method_index: u32,
        list_freeze_method_index: u32,
        list_array_field_index: u32,
        list_size_field_index: u32,
        list_read_only_field_index: u32,
        list_mod_count_field_index: u32,
    },
    FeatureCornerTransform {
        receiver_arg: usize,
        transformer_arg: usize,
        cubics_field_index: u32,
        vertex_field_index: u32,
        rounded_center_field_index: u32,
        convex_field_index: u32,
        corner_type_index: u32,
        cubic_transform_method_index: u32,
        point_transform_method_index: u32,
        list_freeze_method_index: u32,
        list_array_field_index: u32,
        list_size_field_index: u32,
        list_read_only_field_index: u32,
        list_mod_count_field_index: u32,
    },
    FeatureDistanceSquared {
        left_arg: usize,
        right_arg: usize,
        corner_type_index: u32,
        convex_field_index: u32,
        cubics_field_index: u32,
        cubic_points_field_index: u32,
    },
    MeasuredPolygonMeasure {
        measurer_arg: usize,
        polygon_arg: usize,
        polygon_features_field_index: u32,
        feature_cubics_field_index: u32,
        cubic_points_field_index: u32,
        corner_type_index: u32,
        center_x_field_index: u32,
        center_y_field_index: u32,
        measured_polygon_measurer_field_index: u32,
        measured_polygon_cubics_field_index: u32,
        measured_polygon_features_field_index: u32,
        measured_cubic_owner_field_index: u32,
        measured_cubic_cubic_field_index: u32,
        measured_cubic_size_field_index: u32,
        measured_cubic_start_field_index: u32,
        measured_cubic_end_field_index: u32,
        progress_feature_progress_field_index: u32,
        progress_feature_feature_field_index: u32,
        list_freeze_method_index: u32,
        list_array_field_index: u32,
        list_size_field_index: u32,
        list_read_only_field_index: u32,
        list_mod_count_field_index: u32,
    },
    RoundedPolygonConstructor {
        receiver_arg: usize,
        features_arg: usize,
        center_x_arg: usize,
        center_y_arg: usize,
        features_field_index: u32,
        center_x_field_index: u32,
        center_y_field_index: u32,
        cubics_field_index: u32,
        feature_cubics_field_index: u32,
        cubic_points_field_index: u32,
        list_freeze_method_index: u32,
        list_array_field_index: u32,
        list_size_field_index: u32,
        list_read_only_field_index: u32,
        list_mod_count_field_index: u32,
    },
    RoundedPolygonTransform {
        receiver_arg: usize,
        transformer_arg: usize,
        features_field_index: u32,
        center_x_field_index: u32,
        center_y_field_index: u32,
        feature_transform_method_index: u32,
        point_transform_method_index: u32,
        constructor_method_index: u32,
        list_freeze_method_index: u32,
        list_array_field_index: u32,
        list_size_field_index: u32,
        list_read_only_field_index: u32,
        list_mod_count_field_index: u32,
    },
    RoundedPolygonBounds {
        polygon_arg: usize,
        bounds_arg: usize,
        mask_arg: usize,
        cubics_field_index: u32,
        cubic_points_field_index: u32,
    },
    RoundedPolygonNormalized {
        receiver_arg: usize,
        bounds_method_index: u32,
        transformer_type_index: u32,
        transformer_constructor_method_index: u32,
        transform_method_index: u32,
    },
    MeasuredCubicCut {
        receiver_arg: usize,
        progress_arg: usize,
        cubic_field_index: u32,
        size_field_index: u32,
        start_field_index: u32,
        end_field_index: u32,
        owner_field_index: u32,
        owner_center_field_index: u32,
        center_x_field_index: u32,
        center_y_field_index: u32,
        cubic_points_field_index: u32,
        pair_first_field_index: u32,
        pair_second_field_index: u32,
    },
    CircularFloatInterpolation {
        x_arg: usize,
        y_arg: usize,
        progress_arg: usize,
        array_field_index: u32,
        size_field_index: u32,
    },
    ObjectEquals {
        left_arg: usize,
        right_arg: usize,
    },
    ObjectArrayGapInsert {
        receiver_arg: usize,
        index_arg: usize,
        count_arg: usize,
        array_field_index: u32,
        size_field_index: u32,
    },
    ObjectArrayAppend {
        receiver_arg: usize,
        value_arg: usize,
        array_field_index: u32,
        size_field_index: u32,
        read_only_field_index: u32,
        mod_count_field_index: u32,
    },
    SlotObjectGapEnsure {
        receiver_arg: usize,
        count_arg: usize,
        anchor_arg: usize,
        slot_index_field_index: u32,
        slot_limit_field_index: u32,
        object_array_field_index: u32,
        object_gap_start_field_index: u32,
        object_gap_size_field_index: u32,
        object_anchor_field_index: u32,
        group_array_field_index: u32,
        group_gap_start_field_index: u32,
        group_gap_size_field_index: u32,
    },
    SlotObjectUpdate {
        receiver_arg: usize,
        value_arg: usize,
        insert_count_field_index: u32,
        insert_anchor_field_index: u32,
        slot_index_field_index: u32,
        slot_limit_field_index: u32,
        object_array_field_index: u32,
        object_gap_start_field_index: u32,
        object_gap_size_field_index: u32,
        object_anchor_field_index: u32,
        group_array_field_index: u32,
        group_gap_start_field_index: u32,
        group_gap_size_field_index: u32,
    },
    SlotGroupDataIndex {
        receiver_arg: usize,
        groups_arg: usize,
        address_arg: usize,
        group_array_field_index: u32,
        object_array_field_index: u32,
        object_gap_size_field_index: u32,
    },
    SlotGroupSize {
        receiver_arg: usize,
        group_array_field_index: u32,
        group_gap_size_field_index: u32,
    },
    SlotGroupAddress {
        receiver_arg: usize,
        index_arg: usize,
        group_gap_start_field_index: u32,
        group_gap_size_field_index: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IntrinsicFieldValueType {
    Int,
    Long,
    Float,
    Object,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LoadedClass {
    pub(super) dex_file_index: usize,
    pub(super) class_def: DexClassDef,
}

// cite: art class_linker.cc#LinkFields
// cite: art art_field.h#GetOffset
#[derive(Debug, Clone, PartialEq)]
pub(super) struct ResolvedInstanceField {
    pub(super) field: Rc<DexFieldId>,
    pub(super) signature: Rc<str>,
    pub(super) declaring_class: String,
    pub(super) type_descriptor: String,
    pub(super) access_flags: u32,
    pub(super) is_volatile: bool,
    pub(super) slot: u16,
    pub(super) default_value: Option<DexValue>,
}

// cite: art class_linker.cc#LinkStaticFields
// cite: art art_field.h#GetOffset
#[derive(Debug, Clone, PartialEq)]
pub(super) struct ResolvedStaticField {
    pub(super) field: Rc<DexFieldId>,
    pub(super) signature: Rc<str>,
    pub(super) declaring_class: String,
    pub(super) type_descriptor: String,
    pub(super) access_flags: u32,
    pub(super) is_volatile: bool,
    pub(super) slot: u16,
    pub(super) default_value: Option<DexValue>,
}

#[derive(Debug, Default)]
pub(super) struct DexRuntimeCaches {
    pub(super) class_defs: crate::fxhash::FxHashMap<String, Option<Rc<LoadedClass>>>,
    pub(super) code_items: crate::fxhash::FxHashMap<CodeItemKey, Rc<DexCodeItem>>,
    pub(super) field_ids: Vec<Vec<Option<Rc<DexFieldId>>>>,
    pub(super) method_ids: Vec<Vec<Option<Rc<DexMethodId>>>>,
    pub(super) field_signatures: HashMap<DexFieldId, Rc<str>>,
    pub(super) field_signatures_by_index: Vec<Vec<Option<Rc<str>>>>,
    pub(super) method_proto_descriptors: Vec<Vec<Option<Rc<str>>>>,
    pub(super) field_inline_caches: Vec<Vec<Option<(u32, u16)>>>,
    pub(super) resolved_instance_fields: Vec<Vec<Option<Rc<ResolvedInstanceField>>>>,
    pub(super) resolved_static_fields: Vec<Vec<Option<Rc<ResolvedStaticField>>>>,
    pub(super) strings: BTreeMap<StringIdKey, String>,
    pub(super) type_descriptors: BTreeMap<TypeIdKey, String>,
    pub(super) resource_values: BTreeMap<u32, Option<ResourceValue>>,
    pub(super) resource_files: BTreeMap<u32, Option<ResourceFile>>,
    pub(super) exact_methods: BTreeMap<MethodLookupKey, Option<Rc<ResolvedMethod>>>,
    pub(super) named_methods: BTreeMap<NamedMethodLookupKey, Vec<ResolvedMethod>>,
    pub(super) class_hierarchy_methods:
        BTreeMap<HierarchyMethodLookupKey, Option<Rc<ResolvedMethod>>>,
    pub(super) interface_hierarchy_methods:
        BTreeMap<HierarchyMethodLookupKey, Option<Rc<ResolvedMethod>>>,
    pub(super) superclass_descriptors: crate::fxhash::FxHashMap<String, Option<String>>,
    pub(super) interface_descriptors: crate::fxhash::FxHashMap<String, Vec<String>>,
    pub(super) builtin_family_hints: BTreeMap<usize, u8>,
    pub(super) direct_code_methods: crate::fxhash::FxHashSet<u64>,
    pub(super) virtual_code_methods:
        crate::fxhash::FxHashMap<u64, crate::fxhash::FxHashSet<String>>,
    pub(super) direct_resolved_methods:
        crate::fxhash::FxHashMap<u64, std::rc::Rc<super::ResolvedMethod>>,
    pub(super) virtual_resolved_methods: crate::fxhash::FxHashMap<
        u64,
        crate::fxhash::FxHashMap<String, std::rc::Rc<super::ResolvedMethod>>,
    >,
    pub(super) null_assert_args: crate::fxhash::FxHashMap<u64, Option<usize>>,
    pub(super) method_intrinsics: crate::fxhash::FxHashMap<u64, Option<MethodIntrinsic>>,
    pub(super) package_registry: Option<crate::package_registry::PackageRegistry>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct CodeItemKey {
    pub(super) dex_file_index: usize,
    pub(super) code_offset: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct StringIdKey {
    pub(super) dex_file_index: usize,
    pub(super) string_index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct TypeIdKey {
    pub(super) dex_file_index: usize,
    pub(super) type_index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct MethodLookupKey {
    pub(super) class_descriptor: String,
    pub(super) name: String,
    pub(super) proto_descriptor: String,
}

impl MethodLookupKey {
    pub(super) fn from_method(method: &DexMethodId) -> Self {
        Self {
            class_descriptor: method.class_descriptor.clone(),
            name: method.name.clone(),
            proto_descriptor: method.proto.descriptor(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct NamedMethodLookupKey {
    pub(super) class_descriptor: String,
    pub(super) name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct HierarchyMethodLookupKey {
    pub(super) receiver_class_descriptor: String,
    pub(super) method: MethodLookupKey,
}

impl HierarchyMethodLookupKey {
    pub(super) fn from_method(receiver_class_descriptor: &str, method: &DexMethodId) -> Self {
        Self {
            receiver_class_descriptor: receiver_class_descriptor.to_owned(),
            method: MethodLookupKey::from_method(method),
        }
    }
}
