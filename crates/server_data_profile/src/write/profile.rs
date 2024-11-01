use model::{
    AccountIdInternal, Location, ProfileAttributeFilterListUpdateValidated, ProfileSearchAgeRangeValidated, ProfileStateInternal, ProfileUpdateInternal, ValidatedSearchGroups
};
use server_data::{
    cache::CacheError,
    define_server_data_write_commands,
    index::location::LocationIndexIteratorState,
    result::{Result, WrappedContextExt},
    write::WriteCommandsProvider,
    DataError, IntoDataError,
};
use tracing::info;

define_server_data_write_commands!(WriteCommandsProfile);
define_db_read_command_for_write!(WriteCommandsProfile);
define_db_transaction_command!(WriteCommandsProfile);

impl<C: WriteCommandsProvider> WriteCommandsProfile<C> {
    pub async fn profile_update_location(
        self,
        id: AccountIdInternal,
        coordinates: Location,
    ) -> Result<(), DataError> {
        let location = self
            .cache()
            .read_cache(id.as_id(), |e| {
                e.profile.as_ref().map(|p| p.location.clone())
            })
            .await
            .into_data_error(id)?
            .ok_or(DataError::FeatureDisabled.report())?;

        let new_location_key = self.location().coordinates_to_key(&coordinates);
        db_transaction!(self, move |mut cmds| {
            cmds.profile().data().profile_location(id, coordinates)
        })?;

        self.location()
            .update_profile_location(id.as_id(), location.current_position, new_location_key)
            .await?;

        let new_iterator_state = self
            .location_iterator()
            .reset_iterator(LocationIndexIteratorState::new(), new_location_key);
        self.write_cache(id, |entry| {
            let p = entry
                .profile
                .as_mut()
                .ok_or(CacheError::FeatureNotEnabled)?;
            p.location.current_position = new_location_key;
            p.location.current_iterator = new_iterator_state;
            Ok(())
        })
        .await?;

        Ok(())
    }

    /// Updates [model::Profile].
    ///
    /// Updates also [model::ProfileSyncVersion].
    ///
    /// Check also
    /// [crate::write::profile_admin::profile_name_allowlist::WriteCommandsProfileAdminProfileNameAllowlist::moderate_profile_name]
    /// and from other `server_data_all`
    /// `UnlimitedLikesUpdate::update_unlimited_likes_value`
    /// as those also modifies the [model::Profile].
    pub async fn profile(
        self,
        id: AccountIdInternal,
        data: ProfileUpdateInternal,
    ) -> Result<(), DataError> {
        let profile_data = data.clone();
        let config = self.config_arc().clone();
        let account = db_transaction!(self, move |mut cmds| {
            let name_update_detected = {
                let current_profile = cmds.read().profile().data().profile(id)?;
                current_profile.name != profile_data.new_data.name
            };
            cmds.profile().data().profile(id, &profile_data)?;
            cmds.profile()
                .data()
                .upsert_profile_attributes(id, profile_data.new_data.attributes, config.profile_attributes())?;
            cmds.profile().data().increment_profile_sync_version(id)?;
            if name_update_detected {
                cmds.profile()
                    .profile_name_allowlist()
                    .reset_profile_name_accepted_and_denied_values(
                        id,
                        &profile_data.new_data.name,
                        config.profile_name_allowlist(),
                    )?;
            }
            cmds.read().common().account(id)
        })?;

        let (location, profile_data) = self
            .cache()
            .write_cache(id.as_id(), |e| {
                let p = e.profile.as_mut().ok_or(CacheError::FeatureNotEnabled)?;

                p.data.update_from(&data.new_data);
                p.attributes.update_from(&data.new_data);
                p.data.version_uuid = data.version;

                Ok((p.location.current_position, e.location_index_profile_data()?))
            })
            .await
            .into_data_error(id)?;

        if account.profile_visibility().is_currently_public() {
            self.location()
                .update_profile_data(id.as_id(), profile_data, location)
                .await?;
        }

        Ok(())
    }

    async fn modify_profile_state(
        self,
        id: AccountIdInternal,
        action: impl FnOnce(&mut ProfileStateInternal),
    ) -> Result<(), DataError> {
        let mut s = self
            .db_read(move |mut cmd| cmd.profile().data().profile_state(id))
            .await?;
        action(&mut s);
        let s_cloned = s.clone();
        let account = db_transaction!(self, move |mut cmds| {
            cmds.profile().data().profile_state(id, s_cloned)?;
            cmds.read().common().account(id)
        })?;

        let (location, profile_data) = self
            .cache()
            .write_cache(id.as_id(), |e| {
                let p = e.profile.as_mut().ok_or(CacheError::FeatureNotEnabled)?;

                p.state = s.into();

                Ok((p.location.current_position, e.location_index_profile_data()?))
            })
            .await
            .into_data_error(id)?;

        if account.profile_visibility().is_currently_public() {
            self.location()
                .update_profile_data(id.as_id(), profile_data, location)
                .await?;
        }

        Ok(())
    }

