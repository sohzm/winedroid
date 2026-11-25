use super::super::*;

impl DexProgram {
    pub(super) fn invoke_android_platform_misc_builtin(
        &self,
        method: &DexMethodId,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        if method.class_descriptor == "Landroid/util/Log;"
            && method.name == "isLoggable"
            && method.proto.descriptor_is("(Ljava/lang/String;I)Z")
        {
            return Some(DexExecutionResult::Int(0));
        }

        // cite: frameworks-base Log.java#v, d, i, w, e, wtf
        if method.class_descriptor == "Landroid/util/Log;"
            && matches!(method.name.as_str(), "v" | "d" | "i" | "w" | "e" | "wtf")
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/lang/String;Ljava/lang/String;)I"
                    | "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/Throwable;)I"
                    | "(Ljava/lang/String;Ljava/lang/Throwable;)I"
            )
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Landroid/os/Trace;"
            && method.name == "beginSection"
            && method.proto.descriptor_is("(Ljava/lang/String;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/os/Trace;"
            && method.name == "endSection"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/os/Trace;"
            && method.name == "setCounter"
            && method.proto.descriptor_is("(Ljava/lang/String;J)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/os/Trace;"
            && method.name == "isEnabled"
            && method.proto.descriptor_is("()Z")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Landroid/os/StrictMode;"
            && method.name == "getVmPolicy"
            && method
                .proto
                .descriptor_is("()Landroid/os/StrictMode$VmPolicy;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroid/os/StrictMode$VmPolicy;:default"),
            )));
        }

        if method.class_descriptor == "Landroid/os/StrictMode;"
            && method.name == "setVmPolicy"
            && method
                .proto
                .descriptor_is("(Landroid/os/StrictMode$VmPolicy;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base StrictMode.java#noteSlowCall
        if method.matches_builtin_signature(
            "Landroid/os/StrictMode;",
            "noteSlowCall",
            "(Ljava/lang/String;)V",
        ) {
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base NotificationChannel.java#NotificationChannel, setDescription, setShowBadge, setSound, enableLights, enableVibration, getId
        if method.class_descriptor == "Landroid/app/NotificationChannel;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("<init>", "(Ljava/lang/String;Ljava/lang/CharSequence;I)V") => {
                    let invocation = invocation?;
                    let receiver = invocation.arguments.first()?;
                    if let Some(id) = self.string_text(invocation.arguments.get(1)?) {
                        self.write_stored_instance_field(
                            receiver,
                            &DexFieldId {
                                class_descriptor: "Landroid/app/NotificationChannel;".to_owned(),
                                type_descriptor: "Ljava/lang/String;".to_owned(),
                                name: "mId".to_owned(),
                            },
                            DexValue::String(std::rc::Rc::from(id)),
                        );
                    }
                    return Some(DexExecutionResult::Void);
                }
                ("setDescription", "(Ljava/lang/String;)V")
                | ("setShowBadge", "(Z)V")
                | ("setSound", "(Landroid/net/Uri;Landroid/media/AudioAttributes;)V")
                | ("enableLights", "(Z)V")
                | ("enableVibration", "(Z)V") => {
                    return Some(DexExecutionResult::Void);
                }
                ("getId", "()Ljava/lang/String;") => {
                    let invocation = invocation?;
                    let receiver = invocation.arguments.first()?;
                    let id = self
                        .read_stored_instance_field(
                            receiver,
                            &DexFieldId {
                                class_descriptor: "Landroid/app/NotificationChannel;".to_owned(),
                                type_descriptor: "Ljava/lang/String;".to_owned(),
                                name: "mId".to_owned(),
                            },
                        )
                        .unwrap_or(DexValue::Null);
                    return Some(DexExecutionResult::Value(id));
                }
                _ => {}
            }
        }

        // cite: frameworks-base NotificationManager.java#createNotificationChannel, createNotificationChannels, getNotificationChannel, areNotificationsEnabled, notify, cancel, cancelAll
        if method.class_descriptor == "Landroid/app/NotificationManager;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("createNotificationChannel", "(Landroid/app/NotificationChannel;)V")
                | ("createNotificationChannels", "(Ljava/util/List;)V")
                | ("notify", "(ILandroid/app/Notification;)V")
                | ("notify", "(Ljava/lang/String;ILandroid/app/Notification;)V")
                | ("cancel", "(I)V")
                | ("cancel", "(Ljava/lang/String;I)V")
                | ("cancelAll", "()V") => {
                    return Some(DexExecutionResult::Void);
                }
                (
                    "getNotificationChannel",
                    "(Ljava/lang/String;)Landroid/app/NotificationChannel;",
                ) => {
                    return Some(DexExecutionResult::Value(DexValue::Null));
                }
                ("areNotificationsEnabled", "()Z") => {
                    return Some(DexExecutionResult::Int(1));
                }
                _ => {}
            }
        }

        // cite: frameworks-base ActivityManager.java#isLowRamDevice, getMemoryClass, getLargeMemoryClass, getRunningAppProcesses
        if method.class_descriptor == "Landroid/app/ActivityManager;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("isLowRamDevice", "()Z") => {
                    return Some(DexExecutionResult::Int(0));
                }
                ("getMemoryClass", "()I") => {
                    return Some(DexExecutionResult::Int(256));
                }
                ("getLargeMemoryClass", "()I") => {
                    return Some(DexExecutionResult::Int(512));
                }
                ("getRunningAppProcesses", "()Ljava/util/List;") => {
                    let list = self.allocate_object("Ljava/util/ArrayList;".to_owned());
                    self.runtime
                        .borrow_mut()
                        .list_entries
                        .insert(map_identity_key(&list), Vec::new());
                    return Some(DexExecutionResult::Value(list));
                }
                _ => {}
            }
        }

        // cite: frameworks-base Context.java#JOB_SCHEDULER_SERVICE
        if method.class_descriptor == "Landroid/app/job/JobScheduler;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("forNamespace", "(Ljava/lang/String;)Landroid/app/job/JobScheduler;") => {
                    return Some(DexExecutionResult::Value(
                        self.allocate_object("Landroid/app/job/JobScheduler;".to_owned()),
                    ));
                }
                ("getAllPendingJobs", "()Ljava/util/List;") => {
                    let list = self.allocate_object("Ljava/util/ArrayList;".to_owned());
                    self.runtime
                        .borrow_mut()
                        .list_entries
                        .insert(map_identity_key(&list), Vec::new());
                    return Some(DexExecutionResult::Value(list));
                }
                ("schedule", "(Landroid/app/job/JobInfo;)I") => {
                    return Some(DexExecutionResult::Int(1));
                }
                ("cancel", "(I)V") | ("cancelAll", "()V") => {
                    return Some(DexExecutionResult::Void);
                }
                ("getPendingJob", "(I)Landroid/app/job/JobInfo;") => {
                    return Some(DexExecutionResult::Value(DexValue::Null));
                }
                _ => {}
            }
        }

        // cite: frameworks-base PowerManager.java#newWakeLock, isInteractive, isPowerSaveMode, isDeviceIdleMode, isIgnoringBatteryOptimizations
        if method.class_descriptor == "Landroid/os/PowerManager;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("newWakeLock", "(ILjava/lang/String;)Landroid/os/PowerManager$WakeLock;")
                | ("newWakeLock", "(ILjava/lang/String;I)Landroid/os/PowerManager$WakeLock;") => {
                    return Some(DexExecutionResult::Value(
                        self.allocate_object("Landroid/os/PowerManager$WakeLock;".to_owned()),
                    ));
                }
                ("isInteractive", "()Z") => {
                    return Some(DexExecutionResult::Int(1));
                }
                ("isPowerSaveMode", "()Z") | ("isDeviceIdleMode", "()Z") => {
                    return Some(DexExecutionResult::Int(0));
                }
                ("isIgnoringBatteryOptimizations", "(Ljava/lang/String;)Z") => {
                    return Some(DexExecutionResult::Int(1));
                }
                _ => {}
            }
        }

        // cite: frameworks-base PowerManager.java#newWakeLock
        if method.class_descriptor == "Landroid/os/PowerManager$WakeLock;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("acquire", "()V")
                | ("acquire", "(J)V")
                | ("release", "()V")
                | ("release", "(I)V")
                | ("setReferenceCounted", "(Z)V") => {
                    return Some(DexExecutionResult::Void);
                }
                ("isHeld", "()Z") => {
                    return Some(DexExecutionResult::Int(0));
                }
                _ => {}
            }
        }

        if method.class_descriptor == "Landroid/database/ContentObserver;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(Landroid/os/Handler;)V")
        {
            let invocation = invocation?;
            self.initialize_content_observer(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/view/animation/AnimationUtils;"
            && method.name == "currentAnimationTimeMillis"
            && method.proto.descriptor_is("()J")
        {
            return Some(DexExecutionResult::Value(DexValue::Long(0)));
        }

        if method.class_descriptor == "Landroid/os/VibratorManager;"
            && method.name == "getDefaultVibrator"
            && method.proto.descriptor_is("()Landroid/os/Vibrator;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroid/os/Vibrator;:default"),
            )));
        }

        if method.class_descriptor == "Landroid/os/Vibrator;"
            && method.name == "hasVibrator"
            && method.proto.descriptor_is("()Z")
        {
            return Some(DexExecutionResult::Int(1));
        }

        if method.class_descriptor == "Landroid/os/Vibrator;"
            && method.name == "cancel"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/os/Vibrator;"
            && method.name == "vibrate"
            && matches!(
                method.proto.descriptor().as_str(),
                "(J)V"
                    | "(Landroid/os/VibrationEffect;)V"
                    | "(Landroid/os/VibrationEffect;Landroid/media/AudioAttributes;)V"
            )
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base Vibrator.java#areAllPrimitivesSupported, getInfo
        // cite: frameworks-base VibratorInfo.java#EMPTY_VIBRATOR_INFO, isPrimitiveSupported
        if method.class_descriptor == "Landroid/os/Vibrator;"
            && method.name == "areAllPrimitivesSupported"
            && method.proto.descriptor_is("([I)Z")
        {
            let invocation = invocation?;
            if invocation.arguments.len() != 2
                || !dex_value_is_object_like(&invocation.arguments[0])
            {
                return None;
            }
            let length = self.array_length(&invocation.arguments[1])?;

            return Some(DexExecutionResult::Int(i32::from(length == 0)));
        }

        if method.class_descriptor == "Landroid/os/VibrationEffect;"
            && method.name == "createOneShot"
            && method
                .proto
                .descriptor_is("(JI)Landroid/os/VibrationEffect;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroid/os/VibrationEffect;:one-shot"),
            )));
        }

        if method.class_descriptor == "Landroid/os/VibrationEffect;"
            && method.name == "createPredefined"
            && method
                .proto
                .descriptor_is("(I)Landroid/os/VibrationEffect;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroid/os/VibrationEffect;:predefined"),
            )));
        }

        if method.class_descriptor == "Landroid/media/AudioAttributes$Builder;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/media/AudioAttributes$Builder;"
            && matches!(
                method.name.as_str(),
                "setUsage" | "setContentType" | "setFlags"
            )
            && method
                .proto
                .descriptor_is("(I)Landroid/media/AudioAttributes$Builder;")
        {
            let invocation = invocation?;

            return Some(DexExecutionResult::Value(invocation.arguments[0].clone()));
        }

        if method.class_descriptor == "Landroid/media/AudioAttributes$Builder;"
            && method.name == "build"
            && method
                .proto
                .descriptor_is("()Landroid/media/AudioAttributes;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroid/media/AudioAttributes;:default"),
            )));
        }

        if method.class_descriptor == "Landroid/media/SoundPool$Builder;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/media/SoundPool$Builder;"
            && method.name == "setMaxStreams"
            && method
                .proto
                .descriptor_is("(I)Landroid/media/SoundPool$Builder;")
        {
            let invocation = invocation?;

            return Some(DexExecutionResult::Value(invocation.arguments[0].clone()));
        }

        if method.class_descriptor == "Landroid/media/SoundPool$Builder;"
            && method.name == "setAudioAttributes"
            && method.proto.descriptor()
                == "(Landroid/media/AudioAttributes;)Landroid/media/SoundPool$Builder;"
        {
            let invocation = invocation?;

            return Some(DexExecutionResult::Value(invocation.arguments[0].clone()));
        }

        if method.class_descriptor == "Landroid/media/SoundPool$Builder;"
            && method.name == "build"
            && method.proto.descriptor_is("()Landroid/media/SoundPool;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroid/media/SoundPool;:default"),
            )));
        }

        if method.class_descriptor == "Landroid/media/SoundPool;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(III)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/media/SoundPool;"
            && method.name == "load"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Landroid/content/Context;II)I" | "(Ljava/lang/String;I)I"
            )
        {
            return Some(DexExecutionResult::Int(1));
        }

        if method.class_descriptor == "Landroid/media/SoundPool;"
            && method.name == "play"
            && method.proto.descriptor_is("(IFFIIF)I")
        {
            return Some(DexExecutionResult::Int(1));
        }

        if method.class_descriptor == "Landroid/media/SoundPool;"
            && matches!(method.name.as_str(), "pause" | "resume" | "stop" | "unload")
            && method.proto.descriptor_is("(I)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/media/SoundPool;"
            && method.name == "release"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/media/SoundPool;"
            && method.name == "setOnLoadCompleteListener"
            && method
                .proto
                .descriptor_is("(Landroid/media/SoundPool$OnLoadCompleteListener;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/view/accessibility/AccessibilityManager;"
            && matches!(
                method.name.as_str(),
                "isEnabled" | "isTouchExplorationEnabled"
            )
            && method.proto.descriptor_is("()Z")
        {
            return Some(DexExecutionResult::Int(0));
        }

        // cite: frameworks-base OnBackInvokedDispatcher.java#registerOnBackInvokedCallback, unregisterOnBackInvokedCallback
        if method.class_descriptor == "Landroid/window/OnBackInvokedDispatcher;"
            && matches!(
                (method.name.as_str(), method.proto.descriptor().as_str()),
                (
                    "registerOnBackInvokedCallback",
                    "(ILandroid/window/OnBackInvokedCallback;)V"
                ) | (
                    "unregisterOnBackInvokedCallback",
                    "(Landroid/window/OnBackInvokedCallback;)V"
                )
            )
        {
            let invocation = invocation?;
            if !dex_value_is_object_like(invocation.arguments.first()?) {
                return None;
            }

            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base AutofillManager.java#AutofillCallback
        if method.class_descriptor == "Landroid/view/autofill/AutofillManager$AutofillCallback;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base AutofillManager.java#registerCallback, unregisterCallback, hasAutofillFeature
        if method.class_descriptor == "Landroid/view/autofill/AutofillManager;"
            && matches!(
                method.name.as_str(),
                "registerCallback" | "unregisterCallback"
            )
            && method.proto.descriptor()
                == "(Landroid/view/autofill/AutofillManager$AutofillCallback;)V"
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base AccessibilityManager.java#getEnabledAccessibilityServiceList
        if method.class_descriptor == "Landroid/view/accessibility/AccessibilityManager;"
            && method.name == "getEnabledAccessibilityServiceList"
            && method.proto.descriptor_is("(I)Ljava/util/List;")
        {
            let invocation = invocation?;
            if invocation.arguments.len() != 2
                || !dex_value_is_object_like(&invocation.arguments[0])
            {
                return None;
            }
            let DexValue::Int(_) = invocation.arguments[1] else {
                return None;
            };

            return Some(DexExecutionResult::Value(self.collections_empty_list()));
        }

        // cite: frameworks-base AccessibilityManager.java#getRecommendedTimeoutMillis
        if method.class_descriptor == "Landroid/view/accessibility/AccessibilityManager;"
            && method.name == "getRecommendedTimeoutMillis"
            && method.proto.descriptor_is("(II)I")
        {
            let invocation = invocation?;
            let DexValue::Int(original_timeout) = invocation.arguments.get(1)? else {
                return None;
            };

            return Some(DexExecutionResult::Int(*original_timeout));
        }

        if method.class_descriptor == "Landroid/view/accessibility/AccessibilityManager;"
            && method.name == "addAccessibilityStateChangeListener"
            && method.proto.descriptor()
                == "(Landroid/view/accessibility/AccessibilityManager$AccessibilityStateChangeListener;)Z"
        {
            let invocation = invocation?;
            let value = self.add_accessibility_state_change_listener(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/view/accessibility/AccessibilityManager;"
            && method.name == "removeAccessibilityStateChangeListener"
            && method.proto.descriptor()
                == "(Landroid/view/accessibility/AccessibilityManager$AccessibilityStateChangeListener;)Z"
        {
            let invocation = invocation?;
            let value = self.remove_accessibility_state_change_listener(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/view/accessibility/AccessibilityManager;"
            && method.name == "addTouchExplorationStateChangeListener"
            && method.proto.descriptor()
                == "(Landroid/view/accessibility/AccessibilityManager$TouchExplorationStateChangeListener;)Z"
        {
            let invocation = invocation?;
            let value = self.add_accessibility_touch_exploration_state_change_listener(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/view/accessibility/AccessibilityManager;"
            && method.name == "removeTouchExplorationStateChangeListener"
            && method.proto.descriptor()
                == "(Landroid/view/accessibility/AccessibilityManager$TouchExplorationStateChangeListener;)Z"
        {
            let invocation = invocation?;
            let value =
                self.remove_accessibility_touch_exploration_state_change_listener(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/graphics/Color;"
            && method.name == "argb"
            && method.proto.descriptor_is("(IIII)I")
        {
            let invocation = invocation?;
            let value = android_color_argb(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/graphics/ColorSpace;"
            && method.name == "get"
            && method.proto.descriptor()
                == "(Landroid/graphics/ColorSpace$Named;)Landroid/graphics/ColorSpace;"
        {
            let invocation = invocation?;
            let value = android_color_space_get(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: frameworks-base SparseArray.java#SparseArray
        if method.class_descriptor == "Landroid/util/SparseArray;"
            && method.name == "<init>"
            && matches!(method.proto.descriptor().as_str(), "()V" | "(I)V")
        {
            let invocation = invocation?;
            self.initialize_sparse_array(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/util/SparseArray;"
            && method.name == "size"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.sparse_array_size(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: frameworks-base SparseArray.java#put, append
        if method.class_descriptor == "Landroid/util/SparseArray;"
            && matches!(method.name.as_str(), "put" | "append")
            && method.proto.descriptor_is("(ILjava/lang/Object;)V")
        {
            let invocation = invocation?;
            self.sparse_array_put(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base StateSet.java#trimStateSet
        if method.matches_builtin_signature("Landroid/util/StateSet;", "trimStateSet", "([II)[I") {
            let invocation = invocation?;
            let states = invocation.arguments.first()?;
            let DexValue::Int(new_size) = invocation.arguments.get(1)? else {
                return None;
            };
            let new_size = (*new_size).max(0) as usize;
            let values = self
                .runtime
                .borrow()
                .array_entries
                .get(&map_identity_key(states))
                .cloned()
                .unwrap_or_default();
            if values.len() == new_size {
                return Some(DexExecutionResult::Value(states.clone()));
            }
            let mut trimmed = values.into_iter().take(new_size).collect::<Vec<_>>();
            trimmed.resize(new_size, DexValue::Int(0));
            return Some(DexExecutionResult::Value(
                self.allocate_filled_array("[I".to_owned(), trimmed),
            ));
        }

        // cite: frameworks-base AttributeSet.java#getAttributeCount, getClassAttribute
        if method.class_descriptor == "Landroid/util/AttributeSet;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("getAttributeCount", "()I") => return Some(DexExecutionResult::Int(0)),
                ("getClassAttribute", "()Ljava/lang/String;") => {
                    return Some(DexExecutionResult::Value(DexValue::Null));
                }
                _ => {}
            }
        }

        // cite: frameworks-base LinearInterpolator.java#LinearInterpolator
        // cite: frameworks-base DecelerateInterpolator.java#DecelerateInterpolator
        // cite: frameworks-base PathInterpolator.java#PathInterpolator
        if matches!(
            method.class_descriptor.as_str(),
            "Landroid/view/animation/LinearInterpolator;"
                | "Landroid/view/animation/DecelerateInterpolator;"
                | "Landroid/view/animation/AccelerateInterpolator;"
                | "Landroid/view/animation/AccelerateDecelerateInterpolator;"
                | "Landroid/view/animation/PathInterpolator;"
        ) && method.name == "<init>"
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/util/SparseArray;"
            && method.name == "get"
            && method.proto.descriptor_is("(I)Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.sparse_array_get(invocation, None)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/util/SparseArray;"
            && method.name == "get"
            && method
                .proto
                .descriptor_is("(ILjava/lang/Object;)Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let default_value = invocation.arguments.get(2).cloned();
            let value = self.sparse_array_get(invocation, default_value)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/util/SparseArray;"
            && matches!(method.name.as_str(), "remove" | "delete")
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.sparse_array_remove(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/util/SparseArray;"
            && method.name == "removeAt"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.sparse_array_remove_at(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/util/SparseArray;"
            && method.name == "clear"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.sparse_array_clear(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/util/SparseArray;"
            && method.name == "valueAt"
            && method.proto.descriptor_is("(I)Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.sparse_array_value_at(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/util/SparseArray;"
            && method.name == "keyAt"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.sparse_array_key_at(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/util/SparseLongArray;"
            && method.name == "<init>"
            && matches!(method.proto.descriptor().as_str(), "()V" | "(I)V")
        {
            let invocation = invocation?;
            self.initialize_sparse_array(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/util/SparseLongArray;"
            && method.name == "size"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.sparse_array_size(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/util/SparseLongArray;"
            && matches!(method.name.as_str(), "put" | "append")
            && method.proto.descriptor_is("(IJ)V")
        {
            let invocation = invocation?;
            self.sparse_long_array_put(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/util/SparseLongArray;"
            && method.name == "get"
            && method.proto.descriptor_is("(I)J")
        {
            let invocation = invocation?;
            let value = self.sparse_long_array_get(invocation, 0)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Landroid/util/SparseLongArray;"
            && method.name == "get"
            && method.proto.descriptor_is("(IJ)J")
        {
            let invocation = invocation?;
            let value = self.sparse_long_array_get(invocation, 0)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Landroid/util/SparseLongArray;"
            && matches!(method.name.as_str(), "remove" | "delete")
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.sparse_array_remove(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/util/SparseLongArray;"
            && method.name == "removeAt"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.sparse_array_remove_at(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/util/SparseLongArray;"
            && method.name == "clear"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.sparse_array_clear(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/util/SparseLongArray;"
            && method.name == "valueAt"
            && method.proto.descriptor_is("(I)J")
        {
            let invocation = invocation?;
            let value = self.sparse_long_array_value_at(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Landroid/util/SparseLongArray;"
            && method.name == "keyAt"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.sparse_array_key_at(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/util/SparseLongArray;"
            && method.name == "indexOfKey"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.sparse_array_index_of_key(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/util/SparseBooleanArray;"
            && method.name == "<init>"
            && matches!(method.proto.descriptor().as_str(), "()V" | "(I)V")
        {
            let invocation = invocation?;
            self.initialize_sparse_array(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/util/SparseBooleanArray;"
            && method.name == "size"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.sparse_array_size(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/util/SparseBooleanArray;"
            && matches!(method.name.as_str(), "put" | "append")
            && method.proto.descriptor_is("(IZ)V")
        {
            let invocation = invocation?;
            self.sparse_boolean_array_put(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/util/SparseBooleanArray;"
            && method.name == "get"
            && method.proto.descriptor_is("(I)Z")
        {
            let invocation = invocation?;
            let value = self.sparse_boolean_array_get(invocation, 0)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/util/SparseBooleanArray;"
            && method.name == "get"
            && method.proto.descriptor_is("(IZ)Z")
        {
            let invocation = invocation?;
            let value = self.sparse_boolean_array_get(invocation, 0)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/util/SparseBooleanArray;"
            && matches!(method.name.as_str(), "remove" | "delete")
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.sparse_array_remove(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/util/SparseBooleanArray;"
            && method.name == "removeAt"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.sparse_array_remove_at(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/util/SparseBooleanArray;"
            && method.name == "clear"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.sparse_array_clear(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/util/SparseBooleanArray;"
            && method.name == "valueAt"
            && method.proto.descriptor_is("(I)Z")
        {
            let invocation = invocation?;
            let value = self.sparse_boolean_array_value_at(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/util/SparseBooleanArray;"
            && method.name == "keyAt"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.sparse_array_key_at(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/app/FragmentManager;"
            && method.name == "findFragmentByTag"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Landroid/app/Fragment;")
        {
            return Some(DexExecutionResult::Value(DexValue::Null));
        }

        if method.class_descriptor == "Landroid/app/FragmentManager;"
            && method.name == "beginTransaction"
            && method
                .proto
                .descriptor_is("()Landroid/app/FragmentTransaction;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroid/app/FragmentTransaction;"),
            )));
        }

        if method.class_descriptor == "Landroid/app/FragmentManager;"
            && method.name == "executePendingTransactions"
            && method.proto.descriptor_is("()Z")
        {
            return Some(DexExecutionResult::Int(1));
        }

        if method.class_descriptor == "Landroid/app/FragmentTransaction;"
            && method.name == "add"
            && method.proto.descriptor()
                == "(Landroid/app/Fragment;Ljava/lang/String;)Landroid/app/FragmentTransaction;"
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroid/app/FragmentTransaction;"),
            )));
        }

        if method.class_descriptor == "Landroid/app/FragmentTransaction;"
            && method.name == "commit"
            && method.proto.descriptor_is("()I")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Landroid/app/Fragment;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroidx/lifecycle/LifecycleOwner;"
            && method.name == "getLifecycle"
            && method
                .proto
                .descriptor_is("()Landroidx/lifecycle/Lifecycle;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from(method.display_signature()),
            )));
        }

        if method.class_descriptor == "Landroidx/lifecycle/Lifecycle;"
            && method.name == "getCurrentState"
            && method
                .proto
                .descriptor_is("()Landroidx/lifecycle/Lifecycle$State;")
        {
            let receiver_is_dex_object = invocation
                .and_then(|invocation| invocation.arguments.first())
                .is_some_and(|receiver| matches!(receiver, DexValue::Object(_)));
            if receiver_is_dex_object {
                return None;
            }

            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroidx/lifecycle/Lifecycle$State;.INITIALIZED"),
            )));
        }

        if method.class_descriptor == "Landroidx/lifecycle/Lifecycle;"
            && method.name == "addObserver"
            && method
                .proto
                .descriptor_is("(Landroidx/lifecycle/LifecycleObserver;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/os/BaseBundle;"
            && method.name == "containsKey"
            && method.proto.descriptor_is("(Ljava/lang/String;)Z")
        {
            let value = invocation
                .and_then(|invocation| self.bundle_contains_key(invocation))
                .unwrap_or(0);

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/os/BaseBundle;"
            && method.name == "size"
            && method.proto.descriptor_is("()I")
        {
            let value = invocation
                .and_then(|invocation| self.bundle_size(invocation))
                .unwrap_or(0);

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/os/BaseBundle;"
            && method.name == "isEmpty"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = self.bundle_is_empty(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/os/BaseBundle;"
            && method.name == "putString"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;Ljava/lang/String;)V")
        {
            if let Some(invocation) = invocation {
                self.write_bundle_string(invocation)?;
            }

            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base BaseBundle.java#getInt, getLong, getFloat, getDouble, getBoolean
        if method.class_descriptor == "Landroid/os/BaseBundle;"
            && matches!(
                (method.name.as_str(), method.proto.descriptor().as_str()),
                ("getBoolean", "(Ljava/lang/String;)Z")
                    | ("getBoolean", "(Ljava/lang/String;Z)Z")
                    | ("getInt", "(Ljava/lang/String;)I")
                    | ("getInt", "(Ljava/lang/String;I)I")
                    | ("getLong", "(Ljava/lang/String;)J")
                    | ("getLong", "(Ljava/lang/String;J)J")
                    | ("getFloat", "(Ljava/lang/String;)F")
                    | ("getFloat", "(Ljava/lang/String;F)F")
                    | ("getDouble", "(Ljava/lang/String;)D")
                    | ("getDouble", "(Ljava/lang/String;D)D")
            )
        {
            let invocation = invocation?;
            let (box_descriptor, default_value) = match method.name.as_str() {
                "getBoolean" => ("Ljava/lang/Boolean;", DexValue::Int(0)),
                "getInt" => ("Ljava/lang/Integer;", DexValue::Int(0)),
                "getLong" => ("Ljava/lang/Long;", DexValue::Long(0)),
                "getFloat" => ("Ljava/lang/Float;", DexValue::Float(0.0)),
                "getDouble" => (
                    "Ljava/lang/Double;",
                    DexValue::Long(0.0f64.to_bits() as i64),
                ),
                _ => return None,
            };
            let value = self.read_bundle_scalar(invocation, box_descriptor, default_value)?;

            let result = match &value {
                DexValue::Int(value) => DexExecutionResult::Int(*value),
                DexValue::Long(value) => DexExecutionResult::Value(DexValue::Long(*value)),
                DexValue::Float(value) => DexExecutionResult::Value(DexValue::Float(*value)),
                _ => return None,
            };

            return Some(result);
        }

        // cite: frameworks-base BaseBundle.java#getString
        if method.class_descriptor == "Landroid/os/BaseBundle;"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/lang/String;)Ljava/lang/String;"
                    | "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;"
            )
            && method.name == "getString"
        {
            let invocation = invocation?;
            let value = self.read_bundle_string(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: frameworks-base BaseBundle.java#putBoolean, putInt, putLong, putDouble
        if method.class_descriptor == "Landroid/os/BaseBundle;"
            && matches!(
                (method.name.as_str(), method.proto.descriptor().as_str()),
                ("putBoolean", "(Ljava/lang/String;Z)V")
                    | ("putInt", "(Ljava/lang/String;I)V")
                    | ("putLong", "(Ljava/lang/String;J)V")
                    | ("putFloat", "(Ljava/lang/String;F)V")
                    | ("putDouble", "(Ljava/lang/String;D)V")
            )
        {
            if let Some(invocation) = invocation {
                let box_descriptor = match method.name.as_str() {
                    "putBoolean" => "Ljava/lang/Boolean;",
                    "putInt" => "Ljava/lang/Integer;",
                    "putLong" => "Ljava/lang/Long;",
                    "putFloat" => "Ljava/lang/Float;",
                    "putDouble" => "Ljava/lang/Double;",
                    _ => return None,
                };
                self.write_bundle_scalar(invocation, box_descriptor)?;
            }

            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base Bundle.java#putSerializable, putParcelable
        if matches!(
            method.class_descriptor.as_str(),
            "Landroid/os/Bundle;" | "Landroid/os/BaseBundle;"
        ) && matches!(
            (method.name.as_str(), method.proto.descriptor().as_str()),
            (
                "putSerializable",
                "(Ljava/lang/String;Ljava/io/Serializable;)V"
            ) | (
                "putParcelable",
                "(Ljava/lang/String;Landroid/os/Parcelable;)V"
            ) | (
                "putParcelableArray",
                "(Ljava/lang/String;[Landroid/os/Parcelable;)V"
            ) | (
                "putParcelableArrayList",
                "(Ljava/lang/String;Ljava/util/ArrayList;)V"
            ) | (
                "putStringArrayList",
                "(Ljava/lang/String;Ljava/util/ArrayList;)V"
            ) | ("putBundle", "(Ljava/lang/String;Landroid/os/Bundle;)V")
        ) {
            let invocation = invocation?;
            self.write_bundle_object(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base BaseBundle.java#get
        if method.class_descriptor == "Landroid/os/BaseBundle;"
            && method.name == "get"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.bundle_get(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/os/BaseBundle;"
            && method.name == "keySet"
            && method.proto.descriptor_is("()Ljava/util/Set;")
        {
            let value = invocation
                .and_then(|invocation| self.bundle_key_set(invocation))
                .unwrap_or_else(|| DexValue::Reference(std::rc::Rc::from("Ljava/util/Set;.empty")));

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/os/Bundle;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            if let Some(invocation) = invocation {
                self.initialize_bundle(invocation)?;
            }

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/os/Bundle;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(I)V")
        {
            if let Some(invocation) = invocation {
                self.initialize_bundle(invocation)?;
            }

            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base Bundle.java#Bundle
        if method.class_descriptor == "Landroid/os/Bundle;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(Landroid/os/Bundle;)V")
        {
            let invocation = invocation?;
            self.initialize_bundle_copy(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/os/Bundle;"
            && method.name == "getParcelable"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Landroid/os/Parcelable;")
        {
            let value = invocation
                .and_then(|invocation| self.bundle_get_object_entry(invocation))
                .unwrap_or(DexValue::Null);

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/os/Bundle;"
            && method.name == "getParcelable"
            && method.proto.descriptor()
                == "(Ljava/lang/String;Ljava/lang/Class;)Ljava/lang/Object;"
        {
            // cite: frameworks-base Bundle.java#getParcelable
            let value = invocation
                .and_then(|invocation| self.bundle_get_object_entry(invocation))
                .unwrap_or(DexValue::Null);

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/os/Bundle;"
            && method.name == "getBundle"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Landroid/os/Bundle;")
        {
            // cite: frameworks-base Bundle.java#getBundle
            let value = invocation
                .and_then(|invocation| self.bundle_get_object_entry(invocation))
                .unwrap_or(DexValue::Null);

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/os/Bundle;"
            && method.name == "remove"
            && method.proto.descriptor_is("(Ljava/lang/String;)V")
        {
            if let Some(invocation) = invocation {
                self.remove_bundle_entry(invocation)?;
            }

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/os/Bundle;"
            && method.name == "putAll"
            && method.proto.descriptor_is("(Landroid/os/Bundle;)V")
        {
            if let Some(invocation) = invocation {
                self.bundle_put_all(invocation)?;
            }

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/os/Bundle;"
            && method.name == "putCharSequence"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;Ljava/lang/CharSequence;)V")
        {
            let invocation = invocation?;
            self.write_bundle_char_sequence(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/os/Bundle;"
            && method.name == "getCharSequence"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Ljava/lang/CharSequence;")
        {
            // cite: frameworks-base Bundle.java#getCharSequence
            let value = invocation
                .and_then(|invocation| self.bundle_get_object_entry(invocation))
                .unwrap_or(DexValue::Null);

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/os/Bundle;"
            && method.name == "putBundle"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;Landroid/os/Bundle;)V")
        {
            let invocation = invocation?;
            self.write_bundle_bundle(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base Bundle.java#putParcelableArrayList, putStringArrayList
        if method.class_descriptor == "Landroid/os/Bundle;"
            && matches!(
                (method.name.as_str(), method.proto.descriptor().as_str()),
                (
                    "putParcelableArrayList",
                    "(Ljava/lang/String;Ljava/util/ArrayList;)V"
                ) | (
                    "putStringArrayList",
                    "(Ljava/lang/String;Ljava/util/ArrayList;)V"
                )
            )
        {
            let invocation = invocation?;
            self.write_bundle_bundle(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base Bundle.java#getParcelableArrayList, getStringArrayList
        if method.class_descriptor == "Landroid/os/Bundle;"
            && matches!(
                (method.name.as_str(), method.proto.descriptor().as_str()),
                (
                    "getParcelableArrayList",
                    "(Ljava/lang/String;)Ljava/util/ArrayList;"
                ) | (
                    "getStringArrayList",
                    "(Ljava/lang/String;)Ljava/util/ArrayList;"
                )
            )
        {
            let value = invocation
                .and_then(|invocation| self.bundle_get_object_entry(invocation))
                .unwrap_or(DexValue::Null);

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/os/Bundle;"
            && method.name == "setClassLoader"
            && method.proto.descriptor_is("(Ljava/lang/ClassLoader;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/os/Process;"
            && method.name == "myPid"
            && method.proto.descriptor_is("()I")
        {
            return Some(DexExecutionResult::Int(BOOTSTRAP_PROCESS_ID));
        }

        if method.class_descriptor == "Landroid/os/Process;"
            && method.name == "myUid"
            && method.proto.descriptor_is("()I")
        {
            return Some(DexExecutionResult::Int(BOOTSTRAP_PROCESS_UID));
        }

        if method.class_descriptor == "Landroid/database/Observable;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/database/Observable;"
            && method.name == "registerObserver"
            && method.proto.descriptor_is("(Ljava/lang/Object;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/database/Observable;"
            && method.name == "unregisterObserver"
            && method.proto.descriptor_is("(Ljava/lang/Object;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/view/autofill/AutofillValue;"
            && method.name == "forText"
            && method.proto.descriptor()
                == "(Ljava/lang/CharSequence;)Landroid/view/autofill/AutofillValue;"
        {
            let invocation = invocation?;
            let value = self.autofill_value_for_text(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/view/autofill/AutofillValue;"
            && method.name == "forToggle"
            && method
                .proto
                .descriptor_is("(Z)Landroid/view/autofill/AutofillValue;")
        {
            let invocation = invocation?;
            let value = self.autofill_value_for_toggle(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/view/autofill/AutofillManager;"
            && method.name == "notifyViewVisibilityChanged"
            && method.proto.descriptor_is("(Landroid/view/View;IZ)V")
        {
            let invocation = invocation?;
            autofill_manager_notify_view_visibility_changed(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/view/autofill/AutofillManager;"
            && method.name == "notifyValueChanged"
            && method.proto.descriptor()
                == "(Landroid/view/View;ILandroid/view/autofill/AutofillValue;)V"
        {
            let invocation = invocation?;
            autofill_manager_notify_value_changed(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/view/autofill/AutofillManager;"
            && method.name == "notifyViewEntered"
            && method
                .proto
                .descriptor_is("(Landroid/view/View;ILandroid/graphics/Rect;)V")
        {
            let invocation = invocation?;
            autofill_manager_notify_view_entered(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base AutofillManager.java#notifyViewExited, notifyViewVisibilityChanged, cancel, registerCallback, unregisterCallback
        if method.class_descriptor == "Landroid/view/autofill/AutofillManager;"
            && matches!(
                (method.name.as_str(), method.proto.descriptor().as_str()),
                ("notifyViewExited", "(Landroid/view/View;I)V")
                    | ("notifyViewExited", "(Landroid/view/View;)V")
                    | ("notifyViewVisibilityChanged", "(Landroid/view/View;IZ)V")
                    | ("cancel", "()V")
                    | (
                        "registerCallback",
                        "(Landroid/view/autofill/AutofillManager$AutofillCallback;)V"
                    )
                    | (
                        "unregisterCallback",
                        "(Landroid/view/autofill/AutofillManager$AutofillCallback;)V"
                    )
            )
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base EditorInfo.java#setInitialSurroundingSubText, setInitialToolType, setSupportedHandwritingGestures
        if method.class_descriptor == "Landroid/view/inputmethod/EditorInfo;"
            && matches!(
                (method.name.as_str(), method.proto.descriptor().as_str()),
                (
                    "setInitialSurroundingSubText",
                    "(Ljava/lang/CharSequence;I)V"
                ) | ("setInitialSurroundingText", "(Ljava/lang/CharSequence;)V")
                    | ("setInitialToolType", "(I)V")
                    | ("setStylusHandwritingEnabled", "(Z)V")
                    | ("setSupportedHandwritingGestures", "(Ljava/util/List;)V")
                    | (
                        "setSupportedHandwritingGesturePreviews",
                        "(Ljava/util/Set;)V"
                    )
            )
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base InputMethodManager.java#updateSelection, updateCursorAnchorInfo, updateExtractedText
        if method.class_descriptor == "Landroid/view/inputmethod/InputMethodManager;"
            && matches!(
                (method.name.as_str(), method.proto.descriptor().as_str()),
                ("updateSelection", "(Landroid/view/View;IIII)V")
                    | (
                        "updateCursorAnchorInfo",
                        "(Landroid/view/View;Landroid/view/inputmethod/CursorAnchorInfo;)V"
                    )
                    | (
                        "updateExtractedText",
                        "(Landroid/view/View;ILandroid/view/inputmethod/ExtractedText;)V"
                    )
            )
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base InputMethodManager.java#isActive
        if method.class_descriptor == "Landroid/view/inputmethod/InputMethodManager;"
            && method.name == "isActive"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Landroid/view/View;)Z" | "()Z"
            )
        {
            return Some(DexExecutionResult::Int(1));
        }

        if method.class_descriptor == "Landroid/view/inputmethod/InputMethodManager;"
            && method.name == "restartInput"
            && method.proto.descriptor_is("(Landroid/view/View;)V")
        {
            let invocation = invocation?;
            input_method_manager_restart_input(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/view/inputmethod/InputMethodManager;"
            && method.name == "showSoftInput"
            && method.proto.descriptor_is("(Landroid/view/View;I)Z")
        {
            let invocation = invocation?;
            let value = input_method_manager_show_soft_input(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: frameworks-base InputMethodManager.java#hideSoftInputFromWindow, toggleSoftInput
        if method.class_descriptor == "Landroid/view/inputmethod/InputMethodManager;"
            && matches!(
                (method.name.as_str(), method.proto.descriptor().as_str()),
                ("hideSoftInputFromWindow", "(Landroid/os/IBinder;I)Z")
                    | (
                        "hideSoftInputFromWindow",
                        "(Landroid/os/IBinder;ILandroid/os/ResultReceiver;)Z"
                    )
                    | ("hideSoftInputFromView", "(Landroid/view/View;I)Z")
            )
        {
            return Some(DexExecutionResult::Int(1));
        }

        if method.class_descriptor == "Landroid/view/inputmethod/InputMethodManager;"
            && method.name == "toggleSoftInput"
            && method.proto.descriptor_is("(II)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/view/inputmethod/CursorAnchorInfo$Builder;"
            && method.name == "<init>"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            cursor_anchor_info_builder_reference(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteOpenHelper;"
            && method.name == "<init>"
            && method.proto.descriptor()
                == "(Landroid/content/Context;Ljava/lang/String;Landroid/database/sqlite/SQLiteDatabase$CursorFactory;I)V"
        {
            let invocation = invocation?;
            self.initialize_sqlite_open_helper(invocation, false)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteOpenHelper;"
            && method.name == "<init>"
            && method.proto.descriptor()
                == "(Landroid/content/Context;Ljava/lang/String;Landroid/database/sqlite/SQLiteDatabase$CursorFactory;ILandroid/database/DatabaseErrorHandler;)V"
        {
            let invocation = invocation?;
            self.initialize_sqlite_open_helper(invocation, true)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteOpenHelper;"
            && method.name == "getWritableDatabase"
            && method
                .proto
                .descriptor_is("()Landroid/database/sqlite/SQLiteDatabase;")
        {
            let invocation = invocation?;
            let value = match self.sqlite_writable_database(invocation) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("SQLiteOpenHelper lifecycle failed\n  error: {error}");
                    return None;
                }
            };

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteOpenHelper;"
            && method.name == "setWriteAheadLoggingEnabled"
            && method.proto.descriptor_is("(Z)V")
        {
            let invocation = invocation?;
            self.sqlite_open_helper_set_write_ahead_logging_enabled(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteOpenHelper;"
            && method.name == "getDatabaseName"
            && method.proto.descriptor_is("()Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.sqlite_open_helper_database_name(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteOpenHelper;"
            && method.name == "close"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteDatabase;"
            && method.name == "execSQL"
            && method.proto.descriptor_is("(Ljava/lang/String;)V")
        {
            let invocation = invocation?;
            return Some(match self.sqlite_database_exec_sql(invocation)? {
                Ok(()) => DexExecutionResult::Void,
                Err(error) => DexExecutionResult::Throw(self.sqlite_transaction_exception(error)),
            });
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteDatabase;"
            && method.name == "execSQL"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;[Ljava/lang/Object;)V")
        {
            let invocation = invocation?;
            return Some(match self.sqlite_database_exec_sql(invocation)? {
                Ok(()) => DexExecutionResult::Void,
                Err(error) => DexExecutionResult::Throw(self.sqlite_transaction_exception(error)),
            });
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteDatabase;"
            && matches!(
                method.name.as_str(),
                "beginTransaction" | "beginTransactionNonExclusive"
            )
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            let exclusive = method.name == "beginTransaction";
            return Some(
                match self.sqlite_begin_transaction(invocation, exclusive)? {
                    Ok(()) => DexExecutionResult::Void,
                    Err(error) => {
                        DexExecutionResult::Throw(self.sqlite_transaction_exception(error))
                    }
                },
            );
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteDatabase;"
            && method.name == "setTransactionSuccessful"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            return Some(match self.sqlite_set_transaction_successful(invocation)? {
                Ok(()) => DexExecutionResult::Void,
                Err(error) => DexExecutionResult::Throw(self.sqlite_transaction_exception(error)),
            });
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteDatabase;"
            && method.name == "endTransaction"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            return Some(match self.sqlite_end_transaction(invocation)? {
                Ok(()) => DexExecutionResult::Void,
                Err(error) => DexExecutionResult::Throw(self.sqlite_transaction_exception(error)),
            });
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteDatabase;"
            && method.name == "inTransaction"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = self.sqlite_in_transaction(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteDatabase;"
            && method.name == "insert"
            && method.proto.descriptor()
                == "(Ljava/lang/String;Ljava/lang/String;Landroid/content/ContentValues;)J"
        {
            let invocation = invocation?;
            let row_id = self.sqlite_insert_content_values(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(row_id)));
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteDatabase;"
            && method.name == "isOpen"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            sqlite_database_is_open(invocation)?;

            return Some(DexExecutionResult::Int(1));
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteDatabase;"
            && method.name == "getVersion"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.sqlite_database_version(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteDatabase;"
            && method.name == "setVersion"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.sqlite_database_set_version(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteDatabase;"
            && method.name == "setMaxSqlCacheSize"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            sqlite_database_int_configuration(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteDatabase;"
            && method.name == "compileStatement"
            && method.proto.descriptor()
                == "(Ljava/lang/String;)Landroid/database/sqlite/SQLiteStatement;"
        {
            let invocation = invocation?;
            let value = self.sqlite_database_compile_statement(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Landroid/database/sqlite/SQLiteProgram;"
                | "Landroid/database/sqlite/SQLiteQuery;"
                | "Landroid/database/sqlite/SQLiteStatement;"
        ) && method.name == "bindString"
            && method.proto.descriptor_is("(ILjava/lang/String;)V")
        {
            let invocation = invocation?;
            self.sqlite_statement_bind_value(invocation, sqlite_statement_bind_string)?;

            return Some(DexExecutionResult::Void);
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Landroid/database/sqlite/SQLiteProgram;"
                | "Landroid/database/sqlite/SQLiteQuery;"
                | "Landroid/database/sqlite/SQLiteStatement;"
        ) && method.name == "bindLong"
            && method.proto.descriptor_is("(IJ)V")
        {
            let invocation = invocation?;
            self.sqlite_statement_bind_value(invocation, sqlite_statement_bind_long)?;

            return Some(DexExecutionResult::Void);
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Landroid/database/sqlite/SQLiteProgram;"
                | "Landroid/database/sqlite/SQLiteQuery;"
                | "Landroid/database/sqlite/SQLiteStatement;"
        ) && method.name == "bindNull"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.sqlite_statement_bind_value(invocation, sqlite_statement_bind_null)?;

            return Some(DexExecutionResult::Void);
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Landroid/database/sqlite/SQLiteProgram;"
                | "Landroid/database/sqlite/SQLiteQuery;"
                | "Landroid/database/sqlite/SQLiteStatement;"
        ) && method.name == "clearBindings"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.sqlite_statement_clear_bindings(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteStatement;"
            && method.name == "execute"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            return Some(match self.sqlite_statement_execute(invocation)? {
                Ok(()) => DexExecutionResult::Void,
                Err(error) => DexExecutionResult::Throw(self.sqlite_transaction_exception(error)),
            });
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteStatement;"
            && method.name == "executeInsert"
            && method.proto.descriptor_is("()J")
        {
            let invocation = invocation?;
            return Some(match self.sqlite_statement_execute_insert(invocation)? {
                Ok(value) => DexExecutionResult::Value(DexValue::Long(value)),
                Err(error) => DexExecutionResult::Throw(self.sqlite_transaction_exception(error)),
            });
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteCursor;"
            && method.name == "<init>"
            && method.proto.descriptor()
                == "(Landroid/database/sqlite/SQLiteCursorDriver;Ljava/lang/String;Landroid/database/sqlite/SQLiteQuery;)V"
        {
            let invocation = invocation?;
            sqlite_cursor_constructor(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteClosable;"
            && method.name == "close"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            sqlite_statement_reference(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/database/sqlite/SQLiteDatabase;"
            && method.name == "rawQueryWithFactory"
            && method.proto.descriptor()
                == "(Landroid/database/sqlite/SQLiteDatabase$CursorFactory;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;"
        {
            let invocation = invocation?;
            return Some(match self.sqlite_database_raw_query_with_factory(invocation)? {
                Ok(value) => DexExecutionResult::Value(value),
                Err(error) => DexExecutionResult::Throw(self.sqlite_transaction_exception(error)),
            });
        }

        if method.class_descriptor == "Landroid/database/Cursor;"
            && method.name == "getColumnCount"
            && method.proto.descriptor_is("()I")
        {
            let invocation = invocation?;
            let value = self.sqlite_cursor_column_count(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/database/Cursor;"
            && method.name == "getColumnName"
            && method.proto.descriptor_is("(I)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.sqlite_cursor_column_name(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::String(
                std::rc::Rc::from(value),
            )));
        }

        if method.class_descriptor == "Landroid/database/Cursor;"
            && method.name == "getColumnIndex"
            && method.proto.descriptor_is("(Ljava/lang/String;)I")
        {
            let invocation = invocation?;
            let value = self.sqlite_cursor_column_index(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/database/Cursor;"
            && method.name == "getColumnIndexOrThrow"
            && method.proto.descriptor_is("(Ljava/lang/String;)I")
        {
            let invocation = invocation?;
            let value = self.sqlite_cursor_column_index(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/database/Cursor;"
            && method.name == "close"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            sqlite_cursor_reference(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroid/database/Cursor;"
            && method.name == "getString"
            && method.proto.descriptor_is("(I)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.sqlite_cursor_string(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/database/Cursor;"
            && method.name == "getInt"
            && method.proto.descriptor_is("(I)I")
        {
            let invocation = invocation?;
            let value = self.sqlite_cursor_int(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/database/Cursor;"
            && method.name == "getLong"
            && method.proto.descriptor_is("(I)J")
        {
            let invocation = invocation?;
            let value = self.sqlite_cursor_long(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Landroid/database/Cursor;"
            && method.name == "isNull"
            && method.proto.descriptor_is("(I)Z")
        {
            let invocation = invocation?;
            let value = self.sqlite_cursor_is_null(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/database/Cursor;"
            && method.name == "moveToNext"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = self.sqlite_cursor_move_to_next(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: frameworks-base AbstractCursor.java#moveToPosition, moveToFirst, moveToLast, moveToPrevious, getPosition
        if method.class_descriptor == "Landroid/database/Cursor;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("moveToFirst", "()Z") => {
                    let invocation = invocation?;
                    let cursor_key = sqlite_cursor_key(invocation)?;
                    let mut runtime = self.runtime.borrow_mut();
                    let cursor = runtime.sqlite_cursors.get_mut(&cursor_key)?;
                    if cursor.rows.is_empty() {
                        cursor.position = 0;
                        return Some(DexExecutionResult::Int(0));
                    }
                    cursor.position = 0;
                    return Some(DexExecutionResult::Int(1));
                }
                ("moveToLast", "()Z") => {
                    let invocation = invocation?;
                    let cursor_key = sqlite_cursor_key(invocation)?;
                    let mut runtime = self.runtime.borrow_mut();
                    let cursor = runtime.sqlite_cursors.get_mut(&cursor_key)?;
                    if cursor.rows.is_empty() {
                        cursor.position = -1;
                        return Some(DexExecutionResult::Int(0));
                    }
                    cursor.position = (cursor.rows.len() - 1) as isize;
                    return Some(DexExecutionResult::Int(1));
                }
                ("moveToPrevious", "()Z") => {
                    let invocation = invocation?;
                    let cursor_key = sqlite_cursor_key(invocation)?;
                    let mut runtime = self.runtime.borrow_mut();
                    let cursor = runtime.sqlite_cursors.get_mut(&cursor_key)?;
                    let prev = cursor.position - 1;
                    if prev >= 0 && (prev as usize) < cursor.rows.len() {
                        cursor.position = prev;
                        return Some(DexExecutionResult::Int(1));
                    }
                    cursor.position = -1;
                    return Some(DexExecutionResult::Int(0));
                }
                ("moveToPosition", "(I)Z") => {
                    let invocation = invocation?;
                    let cursor_key = map_identity_key(invocation.arguments.first()?);
                    let DexValue::Int(pos) = invocation.arguments.get(1)? else {
                        return None;
                    };
                    let mut runtime = self.runtime.borrow_mut();
                    let cursor = runtime.sqlite_cursors.get_mut(&cursor_key)?;
                    let row_count = isize::try_from(cursor.rows.len()).ok()?;
                    let target = *pos as isize;
                    if target < 0 {
                        cursor.position = -1;
                        return Some(DexExecutionResult::Int(0));
                    }
                    if target >= row_count {
                        cursor.position = row_count;
                        return Some(DexExecutionResult::Int(0));
                    }
                    cursor.position = target;
                    return Some(DexExecutionResult::Int(1));
                }
                ("getPosition", "()I") => {
                    let invocation = invocation?;
                    let cursor_key = sqlite_cursor_key(invocation)?;
                    let runtime = self.runtime.borrow();
                    let cursor = runtime.sqlite_cursors.get(&cursor_key)?;
                    return Some(DexExecutionResult::Int(cursor.position as i32));
                }
                // cite: frameworks-base AbstractCursor.java#getCount
                ("getCount", "()I") => {
                    let invocation = invocation?;
                    let cursor_key = sqlite_cursor_key(invocation)?;
                    let runtime = self.runtime.borrow();
                    let cursor = runtime.sqlite_cursors.get(&cursor_key)?;
                    return Some(DexExecutionResult::Int(cursor.rows.len() as i32));
                }
                _ => {}
            }
        }

        // cite: frameworks-base SQLiteDatabase.java#isWriteAheadLoggingEnabled
        if method.matches_builtin_signature(
            "Landroid/database/sqlite/SQLiteDatabase;",
            "isWriteAheadLoggingEnabled",
            "()Z",
        ) {
            return Some(DexExecutionResult::Int(0));
        }

        // cite: frameworks-base SQLiteStatement.java#executeUpdateDelete
        if method.matches_builtin_signature(
            "Landroid/database/sqlite/SQLiteStatement;",
            "executeUpdateDelete",
            "()I",
        ) {
            let invocation = invocation?;
            let _ = self.sqlite_statement_execute(invocation)?;
            return Some(DexExecutionResult::Int(1));
        }

        // cite: frameworks-base PendingIntent.java#getBroadcast, cancel
        if method.matches_builtin_signature(
            "Landroid/app/PendingIntent;",
            "getBroadcast",
            "(Landroid/content/Context;ILandroid/content/Intent;I)Landroid/app/PendingIntent;",
        ) {
            return Some(DexExecutionResult::Value(
                self.allocate_object("Landroid/app/PendingIntent;".to_owned()),
            ));
        }
        if method.matches_builtin_signature("Landroid/app/PendingIntent;", "cancel", "()V") {
            return Some(DexExecutionResult::Void);
        }

        // cite: frameworks-base ActivityManager.java#getHistoricalProcessExitReasons
        if method.matches_builtin_signature(
            "Landroid/app/ActivityManager;",
            "getHistoricalProcessExitReasons",
            "(Ljava/lang/String;II)Ljava/util/List;",
        ) {
            let list = self.allocate_object("Ljava/util/ArrayList;".to_owned());
            self.runtime
                .borrow_mut()
                .list_entries
                .insert(map_identity_key(&list), Vec::new());
            return Some(DexExecutionResult::Value(list));
        }

        // cite: frameworks-base MimeTypeMap.java#getSingleton
        if method.class_descriptor == "Landroid/webkit/MimeTypeMap;"
            && method.name == "getSingleton"
            && method.proto.descriptor_is("()Landroid/webkit/MimeTypeMap;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroid/webkit/MimeTypeMap;.singleton"),
            )));
        }

        if method.class_descriptor == "Landroid/webkit/MimeTypeMap;"
            && method.name == "getMimeTypeFromExtension"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.mime_type_map_get_mime_type_from_extension(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroid/webkit/MimeTypeMap;"
            && method.name == "hasMimeType"
            && method.proto.descriptor_is("(Ljava/lang/String;)Z")
        {
            let invocation = invocation?;
            let value = self.mime_type_map_has_mime_type(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/webkit/MimeTypeMap;"
            && method.name == "hasExtension"
            && method.proto.descriptor_is("(Ljava/lang/String;)Z")
        {
            let invocation = invocation?;
            let value = self.mime_type_map_has_extension(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Landroid/webkit/MimeTypeMap;"
            && method.name == "getExtensionFromMimeType"
            && method
                .proto
                .descriptor_is("(Ljava/lang/String;)Ljava/lang/String;")
        {
            let invocation = invocation?;
            let value = self.mime_type_map_get_extension_from_mime_type(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        None
    }

    // cite: frameworks-base MimeTypeMap.java#getMimeTypeFromExtension
    pub(super) fn mime_type_map_get_mime_type_from_extension(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }
        let DexValue::String(extension) = &invocation.arguments[1] else {
            return None;
        };

        Some(
            match mime_type_for_extension(&extension.to_ascii_lowercase()) {
                Some(mime_type) => DexValue::String(std::rc::Rc::from(mime_type)),
                None => DexValue::Null,
            },
        )
    }

    pub(super) fn mime_type_map_has_mime_type(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }
        let DexValue::String(mime_type) = &invocation.arguments[1] else {
            return None;
        };

        Some(
            if extension_for_mime_type(&mime_type.to_ascii_lowercase()).is_some() {
                1
            } else {
                0
            },
        )
    }

    pub(super) fn mime_type_map_has_extension(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 {
            return None;
        }
        let DexValue::String(extension) = &invocation.arguments[1] else {
            return None;
        };

        Some(
            if mime_type_for_extension(&extension.to_ascii_lowercase()).is_some() {
                1
            } else {
                0
            },
        )
    }

    // cite: frameworks-base MimeTypeMap.java#getExtensionFromMimeType
    pub(super) fn mime_type_map_get_extension_from_mime_type(
        &self,
        invocation: &DexInvocation,
    ) -> Option<DexValue> {
        if invocation.arguments.len() != 2 {
            return None;
        }
        let DexValue::String(mime_type) = &invocation.arguments[1] else {
            return None;
        };

        Some(
            match extension_for_mime_type(&mime_type.to_ascii_lowercase()) {
                Some(extension) => DexValue::String(std::rc::Rc::from(extension)),
                None => DexValue::Null,
            },
        )
    }

    pub(super) fn add_accessibility_state_change_listener(
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

        let manager_key = map_identity_key(&invocation.arguments[0]);
        let listener = invocation.arguments[1].clone();
        let mut runtime = self.runtime.borrow_mut();
        let listeners = runtime
            .accessibility_state_change_listeners
            .entry(manager_key)
            .or_default();

        if !listeners
            .iter()
            .any(|registered| map_identity_key(registered) == map_identity_key(&listener))
        {
            listeners.push(listener);
        }

        Some(1)
    }

    pub(super) fn remove_accessibility_state_change_listener(
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

        let manager_key = map_identity_key(&invocation.arguments[0]);
        let listener_key = map_identity_key(&invocation.arguments[1]);
        let mut runtime = self.runtime.borrow_mut();
        let Some(listeners) = runtime
            .accessibility_state_change_listeners
            .get_mut(&manager_key)
        else {
            return Some(0);
        };

        let previous_len = listeners.len();
        listeners.retain(|listener| map_identity_key(listener) != listener_key);
        let removed = listeners.len() != previous_len;
        if listeners.is_empty() {
            runtime
                .accessibility_state_change_listeners
                .remove(&manager_key);
        }

        Some(i32::from(removed))
    }

    pub(super) fn add_accessibility_touch_exploration_state_change_listener(
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

        let manager_key = map_identity_key(&invocation.arguments[0]);
        let listener = invocation.arguments[1].clone();
        let mut runtime = self.runtime.borrow_mut();
        let listeners = runtime
            .accessibility_touch_exploration_state_change_listeners
            .entry(manager_key)
            .or_default();

        if !listeners
            .iter()
            .any(|registered| map_identity_key(registered) == map_identity_key(&listener))
        {
            listeners.push(listener);
        }

        Some(1)
    }

    pub(super) fn remove_accessibility_touch_exploration_state_change_listener(
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

        let manager_key = map_identity_key(&invocation.arguments[0]);
        let listener_key = map_identity_key(&invocation.arguments[1]);
        let mut runtime = self.runtime.borrow_mut();
        let Some(listeners) = runtime
            .accessibility_touch_exploration_state_change_listeners
            .get_mut(&manager_key)
        else {
            return Some(0);
        };

        let previous_len = listeners.len();
        listeners.retain(|listener| map_identity_key(listener) != listener_key);
        let removed = listeners.len() != previous_len;
        if listeners.is_empty() {
            runtime
                .accessibility_touch_exploration_state_change_listeners
                .remove(&manager_key);
        }

        Some(i32::from(removed))
    }

    fn initialize_content_observer(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        match &invocation.arguments[1] {
            DexValue::Null => {}
            value if dex_value_is_object_like(value) => {}
            _ => return None,
        }

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &content_observer_handler_field(),
            invocation.arguments[1].clone(),
        )
        .then_some(())
    }

    fn autofill_value_for_text(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        let text = match &invocation.arguments[0] {
            DexValue::Null => DexValue::Null,
            DexValue::String(value) => DexValue::String(std::rc::Rc::from(
                android_trim_no_copy_spans_text(value).to_owned(),
            )),
            DexValue::Object(_) | DexValue::Reference(_) => invocation.arguments[0].clone(),
            DexValue::Uninitialized | DexValue::Int(_) | DexValue::Long(_) | DexValue::Float(_) => {
                return None;
            }
        };

        Some(self.autofill_value_with_payload(1, text))
    }

    fn autofill_value_for_toggle(&self, invocation: &DexInvocation) -> Option<DexValue> {
        let value = boolean_value_of(invocation)?;

        Some(self.autofill_value_with_payload(2, value))
    }

    fn autofill_value_with_payload(&self, value_type: i32, payload: DexValue) -> DexValue {
        let value = self.allocate_object("Landroid/view/autofill/AutofillValue;".to_owned());
        self.write_stored_instance_field(
            &value,
            &autofill_value_type_field(),
            DexValue::Int(value_type),
        );
        self.write_stored_instance_field(&value, &autofill_value_value_field(), payload);

        value
    }

    fn initialize_sqlite_open_helper(
        &self,
        invocation: &DexInvocation,
        has_error_handler: bool,
    ) -> Option<()> {
        let expected_len = if has_error_handler { 6 } else { 5 };
        if invocation.arguments.len() != expected_len
            || !dex_value_is_object_like(&invocation.arguments[0])
        {
            return None;
        }

        if !dex_value_is_nullable_object_like(&invocation.arguments[1]) {
            return None;
        }
        match &invocation.arguments[2] {
            DexValue::String(_) | DexValue::Null => {}
            _ => return None,
        }
        if !dex_value_is_nullable_object_like(&invocation.arguments[3]) {
            return None;
        }
        let DexValue::Int(_) = invocation.arguments[4] else {
            return None;
        };
        if has_error_handler && !dex_value_is_nullable_object_like(&invocation.arguments[5]) {
            return None;
        }

        let receiver = &invocation.arguments[0];
        self.write_stored_instance_field(
            receiver,
            &sqlite_open_helper_context_field(),
            sqlite_nullable_object_value(&invocation.arguments[1]),
        );
        self.write_stored_instance_field(
            receiver,
            &sqlite_open_helper_name_field(),
            invocation.arguments[2].clone(),
        );
        self.write_stored_instance_field(
            receiver,
            &sqlite_open_helper_factory_field(),
            sqlite_nullable_object_value(&invocation.arguments[3]),
        );
        self.write_stored_instance_field(
            receiver,
            &sqlite_open_helper_version_field(),
            invocation.arguments[4].clone(),
        );
        if has_error_handler {
            self.write_stored_instance_field(
                receiver,
                &sqlite_open_helper_error_handler_field(),
                sqlite_nullable_object_value(&invocation.arguments[5]),
            );
        }

        Some(())
    }

    fn sqlite_writable_database(&self, invocation: &DexInvocation) -> Result<DexValue> {
        let helper = invocation.arguments.first().ok_or_else(|| {
            WinedroidError::new("SQLiteOpenHelper.getWritableDatabase missing receiver")
        })?;
        let database = sqlite_writable_database_reference(helper);
        let database_name =
            match self.read_stored_instance_field(helper, &sqlite_open_helper_name_field()) {
                Some(DexValue::String(name)) => Some(name),
                Some(DexValue::Null) | Some(DexValue::Int(0)) | None => None,
                Some(value) => {
                    return Err(WinedroidError::new(format!(
                        "SQLiteOpenHelper database name is not a string\nvalue: {value}"
                    )))
                }
            };
        let context = self
            .read_stored_instance_field(helper, &sqlite_open_helper_context_field())
            .ok_or_else(|| {
                WinedroidError::new("SQLiteOpenHelper has no Context storage identity")
            })?;
        self.register_sqlite_database_name(&database, &context, database_name.as_deref())?;

        self.dispatch_sqlite_open_helper_lifecycle(helper, &database)?;
        Ok(database)
    }

    fn dispatch_sqlite_open_helper_lifecycle(
        &self,
        helper: &DexValue,
        database: &DexValue,
    ) -> Result<()> {
        if self
            .runtime
            .borrow()
            .sqlite_open_helpers_opened
            .contains(&map_identity_key(helper))
        {
            return Ok(());
        }

        let requested_version = match self
            .read_stored_instance_field(helper, &sqlite_open_helper_version_field())
            .unwrap_or(DexValue::Int(0))
        {
            DexValue::Int(version) => version,
            value => {
                return Err(WinedroidError::new(format!(
                    "SQLiteOpenHelper version is not an int\nvalue: {value}"
                )))
            }
        };
        let current_version = self
            .sqlite_database_user_version(database)
            .ok_or_else(|| WinedroidError::new("failed to read SQLite user_version"))?;

        self.execute_optional_sqlite_open_helper_method(helper, database, "onConfigure")?;
        if current_version == 0 {
            self.execute_optional_sqlite_open_helper_method(helper, database, "onCreate")?;
        } else if current_version < requested_version {
            self.execute_optional_sqlite_open_helper_version_method(
                helper,
                database,
                "onUpgrade",
                current_version,
                requested_version,
            )?;
        } else if current_version > requested_version && requested_version > 0 {
            self.execute_optional_sqlite_open_helper_version_method(
                helper,
                database,
                "onDowngrade",
                current_version,
                requested_version,
            )?;
        }
        if requested_version > 0 {
            self.sqlite_set_database_user_version(database, requested_version)
                .ok_or_else(|| WinedroidError::new("failed to update SQLite user_version"))?;
        }
        self.execute_optional_sqlite_open_helper_method(helper, database, "onOpen")?;
        self.runtime
            .borrow_mut()
            .sqlite_open_helpers_opened
            .insert(map_identity_key(helper));

        Ok(())
    }

    fn execute_optional_sqlite_open_helper_method(
        &self,
        helper: &DexValue,
        database: &DexValue,
        method_name: &str,
    ) -> Result<bool> {
        let Some(class_descriptor) = value_class_descriptor(helper) else {
            return Ok(false);
        };
        let method = DexMethodId {
            class_descriptor: class_descriptor.to_owned(),
            name: method_name.to_owned(),
            proto: crate::dex::DexProto {
                shorty: "VL".to_owned(),
                return_type: "V".to_owned(),
                parameters: vec!["Landroid/database/sqlite/SQLiteDatabase;".to_owned()],
            },
        };
        let Some(resolved) =
            self.resolve_exact_method_in_class_hierarchy(class_descriptor, &method)?
        else {
            return Ok(false);
        };
        let result = self.execute_method(
            None,
            &resolved,
            Some(&[helper.clone(), database.clone()]),
            0,
        )?;

        match result {
            DexExecutionResult::Void => Ok(true),
            DexExecutionResult::Throw(value) => Err(WinedroidError::new(format!(
                "SQLiteOpenHelper lifecycle method threw\nclass: {class_descriptor}\nmethod: {}\nthrowable: {}{}",
                method.display_signature(),
                value,
                self.sqlite_throwable_summary(&value)
            ))),
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                Err(WinedroidError::new(format!(
                    "SQLiteOpenHelper lifecycle method parked without a guest-thread scheduler\nclass: {class_descriptor}\nmethod: {}",
                    method.display_signature()
                )))
            }
            result => Err(WinedroidError::new(format!(
                "SQLiteOpenHelper lifecycle method returned a value\nclass: {class_descriptor}\nmethod: {}\nresult: {result}",
                method.display_signature()
            ))),
        }
    }

    fn execute_optional_sqlite_open_helper_version_method(
        &self,
        helper: &DexValue,
        database: &DexValue,
        method_name: &str,
        old_version: i32,
        new_version: i32,
    ) -> Result<bool> {
        let Some(class_descriptor) = value_class_descriptor(helper) else {
            return Ok(false);
        };
        let method = DexMethodId {
            class_descriptor: class_descriptor.to_owned(),
            name: method_name.to_owned(),
            proto: crate::dex::DexProto {
                shorty: "VLII".to_owned(),
                return_type: "V".to_owned(),
                parameters: vec![
                    "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
                    "I".to_owned(),
                    "I".to_owned(),
                ],
            },
        };
        let Some(resolved) =
            self.resolve_exact_method_in_class_hierarchy(class_descriptor, &method)?
        else {
            return Ok(false);
        };
        let result = self.execute_method(
            None,
            &resolved,
            Some(&[
                helper.clone(),
                database.clone(),
                DexValue::Int(old_version),
                DexValue::Int(new_version),
            ]),
            0,
        )?;

        match result {
            DexExecutionResult::Void => Ok(true),
            DexExecutionResult::Throw(value) => Err(WinedroidError::new(format!(
                "SQLiteOpenHelper lifecycle method threw\nclass: {class_descriptor}\nmethod: {}\nthrowable: {}{}",
                method.display_signature(),
                value,
                self.sqlite_throwable_summary(&value)
            ))),
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                Err(WinedroidError::new(format!(
                    "SQLiteOpenHelper lifecycle method parked without a guest-thread scheduler\nclass: {class_descriptor}\nmethod: {}",
                    method.display_signature()
                )))
            }
            result => Err(WinedroidError::new(format!(
                "SQLiteOpenHelper lifecycle method returned a value\nclass: {class_descriptor}\nmethod: {}\nresult: {result}",
                method.display_signature()
            ))),
        }
    }

    fn sqlite_throwable_summary(&self, throwable: &DexValue) -> String {
        let Some(DexValue::String(message)) =
            self.read_stored_instance_field(throwable, &sqlite_throwable_detail_message_field())
        else {
            return String::new();
        };

        format!("\nmessage: {message}")
    }

    fn sqlite_open_helper_set_write_ahead_logging_enabled(
        &self,
        invocation: &DexInvocation,
    ) -> Option<()> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let DexValue::Int(enabled) = invocation.arguments[1] else {
            return None;
        };
        let enabled = if enabled == 0 { 0 } else { 1 };

        self.write_stored_instance_field(
            &invocation.arguments[0],
            &sqlite_open_helper_write_ahead_logging_field(),
            DexValue::Int(enabled),
        )
        .then_some(())
    }

    fn sqlite_open_helper_database_name(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 1 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        self.read_stored_instance_field(&invocation.arguments[0], &sqlite_open_helper_name_field())
    }

    fn sqlite_database_exec_sql(
        &self,
        invocation: &DexInvocation,
    ) -> Option<std::result::Result<(), SqliteTransactionError>> {
        if invocation.arguments.len() < 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }
        let DexValue::String(sql) = &invocation.arguments[1] else {
            return None;
        };
        let bind_args = match invocation.arguments.get(2) {
            Some(value) => self.sqlite_value_args_from_array(value)?,
            None => Vec::new(),
        };

        if std::env::var_os("WINEDROID_SQLITE_TRACE").is_some() {
            eprintln!(
                "SQLite execSQL: database={} sql=`{}` binds={}",
                map_identity_key(&invocation.arguments[0]),
                sql,
                bind_args.len()
            );
        }

        if let Some(result) = self.sqlite_execute_special(&invocation.arguments[0], sql) {
            return Some(result);
        }
        self.sqlite_execute_database_sql(&invocation.arguments[0], sql, &bind_args)?;
        Some(Ok(()))
    }

    fn sqlite_database_version(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 1 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        self.sqlite_database_user_version(&invocation.arguments[0])
    }

    fn sqlite_database_set_version(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }
        let DexValue::Int(version) = invocation.arguments[1] else {
            return None;
        };

        self.sqlite_set_database_user_version(&invocation.arguments[0], version)?;
        Some(())
    }

    fn sqlite_database_compile_statement(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }
        let DexValue::String(sql) = &invocation.arguments[1] else {
            return None;
        };

        let statement = DexValue::Reference(std::rc::Rc::from(format!(
            "Landroid/database/sqlite/SQLiteStatement;:{}:{}",
            map_identity_key(&invocation.arguments[0]),
            stable_hash_i32(sql)
        )));
        let key = map_identity_key(&statement);
        {
            let mut runtime = self.runtime.borrow_mut();
            runtime
                .sqlite_statement_database
                .insert(key.clone(), invocation.arguments[0].clone());
            runtime.sqlite_statement_sql.insert(key, sql.to_string());
        }

        Some(statement)
    }

    fn sqlite_statement_bind_value(
        &self,
        invocation: &DexInvocation,
        value: fn(&DexInvocation) -> Option<(i32, DexValue)>,
    ) -> Option<()> {
        if !dex_value_is_object_like(invocation.arguments.first()?) {
            return None;
        }
        let (index, value) = value(invocation)?;
        self.write_sqlite_bind(&invocation.arguments[0], index, value)
            .then_some(())
    }

    // cite: frameworks-base SQLiteProgram.java#bindString, bindLong
    fn write_sqlite_bind(&self, statement: &DexValue, index: i32, value: DexValue) -> bool {
        let Ok(index) = usize::try_from(index) else {
            return false;
        };
        let key = map_identity_key(statement);
        self.runtime
            .borrow_mut()
            .sqlite_bind_values
            .insert((key, index), value);
        true
    }

    fn sqlite_statement_clear_bindings(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let statement_key = map_identity_key(&invocation.arguments[0]);
        self.runtime
            .borrow_mut()
            .sqlite_bind_values
            .retain(|(owner_key, _), _| owner_key != &statement_key);

        Some(())
    }

    fn sqlite_statement_execute(
        &self,
        invocation: &DexInvocation,
    ) -> Option<std::result::Result<(), SqliteTransactionError>> {
        if invocation.arguments.len() != 1 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }
        let statement = &invocation.arguments[0];
        let sql = self.sqlite_statement_sql(statement)?;

        if std::env::var_os("WINEDROID_SQLITE_TRACE").is_some() {
            eprintln!("SQLite statement execute: sql=`{sql}`");
        }

        let database = self
            .runtime
            .borrow()
            .sqlite_statement_database
            .get(&map_identity_key(statement))
            .cloned()?;
        if let Some(result) = self.sqlite_execute_special(&database, &sql) {
            return Some(result);
        }
        self.sqlite_execute_statement(statement)?;
        Some(Ok(()))
    }

    fn sqlite_statement_execute_insert(
        &self,
        invocation: &DexInvocation,
    ) -> Option<std::result::Result<i64, SqliteTransactionError>> {
        if invocation.arguments.len() != 1 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        let statement = &invocation.arguments[0];
        let database = self
            .runtime
            .borrow()
            .sqlite_statement_database
            .get(&map_identity_key(statement))
            .cloned()?;
        let sql = self.sqlite_statement_sql(statement)?;
        if let Some(result) = self.sqlite_execute_special(&database, &sql) {
            return Some(result.map(|()| 0));
        }
        Some(Ok(self.sqlite_execute_insert_statement(statement)?))
    }

    fn sqlite_database_raw_query_with_factory(
        &self,
        invocation: &DexInvocation,
    ) -> Option<std::result::Result<DexValue, SqliteTransactionError>> {
        if invocation.arguments.len() != 5 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }
        if !dex_value_is_nullable_object_like(&invocation.arguments[1]) {
            return None;
        }
        let DexValue::String(query) = &invocation.arguments[2] else {
            return None;
        };
        if !dex_value_is_nullable_object_like(&invocation.arguments[3]) {
            return None;
        }
        match &invocation.arguments[4] {
            DexValue::String(_) | DexValue::Null | DexValue::Int(0) => {}
            _ => return None,
        }

        let query_program = DexValue::Reference(std::rc::Rc::from(format!(
            "Landroid/database/sqlite/SQLiteQuery;:sqlite:{}:{}",
            map_identity_key(&invocation.arguments[0]),
            stable_hash_i32(query)
        )));
        self.sqlite_bind_selection_args_to_query(&query_program, &invocation.arguments[3])?;

        let cursor = if dex_value_is_object_like(&invocation.arguments[1]) {
            self.sqlite_cursor_from_factory(
                &invocation.arguments[1],
                &invocation.arguments[0],
                query,
                &invocation.arguments[4],
                &query_program,
            )?
        } else {
            DexValue::Reference(std::rc::Rc::from(format!(
                "Landroid/database/Cursor;:sqlite:{}:{}",
                map_identity_key(&invocation.arguments[0]),
                stable_hash_i32(query)
            )))
        };
        let cursor_key = map_identity_key(&cursor);
        let (cursor_state, bind_count) =
            if let Some(result) = self.sqlite_execute_special(&invocation.arguments[0], query) {
                match result {
                    Ok(()) => (
                        SqliteCursorState {
                            columns: Vec::new(),
                            rows: Vec::new(),
                            position: -1,
                        },
                        0,
                    ),
                    Err(error) => return Some(Err(error)),
                }
            } else {
                let bind_args =
                    self.sqlite_bound_query_args(&invocation.arguments[0], query, &query_program)?;
                (
                    self.sqlite_raw_query_result(&invocation.arguments[0], query, &bind_args)?,
                    bind_args.len(),
                )
            };
        if std::env::var_os("WINEDROID_SQLITE_TRACE").is_some() {
            eprintln!(
                "SQLite rawQuery: database={} query=`{}` binds={} columns={} rows={}",
                map_identity_key(&invocation.arguments[0]),
                query,
                bind_count,
                cursor_state.columns.len(),
                cursor_state.rows.len()
            );
        }

        self.runtime
            .borrow_mut()
            .sqlite_cursors
            .insert(cursor_key, cursor_state);

        Some(Ok(cursor))
    }

    fn sqlite_bind_selection_args_to_query(
        &self,
        query_program: &DexValue,
        selection_args: &DexValue,
    ) -> Option<()> {
        let values = match selection_args {
            DexValue::Null | DexValue::Int(0) => Vec::new(),
            DexValue::Object(_) | DexValue::Reference(_) => {
                let key = map_identity_key(selection_args);
                self.runtime.borrow().array_entries.get(&key)?.clone()
            }
            DexValue::Uninitialized
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        };

        for (index, value) in values.into_iter().enumerate() {
            let bind_index = i32::try_from(index).ok()?.checked_add(1)?;
            self.write_sqlite_bind(query_program, bind_index, value);
        }

        Some(())
    }

    fn sqlite_cursor_from_factory(
        &self,
        cursor_factory: &DexValue,
        database: &DexValue,
        query: &str,
        edit_table: &DexValue,
        query_program: &DexValue,
    ) -> Option<DexValue> {
        let class_descriptor = value_class_descriptor(cursor_factory)?.to_owned();
        let method = DexMethodId {
            class_descriptor: class_descriptor.clone(),
            name: "newCursor".to_owned(),
            proto: crate::dex::DexProto {
                shorty: "LLLLL".to_owned(),
                return_type: "Landroid/database/Cursor;".to_owned(),
                parameters: vec![
                    "Landroid/database/sqlite/SQLiteDatabase;".to_owned(),
                    "Landroid/database/sqlite/SQLiteCursorDriver;".to_owned(),
                    "Ljava/lang/String;".to_owned(),
                    "Landroid/database/sqlite/SQLiteQuery;".to_owned(),
                ],
            },
        };
        let resolved =
            match self.resolve_exact_method_in_class_hierarchy(&class_descriptor, &method) {
                Ok(Some(resolved)) => resolved,
                Ok(None) => return None,
                Err(error) => {
                    eprintln!("SQLite CursorFactory lookup failed\n  error: {error}");
                    return None;
                }
            };
        let driver = DexValue::Reference(std::rc::Rc::from(format!(
            "Landroid/database/sqlite/SQLiteCursorDriver;:sqlite:{}:{}",
            map_identity_key(database),
            stable_hash_i32(query)
        )));
        let result = match self.execute_method(
            None,
            &resolved,
            Some(&[
                cursor_factory.clone(),
                database.clone(),
                driver,
                edit_table.clone(),
                query_program.clone(),
            ]),
            0,
        ) {
            Ok(result) => result,
            Err(error) => {
                eprintln!("SQLite CursorFactory execution failed\n  error: {error}");
                return None;
            }
        };

        match result {
            DexExecutionResult::Value(value) if dex_value_is_object_like(&value) => Some(value),
            DexExecutionResult::Throw(value) => {
                eprintln!("SQLite CursorFactory threw\n  throwable: {value}");
                None
            }
            DexExecutionResult::Void
            | DexExecutionResult::Int(_)
            | DexExecutionResult::Value(_) => None,
            DexExecutionResult::Parked(execution) => {
                self.discard_parked_execution(&execution);
                None
            }
        }
    }

    fn sqlite_cursor_column_count(&self, invocation: &DexInvocation) -> Option<i32> {
        let cursor_key = sqlite_cursor_key(invocation)?;
        let columns = self.sqlite_cursor_columns(&cursor_key);

        i32::try_from(columns.len()).ok()
    }

    fn sqlite_cursor_column_name(&self, invocation: &DexInvocation) -> Option<String> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }
        let DexValue::Int(index) = invocation.arguments[1] else {
            return None;
        };
        let index = usize::try_from(index).ok()?;
        let cursor_key = map_identity_key(&invocation.arguments[0]);
        let columns = self.sqlite_cursor_columns(&cursor_key);

        columns.get(index).cloned()
    }

    fn sqlite_cursor_column_index(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }
        let DexValue::String(column_name) = &invocation.arguments[1] else {
            return None;
        };
        let cursor_key = map_identity_key(&invocation.arguments[0]);
        let columns = self.sqlite_cursor_columns(&cursor_key);

        Some(sqlite_column_index(&columns, column_name))
    }

    fn sqlite_cursor_columns(&self, cursor_key: &str) -> Vec<String> {
        self.runtime
            .borrow()
            .sqlite_cursors
            .get(cursor_key)
            .map(|cursor| cursor.columns.clone())
            .unwrap_or_default()
    }

    fn sqlite_cursor_move_to_next(&self, invocation: &DexInvocation) -> Option<i32> {
        let cursor_key = sqlite_cursor_key(invocation)?;
        let mut runtime = self.runtime.borrow_mut();
        let cursor = runtime.sqlite_cursors.get_mut(&cursor_key)?;
        let next_position = cursor.position.saturating_add(1);
        let row_count = isize::try_from(cursor.rows.len()).ok()?;

        if next_position < row_count {
            cursor.position = next_position;
            return Some(1);
        }

        cursor.position = row_count;
        Some(0)
    }

    fn sqlite_cursor_string(&self, invocation: &DexInvocation) -> Option<DexValue> {
        let value = self.sqlite_cursor_value(invocation)?;

        match value {
            DexValue::String(value) => Some(DexValue::String(value)),
            DexValue::Int(value) => Some(DexValue::String(std::rc::Rc::from(value.to_string()))),
            DexValue::Long(value) => Some(DexValue::String(std::rc::Rc::from(value.to_string()))),
            DexValue::Float(value) => Some(DexValue::String(std::rc::Rc::from(value.to_string()))),
            DexValue::Null => Some(DexValue::Null),
            DexValue::Reference(_) | DexValue::Object(_) | DexValue::Uninitialized => None,
        }
    }

    fn sqlite_cursor_int(&self, invocation: &DexInvocation) -> Option<i32> {
        let value = self.sqlite_cursor_value(invocation)?;

        match value {
            DexValue::Int(value) => Some(value),
            DexValue::Long(value) => i32::try_from(value).ok(),
            DexValue::Float(value) => Some(value as i32),
            DexValue::String(value) => value.parse::<i32>().ok(),
            DexValue::Null => Some(0),
            DexValue::Reference(_) | DexValue::Object(_) | DexValue::Uninitialized => None,
        }
    }

    fn sqlite_cursor_long(&self, invocation: &DexInvocation) -> Option<i64> {
        let value = self.sqlite_cursor_value(invocation)?;

        match value {
            DexValue::Int(value) => Some(i64::from(value)),
            DexValue::Long(value) => Some(value),
            DexValue::Float(value) => Some(value as i64),
            DexValue::String(value) => value.parse::<i64>().ok(),
            DexValue::Null => Some(0),
            DexValue::Reference(_) | DexValue::Object(_) | DexValue::Uninitialized => None,
        }
    }

    fn sqlite_cursor_is_null(&self, invocation: &DexInvocation) -> Option<i32> {
        let value = self.sqlite_cursor_value(invocation)?;

        Some(i32::from(matches!(value, DexValue::Null)))
    }

    fn sqlite_cursor_value(&self, invocation: &DexInvocation) -> Option<DexValue> {
        if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }
        let DexValue::Int(column_index) = invocation.arguments[1] else {
            return None;
        };
        let column_index = usize::try_from(column_index).ok()?;
        let cursor_key = map_identity_key(&invocation.arguments[0]);
        let runtime = self.runtime.borrow();
        let cursor = runtime.sqlite_cursors.get(&cursor_key)?;
        let position = usize::try_from(cursor.position).ok()?;
        let row = cursor.rows.get(position)?;

        Some(row.get(column_index).cloned().unwrap_or(DexValue::Null))
    }
}

fn content_observer_handler_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Landroid/database/ContentObserver;".to_owned(),
        type_descriptor: "Landroid/os/Handler;".to_owned(),
        name: "mHandler".to_owned(),
    }
}

fn autofill_value_type_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Landroid/view/autofill/AutofillValue;".to_owned(),
        type_descriptor: "I".to_owned(),
        name: "mType".to_owned(),
    }
}

fn autofill_value_value_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Landroid/view/autofill/AutofillValue;".to_owned(),
        type_descriptor: "Ljava/lang/Object;".to_owned(),
        name: "mValue".to_owned(),
    }
}

fn sqlite_open_helper_context_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Landroid/database/sqlite/SQLiteOpenHelper;".to_owned(),
        type_descriptor: "Landroid/content/Context;".to_owned(),
        name: "mContext".to_owned(),
    }
}

fn sqlite_open_helper_name_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Landroid/database/sqlite/SQLiteOpenHelper;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "mName".to_owned(),
    }
}

fn sqlite_open_helper_factory_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Landroid/database/sqlite/SQLiteOpenHelper;".to_owned(),
        type_descriptor: "Landroid/database/sqlite/SQLiteDatabase$CursorFactory;".to_owned(),
        name: "mFactory".to_owned(),
    }
}

fn sqlite_open_helper_version_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Landroid/database/sqlite/SQLiteOpenHelper;".to_owned(),
        type_descriptor: "I".to_owned(),
        name: "mNewVersion".to_owned(),
    }
}

fn sqlite_open_helper_error_handler_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Landroid/database/sqlite/SQLiteOpenHelper;".to_owned(),
        type_descriptor: "Landroid/database/DatabaseErrorHandler;".to_owned(),
        name: "mErrorHandler".to_owned(),
    }
}

fn sqlite_open_helper_write_ahead_logging_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Landroid/database/sqlite/SQLiteOpenHelper;".to_owned(),
        type_descriptor: "Z".to_owned(),
        name: "mEnableWriteAheadLogging".to_owned(),
    }
}

fn sqlite_writable_database_reference(helper: &DexValue) -> DexValue {
    DexValue::Reference(std::rc::Rc::from(format!(
        "Landroid/database/sqlite/SQLiteDatabase;:{}",
        map_identity_key(helper)
    )))
}

fn sqlite_throwable_detail_message_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Throwable;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "detailMessage".to_owned(),
    }
}

fn sqlite_database_is_open(invocation: &DexInvocation) -> Option<()> {
    (invocation.arguments.len() == 1 && dex_value_is_object_like(&invocation.arguments[0]))
        .then_some(())
}

fn android_trim_no_copy_spans_text(value: &str) -> &str {
    let Some((start, _)) = value.char_indices().find(|(_, ch)| u32::from(*ch) > 0x20) else {
        return "";
    };
    let Some((end_start, end_char)) = value
        .char_indices()
        .rev()
        .find(|(_, ch)| u32::from(*ch) > 0x20)
    else {
        return "";
    };
    let end = end_start + end_char.len_utf8();

    &value[start..end]
}

fn autofill_manager_notify_view_visibility_changed(invocation: &DexInvocation) -> Option<()> {
    if invocation.arguments.len() != 4
        || !dex_value_is_object_like(&invocation.arguments[0])
        || !dex_value_is_object_like(&invocation.arguments[1])
    {
        return None;
    }

    let DexValue::Int(_) = invocation.arguments[2] else {
        return None;
    };
    let DexValue::Int(_) = invocation.arguments[3] else {
        return None;
    };

    Some(())
}

fn autofill_manager_notify_value_changed(invocation: &DexInvocation) -> Option<()> {
    if invocation.arguments.len() != 4
        || !dex_value_is_object_like(&invocation.arguments[0])
        || !dex_value_is_object_like(&invocation.arguments[1])
        || !dex_value_is_nullable_object_like(&invocation.arguments[3])
    {
        return None;
    }

    let DexValue::Int(_) = invocation.arguments[2] else {
        return None;
    };

    Some(())
}

fn autofill_manager_notify_view_entered(invocation: &DexInvocation) -> Option<()> {
    if invocation.arguments.len() != 4
        || !dex_value_is_object_like(&invocation.arguments[0])
        || !dex_value_is_object_like(&invocation.arguments[1])
        || !dex_value_is_nullable_object_like(&invocation.arguments[3])
    {
        return None;
    }

    let DexValue::Int(_) = invocation.arguments[2] else {
        return None;
    };

    Some(())
}

fn input_method_manager_restart_input(invocation: &DexInvocation) -> Option<()> {
    if invocation.arguments.len() != 2
        || !dex_value_is_object_like(&invocation.arguments[0])
        || !dex_value_is_object_like(&invocation.arguments[1])
    {
        return None;
    }

    Some(())
}

fn input_method_manager_show_soft_input(invocation: &DexInvocation) -> Option<i32> {
    if invocation.arguments.len() != 3
        || !dex_value_is_object_like(&invocation.arguments[0])
        || !dex_value_is_object_like(&invocation.arguments[1])
    {
        return None;
    }

    let DexValue::Int(_) = invocation.arguments[2] else {
        return None;
    };

    Some(1)
}

fn sqlite_database_int_configuration(invocation: &DexInvocation) -> Option<()> {
    if invocation.arguments.len() != 2 || !dex_value_is_object_like(&invocation.arguments[0]) {
        return None;
    }
    let DexValue::Int(_) = invocation.arguments[1] else {
        return None;
    };

    Some(())
}

fn sqlite_cursor_reference(invocation: &DexInvocation) -> Option<()> {
    (invocation.arguments.len() == 1 && dex_value_is_object_like(&invocation.arguments[0]))
        .then_some(())
}

fn sqlite_cursor_constructor(invocation: &DexInvocation) -> Option<()> {
    if invocation.arguments.len() != 4 || !dex_value_is_object_like(&invocation.arguments[0]) {
        return None;
    }
    if !dex_value_is_object_like(&invocation.arguments[1]) {
        return None;
    }
    match &invocation.arguments[2] {
        DexValue::String(_) | DexValue::Null | DexValue::Int(0) => {}
        _ => return None,
    }
    if !dex_value_is_object_like(&invocation.arguments[3]) {
        return None;
    }

    Some(())
}

fn cursor_anchor_info_builder_reference(invocation: &DexInvocation) -> Option<()> {
    (invocation.arguments.len() == 1 && dex_value_is_object_like(&invocation.arguments[0]))
        .then_some(())
}

fn sqlite_statement_reference(invocation: &DexInvocation) -> Option<()> {
    (invocation.arguments.len() == 1 && dex_value_is_object_like(&invocation.arguments[0]))
        .then_some(())
}

fn sqlite_statement_bind_string(invocation: &DexInvocation) -> Option<(i32, DexValue)> {
    if invocation.arguments.len() != 3 {
        return None;
    }
    let DexValue::Int(index) = invocation.arguments[1] else {
        return None;
    };
    let DexValue::String(value) = &invocation.arguments[2] else {
        return None;
    };

    Some((index, DexValue::String(value.clone())))
}

fn sqlite_statement_bind_long(invocation: &DexInvocation) -> Option<(i32, DexValue)> {
    if invocation.arguments.len() != 4 {
        return None;
    }
    let DexValue::Int(index) = invocation.arguments[1] else {
        return None;
    };
    let DexValue::Long(value) = invocation.arguments[2] else {
        return None;
    };
    let DexValue::Long(high_slot_value) = invocation.arguments[3] else {
        return None;
    };
    if value != high_slot_value {
        return None;
    }

    Some((index, DexValue::Long(value)))
}

fn sqlite_statement_bind_null(invocation: &DexInvocation) -> Option<(i32, DexValue)> {
    if invocation.arguments.len() != 2 {
        return None;
    }
    let DexValue::Int(index) = invocation.arguments[1] else {
        return None;
    };

    Some((index, DexValue::Null))
}

fn sqlite_cursor_key(invocation: &DexInvocation) -> Option<String> {
    (invocation.arguments.len() == 1 && dex_value_is_object_like(&invocation.arguments[0]))
        .then(|| map_identity_key(&invocation.arguments[0]))
}

fn sqlite_column_index(columns: &[String], column_name: &str) -> i32 {
    let target = normalize_sqlite_identifier(column_name);
    columns
        .iter()
        .position(|column| normalize_sqlite_identifier(column) == target)
        .map(|index| i32::try_from(index).unwrap_or(-1))
        .unwrap_or(-1)
}

// cite: frameworks-base MimeTypeMap.java#getMimeTypeFromExtension
const MIME_TYPE_TABLE: &[(&str, &str)] = &[
    ("txt", "text/plain"),
    ("html", "text/html"),
    ("htm", "text/html"),
    ("css", "text/css"),
    ("csv", "text/csv"),
    ("xml", "text/xml"),
    ("json", "application/json"),
    ("pdf", "application/pdf"),
    ("zip", "application/zip"),
    ("gz", "application/gzip"),
    ("tar", "application/x-tar"),
    ("apk", "application/vnd.android.package-archive"),
    ("doc", "application/msword"),
    ("xls", "application/vnd.ms-excel"),
    ("ppt", "application/vnd.ms-powerpoint"),
    (
        "docx",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ),
    (
        "xlsx",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    ),
    (
        "pptx",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    ),
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("bmp", "image/bmp"),
    ("webp", "image/webp"),
    ("svg", "image/svg+xml"),
    ("ico", "image/x-icon"),
    ("mp3", "audio/mpeg"),
    ("ogg", "audio/ogg"),
    ("wav", "audio/x-wav"),
    ("flac", "audio/flac"),
    ("m4a", "audio/mp4"),
    ("mp4", "video/mp4"),
    ("3gp", "video/3gpp"),
    ("mkv", "video/x-matroska"),
    ("webm", "video/webm"),
    ("avi", "video/avi"),
    ("mov", "video/quicktime"),
];

fn mime_type_for_extension(extension: &str) -> Option<&'static str> {
    MIME_TYPE_TABLE
        .iter()
        .find(|(candidate, _)| *candidate == extension)
        .map(|(_, mime_type)| *mime_type)
}

fn extension_for_mime_type(mime_type: &str) -> Option<&'static str> {
    MIME_TYPE_TABLE
        .iter()
        .find(|(_, candidate)| *candidate == mime_type)
        .map(|(extension, _)| *extension)
}

fn normalize_sqlite_identifier(identifier: &str) -> String {
    let trimmed = identifier
        .trim()
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('[')
        .trim_matches(']');
    let local_name = trimmed.rsplit('.').next().unwrap_or(trimmed);

    local_name.to_ascii_lowercase()
}

fn sqlite_nullable_object_value(value: &DexValue) -> DexValue {
    match value {
        DexValue::Null | DexValue::Int(0) => DexValue::Null,
        _ => value.clone(),
    }
}

fn android_color_argb(invocation: &DexInvocation) -> Option<i32> {
    if invocation.arguments.len() != 4 {
        return None;
    }

    let DexValue::Int(alpha) = invocation.arguments[0] else {
        return None;
    };
    let DexValue::Int(red) = invocation.arguments[1] else {
        return None;
    };
    let DexValue::Int(green) = invocation.arguments[2] else {
        return None;
    };
    let DexValue::Int(blue) = invocation.arguments[3] else {
        return None;
    };

    let color =
        ((alpha & 0xff) << 24) | ((red & 0xff) << 16) | ((green & 0xff) << 8) | (blue & 0xff);

    Some(color)
}

fn android_color_space_get(invocation: &DexInvocation) -> Option<DexValue> {
    if invocation.arguments.len() != 1 {
        return None;
    }

    match &invocation.arguments[0] {
        DexValue::Reference(_) | DexValue::Object(_) => {
            Some(DexValue::Reference(std::rc::Rc::from(format!(
                "Landroid/graphics/ColorSpace;:{}",
                map_identity_key(&invocation.arguments[0])
            ))))
        }
        _ => None,
    }
}
