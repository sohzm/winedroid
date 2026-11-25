use super::super::*;

impl DexProgram {
    pub(super) fn invoke_java_concurrency_atomic_builtin(
        &self,
        method: &DexMethodId,
        invocation: Option<&DexInvocation>,
    ) -> Option<DexExecutionResult> {
        if method.matches_builtin_signature(
            "Ljava/util/concurrent/Executors;",
            "newFixedThreadPool",
            "(ILjava/util/concurrent/ThreadFactory;)Ljava/util/concurrent/ExecutorService;",
        ) {
            let invocation = invocation?;
            let value = fixed_thread_pool_reference(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/concurrent/FutureTask;"
            && method.name == "<init>"
            && method
                .proto
                .descriptor_is("(Ljava/util/concurrent/Callable;)V")
        {
            let invocation = invocation?;
            self.future_task_init(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/concurrent/FutureTask;"
            && method.name == "<init>"
            && method
                .proto
                .descriptor_is("(Ljava/lang/Runnable;Ljava/lang/Object;)V")
        {
            let invocation = invocation?;
            self.future_task_init_runnable(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/concurrent/FutureTask;"
            && method.name == "cancel"
            && method.proto.descriptor_is("(Z)Z")
        {
            let invocation = invocation?;
            let value = self.future_task_cancel(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/concurrent/FutureTask;"
            && method.name == "isCancelled"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = self.future_task_is_cancelled(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/concurrent/FutureTask;"
            && method.name == "isDone"
            && method.proto.descriptor_is("()Z")
        {
            let invocation = invocation?;
            let value = self.future_task_is_done(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/concurrent/Executors;"
            && method.name == "newCachedThreadPool"
            && matches!(
                method.proto.descriptor().as_str(),
                "()Ljava/util/concurrent/ExecutorService;"
                    | "(Ljava/util/concurrent/ThreadFactory;)Ljava/util/concurrent/ExecutorService;"
            )
        {
            // cite: libcore Executors.java#newCachedThreadPool
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Ljava/util/concurrent/ExecutorService;.cached"),
            )));
        }

        if method.class_descriptor == "Ljava/util/concurrent/Executors;"
            && method.name == "newSingleThreadExecutor"
            && matches!(
                method.proto.descriptor().as_str(),
                "()Ljava/util/concurrent/ExecutorService;"
                    | "(Ljava/util/concurrent/ThreadFactory;)Ljava/util/concurrent/ExecutorService;"
            )
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Ljava/util/concurrent/ExecutorService;.single"),
            )));
        }

        if method.class_descriptor == "Ljava/util/concurrent/Executors;"
            && method.name == "newSingleThreadScheduledExecutor"
            && matches!(
                method.proto.descriptor().as_str(),
                "()Ljava/util/concurrent/ScheduledExecutorService;"
                    | "(Ljava/util/concurrent/ThreadFactory;)Ljava/util/concurrent/ScheduledExecutorService;"
            )
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Ljava/util/concurrent/ScheduledExecutorService;.single"),
            )));
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/concurrent/Executor;"
                | "Ljava/util/concurrent/ExecutorService;"
                | "Ljava/util/concurrent/ScheduledExecutorService;"
        ) && method.name == "execute"
            && method.proto.descriptor_is("(Ljava/lang/Runnable;)V")
        {
            let invocation = invocation?;
            self.executor_execute(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/locks/LockSupport;",
            "parkNanos",
            "(J)V",
        ) {
            let invocation = invocation?;
            let should_park = lock_support_park_nanos(invocation)?;

            if !should_park {
                return Some(DexExecutionResult::Void);
            }

            let DexValue::Long(nanos) = invocation.arguments[0] else {
                return None;
            };

            return Some(DexExecutionResult::Parked(
                DexParkedExecution::with_park_timeout(nanos),
            ));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/locks/LockSupport;",
            "parkNanos",
            "(Ljava/lang/Object;J)V",
        ) {
            let invocation = invocation?;
            let should_park = lock_support_park_nanos_with_blocker(invocation)?;

            if !should_park {
                return Some(DexExecutionResult::Void);
            }

            let DexValue::Long(nanos) = invocation.arguments[1] else {
                return None;
            };

            return Some(DexExecutionResult::Parked(
                DexParkedExecution::with_park_timeout(nanos),
            ));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/locks/LockSupport;",
            "unpark",
            "(Ljava/lang/Thread;)V",
        ) {
            let invocation = invocation?;
            self.lock_support_unpark(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/concurrent/locks/ReentrantLock;"
            && method.name == "<init>"
            && matches!(method.proto.descriptor().as_str(), "()V" | "(Z)V")
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: libcore ReentrantReadWriteLock.java#ReentrantReadWriteLock, readLock, writeLock
        if method.class_descriptor == "Ljava/util/concurrent/locks/ReentrantReadWriteLock;" {
            match (method.name.as_str(), method.proto.descriptor().as_str()) {
                ("<init>", "()V") | ("<init>", "(Z)V") => {
                    return Some(DexExecutionResult::Void);
                }
                (
                    "readLock",
                    "()Ljava/util/concurrent/locks/ReentrantReadWriteLock$ReadLock;"
                    | "()Ljava/util/concurrent/locks/Lock;",
                ) => {
                    return Some(DexExecutionResult::Value(self.allocate_object(
                        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$ReadLock;".to_owned(),
                    )));
                }
                (
                    "writeLock",
                    "()Ljava/util/concurrent/locks/ReentrantReadWriteLock$WriteLock;"
                    | "()Ljava/util/concurrent/locks/Lock;",
                ) => {
                    return Some(DexExecutionResult::Value(self.allocate_object(
                        "Ljava/util/concurrent/locks/ReentrantReadWriteLock$WriteLock;".to_owned(),
                    )));
                }
                _ => {}
            }
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/concurrent/locks/Lock;"
                | "Ljava/util/concurrent/locks/ReentrantLock;"
                | "Ljava/util/concurrent/locks/ReentrantReadWriteLock$ReadLock;"
                | "Ljava/util/concurrent/locks/ReentrantReadWriteLock$WriteLock;"
        ) && matches!(method.name.as_str(), "lock" | "unlock")
            && method.proto.descriptor_is("()V")
        {
            return Some(DexExecutionResult::Void);
        }

        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/util/concurrent/locks/Lock;"
                | "Ljava/util/concurrent/locks/ReentrantLock;"
                | "Ljava/util/concurrent/locks/ReentrantReadWriteLock$ReadLock;"
                | "Ljava/util/concurrent/locks/ReentrantReadWriteLock$WriteLock;"
        ) && method.name == "tryLock"
            && method.proto.descriptor_is("()Z")
        {
            return Some(DexExecutionResult::Int(1));
        }

        // cite: libcore SynchronousQueue.java#SynchronousQueue
        if method.class_descriptor == "Ljava/util/concurrent/SynchronousQueue;"
            && method.name == "<init>"
            && matches!(method.proto.descriptor().as_str(), "()V" | "(Z)V")
        {
            return Some(DexExecutionResult::Void);
        }

        // cite: libcore ConcurrentLinkedQueue.java#ConcurrentLinkedQueue
        if method.matches_builtin_signature(
            "Ljava/util/concurrent/ConcurrentLinkedQueue;",
            "<init>",
            "()V",
        ) {
            return Some(DexExecutionResult::Void);
        }

        // cite: libcore ThreadPoolExecutor.java#ThreadPoolExecutor
        if method.class_descriptor == "Ljava/util/concurrent/ThreadPoolExecutor;"
            && method.name == "<init>"
            && matches!(
                method.proto.descriptor().as_str(),
                "(IIJLjava/util/concurrent/TimeUnit;Ljava/util/concurrent/BlockingQueue;)V"
                    | "(IIJLjava/util/concurrent/TimeUnit;Ljava/util/concurrent/BlockingQueue;Ljava/util/concurrent/ThreadFactory;)V"
                    | "(IIJLjava/util/concurrent/TimeUnit;Ljava/util/concurrent/BlockingQueue;Ljava/util/concurrent/RejectedExecutionHandler;)V"
                    | "(IIJLjava/util/concurrent/TimeUnit;Ljava/util/concurrent/BlockingQueue;Ljava/util/concurrent/ThreadFactory;Ljava/util/concurrent/RejectedExecutionHandler;)V"
            )
        {
            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicBoolean;",
            "<init>",
            "()V",
        ) {
            let invocation = invocation?;
            self.initialize_atomic_boolean(invocation, 0)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicBoolean;",
            "<init>",
            "(Z)V",
        ) {
            let invocation = invocation?;
            let DexValue::Int(initial_value) = invocation.arguments.get(1)? else {
                return None;
            };
            self.initialize_atomic_boolean(invocation, *initial_value)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicBoolean;",
            "get",
            "()Z",
        ) {
            let invocation = invocation?;
            let value = self.atomic_boolean_get(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/concurrent/atomic/AtomicBoolean;"
            && matches!(method.name.as_str(), "set" | "lazySet")
            && method.proto.descriptor_is("(Z)V")
        {
            let invocation = invocation?;
            self.atomic_boolean_set(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicBoolean;",
            "getAndSet",
            "(Z)Z",
        ) {
            let invocation = invocation?;
            let value = self.atomic_boolean_get_and_set(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/concurrent/atomic/AtomicBoolean;"
            && matches!(method.name.as_str(), "compareAndSet" | "weakCompareAndSet")
            && method.proto.descriptor_is("(ZZ)Z")
        {
            let invocation = invocation?;
            let value = self.atomic_boolean_compare_and_set(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicInteger;",
            "<init>",
            "()V",
        ) {
            let invocation = invocation?;
            self.initialize_atomic_integer(invocation, 0)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicInteger;",
            "<init>",
            "(I)V",
        ) {
            let invocation = invocation?;
            let DexValue::Int(initial_value) = invocation.arguments.get(1)? else {
                return None;
            };
            self.initialize_atomic_integer(invocation, *initial_value)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicInteger;",
            "incrementAndGet",
            "()I",
        ) {
            let invocation = invocation?;
            let value = self.increment_atomic_integer(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicInteger;",
            "getAndIncrement",
            "()I",
        ) {
            let invocation = invocation?;
            let value = self.get_and_increment_atomic_integer(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicInteger;",
            "decrementAndGet",
            "()I",
        ) {
            let invocation = invocation?;
            let value = self.decrement_atomic_integer(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicInteger;",
            "get",
            "()I",
        ) {
            let invocation = invocation?;
            let value = self.atomic_integer_get(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/concurrent/atomic/AtomicInteger;"
            && matches!(method.name.as_str(), "set" | "lazySet")
            && method.proto.descriptor_is("(I)V")
        {
            let invocation = invocation?;
            self.atomic_integer_set(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicInteger;",
            "addAndGet",
            "(I)I",
        ) {
            let invocation = invocation?;
            let (_previous, current) = self.atomic_integer_add(invocation)?;

            return Some(DexExecutionResult::Int(current));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicInteger;",
            "getAndAdd",
            "(I)I",
        ) {
            let invocation = invocation?;
            let (previous, _current) = self.atomic_integer_add(invocation)?;

            return Some(DexExecutionResult::Int(previous));
        }

        if method.class_descriptor == "Ljava/util/concurrent/atomic/AtomicInteger;"
            && matches!(method.name.as_str(), "compareAndSet" | "weakCompareAndSet")
            && method.proto.descriptor_is("(II)Z")
        {
            let invocation = invocation?;
            let value = self.atomic_integer_compare_and_set(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicReference;",
            "<init>",
            "()V",
        ) {
            let invocation = invocation?;
            self.initialize_empty_atomic_reference(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicReference;",
            "<init>",
            "(Ljava/lang/Object;)V",
        ) {
            let invocation = invocation?;
            self.initialize_atomic_reference(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicReference;",
            "get",
            "()Ljava/lang/Object;",
        ) {
            let invocation = invocation?;
            let value = self.atomic_reference_get(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicReference;",
            "set",
            "(Ljava/lang/Object;)V",
        ) {
            let invocation = invocation?;
            self.atomic_reference_set(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicReference;",
            "getAndSet",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
        ) {
            let invocation = invocation?;
            let value = self.atomic_reference_get_and_set(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicReference;",
            "compareAndSet",
            "(Ljava/lang/Object;Ljava/lang/Object;)Z",
        ) {
            let invocation = invocation?;
            let value = self.atomic_reference_compare_and_set(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicReferenceArray;",
            "<init>",
            "(I)V",
        ) {
            let invocation = invocation?;
            self.initialize_atomic_reference_array(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicReferenceArray;",
            "length",
            "()I",
        ) {
            let invocation = invocation?;
            let value = self.atomic_reference_array_length(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicReferenceArray;",
            "get",
            "(I)Ljava/lang/Object;",
        ) {
            let invocation = invocation?;
            let value = self.atomic_reference_array_get(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/concurrent/atomic/AtomicReferenceArray;"
            && matches!(method.name.as_str(), "set" | "lazySet")
            && method.proto.descriptor_is("(ILjava/lang/Object;)V")
        {
            let invocation = invocation?;
            self.atomic_reference_array_set(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicReferenceArray;",
            "getAndSet",
            "(ILjava/lang/Object;)Ljava/lang/Object;",
        ) {
            let invocation = invocation?;
            let value = self.atomic_reference_array_get_and_set(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicReferenceArray;",
            "compareAndSet",
            "(ILjava/lang/Object;Ljava/lang/Object;)Z",
        ) {
            let invocation = invocation?;
            let value = self.atomic_reference_array_compare_and_set(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicReferenceFieldUpdater;",
            "newUpdater",
            "(Ljava/lang/Class;Ljava/lang/Class;Ljava/lang/String;)Ljava/util/concurrent/atomic/AtomicReferenceFieldUpdater;",
        )
        {
            let invocation = invocation?;
            let value = atomic_reference_field_updater_new(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicReferenceFieldUpdater;",
            "get",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
        ) {
            let invocation = invocation?;
            let value = self.atomic_reference_field_updater_get(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: libcore AtomicReferenceFieldUpdater.java#lazySet
        if method.class_descriptor == "Ljava/util/concurrent/atomic/AtomicReferenceFieldUpdater;"
            && matches!(method.name.as_str(), "set" | "lazySet")
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;Ljava/lang/Object;)V")
        {
            let invocation = invocation?;
            self.atomic_reference_field_updater_set(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicReferenceFieldUpdater;",
            "getAndSet",
            "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        ) {
            let invocation = invocation?;
            let value = self.atomic_reference_field_updater_get_and_set(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.class_descriptor == "Ljava/util/concurrent/atomic/AtomicReferenceFieldUpdater;"
            && matches!(method.name.as_str(), "compareAndSet" | "weakCompareAndSet")
            && method.proto.descriptor()
                == "(Ljava/lang/Object;Ljava/lang/Object;Ljava/lang/Object;)Z"
        {
            let invocation = invocation?;
            let value = self.atomic_reference_field_updater_compare_and_set(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicIntegerFieldUpdater;",
            "newUpdater",
            "(Ljava/lang/Class;Ljava/lang/String;)Ljava/util/concurrent/atomic/AtomicIntegerFieldUpdater;",
        ) {
            let invocation = invocation?;
            let value = atomic_integer_field_updater_new(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicIntegerFieldUpdater;",
            "get",
            "(Ljava/lang/Object;)I",
        ) {
            let invocation = invocation?;
            let value = self.atomic_integer_field_updater_get(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.class_descriptor == "Ljava/util/concurrent/atomic/AtomicIntegerFieldUpdater;"
            && matches!(method.name.as_str(), "set" | "lazySet")
            && method.proto.descriptor_is("(Ljava/lang/Object;I)V")
        {
            let invocation = invocation?;
            self.atomic_integer_field_updater_set(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/concurrent/atomic/AtomicIntegerFieldUpdater;"
            && matches!(method.name.as_str(), "compareAndSet" | "weakCompareAndSet")
            && method.proto.descriptor_is("(Ljava/lang/Object;II)Z")
        {
            let invocation = invocation?;
            let value = self.atomic_integer_field_updater_compare_and_set(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicIntegerFieldUpdater;",
            "incrementAndGet",
            "(Ljava/lang/Object;)I",
        ) {
            let invocation = invocation?;
            let value = self.atomic_integer_field_updater_increment_and_get(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicIntegerFieldUpdater;",
            "getAndIncrement",
            "(Ljava/lang/Object;)I",
        ) {
            let invocation = invocation?;
            let value = self.atomic_integer_field_updater_get_and_increment(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicIntegerFieldUpdater;",
            "getAndDecrement",
            "(Ljava/lang/Object;)I",
        ) {
            let invocation = invocation?;
            let value = self.atomic_integer_field_updater_get_and_decrement(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicIntegerFieldUpdater;",
            "addAndGet",
            "(Ljava/lang/Object;I)I",
        ) {
            let invocation = invocation?;
            let value = self.atomic_integer_field_updater_add_and_get(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicIntegerFieldUpdater;",
            "decrementAndGet",
            "(Ljava/lang/Object;)I",
        ) {
            let invocation = invocation?;
            let value = self.atomic_integer_field_updater_decrement_and_get(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicLongFieldUpdater;",
            "newUpdater",
            "(Ljava/lang/Class;Ljava/lang/String;)Ljava/util/concurrent/atomic/AtomicLongFieldUpdater;",
        ) {
            let invocation = invocation?;
            let value = atomic_long_field_updater_new(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicLongFieldUpdater;",
            "get",
            "(Ljava/lang/Object;)J",
        ) {
            let invocation = invocation?;
            let value = self.atomic_long_field_updater_get(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicLongFieldUpdater;",
            "getAndIncrement",
            "(Ljava/lang/Object;)J",
        ) {
            let invocation = invocation?;
            let value = self.atomic_long_field_updater_get_and_increment(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicLongFieldUpdater;",
            "incrementAndGet",
            "(Ljava/lang/Object;)J",
        ) {
            let invocation = invocation?;
            let value = self.atomic_long_field_updater_increment_and_get(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicLongFieldUpdater;",
            "getAndDecrement",
            "(Ljava/lang/Object;)J",
        ) {
            let invocation = invocation?;
            let value = self.atomic_long_field_updater_get_and_decrement(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicLongFieldUpdater;",
            "decrementAndGet",
            "(Ljava/lang/Object;)J",
        ) {
            let invocation = invocation?;
            let value = self.atomic_long_field_updater_decrement_and_get(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.matches_builtin_signature(
            "Ljava/util/concurrent/atomic/AtomicLongFieldUpdater;",
            "addAndGet",
            "(Ljava/lang/Object;J)J",
        ) {
            let invocation = invocation?;
            let value = self.atomic_long_field_updater_add_and_get(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.class_descriptor == "Ljava/util/concurrent/atomic/AtomicLongFieldUpdater;"
            && matches!(method.name.as_str(), "set" | "lazySet")
            && method.proto.descriptor_is("(Ljava/lang/Object;J)V")
        {
            let invocation = invocation?;
            self.atomic_long_field_updater_set(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.class_descriptor == "Ljava/util/concurrent/atomic/AtomicLongFieldUpdater;"
            && matches!(method.name.as_str(), "compareAndSet" | "weakCompareAndSet")
            && method.proto.descriptor_is("(Ljava/lang/Object;JJ)Z")
        {
            let invocation = invocation?;
            let value = self.atomic_long_field_updater_compare_and_set(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore Unsafe.java#getUnsafe
        if method.matches_builtin_signature("Lsun/misc/Unsafe;", "getUnsafe", "()Lsun/misc/Unsafe;")
        {
            return Some(DexExecutionResult::Value(DexValue::Reference(
                std::rc::Rc::from("Lsun/misc/Unsafe;:theUnsafe"),
            )));
        }

        // cite: libcore AbstractExecutorService.java#AbstractExecutorService
        if method.matches_builtin_signature(
            "Ljava/util/concurrent/AbstractExecutorService;",
            "<init>",
            "()V",
        ) {
            return Some(DexExecutionResult::Void);
        }

        // cite: libcore LinkedBlockingQueue.java#LinkedBlockingQueue
        if method.class_descriptor == "Ljava/util/concurrent/LinkedBlockingQueue;"
            && method.name == "<init>"
            && matches!(method.proto.descriptor().as_str(), "()V" | "(I)V")
        {
            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Lsun/misc/Unsafe;",
            "objectFieldOffset",
            "(Ljava/lang/reflect/Field;)J",
        ) {
            let invocation = invocation?;
            let value = self.unsafe_object_field_offset(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        if method.matches_builtin_signature(
            "Lsun/misc/Unsafe;",
            "compareAndSwapObject",
            "(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z",
        ) {
            let invocation = invocation?;
            let value = self.unsafe_compare_and_swap_object(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore Unsafe.java#compareAndSwapInt
        if method.matches_builtin_signature(
            "Lsun/misc/Unsafe;",
            "compareAndSwapInt",
            "(Ljava/lang/Object;JII)Z",
        ) {
            let invocation = invocation?;
            let value = self.unsafe_compare_and_swap_int(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore Unsafe.java#compareAndSwapLong
        if method.matches_builtin_signature(
            "Lsun/misc/Unsafe;",
            "compareAndSwapLong",
            "(Ljava/lang/Object;JJJ)Z",
        ) {
            let invocation = invocation?;
            let value = self.unsafe_compare_and_swap_long(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore Unsafe.java#arrayBaseOffset
        if method.matches_builtin_signature(
            "Lsun/misc/Unsafe;",
            "arrayBaseOffset",
            "(Ljava/lang/Class;)I",
        ) {
            let invocation = invocation?;
            let value = self.unsafe_array_base_offset(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore Unsafe.java#arrayIndexScale
        if method.matches_builtin_signature(
            "Lsun/misc/Unsafe;",
            "arrayIndexScale",
            "(Ljava/lang/Class;)I",
        ) {
            let invocation = invocation?;
            let value = self.unsafe_array_index_scale(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        // cite: libcore Unsafe.java#putObjectVolatile, putObject, putOrderedObject
        if method.class_descriptor == "Lsun/misc/Unsafe;"
            && matches!(
                method.name.as_str(),
                "putObjectVolatile" | "putObject" | "putOrderedObject"
            )
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;JLjava/lang/Object;)V")
        {
            let invocation = invocation?;
            self.unsafe_put_object_volatile(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        // cite: libcore Unsafe.java#getObjectVolatile, getObject
        if method.class_descriptor == "Lsun/misc/Unsafe;"
            && matches!(method.name.as_str(), "getObjectVolatile" | "getObject")
            && method
                .proto
                .descriptor_is("(Ljava/lang/Object;J)Ljava/lang/Object;")
        {
            let invocation = invocation?;
            let value = self.unsafe_get_object_volatile(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        // cite: libcore Unsafe.java#putLong, putLongVolatile
        if method.class_descriptor == "Lsun/misc/Unsafe;"
            && matches!(method.name.as_str(), "putLong" | "putLongVolatile")
            && method.proto.descriptor_is("(Ljava/lang/Object;JJ)V")
        {
            let invocation = invocation?;
            self.unsafe_put_long(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        // cite: libcore Unsafe.java#getLong, getLongVolatile
        if method.class_descriptor == "Lsun/misc/Unsafe;"
            && matches!(method.name.as_str(), "getLong" | "getLongVolatile")
            && method.proto.descriptor_is("(Ljava/lang/Object;J)J")
        {
            let invocation = invocation?;
            let value = self.unsafe_get_long(invocation)?;

            return Some(DexExecutionResult::Value(DexValue::Long(value)));
        }

        // cite: libcore Unsafe.java#putInt, putIntVolatile
        if method.class_descriptor == "Lsun/misc/Unsafe;"
            && matches!(method.name.as_str(), "putInt" | "putIntVolatile")
            && method.proto.descriptor_is("(Ljava/lang/Object;JI)V")
        {
            let invocation = invocation?;
            self.unsafe_put_int(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        // cite: libcore Unsafe.java#getInt, getIntVolatile
        if method.class_descriptor == "Lsun/misc/Unsafe;"
            && matches!(method.name.as_str(), "getInt" | "getIntVolatile")
            && method.proto.descriptor_is("(Ljava/lang/Object;J)I")
        {
            let invocation = invocation?;
            let value = self.unsafe_get_int(invocation)?;

            return Some(DexExecutionResult::Int(value));
        }

        if method.matches_builtin_signature(
            "Lsun/misc/Unsafe;",
            "getAndSetObject",
            "(Ljava/lang/Object;JLjava/lang/Object;)Ljava/lang/Object;",
        ) {
            let invocation = invocation?;
            let value = self.unsafe_get_and_set_object(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature("Ljava/lang/ref/ReferenceQueue;", "<init>", "()V") {
            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/lang/ref/ReferenceQueue;",
            "poll",
            "()Ljava/lang/ref/Reference;",
        ) {
            return Some(DexExecutionResult::Value(DexValue::Null));
        }

        if method.matches_builtin_signature(
            "Ljava/lang/ref/WeakReference;",
            "<init>",
            "(Ljava/lang/Object;)V",
        ) {
            let invocation = invocation?;
            self.initialize_weak_reference(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/lang/ref/WeakReference;",
            "<init>",
            "(Ljava/lang/Object;Ljava/lang/ref/ReferenceQueue;)V",
        ) {
            let invocation = invocation?;
            self.initialize_weak_reference_with_queue(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        // cite: libcore Reference.java#clear
        if matches!(
            method.class_descriptor.as_str(),
            "Ljava/lang/ref/Reference;"
                | "Ljava/lang/ref/WeakReference;"
                | "Ljava/lang/ref/SoftReference;"
        ) && method.name == "clear"
            && method.proto.descriptor_is("()V")
        {
            let invocation = invocation?;
            self.weak_reference_clear(invocation)?;

            return Some(DexExecutionResult::Void);
        }

        if method.matches_builtin_signature(
            "Ljava/lang/ref/WeakReference;",
            "get",
            "()Ljava/lang/Object;",
        ) {
            let value = self.weak_reference_get(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        if method.matches_builtin_signature(
            "Ljava/lang/ref/Reference;",
            "get",
            "()Ljava/lang/Object;",
        ) {
            let value = self.weak_reference_get(invocation)?;

            return Some(DexExecutionResult::Value(value));
        }

        None
    }
}

fn lock_support_park_nanos(invocation: &DexInvocation) -> Option<bool> {
    if invocation.arguments.len() != 2 {
        return None;
    }

    let DexValue::Long(nanos) = invocation.arguments[0] else {
        return None;
    };
    let DexValue::Long(high_slot_nanos) = invocation.arguments[1] else {
        return None;
    };
    if nanos != high_slot_nanos {
        return None;
    }

    Some(nanos > 0)
}

fn lock_support_park_nanos_with_blocker(invocation: &DexInvocation) -> Option<bool> {
    if invocation.arguments.len() != 3 {
        return None;
    }

    if !dex_value_is_nullable_object_like(&invocation.arguments[0]) {
        return None;
    }

    let DexValue::Long(nanos) = invocation.arguments[1] else {
        return None;
    };
    let DexValue::Long(high_slot_nanos) = invocation.arguments[2] else {
        return None;
    };
    if nanos != high_slot_nanos {
        return None;
    }

    Some(nanos > 0)
}

impl DexProgram {
    fn lock_support_unpark(&self, invocation: &DexInvocation) -> Option<()> {
        if invocation.arguments.len() != 1 {
            return None;
        }

        match &invocation.arguments[0] {
            DexValue::Null | DexValue::Int(0) => Some(()),
            value if dex_value_is_object_like(value) => {
                self.runtime.borrow_mut().unpark_thread(value)?;
                Some(())
            }
            _ => None,
        }
    }
}
