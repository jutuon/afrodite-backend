//! Database writing commands
//!

use std::{
    future::Future,
    sync::{Arc, OnceLock},
};

use config::Config;
use model::AccountId;
use tokio::sync::{mpsc, Mutex, OwnedMutexGuard};

use super::{
    db_manager::{RouterDatabaseWriteHandle, SyncWriteHandle},
    write_concurrent::{
        ConcurrentWriteAction, ConcurrentWriteCommandHandle, ConcurrentWriteSelectorHandle,
    },
};
use crate::{
    db_manager::SyncWriteHandleRef,
    result::{WrappedContextExt, WrappedResultExt},
    DataError,
};

pub type WriteCmds = Cmds;

fn get_quit_lock() -> &'static Mutex<Option<mpsc::Sender<()>>> {
    /// Use static for storing the data writing quit lock as storing the Sender
    /// in WriteCommandRunnerHandle causes ongoing HTTP connections to
    /// prevent the server from shutting down.
    static QUIT_LOCK: OnceLock<Mutex<Option<mpsc::Sender<()>>>> = OnceLock::new();
    QUIT_LOCK.get_or_init(|| Mutex::new(None))
}

/// Make VSCode rust-analyzer code type annotation shorter.
/// The annotation is displayed when calling write() method.
pub struct Cmds {
    pub write: OwnedMutexGuard<SyncWriteHandle>,
}

impl std::ops::Deref for Cmds {
    type Target = OwnedMutexGuard<SyncWriteHandle>;

    fn deref(&self) -> &Self::Target {
        &self.write
    }
}

// pub struct Cmds<'a> {
//     pub write: SyncWriteHandleRef<'a>,
// }

// impl <'a> std::ops::Deref for Cmds<'a> {
//     type Target = SyncWriteHandleRef<'a>;

//     fn deref(&self) -> &Self::Target {
//         &self.write
//     }
// }

#[derive(Debug)]
pub struct WriteCommandRunnerHandle {
    sync_write_mutex: Arc<Mutex<SyncWriteHandle>>,
    concurrent_write: ConcurrentWriteCommandHandle,
}

impl WriteCommandRunnerHandle {
    pub async fn new(write: RouterDatabaseWriteHandle, config: &Config) -> (Self, WriteCmdWatcher) {
        let (quit_lock, quit_handle) = mpsc::channel::<()>(1);
        *get_quit_lock().lock().await = Some(quit_lock);

        let cmd_watcher = WriteCmdWatcher::new(quit_handle);

        let runner_handle = Self {
            sync_write_mutex: Mutex::new(write.clone().into_sync_handle()).into(),
            concurrent_write: ConcurrentWriteCommandHandle::new(write.clone(), config),
        };
        (runner_handle, cmd_watcher)
    }

    pub async fn write<
        CmdResult: Send + 'static,
        Cmd: Future<Output = crate::result::Result<CmdResult, DataError>> + Send,
        GetCmd: FnOnce(WriteCmds) -> Cmd + Send + 'static,
    >(
        &self,
        write_cmd: GetCmd,
    ) -> crate::result::Result<CmdResult, DataError> {
        let quit_lock_storage = get_quit_lock().lock().await;
        let quit_lock = quit_lock_storage
            .clone()
            .ok_or(DataError::ServerClosingInProgress.report())?;
        drop(quit_lock_storage);

        let lock = self.sync_write_mutex.clone().lock_owned().await;
        let handle = tokio::spawn(async move {
            let result = write_cmd(Cmds { write: lock }).await;
            drop(quit_lock); // Write completed, so release the quit lock.
            result
        });

        handle
            .await
            .change_context(DataError::CommandResultReceivingFailed)?
    }

    pub async fn write_with_ref_handle<
        CmdResult: Send + 'static,
        Cmd: Future<Output = crate::result::Result<CmdResult, DataError>> + Send,
        GetCmd,
    >(
        &self,
        write_cmd: GetCmd,
    ) -> crate::result::Result<CmdResult, DataError>
    where
        GetCmd: FnOnce(SyncWriteHandleRef<'_>) -> Cmd + Send + 'static,
    {
        self.write(|cmds| async move {
            let ref_handle = cmds.to_ref_handle();
            write_cmd(ref_handle).await
        })
        .await
    }

    // pub async fn test(&self) {
    //     self.write_with_ref_handle(move|cmds| async move {
    //         cmds.read().common().account_access_token(AccountId::for_debugging_only_zero()).await
    //     }).await.unwrap();
    // }

    pub async fn concurrent_write<
        CmdResult: Send + 'static,
        Cmd: Future<Output = ConcurrentWriteAction<CmdResult>> + Send,
        GetCmd: FnOnce(ConcurrentWriteSelectorHandle) -> Cmd + Send + 'static,
    >(
        &self,
        account: AccountId,
        write_cmd: GetCmd,
    ) -> crate::result::Result<CmdResult, DataError> {
        let quit_lock_storage = get_quit_lock().lock().await;
        let quit_lock = quit_lock_storage
            .clone()
            .ok_or(DataError::ServerClosingInProgress.report())?;
        drop(quit_lock_storage);

        let lock = self.concurrent_write.accquire(account).await;
        let action = write_cmd(lock).await;

        let handle = tokio::spawn(async move {
            let action_future = match action {
                ConcurrentWriteAction::Image { handle, action } => action(handle),
                ConcurrentWriteAction::Profile { handle, action } => action(handle),
            };

            let result = Box::into_pin(action_future).await;
            drop(quit_lock); // Write completed, so release the quit lock.
            result
        });

        handle
            .await
            .change_context(DataError::CommandResultReceivingFailed)
    }
}

pub struct WriteCmdWatcher {
    receiver: mpsc::Receiver<()>,
}

impl WriteCmdWatcher {
    pub fn new(receiver: mpsc::Receiver<()>) -> Self {
        Self { receiver }
    }

    pub async fn wait_untill_all_writing_ends(mut self) {
        let mut quit_lock_storage = get_quit_lock().lock().await;
        let quit_lock = quit_lock_storage.take();
        drop(quit_lock);
        drop(quit_lock_storage);

        loop {
            match self.receiver.recv().await {
                Some(_) => (),
                None => break,
            }
        }
    }
}