use server_data::{cache::{account::CachedAccountComponentData, CacheError}, db_manager::InternalReading};

use model::AccountId;

use error_stack::Result;

pub trait CacheWriteAccount {
    async fn write_cache_account<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CachedAccountComponentData) -> Result<T, CacheError>,
    ) -> Result<T, CacheError>;
}

impl <I: InternalReading> CacheWriteAccount for I {
    async fn write_cache_account<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CachedAccountComponentData) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        self.cache().write_cache(id, |e| {
            let a = e.account_data_mut()?;
            cache_operation(a)
        }).await
    }
}