    pub async fn update_profile_attribute_filters(
        self,
        id: AccountIdInternal,
        filters: ProfileAttributeFilterListUpdateValidated,
    ) -> Result<(), DataError> {
        let config = self.config_arc().clone();
        let new_filters = db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .data()
                .upsert_profile_attribute_filters(id, filters.filters, config.profile_attributes())?;
            cmds.profile()
                .data()
                .update_last_seen_time_filter(id, filters.last_seen_time_filter)?;
            cmds.profile()
                .data()
                .update_unlimited_likes_filter(id, filters.unlimited_likes_filter)?;
            cmds.read().profile().data().profile_attribute_filters(id)
        })?;

        self.cache()
            .write_cache(id.as_id(), |e| {
                let p = e.profile.as_mut().ok_or(CacheError::FeatureNotEnabled)?;
                p.filters = new_filters;
                p.state.last_seen_time_filter = filters.last_seen_time_filter;
                p.state.unlimited_likes_filter = filters.unlimited_likes_filter;
                Ok(())
            })
            .await
            .into_data_error(id)?;

        Ok(())
    }

    pub async fn profile_name(self, id: AccountIdInternal, data: String) -> Result<(), DataError> {
        let profile_data = data.clone();
        db_transaction!(self, move |mut cmds| {
            cmds.profile().data().profile_name(id, profile_data)
        })?;

        self.cache()
            .write_cache(id.as_id(), |e| {
                let p = e.profile.as_mut().ok_or(CacheError::FeatureNotEnabled)?;
                p.data.name = data;
                Ok(())
            })
            .await
            .into_data_error(id)?;

        Ok(())
    }

    pub async fn insert_favorite_profile(
        self,
        id: AccountIdInternal,
        favorite: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .favorite()
                .insert_favorite_profile(id, favorite)
        })
    }

    pub async fn remove_favorite_profile(
        self,
        id: AccountIdInternal,
        favorite: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .favorite()
                .remove_favorite_profile(id, favorite)
        })
    }

    /// Updates the profile attributes sha256 and sync version for it for every
    /// account if needed.
    pub async fn update_profile_attributes_sha256_and_sync_versions(
        self,
        sha256: String,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            let current_hash = cmds.read().profile().data().attribute_file_hash()?;

            if current_hash.as_deref() != Some(&sha256) {
                info!(
                    "Profile attributes file hash changed from {:?} to {:?}",
                    current_hash,
                    Some(&sha256)
                );

                cmds.profile()
                    .data()
                    .upsert_profile_attributes_file_hash(&sha256)?;

                cmds.profile()
                    .data()
                    .increment_profile_attributes_sync_version_for_every_account()?;
            }

            Ok(())
        })
    }

    /// Only server WebSocket code should call this method.
    pub async fn reset_profile_attributes_sync_version(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .data()
                .reset_profile_attributes_sync_version(id)
        })
    }

    /// Only server WebSocket code should call this method.
    pub async fn reset_profile_sync_version(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .data()
                .reset_profile_sync_version(id)
        })
    }

    pub async fn update_search_groups(
        self,
        id: AccountIdInternal,
        search_groups: ValidatedSearchGroups,
    ) -> Result<(), DataError> {
        self.modify_profile_state(id, |s| s.search_group_flags = search_groups.into())
            .await
    }

    pub async fn update_search_age_range(
        self,
        id: AccountIdInternal,
        range: ProfileSearchAgeRangeValidated,
    ) -> Result<(), DataError> {
        self.modify_profile_state(id, |s| {
            s.search_age_range_min = range.min();
            s.search_age_range_max = range.max();
        })
        .await
    }

    pub async fn benchmark_update_profile_bypassing_cache(
        self,
        id: AccountIdInternal,
        data: ProfileUpdateInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile().data().profile(id, &data)
        })
    }

    pub async fn update_last_seen_time_from_cache_to_database(
        self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        let last_seen_time = self.cache()
            .read_cache(id, |e| e.last_seen_time_for_db())
            .await?;

        db_transaction!(self, move |mut cmds| {
            cmds.profile().data().profile_last_seen_time(id, last_seen_time)
        })
    }

    pub async fn set_initial_profile_age_from_current_profile(
        self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            let profile = cmds.read().profile().data().profile(id)?;
            cmds.profile().data().initial_profile_age(id, profile.age)
        })
    }
}
