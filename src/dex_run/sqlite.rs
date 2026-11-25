#[cfg(test)]
use std::fs;
#[cfg(target_os = "emscripten")]
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use rusqlite::hooks::{AuthAction, AuthContext, Authorization};
use rusqlite::params_from_iter;
use rusqlite::types::{Value, ValueRef};
use rusqlite::Connection;
#[cfg(target_os = "emscripten")]
use rusqlite::OpenFlags;

use super::*;

#[cfg(target_os = "emscripten")]
static SQLITE_PROCESS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(target_os = "emscripten")]
struct BrowserSqliteParent {
    host_path: PathBuf,
    resolved: crate::guest_filesystem::ResolvedGuestPath,
    parent: crate::capability_fs::Directory,
    leaf: std::ffi::CString,
    parent_identity: crate::capability_fs::FileIdentity,
}

#[cfg(target_os = "emscripten")]
impl BrowserSqliteParent {
    fn new(
        resolved: crate::guest_filesystem::ResolvedGuestPath,
        host_path: PathBuf,
    ) -> std::io::Result<Self> {
        let host_parent = host_path.parent().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "browser SQLite database path has no parent",
            )
        })?;
        let (parent, leaf) = resolved.directory_capability_and_leaf()?;
        let parent_identity = parent.identity()?;
        require_browser_sqlite_identity(
            ambient_file_identity(std::fs::metadata(host_parent)?),
            parent_identity,
            "database parent",
        )?;

        Ok(Self {
            host_path,
            resolved,
            parent,
            leaf,
            parent_identity,
        })
    }

    fn finish(self) -> std::io::Result<BrowserSqlitePath> {
        let database_identity = self.parent.child_identity(&self.leaf)?;
        require_browser_sqlite_identity(
            ambient_file_identity(std::fs::symlink_metadata(&self.host_path)?),
            database_identity,
            "database file",
        )?;

        Ok(BrowserSqlitePath {
            host_path: self.host_path,
            _resolved: self.resolved,
            parent: self.parent,
            leaf: self.leaf,
            parent_identity: self.parent_identity,
            database_identity,
        })
    }
}

#[cfg(target_os = "emscripten")]
impl BrowserSqlitePath {
    pub(super) fn validate(&self) -> std::io::Result<()> {
        let host_parent = self.host_path.parent().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "browser SQLite database path has no parent",
            )
        })?;
        require_browser_sqlite_identity(
            self.parent.identity()?,
            self.parent_identity,
            "retained database parent",
        )?;
        require_browser_sqlite_identity(
            ambient_file_identity(std::fs::metadata(host_parent)?),
            self.parent_identity,
            "database parent",
        )?;
        require_browser_sqlite_identity(
            self.parent.child_identity(&self.leaf)?,
            self.database_identity,
            "retained database file",
        )?;
        require_browser_sqlite_identity(
            ambient_file_identity(std::fs::symlink_metadata(&self.host_path)?),
            self.database_identity,
            "database file",
        )
    }
}

#[cfg(target_os = "emscripten")]
fn ambient_file_identity(metadata: std::fs::Metadata) -> crate::capability_fs::FileIdentity {
    crate::capability_fs::FileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    }
}

#[cfg(target_os = "emscripten")]
fn require_browser_sqlite_identity(
    actual: crate::capability_fs::FileIdentity,
    expected: crate::capability_fs::FileIdentity,
    target: &str,
) -> std::io::Result<()> {
    if actual == expected {
        return Ok(());
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        format!("browser SQLite {target} identity changed"),
    ))
}

#[cfg(not(target_os = "emscripten"))]
fn sqlite_process_lock() -> impl Drop {
    crate::capability_sqlite::process_lock()
}

#[cfg(target_os = "emscripten")]
fn sqlite_process_lock() -> std::sync::MutexGuard<'static, ()> {
    match SQLITE_PROCESS_LOCK.try_lock() {
        Ok(guard) => guard,
        Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
        Err(std::sync::TryLockError::WouldBlock) => {
            panic!("browser SQLite operation re-entered the inline runtime")
        }
    }
}

