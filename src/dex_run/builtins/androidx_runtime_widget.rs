use super::super::*;

impl DexProgram {
    pub(super) fn invoke_androidx_runtime_widget_builtin(
        &self,
        method: &DexMethodId,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        if method.class_descriptor == "Landroidx/lifecycle/Lifecycle$Event;"
            && method.name == "getTargetState"
            && method
                .proto
                .descriptor_is("()Landroidx/lifecycle/Lifecycle$State;")
        {
            let invocation = invocation?;
            return lifecycle_event_target_state(&invocation.arguments)
                .map(DexExecutionResult::Value);
        }

        if method.class_descriptor == "Landroidx/lifecycle/k;"
            && method.name == "a"
            && method.proto.descriptor_is("()Landroidx/lifecycle/l;")
        {
            let invocation = invocation?;
            return obfuscated_lifecycle_event_target_state(&invocation.arguments)
                .map(DexExecutionResult::Value);
        }

        if method.class_descriptor == "Landroidx/lifecycle/LifecycleRegistry;"
            && method.name == "handleLifecycleEvent"
            && method
                .proto
                .descriptor_is("(Landroidx/lifecycle/Lifecycle$Event;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroidx/lifecycle/LifecycleOwnerKt;"
            && method.name == "getLifecycleScope"
            && method.proto.descriptor()
                == "(Landroidx/lifecycle/LifecycleOwner;)Landroidx/lifecycle/LifecycleCoroutineScope;"
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroidx/lifecycle/LifecycleCoroutineScope;.bootstrap"),
            )));
        }

        if method.class_descriptor == "Landroidx/lifecycle/LifecycleKt;"
            && method.name == "getCoroutineScope"
            && method.proto.descriptor()
                == "(Landroidx/lifecycle/Lifecycle;)Landroidx/lifecycle/LifecycleCoroutineScope;"
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroidx/lifecycle/LifecycleCoroutineScope;.bootstrap"),
            )));
        }

        if method.class_descriptor == "Landroidx/work/WorkManager$Companion;"
            && method.name == "getInstance"
            && method
                .proto
                .descriptor_is("(Landroid/content/Context;)Landroidx/work/WorkManager;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Landroidx/work/WorkManager;.bootstrap"),
            )));
        }

        if method.class_descriptor == "Landroidx/work/WorkManager;"
            && method.name == "getWorkInfosByTag"
            && method.proto.descriptor()
                == "(Ljava/lang/String;)Lcom/google/common/util/concurrent/ListenableFuture;"
        {
            let invocation = invocation?;
            let value = work_infos_by_tag_future(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroidx/work/WorkManager;"
            && method.name == "enqueueUniqueWork"
            && method.proto.descriptor()
                == "(Ljava/lang/String;Landroidx/work/ExistingWorkPolicy;Ljava/util/List;)Landroidx/work/Operation;"
        {
            let invocation = invocation?;
            let value = work_manager_enqueue_unique_work(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/concurrent/Future;"
            && method.name == "get"
            && method.proto.descriptor_is("()Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = future_get(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroidx/work/WorkRequest$Builder;"
            && method.name == "getThisObject$work_runtime_release"
            && method
                .proto
                .descriptor_is("()Landroidx/work/WorkRequest$Builder;")
        {
            let invocation = invocation?;
            let value = invocation.arguments.first()?.clone();

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroidx/work/WorkRequest$Builder;"
            && method.name == "buildInternal$work_runtime_release"
            && method.proto.descriptor_is("()Landroidx/work/WorkRequest;")
        {
            let invocation = invocation?;
            let value = work_request_builder_build_internal(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroidx/fragment/app/FragmentController;"
            && method.name == "dispatchCreate"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ld0/g;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(Lg/F;)V")
        {
            let invocation = invocation?;
            self.store_constructor_argument_in_field(
                invocation,
                "Ld0/g;",
                "a",
                "Ljava/lang/Object;",
            )?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroidx/appcompat/app/AppCompatDelegate;"
            && method.name == "installViewFactory"
            && method.proto.descriptor_is("()V")
        {
            // cite: androidx AppCompatDelegateImpl.java#installViewFactory
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroidx/appcompat/app/AppCompatDelegate;"
            && method.name == "setSupportActionBar"
            && method
                .proto
                .descriptor_is("(Landroidx/appcompat/widget/Toolbar;)V")
        {
            // cite: androidx AppCompatDelegateImpl.java#setSupportActionBar
            if let Err(error) = self.install_support_toolbar_action_bar() {
                return Some(DexExecutionResult::Throw(
                    self.resource_not_found_exception(error),
                ));
            }

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroidx/appcompat/app/AppCompatDelegate;"
            && method.name == "onCreate"
            && method.proto.descriptor_is("(Landroid/os/Bundle;)V")
        {
            // cite: androidx AppCompatDelegateImpl.java#onCreate
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroidx/appcompat/app/AppCompatDelegate;"
            && method.name == "getMenuInflater"
            && method.proto.descriptor_is("()Landroid/view/MenuInflater;")
        {
            // cite: androidx AppCompatDelegateImpl.java#getMenuInflater
            return Some(DexExecutionResult::Value(
                self.allocate_object("Landroid/view/MenuInflater;".to_owned()),
            ));
        }

        if method.class_descriptor == "Landroidx/appcompat/app/AppCompatDelegate;"
            && method.name == "onPostCreate"
            && method.proto.descriptor_is("(Landroid/os/Bundle;)V")
        {
            // cite: androidx AppCompatDelegateImpl.java#onPostCreate
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroidx/appcompat/app/AppCompatDelegate;"
            && matches!(
                method.name.as_str(),
                "onStart" | "onStop" | "onPostResume" | "onDestroy" | "invalidateOptionsMenu"
            )
            && method.proto.descriptor_is("()V")
        {
            // cite: androidx AppCompatDelegateImpl.java#onStart, onStop, onPostResume, onDestroy, invalidateOptionsMenu
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroidx/appcompat/app/AppCompatDelegate;"
            && method.name == "setContentView"
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.record_content_layout(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroidx/appcompat/app/AppCompatDelegate;"
            && method.name == "findViewById"
            && method.proto.descriptor_is("(I)Landroid/view/View;")
        {
            let invocation = invocation?;
            let value = self.appcompat_delegate_find_view_by_id(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Landroidx/recyclerview/widget/LinearLayoutManager;"
            && method.name == "<init>"
            && method.proto.descriptor_is("(Landroid/content/Context;IZ)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroidx/recyclerview/widget/RecyclerView;"
            && method.name == "setLayoutManager"
            && method.proto.descriptor()
                == "(Landroidx/recyclerview/widget/RecyclerView$LayoutManager;)V"
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroidx/recyclerview/widget/RecyclerView;"
            && method.name == "addItemDecoration"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Landroidx/recyclerview/widget/RecyclerView$ItemDecoration;)V"
                    | "(Landroidx/recyclerview/widget/RecyclerView$ItemDecoration;I)V"
            )
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroidx/recyclerview/widget/RecyclerView;"
            && method.name == "setAdapter"
            && method
                .proto
                .descriptor_is("(Landroidx/recyclerview/widget/RecyclerView$Adapter;)V")
        {
            if let Some(invocation) = invocation {
                self.recycler_view_set_adapter(invocation)?;
            }

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroidx/recyclerview/widget/RecyclerView;"
            && method.name == "scrollToPosition"
            && method.proto.descriptor_is("(I)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroidx/recyclerview/widget/RecyclerView;"
            && method.name == "stopScroll"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Landroidx/recyclerview/widget/ItemTouchHelper;"
            && method.name == "attachToRecyclerView"
            && method
                .proto
                .descriptor_is("(Landroidx/recyclerview/widget/RecyclerView;)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Lcom/google/android/material/tabs/TabLayout;"
            && method.name == "removeAllTabs"
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Lcom/google/android/material/tabs/TabLayout;"
            && method.name == "newTab"
            && method
                .proto
                .descriptor_is("()Lcom/google/android/material/tabs/TabLayout$Tab;")
        {
            let value = self.tab_layout_new_tab();

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Lcom/google/android/material/tabs/TabLayout;"
            && method.name == "addTab"
            && matches!(
                method.proto.descriptor().as_str(),
                "(Lcom/google/android/material/tabs/TabLayout$Tab;)V"
                    | "(Lcom/google/android/material/tabs/TabLayout$Tab;Z)V"
                    | "(Lcom/google/android/material/tabs/TabLayout$Tab;IZ)V"
            )
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Lcom/google/android/material/tabs/TabLayout;"
            && method.name == "getTabCount"
            && method.proto.descriptor_is("()I")
        {
            return Some(DexExecutionResult::Int(0));
        }

        if method.class_descriptor == "Lcom/google/android/material/tabs/TabLayout;"
            && method.name == "getSelectedTabPosition"
            && method.proto.descriptor_is("()I")
        {
            return Some(DexExecutionResult::Int(-1));
        }

        if method.class_descriptor == "Lcom/google/android/material/tabs/TabLayout;"
            && method.name == "setScrollPosition"
            && matches!(
                method.proto.descriptor().as_str(),
                "(IFZ)V" | "(IFZZ)V" | "(IFZZZ)V"
            )
        {
            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Lcom/google/android/material/tabs/TabLayout$Tab;"
            && matches!(
                method.name.as_str(),
                "setContentDescription" | "setText" | "setIcon"
            )
            && matches!(
                method.proto.descriptor().as_str(),
                "(Ljava/lang/CharSequence;)Lcom/google/android/material/tabs/TabLayout$Tab;"
                    | "(Landroid/graphics/drawable/Drawable;)Lcom/google/android/material/tabs/TabLayout$Tab;"
                    | "(I)Lcom/google/android/material/tabs/TabLayout$Tab;"
            )
        {
            let invocation = invocation?;
            let value = invocation.arguments.first()?.clone();

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Lt2/g;"
            && method.name == "c"
            && method.proto.descriptor_is("()Lt2/g;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Lt2/g;.default"),
            )));
        }

        if method.class_descriptor == "Lt2/g;"
            && method.name == "b"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Class;)Ljava/lang/Object;")
        {
            let invocation = invocation?;

            if invocation.arguments.get(1)
                == Some(&DexValue::Reference(std::rc::Rc::from(
                    "Ljava/lang/Class;:La3/e;",
                )))
            {
                return Some(DexExecutionResult::Value(DexValue::Reference(
                    std::rc::Rc::from("La3/e;.bootstrap"),
                )));
            }
        }

        None
    }
}
