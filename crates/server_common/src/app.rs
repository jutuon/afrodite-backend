use config::file::ConfigFileError;
use error_stack::Result;
use model::{AccountId, AccountIdInternal, BackendConfig, BackendVersion, EmailMessages};
use simple_backend::email::EmailSender;

use crate::data::DataError;

pub trait WriteDynamicConfig {
    fn write_config(
        &self,
        config: BackendConfig,
    ) -> impl std::future::Future<Output = error_stack::Result<(), ConfigFileError>> + Send;
}

pub trait ReadDynamicConfig {
    fn read_config(
        &self,
    ) -> impl std::future::Future<Output = error_stack::Result<BackendConfig, ConfigFileError>> + Send;
}

pub trait BackendVersionProvider {
    fn backend_version(&self) -> BackendVersion;
}

/// All accounts registered in the service.
pub trait GetAccounts {
    fn get_internal_id(
        &self,
        id: AccountId,
    ) -> impl std::future::Future<Output = Result<AccountIdInternal, DataError>> + Send;

    fn get_internal_id_optional(
        &self,
        id: AccountId,
    ) -> impl std::future::Future<Output = Option<AccountIdInternal>> + Send;
}

// pub trait FileAccessProvider {
//     fn file_access(&self) -> &FileDir;
// }

// impl FileAccessProvider for S {
//     fn file_access(&self) -> &FileDir {
//         &self.business_logic_state().
//     }
// }

pub type EmailSenderImpl = EmailSender<AccountIdInternal, EmailMessages>;