enum SqliteDatabaseStorage {
    Guest {
        display_path: PathBuf,
        resolved: crate::guest_filesystem::ResolvedGuestPath,
    },
    #[cfg(test)]
    Host(PathBuf),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct SqliteSessionKey {
    pub(super) database: String,
    pub(super) thread: String,
}

pub(super) struct SqliteTransactionState {
    frames: Vec<SqliteTransactionFrame>,
}

struct SqliteTransactionFrame {
    marked_successful: bool,
    child_failed: bool,
}

enum SqliteTransactionEnd {
    Nested,
    Commit,
    Rollback,
}

pub(super) enum SqliteTransactionError {
    IllegalState(&'static str),
    Database(String),
    Unsupported(&'static str),
}

const SQLITE_NO_TRANSACTION_MESSAGE: &str =
    "Cannot perform this operation because there is no current transaction.";
const SQLITE_ALREADY_SUCCESSFUL_MESSAGE: &str = "Cannot perform this operation because the transaction has already been marked successful.  The only thing you can do now is call endTransaction().";
pub(super) const SQLITE_CONNECTION_WAIT_MESSAGE: &str = "SQLite connection-pool waiting across guest threads is not implemented yet\nnext missing primitive: SQLite connection-pool guest-thread waiting";

impl SqliteTransactionState {
    fn new() -> Self {
        Self {
            frames: vec![SqliteTransactionFrame {
                marked_successful: false,
                child_failed: false,
            }],
        }
    }

    fn begin_nested(&mut self) -> std::result::Result<(), SqliteTransactionError> {
        // cite: frameworks-base SQLiteSession.java#beginTransaction, beginTransactionUnchecked, throwIfTransactionMarkedSuccessful, Transaction
        if self
            .frames
            .last()
            .is_some_and(|frame| frame.marked_successful)
        {
            return Err(SqliteTransactionError::IllegalState(
                SQLITE_ALREADY_SUCCESSFUL_MESSAGE,
            ));
        }
        self.frames.push(SqliteTransactionFrame {
            marked_successful: false,
            child_failed: false,
        });
        Ok(())
    }

    fn mark_successful(&mut self) -> std::result::Result<(), SqliteTransactionError> {
        // cite: frameworks-base SQLiteSession.java#setTransactionSuccessful, throwIfNoTransaction, throwIfTransactionMarkedSuccessful
        let Some(frame) = self.frames.last_mut() else {
            return Err(SqliteTransactionError::IllegalState(
                SQLITE_NO_TRANSACTION_MESSAGE,
            ));
        };
        if frame.marked_successful {
            return Err(SqliteTransactionError::IllegalState(
                SQLITE_ALREADY_SUCCESSFUL_MESSAGE,
            ));
        }
        frame.marked_successful = true;
        Ok(())
    }

    fn end(&mut self) -> std::result::Result<SqliteTransactionEnd, SqliteTransactionError> {
        // cite: frameworks-base SQLiteSession.java#endTransaction, endTransactionUnchecked, throwIfNoTransaction, Transaction
        let Some(frame) = self.frames.pop() else {
            return Err(SqliteTransactionError::IllegalState(
                SQLITE_NO_TRANSACTION_MESSAGE,
            ));
        };
        let successful = frame.marked_successful && !frame.child_failed;
        if let Some(parent) = self.frames.last_mut() {
            if !successful {
                parent.child_failed = true;
            }
            return Ok(SqliteTransactionEnd::Nested);
        }
        if successful {
            Ok(SqliteTransactionEnd::Commit)
        } else {
            Ok(SqliteTransactionEnd::Rollback)
        }
    }
}

impl DexProgram {
    pub(super) fn register_sqlite_database_name(
        &self,
        database: &DexValue,
        context: &DexValue,
        name: Option<&str>,
    ) -> Result<PathBuf> {
        // cite: frameworks-base SQLiteOpenHelper.java#getDatabaseLocked
        // cite: frameworks-base ContextImpl.java#getDatabasePath, getDatabasesDir, getDataDir
        let storage = self
            .context_private_storage_kind(context)
            .ok_or_else(|| WinedroidError::new("SQLite Context has no private-storage identity"))?;
        match self.sqlite_database_storage_path(name, storage)? {
            SqliteDatabaseStorage::Guest {
                display_path,
                resolved,
            } => {
                self.register_sqlite_guest_database_path(database, &display_path, resolved)?;
                Ok(display_path)
            }
            #[cfg(test)]
            SqliteDatabaseStorage::Host(path) => {
                self.register_sqlite_database_path(database, &path)?;
                Ok(path)
            }
        }
    }

    pub(super) fn sqlite_database_user_version(&self, database: &DexValue) -> Option<i32> {
        self.with_sqlite_connection(database, |connection| {
            connection
                .query_row("PRAGMA user_version", [], |row| row.get::<_, i32>(0))
                .ok()
        })
    }

    pub(super) fn sqlite_set_database_user_version(
        &self,
        database: &DexValue,
        version: i32,
    ) -> Option<()> {
        self.with_sqlite_connection(database, |connection| {
            connection
                .pragma_update(None, "user_version", version)
                .map_err(|error| self.trace_sqlite_error("set user_version", &error))
                .ok()?;

            Some(())
        })
    }

    pub(super) fn sqlite_execute_database_sql(
        &self,
        database: &DexValue,
        sql: &str,
        bind_args: &[Value],
    ) -> Option<()> {
        self.with_sqlite_connection(database, |connection| {
            // cite: frameworks-base SQLiteDatabase.java#execSQL
            // cite: frameworks-base SQLiteConnection.java#acquirePreparedStatement
            let mut statement = connection
                .prepare(sql)
                .map_err(|error| self.trace_sqlite_error("prepare execSQL", &error))
                .ok()?;
            if bind_args.is_empty() {
                statement
                    .raw_execute()
                    .map_err(|error| self.trace_sqlite_error("execSQL", &error))
                    .ok()?;
                return Some(());
            }

            statement
                .execute(params_from_iter(bind_args.iter()))
                .map_err(|error| self.trace_sqlite_error("execSQL", &error))
                .ok()?;
            Some(())
        })
    }

    pub(super) fn sqlite_execute_statement(&self, statement_ref: &DexValue) -> Option<()> {
        let database = self
            .runtime
            .borrow()
            .sqlite_statement_database
            .get(&map_identity_key(statement_ref))
            .cloned()?;
        let sql = self.sqlite_statement_sql(statement_ref)?;
        self.with_sqlite_connection(&database, |connection| {
            let mut statement = connection
                .prepare(&sql)
                .map_err(|error| self.trace_sqlite_error("prepare statement", &error))
                .ok()?;
            let values =
                self.sqlite_statement_bind_values(statement_ref, statement.parameter_count())?;

            statement
                .execute(params_from_iter(values.iter()))
                .map_err(|error| self.trace_sqlite_error("execute statement", &error))
                .ok()?;
            Some(())
        })
    }

    pub(super) fn sqlite_execute_insert_statement(&self, statement_ref: &DexValue) -> Option<i64> {
        let database = self
            .runtime
            .borrow()
            .sqlite_statement_database
            .get(&map_identity_key(statement_ref))
            .cloned()?;
        let sql = self.sqlite_statement_sql(statement_ref)?;
        self.with_sqlite_connection(&database, |connection| {
            let mut statement = connection
                .prepare(&sql)
                .map_err(|error| self.trace_sqlite_error("prepare insert statement", &error))
                .ok()?;
            let values =
                self.sqlite_statement_bind_values(statement_ref, statement.parameter_count())?;

            statement
                .execute(params_from_iter(values.iter()))
                .map_err(|error| self.trace_sqlite_error("execute insert statement", &error))
                .ok()?;
            Some(connection.last_insert_rowid())
        })
    }

    pub(super) fn sqlite_insert_content_values(&self, invocation: &DexInvocation) -> Option<i64> {
        if invocation.arguments.len() != 4 {
            return None;
        }
        if !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }
        let DexValue::String(table) = &invocation.arguments[1] else {
            return None;
        };
        let null_column_hack = match &invocation.arguments[2] {
            DexValue::String(value) => Some(value.clone()),
            DexValue::Null | DexValue::Int(0) => None,
            DexValue::Uninitialized
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::Object(_)
            | DexValue::Reference(_) => return None,
        };
        let values_key = match &invocation.arguments[3] {
            DexValue::Object(_) | DexValue::Reference(_) => {
                map_identity_key(&invocation.arguments[3])
            }
            DexValue::Uninitialized
            | DexValue::Null
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => return None,
        };

        let mut entries = Vec::new();
        {
            let runtime = self.runtime.borrow();
            for ((content_values_key, entry_key), value) in &runtime.content_values_entries {
                if content_values_key != &values_key {
                    continue;
                }

                entries.push((entry_key.clone(), value.clone()));
            }
        }
        entries.sort_by(|left, right| left.0.cmp(&right.0));

        let (sql, values) = sqlite_insert_sql(table, null_column_hack.as_deref(), entries)?;
        if std::env::var_os("WINEDROID_SQLITE_TRACE").is_some() {
            eprintln!(
                "SQLite insert: database={} sql=`{}` values={}",
                map_identity_key(&invocation.arguments[0]),
                sql,
                values.len()
            );
        }

        self.with_sqlite_connection(&invocation.arguments[0], |connection| {
            connection
                .execute(&sql, params_from_iter(values.iter()))
                .map_err(|error| self.trace_sqlite_error("insert", &error))
                .ok()?;
            Some(connection.last_insert_rowid())
        })
    }

    pub(super) fn sqlite_raw_query_result(
        &self,
        database: &DexValue,
        query: &str,
        bind_args: &[Value],
    ) -> Option<SqliteCursorState> {
        self.with_sqlite_connection(database, |connection| {
            let mut statement = connection
                .prepare(query)
                .map_err(|error| self.trace_sqlite_error("prepare query", &error))
                .ok()?;
            let columns = statement
                .column_names()
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            let column_count = columns.len();
            let mut rows = statement
                .query(params_from_iter(bind_args.iter()))
                .map_err(|error| self.trace_sqlite_error("query", &error))
                .ok()?;
            let mut values = Vec::new();

            while let Some(row) = rows
                .next()
                .map_err(|error| self.trace_sqlite_error("read query row", &error))
                .ok()?
            {
                let mut row_values = Vec::with_capacity(column_count);
                for index in 0..column_count {
                    let value = row
                        .get_ref(index)
                        .map(sqlite_value_ref_to_dex_value)
                        .map_err(|error| self.trace_sqlite_error("read query value", &error))
                        .ok()?;
                    row_values.push(value);
                }
                values.push(row_values);
            }

            Some(SqliteCursorState {
                columns,
                rows: values,
                position: -1,
            })
        })
    }

    pub(super) fn sqlite_bound_query_args(
        &self,
        database: &DexValue,
        query: &str,
        query_program: &DexValue,
    ) -> Option<Vec<Value>> {
        self.with_sqlite_connection(database, |connection| {
            let statement = connection
                .prepare(query)
                .map_err(|error| self.trace_sqlite_error("prepare query bindings", &error))
                .ok()?;

            self.sqlite_statement_bind_values(query_program, statement.parameter_count())
        })
    }

    pub(super) fn sqlite_value_args_from_array(&self, value: &DexValue) -> Option<Vec<Value>> {
        match value {
            DexValue::Null | DexValue::Int(0) => Some(Vec::new()),
            DexValue::Object(_) | DexValue::Reference(_) => {
                let key = map_identity_key(value);
                let runtime = self.runtime.borrow();
                let values = runtime.array_entries.get(&key)?;

                values.iter().map(dex_value_to_sqlite_value).collect()
            }
            DexValue::Uninitialized
            | DexValue::Int(_)
            | DexValue::Long(_)
            | DexValue::Float(_)
            | DexValue::String(_) => None,
        }
    }

    pub(super) fn sqlite_statement_sql(&self, statement: &DexValue) -> Option<String> {
        self.runtime
            .borrow()
            .sqlite_statement_sql
            .get(&map_identity_key(statement))
            .cloned()
    }

    pub(super) fn sqlite_begin_transaction(
        &self,
        invocation: &DexInvocation,
        exclusive: bool,
    ) -> Option<std::result::Result<(), SqliteTransactionError>> {
        if invocation.arguments.len() != 1 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        self.sqlite_begin_transaction_for_database(&invocation.arguments[0], exclusive)
    }

    fn sqlite_begin_transaction_for_database(
        &self,
        database: &DexValue,
        exclusive: bool,
    ) -> Option<std::result::Result<(), SqliteTransactionError>> {
        let session_key = self.sqlite_session_key(database);
        let _process_lock = sqlite_process_lock();
        // cite: frameworks-base SQLiteConnectionPool.java#waitForConnection, setMaxConnectionPoolSizeLocked
        if self.sqlite_sessions.borrow().iter().any(|(key, session)| {
            key.database == session_key.database
                && key.thread != session_key.thread
                && session.transaction.is_some()
        }) {
            return Some(Err(SqliteTransactionError::Unsupported(
                SQLITE_CONNECTION_WAIT_MESSAGE,
            )));
        }
        self.ensure_sqlite_session(database, &session_key)?;
        let mut sessions = self.sqlite_sessions.borrow_mut();
        let session = sessions.get_mut(&session_key)?;
        if let Some(transaction) = session.transaction.as_mut() {
            return Some(transaction.begin_nested());
        }
        if let Err(error) = session.connection.validate_path() {
            return Some(Err(SqliteTransactionError::Database(format!(
                "SQLite capability validation failed: {error}"
            ))));
        }
        // cite: frameworks-base SQLiteDatabase.java#beginTransaction, beginTransactionNonExclusive
        // cite: frameworks-base SQLiteSession.java#beginTransactionUnchecked
        let sql = if exclusive {
            "BEGIN EXCLUSIVE;"
        } else {
            "BEGIN IMMEDIATE;"
        };
        match session.connection.connection().execute_batch(sql) {
            Ok(()) => {
                session.transaction = Some(SqliteTransactionState::new());
                Some(Ok(()))
            }
            Err(error) => {
                self.trace_sqlite_error("begin transaction", &error);
                Some(Err(SqliteTransactionError::Database(error.to_string())))
            }
        }
    }

    pub(super) fn sqlite_set_transaction_successful(
        &self,
        invocation: &DexInvocation,
    ) -> Option<std::result::Result<(), SqliteTransactionError>> {
        if invocation.arguments.len() != 1 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        self.sqlite_set_transaction_successful_for_database(&invocation.arguments[0])
    }

    fn sqlite_set_transaction_successful_for_database(
        &self,
        database: &DexValue,
    ) -> Option<std::result::Result<(), SqliteTransactionError>> {
        let session_key = self.sqlite_session_key(database);
        let _process_lock = sqlite_process_lock();
        let mut sessions = self.sqlite_sessions.borrow_mut();
        let Some(session) = sessions.get_mut(&session_key) else {
            return Some(Err(SqliteTransactionError::IllegalState(
                SQLITE_NO_TRANSACTION_MESSAGE,
            )));
        };
        let Some(transaction) = session.transaction.as_mut() else {
            return Some(Err(SqliteTransactionError::IllegalState(
                SQLITE_NO_TRANSACTION_MESSAGE,
            )));
        };
        Some(transaction.mark_successful())
    }

    pub(super) fn sqlite_end_transaction(
        &self,
        invocation: &DexInvocation,
    ) -> Option<std::result::Result<(), SqliteTransactionError>> {
        if invocation.arguments.len() != 1 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        self.sqlite_end_transaction_for_database(&invocation.arguments[0])
    }

    fn sqlite_end_transaction_for_database(
        &self,
        database: &DexValue,
    ) -> Option<std::result::Result<(), SqliteTransactionError>> {
        let session_key = self.sqlite_session_key(database);
        let _process_lock = sqlite_process_lock();
        let mut sessions = self.sqlite_sessions.borrow_mut();
        let (result, evict) = {
            let Some(session) = sessions.get_mut(&session_key) else {
                return Some(Err(SqliteTransactionError::IllegalState(
                    SQLITE_NO_TRANSACTION_MESSAGE,
                )));
            };
            let Some(transaction) = session.transaction.as_mut() else {
                return Some(Err(SqliteTransactionError::IllegalState(
                    SQLITE_NO_TRANSACTION_MESSAGE,
                )));
            };
            if let Err(error) = session.connection.validate_path() {
                return Some(Err(SqliteTransactionError::Database(format!(
                    "SQLite capability validation failed: {error}"
                ))));
            }
            let end = match transaction.end() {
                Ok(end) => end,
                Err(error) => return Some(Err(error)),
            };
            match end {
                SqliteTransactionEnd::Nested => return Some(Ok(())),
                SqliteTransactionEnd::Commit => {
                    session.transaction = None;
                    match session.connection.connection().execute_batch("COMMIT;") {
                        Ok(()) => (Ok(()), false),
                        Err(error) => {
                            self.trace_sqlite_error("commit transaction", &error);
                            let cleanup_failed = session
                                .connection
                                .connection()
                                .execute_batch("ROLLBACK;")
                                .is_err();
                            (
                                Err(SqliteTransactionError::Database(error.to_string())),
                                cleanup_failed,
                            )
                        }
                    }
                }
                SqliteTransactionEnd::Rollback => {
                    session.transaction = None;
                    match session.connection.connection().execute_batch("ROLLBACK;") {
                        Ok(()) => (Ok(()), false),
                        Err(error) => {
                            self.trace_sqlite_error("rollback transaction", &error);
                            (
                                Err(SqliteTransactionError::Database(error.to_string())),
                                true,
                            )
                        }
                    }
                }
            }
        };
        let evicted_session = if evict {
            sessions.remove(&session_key)
        } else {
            None
        };
        drop(sessions);
        drop(_process_lock);
        drop(evicted_session);
        Some(result)
    }

    pub(super) fn sqlite_execute_special(
        &self,
        database: &DexValue,
        sql: &str,
    ) -> Option<std::result::Result<(), SqliteTransactionError>> {
        // cite: frameworks-base SQLiteSession.java#executeSpecial
        // cite: frameworks-base DatabaseUtils.java#getSqlStatementType, STATEMENT_BEGIN, STATEMENT_COMMIT, STATEMENT_ABORT
        let sql = sql.trim();
        let prefix = sql.get(..3)?.to_ascii_uppercase();
        match prefix.as_str() {
            "BEG" => self.sqlite_begin_transaction_for_database(database, true),
            "COM" | "END" => {
                match self.sqlite_set_transaction_successful_for_database(database)? {
                    Ok(()) => self.sqlite_end_transaction_for_database(database),
                    Err(error) => Some(Err(error)),
                }
            }
            "ROL" if !sql.to_ascii_uppercase().contains(" TO ") => {
                self.sqlite_end_transaction_for_database(database)
            }
            _ => None,
        }
    }

    pub(super) fn sqlite_transaction_exception(&self, error: SqliteTransactionError) -> DexValue {
        let (class_descriptor, message) = match error {
            SqliteTransactionError::IllegalState(message) => {
                ("Ljava/lang/IllegalStateException;", message.to_owned())
            }
            SqliteTransactionError::Database(message) => {
                ("Landroid/database/sqlite/SQLiteException;", message)
            }
            SqliteTransactionError::Unsupported(message) => (
                "Ljava/lang/UnsupportedOperationException;",
                message.to_owned(),
            ),
        };
        let exception = self.allocate_object(class_descriptor.to_owned());
        self.write_stored_instance_field(
            &exception,
            &sqlite_transaction_detail_message_field(),
            DexValue::String(std::rc::Rc::from(message)),
        );
        exception
    }

    pub(super) fn sqlite_in_transaction(&self, invocation: &DexInvocation) -> Option<i32> {
        if invocation.arguments.len() != 1 || !dex_value_is_object_like(&invocation.arguments[0]) {
            return None;
        }

        // cite: frameworks-base SQLiteDatabase.java#mThreadSession, getThreadSession, inTransaction
        // cite: frameworks-base SQLiteSession.java#hasTransaction
        let session_key = self.sqlite_session_key(&invocation.arguments[0]);
        let in_transaction = self
            .sqlite_sessions
            .borrow()
            .get(&session_key)
            .is_some_and(|session| session.transaction.is_some());

        Some(i32::from(in_transaction))
    }

    fn with_sqlite_connection<T>(
        &self,
        database: &DexValue,
        operation: impl FnOnce(&Connection) -> Option<T>,
    ) -> Option<T> {
        let _process_lock = sqlite_process_lock();
        let session_key = self.sqlite_session_key(database);
        self.ensure_sqlite_session(database, &session_key)?;
        let sessions = self.sqlite_sessions.borrow();
        let runtime_connection = &sessions.get(&session_key)?.connection;
        if let Err(error) = runtime_connection.validate_path() {
            eprintln!("SQLite capability validation failed\n  error: {error}");
            return None;
        }
        operation(runtime_connection.connection())
    }

    pub(super) fn sqlite_session_key(&self, database: &DexValue) -> SqliteSessionKey {
        // cite: frameworks-base SQLiteDatabase.java#mThreadSession, getThreadSession
        SqliteSessionKey {
            database: map_identity_key(database),
            thread: map_identity_key(&self.runtime.borrow().current_guest_thread),
        }
    }

    fn ensure_sqlite_session(&self, database: &DexValue, key: &SqliteSessionKey) -> Option<()> {
        if self.sqlite_sessions.borrow().contains_key(key) {
            return Some(());
        }
        let connection = self.open_runtime_sqlite_connection(database)?;
        self.sqlite_sessions.borrow_mut().insert(
            key.clone(),
            RuntimeSqliteSession {
                connection,
                transaction: None,
            },
        );
        Some(())
    }

    fn open_runtime_sqlite_connection(
        &self,
        database: &DexValue,
    ) -> Option<RuntimeSqliteConnection> {
        let path = self.sqlite_database_path(database).ok()?;
        let guest_path = self
            .sqlite_guest_database_paths
            .borrow()
            .get(&map_identity_key(database))
            .cloned();
        #[cfg(target_os = "emscripten")]
        let connection = {
            let guest_path = guest_path?;
            if guest_path.host_path() != path {
                eprintln!(
                    "SQLite browser path did not match its guest capability\n  path: {}",
                    path.display()
                );
                return None;
            }
            match guest_path.metadata() {
                Ok(metadata) if metadata.kind == crate::capability_fs::MetadataKind::File => {}
                Ok(_) => {
                    eprintln!(
                        "SQLite browser path is not a regular file\n  path: {}",
                        path.display()
                    );
                    return None;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    eprintln!(
                        "SQLite browser path validation failed\n  path: {}\n  error: {error}",
                        path.display()
                    );
                    return None;
                }
            }
            let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_NOFOLLOW;
            let browser_parent = BrowserSqliteParent::new(guest_path, path.clone())
                .map_err(|error| {
                    eprintln!(
                        "SQLite browser capability pin failed\n  path: {}\n  error: {error}",
                        path.display()
                    );
                })
                .ok()?;
            let connection = Connection::open_with_flags(&path, flags)
                .map_err(|error| {
                    eprintln!(
                        "SQLite open failed\n  path: {}\n  error: {error}",
                        path.display()
                    );
                })
                .ok()?;
            connection
                .busy_timeout(std::time::Duration::ZERO)
                .map_err(|error| {
                    eprintln!(
                        "SQLite browser busy policy failed\n  path: {}\n  error: {error}",
                        path.display()
                    );
                })
                .ok()?;
            let browser_path = browser_parent
                .finish()
                .map_err(|error| {
                    eprintln!(
                        "SQLite browser capability pin failed\n  path: {}\n  error: {error}",
                        path.display()
                    );
                })
                .ok()?;
            RuntimeSqliteConnection::Browser {
                connection: std::mem::ManuallyDrop::new(connection),
                path: browser_path,
            }
        };
        #[cfg(not(target_os = "emscripten"))]
        let connection = if let Some(guest_path) = guest_path {
            let (directory, leaf) = guest_path.directory_capability_and_leaf().ok()?;
            crate::capability_sqlite::CapabilityConnection::open(directory, leaf)
                .map_err(|error| {
                    eprintln!("SQLite capability VFS failed\n  error: {error}");
                })
                .ok()
                .map(RuntimeSqliteConnection::Guest)?
        } else {
            #[cfg(test)]
            {
                Connection::open(&path)
                    .map_err(|error| {
                        eprintln!(
                            "SQLite open failed\n  path: {}\n  error: {error}",
                            path.display()
                        );
                    })
                    .ok()
                    .map(RuntimeSqliteConnection::Host)?
            }
            #[cfg(not(test))]
            {
                let _ = path;
                return None;
            }
        };
        connection
            .connection()
            .authorizer(Some(sqlite_host_path_authorization));
        Some(connection)
    }

    pub(in crate::dex_run) fn sqlite_database_path(&self, database: &DexValue) -> Result<PathBuf> {
        let key = map_identity_key(database);
        if let Some(path) = self
            .runtime
            .borrow()
            .sqlite_database_paths
            .get(&key)
            .cloned()
        {
            return Ok(path);
        }

        Err(WinedroidError::new(format!(
            "SQLite database `{key}` has no registered Context path"
        )))
    }

    fn sqlite_database_storage_path(
        &self,
        name: Option<&str>,
        storage: crate::guest_filesystem::PrivateStorageKind,
    ) -> Result<SqliteDatabaseStorage> {
        // cite: frameworks-base SQLiteOpenHelper.java#getDatabaseLocked
        // cite: frameworks-base ContextImpl.java#getDatabasePath, makeFilename
        let Some(database_name) = name else {
            return Err(WinedroidError::new(
                "SQLiteOpenHelper in-memory databases are not implemented yet\nnext missing primitive: SQLiteOpenHelper in-memory databases",
            ));
        };
        if database_name.is_empty() {
            return Err(WinedroidError::new("SQLite database name is empty"));
        }
        if !database_name.starts_with('/') && database_name.contains('/') {
            return Err(WinedroidError::new(format!(
                "SQLite database name `{database_name}` contains a path separator"
            )));
        }

        if let Some(filesystem) = &self.guest_filesystem {
            // cite: frameworks-base SQLiteOpenHelper.java#getDatabaseLocked
            // cite: frameworks-base ContextImpl.java#getDatabasePath, getDatabasesDir, getDataDir
            let (directory, guest_path) = if database_name.starts_with('/') {
                let parent = database_name
                    .rsplit_once('/')
                    .map(|(dir, _)| if dir.is_empty() { "/" } else { dir })
                    .unwrap_or("/");
                (parent.to_owned(), database_name.to_owned())
            } else {
                let directory = filesystem.private_guest_path(storage, "databases");
                let guest_path = format!("{directory}/{database_name}");
                (directory, guest_path)
            };
            filesystem
                .resolve(
                    &directory,
                    crate::external_storage::ExternalStorageAccess::Write,
                    &self.external_storage_policy,
                )?
                .mkdir_all()
                .map_err(|error| {
                    WinedroidError::with_source("failed to create SQLite database directory", error)
                })?;
            let resolved = filesystem.resolve(
                &guest_path,
                crate::external_storage::ExternalStorageAccess::Write,
                &self.external_storage_policy,
            )?;
            #[cfg(any(test, target_os = "emscripten"))]
            let display_path = resolved.host_path().to_path_buf();
            #[cfg(all(not(test), not(target_os = "emscripten")))]
            let display_path = PathBuf::from(&guest_path);
            return Ok(SqliteDatabaseStorage::Guest {
                display_path,
                resolved,
            });
        }

        #[cfg(test)]
        {
            let Some(data_dir) = (match storage {
                crate::guest_filesystem::PrivateStorageKind::CredentialProtected => {
                    self._test_app_data_dir.as_ref()
                }
                crate::guest_filesystem::PrivateStorageKind::DeviceProtected => {
                    self.app_device_data_dir.as_ref()
                }
            }) else {
                return Err(WinedroidError::new(format!(
                    "SQLite database `{database_name}` has no private-storage root"
                )));
            };

            return Ok(SqliteDatabaseStorage::Host(
                data_dir.join("databases").join(database_name),
            ));
        }

        #[cfg(not(test))]
        Err(WinedroidError::new(format!(
            "SQLite database `{database_name}` has no guest filesystem"
        )))
    }

    #[cfg(test)]
    pub(super) fn register_sqlite_database_path(
        &self,
        database: &DexValue,
        path: &Path,
    ) -> Result<()> {
        let parent = path.parent().ok_or_else(|| {
            WinedroidError::new(format!(
                "SQLite database path has no parent directory `{}`",
                path.display()
            ))
        })?;
        fs::create_dir_all(parent).map_err(|error| {
            WinedroidError::with_source(
                format!(
                    "failed to create SQLite database directory `{}`",
                    parent.display()
                ),
                error,
            )
        })?;

        let key = map_identity_key(database);
        let old_path = self
            .runtime
            .borrow()
            .sqlite_database_paths
            .get(&key)
            .cloned();
        self.runtime
            .borrow_mut()
            .sqlite_database_paths
            .insert(key.clone(), path.to_path_buf());
        let was_guest = self
            .sqlite_guest_database_paths
            .borrow_mut()
            .remove(&key)
            .is_some();
        if old_path.as_deref() != Some(path) || was_guest {
            self.sqlite_sessions
                .borrow_mut()
                .retain(|session, _| session.database != key);
        }
        Ok(())
    }

    fn register_sqlite_guest_database_path(
        &self,
        database: &DexValue,
        display_path: &Path,
        resolved: crate::guest_filesystem::ResolvedGuestPath,
    ) -> Result<()> {
        let key = map_identity_key(database);
        let old_path = self
            .runtime
            .borrow()
            .sqlite_database_paths
            .get(&key)
            .cloned();
        self.runtime
            .borrow_mut()
            .sqlite_database_paths
            .insert(key.clone(), display_path.to_path_buf());
        let was_guest = self
            .sqlite_guest_database_paths
            .borrow_mut()
            .insert(key.clone(), resolved)
            .is_some();
        if old_path.as_deref() != Some(display_path) || !was_guest {
            self.sqlite_sessions
                .borrow_mut()
                .retain(|session, _| session.database != key);
        }
        Ok(())
    }

    fn sqlite_statement_bind_values(
        &self,
        statement_ref: &DexValue,
        parameter_count: usize,
    ) -> Option<Vec<Value>> {
        let mut values = vec![Value::Null; parameter_count];
        let owner_key = instance_field_owner_key(statement_ref)?;
        let runtime = self.runtime.borrow();

        for ((bind_owner_key, index), value) in runtime.sqlite_bind_values.iter() {
            if bind_owner_key != &owner_key {
                continue;
            }
            if *index == 0 || *index > parameter_count {
                continue;
            }

            values[*index - 1] = dex_value_to_sqlite_value(value)?;
        }

        Some(values)
    }

    fn trace_sqlite_error(&self, operation: &str, error: &rusqlite::Error) {
        eprintln!("SQLite {operation} failed\n  error: {error}");
    }
}

fn sqlite_host_path_authorization(context: AuthContext<'_>) -> Authorization {
    match context.action {
        AuthAction::Attach { filename } if filename.is_empty() => Authorization::Allow,
        AuthAction::Attach { .. } => Authorization::Deny,
        AuthAction::Unknown { code, .. } if code == rusqlite::ffi::SQLITE_ATTACH => {
            Authorization::Deny
        }
        AuthAction::Pragma { pragma_name, .. }
            if sqlite_pragma_accepts_or_exposes_host_path(pragma_name) =>
        {
            Authorization::Deny
        }
        AuthAction::Pragma {
            pragma_name,
            pragma_value: Some(_),
        } if cfg!(target_os = "emscripten") && pragma_name.eq_ignore_ascii_case("busy_timeout") => {
            Authorization::Deny
        }
        _ => Authorization::Allow,
    }
}

fn sqlite_transaction_detail_message_field() -> DexFieldId {
    DexFieldId {
        class_descriptor: "Ljava/lang/Throwable;".to_owned(),
        type_descriptor: "Ljava/lang/String;".to_owned(),
        name: "detailMessage".to_owned(),
    }
}

fn sqlite_pragma_accepts_or_exposes_host_path(pragma_name: &str) -> bool {
    [
        "database_list",
        "data_store_directory",
        "lock_proxy_file",
        "temp_store_directory",
    ]
    .iter()
    .any(|name| pragma_name.eq_ignore_ascii_case(name))
}

fn sqlite_insert_sql(
    table: &str,
    null_column_hack: Option<&str>,
    entries: Vec<(String, DexValue)>,
) -> Option<(String, Vec<Value>)> {
    if entries.is_empty() {
        if let Some(column) = null_column_hack {
            let sql = format!(
                "INSERT INTO {} ({}) VALUES (NULL)",
                sqlite_quote_identifier(table),
                sqlite_quote_identifier(column)
            );
            return Some((sql, Vec::new()));
        }

        return Some((
            format!(
                "INSERT INTO {} DEFAULT VALUES",
                sqlite_quote_identifier(table)
            ),
            Vec::new(),
        ));
    }

    let mut columns = Vec::with_capacity(entries.len());
    let mut placeholders = Vec::with_capacity(entries.len());
    let mut values = Vec::with_capacity(entries.len());

    for (column, value) in entries {
        columns.push(sqlite_quote_identifier(&column));
        placeholders.push("?".to_owned());
        values.push(dex_value_to_sqlite_value(&value)?);
    }

    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        sqlite_quote_identifier(table),
        columns.join(", "),
        placeholders.join(", ")
    );
    Some((sql, values))
}

fn dex_value_to_sqlite_value(value: &DexValue) -> Option<Value> {
    match value {
        DexValue::Null => Some(Value::Null),
        DexValue::Int(value) => Some(Value::Integer(i64::from(*value))),
        DexValue::Long(value) => Some(Value::Integer(*value)),
        DexValue::Float(value) => Some(Value::Real(f64::from(*value))),
        DexValue::String(value) => Some(Value::Text(value.clone().to_string())),
        DexValue::Reference(reference) if reference.as_ref() == "Ljava/lang/Boolean;:true" => {
            Some(Value::Integer(1))
        }
        DexValue::Reference(reference) if reference.as_ref() == "Ljava/lang/Boolean;:false" => {
            Some(Value::Integer(0))
        }
        DexValue::Reference(reference) => {
            if let Some(value) = boxed_integer_reference_value(reference) {
                return Some(Value::Integer(value));
            }
            if let Some(value) = boxed_float_reference_value(reference) {
                return Some(Value::Real(f64::from(value)));
            }

            None
        }
        DexValue::Object(_) | DexValue::Uninitialized => None,
    }
}

fn sqlite_value_ref_to_dex_value(value: ValueRef<'_>) -> DexValue {
    match value {
        ValueRef::Null => DexValue::Null,
        ValueRef::Integer(value) => DexValue::Long(value),
        ValueRef::Real(value) => DexValue::Float(value as f32),
        ValueRef::Text(value) => DexValue::String(std::rc::Rc::from(
            String::from_utf8_lossy(value).into_owned(),
        )),
        ValueRef::Blob(_) => DexValue::Null,
    }
}

fn sqlite_quote_identifier(value: &str) -> String {
    format!("`{}`", value.replace('`', "``"))
}
