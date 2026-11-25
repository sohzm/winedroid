use super::*;

#[cfg(test)]
pub(in crate::dex_run) fn bootstrap_instance_field(field: &DexFieldId) -> Option<DexValue> {
    let system_metrics =
        DexValue::Reference(std::rc::Rc::from("Landroid/util/DisplayMetrics;.bootstrap"));
    bootstrap_instance_field_with_display_size(&system_metrics, field, DisplaySize::default())
}

pub(in crate::dex_run) fn bootstrap_instance_field_with_display_size(
    object: &DexValue,
    field: &DexFieldId,
    display_size: DisplaySize,
) -> Option<DexValue> {
    if field.class_descriptor == "Landroid/util/DisplayMetrics;" {
        let system_metrics = matches!(
            object,
            DexValue::Reference(reference)
                if reference.as_ref() == "Landroid/util/DisplayMetrics;.bootstrap"
        );

        if !system_metrics {
            // cite: frameworks-base DisplayMetrics.java#DisplayMetrics
            if matches!(
                field.name.as_str(),
                "density"
                    | "scaledDensity"
                    | "xdpi"
                    | "ydpi"
                    | "noncompatDensity"
                    | "noncompatScaledDensity"
                    | "noncompatXdpi"
                    | "noncompatYdpi"
            ) && field.type_descriptor == "F"
            {
                return Some(DexValue::Float(0.0));
            }

            if matches!(
                field.name.as_str(),
                "densityDpi"
                    | "noncompatDensityDpi"
                    | "widthPixels"
                    | "noncompatWidthPixels"
                    | "heightPixels"
                    | "noncompatHeightPixels"
            ) && field.type_descriptor == "I"
            {
                return Some(DexValue::Int(0));
            }

            return None;
        }

        // cite: frameworks-base ResourcesManager.java#getDisplayMetrics
        // cite: frameworks-base DisplayInfo.java#getAppMetrics, getMetricsWithSize
        // cite: frameworks-base DisplayMetrics.java#setToDefaults
        if matches!(
            field.name.as_str(),
            "density"
                | "scaledDensity"
                | "xdpi"
                | "ydpi"
                | "noncompatDensity"
                | "noncompatScaledDensity"
                | "noncompatXdpi"
                | "noncompatYdpi"
        ) && field.type_descriptor == "F"
        {
            return Some(DexValue::Float(display_metrics_float_field(&field.name)?));
        }

        if matches!(field.name.as_str(), "densityDpi" | "noncompatDensityDpi")
            && field.type_descriptor == "I"
        {
            return Some(DexValue::Int(BOOTSTRAP_DISPLAY_DENSITY_DPI));
        }

        if matches!(field.name.as_str(), "widthPixels" | "noncompatWidthPixels")
            && field.type_descriptor == "I"
        {
            return Some(DexValue::Int(display_size.width_i32()));
        }

        if matches!(
            field.name.as_str(),
            "heightPixels" | "noncompatHeightPixels"
        ) && field.type_descriptor == "I"
        {
            return Some(DexValue::Int(display_size.height_i32()));
        }
    }

    if field.class_descriptor == "Landroid/content/res/Configuration;"
        && field.name == "fontScale"
        && field.type_descriptor == "F"
    {
        return Some(DexValue::Float(BOOTSTRAP_FONT_SCALE));
    }

    if field.class_descriptor == "Landroidx/activity/ComponentActivity;"
        && field.name == "mSavedStateRegistryController"
        && field.type_descriptor == "Landroidx/savedstate/SavedStateRegistryController;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/activity/ComponentActivity;"
        && field.name == "mContextAwareHelper"
        && field.type_descriptor == "Landroidx/activity/contextaware/ContextAwareHelper;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/activity/ComponentActivity;"
        && field.name == "mOnBackPressedDispatcher"
        && field.type_descriptor == "Landroidx/activity/OnBackPressedDispatcher;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/activity/ComponentActivity;"
        && field.name == "mContentLayoutId"
        && field.type_descriptor == "I"
    {
        return Some(DexValue::Int(0));
    }

    if field.class_descriptor == "Landroidx/activity/contextaware/ContextAwareHelper;"
        && field.name == "listeners"
        && field.type_descriptor == "Ljava/util/Set;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/activity/contextaware/ContextAwareHelper;"
        && field.name == "context"
        && field.type_descriptor == "Landroid/content/Context;"
    {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Landroidx/work/Data;"
        && field.name == "values"
        && field.type_descriptor == "Ljava/util/Map;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroid/database/Observable;"
        && field.name == "mObservers"
        && field.type_descriptor == "Ljava/util/ArrayList;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/fragment/app/FragmentActivity;"
        && field.name == "mFragmentLifecycleRegistry"
        && field.type_descriptor == "Landroidx/lifecycle/LifecycleRegistry;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/fragment/app/FragmentManager;"
        && field.name == "mNonConfig"
        && field.type_descriptor == "Landroidx/fragment/app/FragmentManagerViewModel;"
    {
        // cite: androidx FragmentManager.java#attachController
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/fragment/app/FragmentActivity;"
        && field.name == "mFragments"
        && field.type_descriptor == "Landroidx/fragment/app/FragmentController;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/appcompat/app/AppCompatActivity;"
        && field.name == "mDelegate"
        && field.type_descriptor == "Landroidx/appcompat/app/AppCompatDelegate;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/appcompat/app/AppCompatActivity;"
        && field.name == "mResources"
        && field.type_descriptor == "Landroid/content/res/Resources;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/appcompat/widget/ContentFrameLayout;"
        && field.name == "g"
        && field.type_descriptor == "Landroid/graphics/Rect;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "LI1/c;" && field.name == "e" && field.type_descriptor == "LI1/c;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/appcompat/widget/ContentFrameLayout;"
        && matches!(field.name.as_str(), "a" | "b" | "c" | "d" | "e" | "f")
        && field.type_descriptor == "Landroid/util/TypedValue;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/viewpager2/widget/ViewPager2;"
        && field.name == "mRecyclerView"
        && field.type_descriptor == "Landroidx/recyclerview/widget/RecyclerView;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/viewpager2/widget/ViewPager2;"
        && field.name == "mAccessibilityProvider"
        && field.type_descriptor == "Landroidx/viewpager2/widget/ViewPager2$AccessibilityProvider;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/viewpager2/widget/ViewPager2;"
        && field.name == "mCurrentItemDataSetChangeObserver"
        && field.type_descriptor
            == "Landroidx/recyclerview/widget/RecyclerView$AdapterDataObserver;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/viewpager2/widget/ViewPager2;"
        && field.name == "mExternalPageChangeCallbacks"
        && field.type_descriptor == "Landroidx/viewpager2/widget/CompositeOnPageChangeCallback;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/viewpager2/widget/ViewPager2;"
        && field.name == "mPendingAdapterState"
        && field.type_descriptor == "Landroid/os/Parcelable;"
    {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Landroidx/recyclerview/widget/RecyclerView;"
        && field.name == "mAdapter"
        && field.type_descriptor == "Landroidx/recyclerview/widget/RecyclerView$Adapter;"
    {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Landroidx/recyclerview/widget/RecyclerView;"
        && field.name == "e0"
        && field.type_descriptor == "Lm0/f0;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/recyclerview/widget/RecyclerView;"
        && field.name == "c"
        && field.type_descriptor == "Lm0/W;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/recyclerview/widget/RecyclerView;"
        && field.name == "e"
        && field.type_descriptor == "Lm0/b;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/recyclerview/widget/RecyclerView;"
        && field.name == "f"
        && field.type_descriptor == "LP/m;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/recyclerview/widget/RecyclerView;"
        && field.name == "h0"
        && field.type_descriptor == "Lm0/c0;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/recyclerview/widget/RecyclerView;"
        && field.name == "g0"
        && field.type_descriptor == "Lm0/n;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lm0/f0;"
        && field.name == "g"
        && field.type_descriptor == "Landroidx/recyclerview/widget/RecyclerView;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            "Landroidx/recyclerview/widget/RecyclerView;.bootstrap",
        )));
    }

    if field.class_descriptor == "Lm0/f0;"
        && field.name == "c"
        && field.type_descriptor == "Landroid/widget/OverScroller;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lm0/W;"
        && matches!(field.name.as_str(), "a" | "c")
        && field.type_descriptor == "Ljava/util/ArrayList;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lm0/b;"
        && matches!(field.name.as_str(), "c" | "d")
        && field.type_descriptor == "Ljava/lang/Object;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "LP/m;"
        && field.name == "c"
        && field.type_descriptor == "Ljava/lang/Object;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from("Lm0/c;.bootstrap")));
    }

    if field.class_descriptor == "Landroidx/viewpager2/widget/CompositeOnPageChangeCallback;"
        && field.name == "mCallbacks"
        && field.type_descriptor == "Ljava/util/List;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lcom/google/android/material/tabs/TabLayout;"
        && field.name == "selectedListeners"
        && field.type_descriptor == "Ljava/util/ArrayList;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/LifecycleRegistry;"
        && field.name == "enforceMainThread"
        && field.type_descriptor == "Z"
    {
        return Some(DexValue::Int(0));
    }

    if field.class_descriptor == "Landroidx/lifecycle/LifecycleRegistry;"
        && field.name == "state"
        && field.type_descriptor == "Landroidx/lifecycle/Lifecycle$State;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            "Landroidx/lifecycle/Lifecycle$State;.INITIALIZED",
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/u;"
        && field.name == "d"
        && field.type_descriptor == "Landroidx/lifecycle/l;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            "Landroidx/lifecycle/l;.b",
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/LifecycleRegistry;"
        && field.name == "lifecycleOwner"
        && field.type_descriptor == "Ljava/lang/ref/WeakReference;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/c;"
        && matches!(field.name.as_str(), "a" | "b")
        && field.type_descriptor == "Ljava/util/HashMap;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lp/f;"
        && matches!(field.name.as_str(), "a" | "b")
        && field.type_descriptor == "Lp/c;"
    {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Lp/c;"
        && matches!(field.name.as_str(), "c" | "d")
        && field.type_descriptor == "Lp/c;"
    {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Lb/a;" && field.name == "b" && field.type_descriptor == "La/m;" {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Ld0/g;"
        && field.name == "b"
        && field.type_descriptor == "Ljava/lang/Object;"
    {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Ld0/F;" && field.name == "t" && field.type_descriptor == "Ld0/s;"
    {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Ld0/F;"
        && matches!(field.name.as_str(), "d" | "I" | "J")
        && field.type_descriptor == "Ljava/util/ArrayList;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "La/m;"
        && field.name == "f"
        && field.type_descriptor == "Landroidx/lifecycle/K;"
    {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Lg0/b;"
        && field.name == "a"
        && field.type_descriptor == "Ljava/util/LinkedHashMap;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lg/m;" && field.name == "w" && field.type_descriptor == "Lg/F;" {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Lg/F;" && field.name == "o" && field.type_descriptor == "Lg/P;" {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Lg/F;" && field.name == "M" && field.type_descriptor == "[Lg/E;" {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Lg/F;"
        && matches!(field.name.as_str(), "X" | "Y")
        && field.type_descriptor == "Lg/B;"
    {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Lg/F;"
        && field.name == "k"
        && field.type_descriptor == "Landroid/content/Context;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            "Landroid/content/Context;.bootstrap",
        )));
    }

    if field.class_descriptor == "Lg/F;"
        && field.name == "l"
        && field.type_descriptor == "Landroid/view/Window;"
    {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Lg/F;" && field.name == "m" && field.type_descriptor == "Lg/A;" {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lg/F;"
        && field.name == "S"
        && field.type_descriptor == "Landroid/content/res/Configuration;"
    {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Landroidx/savedstate/SavedStateRegistryController;"
        && field.name == "impl"
        && field.type_descriptor == "Landroidx/savedstate/internal/SavedStateRegistryImpl;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lt2/g;"
        && field.name == "f"
        && field.type_descriptor == "Ljava/util/concurrent/atomic/AtomicBoolean;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lt2/g;" && field.name == "d" && field.type_descriptor == "Ly2/f;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Ly2/f;"
        && field.name == "b"
        && field.type_descriptor == "Ljava/util/HashMap;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "La3/e;" && field.name == "a" && field.type_descriptor == "Le3/r;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Le3/r;" && field.name == "b" && field.type_descriptor == "Le3/u;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Le3/u;"
        && field.name == "a"
        && field.type_descriptor == "Landroid/content/SharedPreferences;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Le3/u;"
        && field.name == "c"
        && field.type_descriptor == "Ljava/lang/Object;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lcom/google/firebase/analytics/FirebaseAnalytics;"
        && field.name == "a"
        && field.type_descriptor == "Lcom/google/android/gms/internal/measurement/j7;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroid/content/pm/PackageInfo;"
        && field.name == "applicationInfo"
        && field.type_descriptor == "Landroid/content/pm/ApplicationInfo;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroid/content/pm/ActivityInfo;"
        && field.name == "parentActivityName"
        && field.type_descriptor == "Ljava/lang/String;"
    {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Landroid/content/pm/ActivityInfo;"
        && field.name == "metaData"
        && field.type_descriptor == "Landroid/os/Bundle;"
    {
        return Some(DexValue::Null);
    }

    if field.class_descriptor == "Landroidx/savedstate/internal/SavedStateRegistryImpl;"
        && field.name == "attached"
        && field.type_descriptor == "Z"
    {
        return Some(DexValue::Int(0));
    }

    if field.class_descriptor == "Landroidx/savedstate/internal/SavedStateRegistryImpl;"
        && field.name == "isRestored"
        && field.type_descriptor == "Z"
    {
        return Some(DexValue::Int(0));
    }

    if field.class_descriptor == "Landroidx/savedstate/internal/SavedStateRegistryImpl;"
        && field.name == "owner"
        && field.type_descriptor == "Landroidx/savedstate/SavedStateRegistryOwner;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    None
}

pub(in crate::dex_run) fn display_metrics_float_field(name: &str) -> Option<f32> {
    match name {
        "density" | "scaledDensity" | "noncompatDensity" | "noncompatScaledDensity" => {
            Some(BOOTSTRAP_DISPLAY_DENSITY)
        }
        "xdpi" | "ydpi" | "noncompatXdpi" | "noncompatYdpi" => {
            Some(BOOTSTRAP_DISPLAY_DENSITY_DPI as f32)
        }
        _ => None,
    }
}

pub(in crate::dex_run) fn bootstrap_instance_field_for_object(
    object: &DexValue,
    field: &DexFieldId,
) -> Option<DexValue> {
    let DexValue::Reference(reference) = object else {
        return None;
    };

    if field.class_descriptor == "LH1/h;" && field.name == "b" && field.type_descriptor == "I" {
        if reference.as_ref() == "LH1/h;.c:LH1/h;" {
            return Some(DexValue::Int(0));
        }

        if reference.as_ref() == "LH1/h;.d:LH1/h;" {
            return Some(DexValue::Int(1));
        }
    }

    if field.class_descriptor == "LL1/s;" && field.name == "b" && field.type_descriptor == "I" {
        if reference.as_ref() == "LL1/s;.c:LL1/s;" {
            return Some(DexValue::Int(0));
        }

        if reference.as_ref() == "LL1/s;.d:LL1/s;" {
            return Some(DexValue::Int(1));
        }

        if reference.as_ref() == "LL1/s;.e:LL1/s;" {
            return Some(DexValue::Int(2));
        }
    }

    None
}

pub(in crate::dex_run) fn bootstrap_instance_field_write(field: &DexFieldId) -> bool {
    if field.class_descriptor == "Landroidx/savedstate/internal/SavedStateRegistryImpl;"
        && field.name == "attached"
        && field.type_descriptor == "Z"
    {
        return true;
    }

    if field.class_descriptor == "Landroidx/savedstate/internal/SavedStateRegistryImpl;"
        && field.name == "isRestored"
        && field.type_descriptor == "Z"
    {
        return true;
    }

    if field.class_descriptor == "Landroidx/savedstate/internal/SavedStateRegistryImpl;"
        && field.name == "restoredState"
        && field.type_descriptor == "Landroid/os/Bundle;"
    {
        return true;
    }

    if field.class_descriptor == "Landroidx/activity/contextaware/ContextAwareHelper;"
        && field.name == "context"
        && field.type_descriptor == "Landroid/content/Context;"
    {
        return true;
    }

    if field.class_descriptor == "Landroidx/lifecycle/LifecycleRegistry;"
        && field.name == "state"
        && field.type_descriptor == "Landroidx/lifecycle/Lifecycle$State;"
    {
        return true;
    }

    false
}

pub(in crate::dex_run) fn default_field_value(field: &DexFieldId) -> Option<DexValue> {
    if field.type_descriptor.starts_with('L') || field.type_descriptor.starts_with('[') {
        return Some(DexValue::Null);
    }

    default_primitive_field_value(field)
}

pub(in crate::dex_run) fn default_array_value(class_descriptor: &str) -> DexValue {
    match class_descriptor
        .strip_prefix('[')
        .and_then(|descriptor| descriptor.chars().next())
    {
        Some('Z' | 'B' | 'C' | 'S' | 'I') => DexValue::Int(0),
        Some('J') => DexValue::Long(0),
        Some('F') => DexValue::Int(0),
        Some('L' | '[') => DexValue::Null,
        _ => DexValue::Uninitialized,
    }
}

pub(in crate::dex_run) fn normalize_array_value_for_descriptor(
    class_descriptor: &str,
    value: DexValue,
) -> Option<DexValue> {
    match class_descriptor {
        "[F" => match value {
            DexValue::Float(value) => Some(DexValue::Int(value.to_bits() as i32)),
            DexValue::Int(_) => Some(value),
            _ => None,
        },
        _ => Some(value),
    }
}

pub(in crate::dex_run) fn default_primitive_field_value(field: &DexFieldId) -> Option<DexValue> {
    if field.type_descriptor == "J" {
        return Some(DexValue::Long(0));
    }

    if field.type_descriptor == "F" {
        return Some(DexValue::Float(0.0));
    }

    if matches!(field.type_descriptor.as_str(), "Z" | "B" | "C" | "S" | "I") {
        return Some(DexValue::Int(0));
    }

    None
}

pub(in crate::dex_run) fn bootstrap_enum_static_field(
    field: &DexFieldId,
    class_descriptor: &str,
    names: &[&str],
) -> Option<DexValue> {
    if field.class_descriptor != class_descriptor || field.type_descriptor != class_descriptor {
        return None;
    }

    if !names.contains(&field.name.as_str()) {
        return None;
    }

    Some(DexValue::Reference(std::rc::Rc::from(format!(
        "{}.{}",
        class_descriptor, field.name
    ))))
}

pub(in crate::dex_run) fn bootstrap_enum_names(
    class_descriptor: &str,
) -> Option<&'static [&'static str]> {
    match class_descriptor {
        "Ljava/util/concurrent/TimeUnit;" => Some(&[
            "NANOSECONDS",
            "MICROSECONDS",
            "MILLISECONDS",
            "SECONDS",
            "MINUTES",
            "HOURS",
            "DAYS",
        ]),
        "Ljava/time/format/TextStyle;" => Some(&[
            "FULL",
            "FULL_STANDALONE",
            "SHORT",
            "SHORT_STANDALONE",
            "NARROW",
            "NARROW_STANDALONE",
        ]),
        "Ljava/time/Month;" => Some(&[
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
        ]),
        "Ljava/time/DayOfWeek;" => Some(&[
            "MONDAY",
            "TUESDAY",
            "WEDNESDAY",
            "THURSDAY",
            "FRIDAY",
            "SATURDAY",
            "SUNDAY",
        ]),
        "Ljava/math/RoundingMode;" => Some(&[
            "UP",
            "DOWN",
            "CEILING",
            "FLOOR",
            "HALF_UP",
            "HALF_DOWN",
            "HALF_EVEN",
            "UNNECESSARY",
        ]),
        "Landroid/graphics/Path$FillType;" => {
            Some(&["WINDING", "EVEN_ODD", "INVERSE_WINDING", "INVERSE_EVEN_ODD"])
        }
        "Landroid/graphics/Path$Direction;" => Some(&["CW", "CCW"]),
        "Landroid/graphics/Path$Op;" => Some(&[
            "DIFFERENCE",
            "INTERSECT",
            "UNION",
            "XOR",
            "REVERSE_DIFFERENCE",
        ]),
        "Landroid/graphics/Paint$Style;" => Some(&["FILL", "STROKE", "FILL_AND_STROKE"]),
        "Landroid/graphics/Paint$Cap;" => Some(&["BUTT", "ROUND", "SQUARE"]),
        "Landroid/graphics/Paint$Join;" => Some(&["MITER", "ROUND", "BEVEL"]),
        "Landroid/graphics/Region$Op;" => Some(&[
            "DIFFERENCE",
            "INTERSECT",
            "UNION",
            "XOR",
            "REVERSE_DIFFERENCE",
            "REPLACE",
        ]),
        "Landroid/graphics/Bitmap$Config;" => Some(&[
            "ALPHA_8",
            "RGB_565",
            "ARGB_4444",
            "ARGB_8888",
            "RGBA_F16",
            "HARDWARE",
        ]),
        "Landroid/text/Layout$Alignment;" => {
            Some(&["ALIGN_NORMAL", "ALIGN_OPPOSITE", "ALIGN_CENTER"])
        }
        "Landroid/graphics/BlendMode;" => Some(&[
            "CLEAR",
            "SRC",
            "DST",
            "SRC_OVER",
            "DST_OVER",
            "SRC_IN",
            "DST_IN",
            "SRC_OUT",
            "DST_OUT",
            "SRC_ATOP",
            "DST_ATOP",
            "XOR",
            "PLUS",
            "MODULATE",
            "SCREEN",
            "OVERLAY",
            "DARKEN",
            "LIGHTEN",
            "COLOR_DODGE",
            "COLOR_BURN",
            "HARD_LIGHT",
            "SOFT_LIGHT",
            "DIFFERENCE",
            "EXCLUSION",
            "MULTIPLY",
            "HUE",
            "SATURATION",
            "COLOR",
            "LUMINOSITY",
        ]),
        "Landroid/graphics/ColorSpace$Named;" => Some(&[
            "SRGB",
            "LINEAR_SRGB",
            "EXTENDED_SRGB",
            "LINEAR_EXTENDED_SRGB",
            "BT709",
            "BT2020",
            "DCI_P3",
            "DISPLAY_P3",
            "NTSC_1953",
            "SMPTE_C",
            "ADOBE_RGB",
            "PRO_PHOTO_RGB",
            "ACES",
            "ACESCG",
            "CIE_XYZ",
            "CIE_LAB",
        ]),
        "Landroidx/work/BackoffPolicy;" => Some(&["EXPONENTIAL", "LINEAR"]),
        "Landroidx/work/NetworkType;" => Some(&[
            "NOT_REQUIRED",
            "CONNECTED",
            "UNMETERED",
            "NOT_ROAMING",
            "METERED",
            "TEMPORARILY_UNMETERED",
        ]),
        "Landroidx/work/ExistingWorkPolicy;" => {
            Some(&["REPLACE", "KEEP", "APPEND", "APPEND_OR_REPLACE"])
        }
        _ => None,
    }
}

pub(in crate::dex_run) fn bootstrap_enum_reference_ordinal(value: &DexValue) -> Option<i32> {
    let DexValue::Reference(reference) = value else {
        return None;
    };

    let separator = reference.find(";.")?;
    let class_descriptor = reference.get(..=separator)?;
    let name_start = separator + 2;
    let name_end = reference[name_start..]
        .find(':')
        .map(|offset| name_start + offset)
        .unwrap_or(reference.len());
    let name = reference.get(name_start..name_end)?;

    let ordinal = bootstrap_enum_names(class_descriptor)?
        .iter()
        .position(|candidate| *candidate == name)?;

    i32::try_from(ordinal).ok()
}

pub(in crate::dex_run) fn bootstrap_static_field(field: &DexFieldId) -> Option<DexValue> {
    // cite: frameworks-base Typeface.java#DEFAULT, DEFAULT_BOLD, SANS_SERIF, SERIF, MONOSPACE
    if field.class_descriptor == "Landroid/graphics/Typeface;"
        && field.type_descriptor == "Landroid/graphics/Typeface;"
    {
        let reference = match field.name.as_str() {
            "DEFAULT" => Some("Landroid/graphics/Typeface;:family/sans-serif:style/0"),
            "DEFAULT_BOLD" => Some("Landroid/graphics/Typeface;:family/sans-serif:style/1"),
            "SANS_SERIF" => Some("Landroid/graphics/Typeface;:family/sans-serif:style/0"),
            "SERIF" => Some("Landroid/graphics/Typeface;:family/serif:style/0"),
            "MONOSPACE" => Some("Landroid/graphics/Typeface;:family/monospace:style/0"),
            _ => None,
        };
        if let Some(reference) = reference {
            return Some(DexValue::Reference(std::rc::Rc::from(reference.to_owned())));
        }
    }

    // cite: libcore Collections.java#EMPTY_LIST, EMPTY_SET, EMPTY_MAP
    if field.class_descriptor == "Ljava/util/Collections;" {
        match (field.name.as_str(), field.type_descriptor.as_str()) {
            ("EMPTY_LIST", "Ljava/util/List;") => {
                return Some(DexValue::Reference(std::rc::Rc::from(
                    "Ljava/util/List;:collections:emptyList",
                )));
            }
            ("EMPTY_SET", "Ljava/util/Set;") => {
                return Some(DexValue::Reference(std::rc::Rc::from(
                    "Ljava/util/Set;:collections:emptySet",
                )));
            }
            ("EMPTY_MAP", "Ljava/util/Map;") => {
                return Some(DexValue::Reference(std::rc::Rc::from(
                    "Ljava/util/Map;:collections:emptyMap",
                )));
            }
            _ => {}
        }
    }

    // cite: libcore Boolean.java#TYPE
    if field.name == "TYPE" && field.type_descriptor == "Ljava/lang/Class;" {
        let primitive = match field.class_descriptor.as_str() {
            "Ljava/lang/Boolean;" => Some("Z"),
            "Ljava/lang/Byte;" => Some("B"),
            "Ljava/lang/Character;" => Some("C"),
            "Ljava/lang/Short;" => Some("S"),
            "Ljava/lang/Integer;" => Some("I"),
            "Ljava/lang/Long;" => Some("J"),
            "Ljava/lang/Float;" => Some("F"),
            "Ljava/lang/Double;" => Some("D"),
            "Ljava/lang/Void;" => Some("V"),
            _ => None,
        };
        if let Some(primitive) = primitive {
            return Some(DexValue::Reference(std::rc::Rc::from(format!(
                "Ljava/lang/Class;:{primitive}"
            ))));
        }
    }

    if let Some(names) = bootstrap_enum_names(&field.class_descriptor) {
        if let Some(value) = bootstrap_enum_static_field(field, &field.class_descriptor, names) {
            return Some(value);
        }
    }

    if let Some(value) = bootstrap_enum_static_field(
        field,
        "Ljava/util/concurrent/TimeUnit;",
        &[
            "NANOSECONDS",
            "MICROSECONDS",
            "MILLISECONDS",
            "SECONDS",
            "MINUTES",
            "HOURS",
            "DAYS",
        ],
    ) {
        return Some(value);
    }

    if let Some(value) = bootstrap_enum_static_field(
        field,
        "Ljava/math/RoundingMode;",
        &[
            "UP",
            "DOWN",
            "CEILING",
            "FLOOR",
            "HALF_UP",
            "HALF_DOWN",
            "HALF_EVEN",
            "UNNECESSARY",
        ],
    ) {
        return Some(value);
    }

    if let Some(value) = bootstrap_enum_static_field(
        field,
        "Landroid/graphics/Path$FillType;",
        &["WINDING", "EVEN_ODD", "INVERSE_WINDING", "INVERSE_EVEN_ODD"],
    ) {
        return Some(value);
    }

    if let Some(value) =
        bootstrap_enum_static_field(field, "Landroid/graphics/Path$Direction;", &["CW", "CCW"])
    {
        return Some(value);
    }

    if let Some(value) = bootstrap_enum_static_field(
        field,
        "Landroid/graphics/Path$Op;",
        &[
            "DIFFERENCE",
            "INTERSECT",
            "UNION",
            "XOR",
            "REVERSE_DIFFERENCE",
        ],
    ) {
        return Some(value);
    }

    if let Some(value) = bootstrap_enum_static_field(
        field,
        "Landroid/graphics/Paint$Style;",
        &["FILL", "STROKE", "FILL_AND_STROKE"],
    ) {
        return Some(value);
    }

    if let Some(value) = bootstrap_enum_static_field(
        field,
        "Landroid/widget/ImageView$ScaleType;",
        &[
            "MATRIX",
            "FIT_XY",
            "FIT_START",
            "FIT_CENTER",
            "FIT_END",
            "CENTER",
            "CENTER_CROP",
            "CENTER_INSIDE",
        ],
    ) {
        return Some(value);
    }

    if let Some(value) = bootstrap_enum_static_field(
        field,
        "Landroid/graphics/Region$Op;",
        &[
            "DIFFERENCE",
            "INTERSECT",
            "UNION",
            "XOR",
            "REVERSE_DIFFERENCE",
            "REPLACE",
        ],
    ) {
        return Some(value);
    }

    if let Some(value) = bootstrap_enum_static_field(
        field,
        "Landroid/graphics/Bitmap$Config;",
        &[
            "ALPHA_8",
            "RGB_565",
            "ARGB_4444",
            "ARGB_8888",
            "RGBA_F16",
            "HARDWARE",
        ],
    ) {
        return Some(value);
    }

    if let Some(value) = bootstrap_enum_static_field(
        field,
        "Landroid/text/Layout$Alignment;",
        &["ALIGN_NORMAL", "ALIGN_OPPOSITE", "ALIGN_CENTER"],
    ) {
        return Some(value);
    }

    // cite: libcore ZoneOffset.java#UTC, MIN, MAX
    if field.class_descriptor == "Ljava/time/ZoneOffset;"
        && field.type_descriptor == "Ljava/time/ZoneOffset;"
    {
        let total_seconds = match field.name.as_str() {
            "UTC" => 0,
            "MIN" => -64800,
            "MAX" => 64800,
            _ => return None,
        };

        return Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Ljava/time/ZoneOffset;:{total_seconds}"
        ))));
    }

    // cite: libcore LocalTime.java#MIN, MAX, MIDNIGHT, NOON
    if field.class_descriptor == "Ljava/time/LocalTime;"
        && field.type_descriptor == "Ljava/time/LocalTime;"
    {
        let payload = match field.name.as_str() {
            "MIN" | "MIDNIGHT" => "0:0",
            "NOON" => "43200:0",
            "MAX" => "86399:999999999",
            _ => return None,
        };

        return Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Ljava/time/LocalTime;:{payload}"
        ))));
    }

    if field.class_descriptor == "Landroid/text/TextDirectionHeuristics;"
        && field.type_descriptor == "Landroid/text/TextDirectionHeuristic;"
        && matches!(
            field.name.as_str(),
            "LTR" | "RTL" | "FIRSTSTRONG_LTR" | "FIRSTSTRONG_RTL" | "ANYRTL_LTR" | "LOCALE"
        )
    {
        return Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Landroid/text/TextDirectionHeuristics;.{}",
            field.name
        ))));
    }

    if let Some(value) = bootstrap_enum_static_field(
        field,
        "Landroid/text/TextUtils$TruncateAt;",
        &["START", "MIDDLE", "END", "MARQUEE", "END_SMALL"],
    ) {
        return Some(value);
    }

    if let Some(value) = bootstrap_enum_static_field(
        field,
        "Landroid/graphics/BlendMode;",
        &[
            "CLEAR",
            "COLOR",
            "COLOR_BURN",
            "COLOR_DODGE",
            "DARKEN",
            "DIFFERENCE",
            "DST",
            "DST_ATOP",
            "DST_IN",
            "DST_OUT",
            "DST_OVER",
            "EXCLUSION",
            "HARD_LIGHT",
            "HUE",
            "LIGHTEN",
            "LUMINOSITY",
            "MODULATE",
            "MULTIPLY",
            "OVERLAY",
            "PLUS",
            "SATURATION",
            "SCREEN",
            "SOFT_LIGHT",
            "SRC",
            "SRC_ATOP",
            "SRC_IN",
            "SRC_OUT",
            "SRC_OVER",
            "XOR",
        ],
    ) {
        return Some(value);
    }

    if let Some(value) = bootstrap_enum_static_field(
        field,
        "Landroid/graphics/ColorSpace$Named;",
        &[
            "SRGB",
            "LINEAR_SRGB",
            "EXTENDED_SRGB",
            "LINEAR_EXTENDED_SRGB",
            "BT709",
            "BT2020",
            "DCI_P3",
            "DISPLAY_P3",
            "NTSC_1953",
            "SMPTE_C",
            "ADOBE_RGB",
            "PRO_PHOTO_RGB",
            "ACES",
            "ACESCG",
            "CIE_XYZ",
            "CIE_LAB",
        ],
    ) {
        return Some(value);
    }

    if field.class_descriptor == "Ljava/math/MathContext;"
        && field.type_descriptor == "Ljava/math/MathContext;"
        && matches!(
            field.name.as_str(),
            "DECIMAL32" | "DECIMAL64" | "DECIMAL128" | "UNLIMITED"
        )
    {
        return Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Ljava/math/MathContext;.{}",
            field.name
        ))));
    }

    if let Some(value) = bootstrap_enum_static_field(
        field,
        "Landroidx/work/BackoffPolicy;",
        &["EXPONENTIAL", "LINEAR"],
    ) {
        return Some(value);
    }

    if let Some(value) = bootstrap_enum_static_field(
        field,
        "Landroidx/work/NetworkType;",
        &[
            "NOT_REQUIRED",
            "CONNECTED",
            "UNMETERED",
            "NOT_ROAMING",
            "METERED",
            "TEMPORARILY_UNMETERED",
        ],
    ) {
        return Some(value);
    }

    if let Some(value) = bootstrap_enum_static_field(
        field,
        "Landroidx/work/ExistingWorkPolicy;",
        &["REPLACE", "KEEP", "APPEND", "APPEND_OR_REPLACE"],
    ) {
        return Some(value);
    }

    if field.class_descriptor == "Ljava/lang/Boolean;"
        && field.name == "TRUE"
        && field.type_descriptor == "Ljava/lang/Boolean;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            "Ljava/lang/Boolean;:true",
        )));
    }

    if field.class_descriptor == "Ljava/lang/Boolean;"
        && field.name == "FALSE"
        && field.type_descriptor == "Ljava/lang/Boolean;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            "Ljava/lang/Boolean;:false",
        )));
    }

    if field.class_descriptor == "Ljava/util/Locale;"
        && field.type_descriptor == "Ljava/util/Locale;"
        && matches!(
            field.name.as_str(),
            "ROOT" | "US" | "UK" | "ENGLISH" | "CANADA" | "CANADA_FRENCH" | "FRANCE" | "GERMAN"
        )
    {
        return Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Ljava/util/Locale;:{}",
            field.name
        ))));
    }

    if field.class_descriptor == "Ljava/lang/System;"
        && matches!(field.name.as_str(), "out" | "err")
        && field.type_descriptor == "Ljava/io/PrintStream;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Ljava/io/PrintStream;:System.{}",
            field.name
        ))));
    }

    if field.class_descriptor == "Ljava/math/BigDecimal;"
        && field.type_descriptor == "Ljava/math/BigDecimal;"
    {
        let value = match field.name.as_str() {
            "ZERO" => "0",
            "ONE" => "1",
            "TEN" => "10",
            _ => "",
        };
        if !value.is_empty() {
            return Some(DexValue::Reference(std::rc::Rc::from(format!(
                "Ljava/math/BigDecimal;:{value}"
            ))));
        }
    }

    if field.class_descriptor == "Landroid/view/WindowInsets;"
        && field.name == "CONSUMED"
        && field.type_descriptor == "Landroid/view/WindowInsets;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            "Landroid/view/WindowInsets;.CONSUMED",
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/Lifecycle$State;"
        && field.name == "INITIALIZED"
        && field.type_descriptor == "Landroidx/lifecycle/Lifecycle$State;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            "Landroidx/lifecycle/Lifecycle$State;.INITIALIZED",
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/Lifecycle$State;"
        && field.name == "DESTROYED"
        && field.type_descriptor == "Landroidx/lifecycle/Lifecycle$State;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            "Landroidx/lifecycle/Lifecycle$State;.DESTROYED",
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/Lifecycle$State;"
        && field.name == "CREATED"
        && field.type_descriptor == "Landroidx/lifecycle/Lifecycle$State;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            "Landroidx/lifecycle/Lifecycle$State;.CREATED",
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/Lifecycle$State;"
        && field.name == "STARTED"
        && field.type_descriptor == "Landroidx/lifecycle/Lifecycle$State;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            "Landroidx/lifecycle/Lifecycle$State;.STARTED",
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/Lifecycle$State;"
        && field.name == "RESUMED"
        && field.type_descriptor == "Landroidx/lifecycle/Lifecycle$State;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            "Landroidx/lifecycle/Lifecycle$State;.RESUMED",
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/Lifecycle$Event;"
        && field.name == "ON_CREATE"
        && field.type_descriptor == "Landroidx/lifecycle/Lifecycle$Event;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            "Landroidx/lifecycle/Lifecycle$Event;.ON_CREATE",
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/k;"
        && matches!(
            field.name.as_str(),
            "ON_ANY"
                | "ON_CREATE"
                | "ON_DESTROY"
                | "ON_PAUSE"
                | "ON_RESUME"
                | "ON_START"
                | "ON_STOP"
        )
        && field.type_descriptor == "Landroidx/lifecycle/k;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(format!(
            "Landroidx/lifecycle/k;.{}",
            field.name
        ))));
    }

    if field.class_descriptor == "Landroidx/lifecycle/k;"
        && field.name == "Companion"
        && field.type_descriptor == "Landroidx/lifecycle/i;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/E;"
        && field.name == "a"
        && field.type_descriptor == "LC/b;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/E$a;"
        && field.name == "Companion"
        && field.type_descriptor == "Landroidx/lifecycle/D;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Ld0/H;" && field.name == "h" && field.type_descriptor == "LW0/f;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from("LW0/f;:Ld0/H.h")));
    }

    if field.class_descriptor == "Lg0/a;" && field.name == "b" && field.type_descriptor == "Lg0/a;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lz1/k;" && field.name == "a" && field.type_descriptor == "Lz1/l;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lz1/d;"
        && field.name == "b"
        && field.type_descriptor == "Ljava/util/Map;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "LH1/h;"
        && matches!(field.name.as_str(), "c" | "d")
        && field.type_descriptor == "LH1/h;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lp1/f;" && field.name == "c" && field.type_descriptor == "Lp1/f;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Ls1/a;" && field.name == "a" && field.type_descriptor == "Ls1/a;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lq1/n;" && field.name == "a" && field.type_descriptor == "Lq1/n;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lg/t;" && field.name == "g" && field.type_descriptor == "Ls/e;" {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lg/t;"
        && field.name == "h"
        && field.type_descriptor == "Ljava/lang/Object;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lx0/b;" && field.name == "a" && field.type_descriptor == "Le1/e;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lx0/b;" && field.name == "b" && field.type_descriptor == "Lx0/b;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lf/a;" && field.name == "j" && field.type_descriptor == "[I" {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "LB1/e;" && field.name == "a" && field.type_descriptor == "LB1/d;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from("LB1/d;:LB1/e.a")));
    }

    if field.class_descriptor == "LB1/e;" && field.name == "b" && field.type_descriptor == "LB1/a;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from("LB1/a;:LB1/e.b")));
    }

    if field.class_descriptor == "LH1/u;" && field.name == "a" && field.type_descriptor == "LN1/d;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "LL1/l;" && field.name == "a" && field.type_descriptor == "LI1/c;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lr1/j;" && field.name == "a" && field.type_descriptor == "Lr1/j;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lr1/b;" && field.name == "d" && field.type_descriptor == "Lr1/b;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Lr1/e;" && field.name == "a" && field.type_descriptor == "Lr1/e;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "LH1/l;" && field.name == "b" && field.type_descriptor == "LH1/l;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "LH1/X;" && field.name == "a" && field.type_descriptor == "LH1/X;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "LL1/s;"
        && matches!(field.name.as_str(), "c" | "d" | "e")
        && field.type_descriptor == "LL1/s;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/l;"
        && matches!(field.name.as_str(), "a" | "b" | "c" | "d" | "e")
        && field.type_descriptor == "Landroidx/lifecycle/l;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/ReportFragment;"
        && field.name == "Companion"
        && field.type_descriptor == "Landroidx/lifecycle/ReportFragment$Companion;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroidx/lifecycle/ReportFragment$LifecycleCallbacks;"
        && field.name == "Companion"
        && field.type_descriptor
            == "Landroidx/lifecycle/ReportFragment$LifecycleCallbacks$Companion;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            field.display_signature(),
        )));
    }

    if field.class_descriptor == "Landroid/os/Build$VERSION;"
        && field.name == "SDK_INT"
        && field.type_descriptor == "I"
    {
        return Some(DexValue::Int(BOOTSTRAP_ANDROID_SDK_INT));
    }

    // cite: frameworks-base Build.java#FINGERPRINT, MANUFACTURER, MODEL, BRAND, DEVICE, PRODUCT
    if field.class_descriptor == "Landroid/os/Build;"
        && field.type_descriptor == "Ljava/lang/String;"
    {
        let value = match field.name.as_str() {
            "FINGERPRINT" => Some("winedroid/generic/generic:14/UNKNOWN/0:user/release-keys"),
            "MANUFACTURER" | "BRAND" => Some("winedroid"),
            "MODEL" | "DEVICE" | "PRODUCT" | "HARDWARE" | "BOARD" => Some("generic"),
            _ => None,
        };
        if let Some(value) = value {
            return Some(DexValue::String(std::rc::Rc::from(value.to_owned())));
        }
    }

    if field.class_descriptor == "Lsun/misc/Unsafe;"
        && field.name == "theUnsafe"
        && field.type_descriptor == "Lsun/misc/Unsafe;"
    {
        return Some(DexValue::Reference(std::rc::Rc::from(
            "Lsun/misc/Unsafe;:theUnsafe",
        )));
    }

    // cite: frameworks-base DisplayMetrics.java#DENSITY_DEVICE_STABLE, DENSITY_DEFAULT
    if field.class_descriptor == "Landroid/util/DisplayMetrics;"
        && field.type_descriptor == "I"
        && matches!(
            field.name.as_str(),
            "DENSITY_DEVICE_STABLE" | "DENSITY_DEFAULT" | "DENSITY_MEDIUM"
        )
    {
        return Some(DexValue::Int(BOOTSTRAP_DISPLAY_DENSITY_DPI));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // cite: frameworks-base ResourcesManager.java#getDisplayMetrics
    // cite: frameworks-base DisplayInfo.java#getAppMetrics, getMetricsWithSize
    // cite: frameworks-base DisplayMetrics.java#setToDefaults
    #[test]
    fn system_display_metrics_fields_use_configured_non_default_dimensions() {
        let program =
            DexProgram::with_config(Vec::new(), WinedroidConfig::with_display_size(600, 1000));
        let system_metrics =
            DexValue::Reference(std::rc::Rc::from("Landroid/util/DisplayMetrics;.bootstrap"));

        for (name, expected) in [
            ("densityDpi", BOOTSTRAP_DISPLAY_DENSITY_DPI),
            ("noncompatDensityDpi", BOOTSTRAP_DISPLAY_DENSITY_DPI),
            ("widthPixels", 600),
            ("noncompatWidthPixels", 600),
            ("heightPixels", 1000),
            ("noncompatHeightPixels", 1000),
        ] {
            let field = DexFieldId {
                class_descriptor: "Landroid/util/DisplayMetrics;".to_owned(),
                type_descriptor: "I".to_owned(),
                name: name.to_owned(),
            };

            assert_eq!(
                program.read_bootstrap_instance_field(&system_metrics, &field),
                Some(DexValue::Int(expected))
            );
        }

        for name in [
            "density",
            "scaledDensity",
            "noncompatDensity",
            "noncompatScaledDensity",
        ] {
            let field = DexFieldId {
                class_descriptor: "Landroid/util/DisplayMetrics;".to_owned(),
                type_descriptor: "F".to_owned(),
                name: name.to_owned(),
            };

            assert_eq!(
                program.read_bootstrap_instance_field(&system_metrics, &field),
                Some(DexValue::Float(BOOTSTRAP_DISPLAY_DENSITY))
            );
        }
    }

    // cite: frameworks-base DisplayMetrics.java#DisplayMetrics
    #[test]
    fn arbitrary_display_metrics_instances_keep_java_zero_defaults() {
        let program =
            DexProgram::with_config(Vec::new(), WinedroidConfig::with_display_size(600, 1000));
        let metrics = DexValue::Object(DexObjectRef {
            id: 1,
            class_descriptor: std::rc::Rc::from("Landroid/util/DisplayMetrics;"),
        });

        for name in [
            "densityDpi",
            "noncompatDensityDpi",
            "widthPixels",
            "noncompatWidthPixels",
            "heightPixels",
            "noncompatHeightPixels",
        ] {
            let field = DexFieldId {
                class_descriptor: "Landroid/util/DisplayMetrics;".to_owned(),
                type_descriptor: "I".to_owned(),
                name: name.to_owned(),
            };

            assert_eq!(
                program.read_bootstrap_instance_field(&metrics, &field),
                Some(DexValue::Int(0))
            );
        }

        for name in [
            "density",
            "scaledDensity",
            "xdpi",
            "ydpi",
            "noncompatDensity",
            "noncompatScaledDensity",
            "noncompatXdpi",
            "noncompatYdpi",
        ] {
            let field = DexFieldId {
                class_descriptor: "Landroid/util/DisplayMetrics;".to_owned(),
                type_descriptor: "F".to_owned(),
                name: name.to_owned(),
            };

            assert_eq!(
                program.read_bootstrap_instance_field(&metrics, &field),
                Some(DexValue::Float(0.0))
            );
        }
    }
}
