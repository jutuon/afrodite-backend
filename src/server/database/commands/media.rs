use super::{WriteCommandRunnerHandle, ResultSender, WriteCommandRunner, SendBack};



use std::{collections::HashSet, future::Future, net::SocketAddr, sync::Arc};

use axum::extract::BodyStream;
use error_stack::Result;

use tokio::{
    sync::{mpsc, oneshot, OwnedSemaphorePermit, RwLock, Semaphore},
    task::JoinHandle,
};
use tokio_stream::StreamExt;

use crate::{
    api::{
        media::data::{HandleModerationRequest, Moderation},
        model::{
            Account, AccountIdInternal, AccountIdLight, AccountSetup, AuthPair, ContentId,
            Location, ModerationRequestContent, ProfileLink,
            ProfileUpdateInternal, SignInWithInfo,
        },
    },
    config::Config,
    server::database::{write::WriteCommands, DatabaseError},
    utils::{ErrorConversion, IntoReportExt},
};

use super::{super::file::file::ImageSlot, RouterDatabaseWriteHandle};



/// Synchronized write commands.
#[derive(Debug)]
pub enum MediaWriteCommand {
    SetModerationRequest {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        request: ModerationRequestContent,
    },
    GetModerationListAndCreateNewIfNecessary {
        s: ResultSender<Vec<Moderation>>,
        account_id: AccountIdInternal,
    },
    SaveToSlot {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    },
    UpdateModeration {
        s: ResultSender<()>,
        moderator_id: AccountIdInternal,
        moderation_request_owner: AccountIdInternal,
        result: HandleModerationRequest,
    },
}


#[derive(Debug, Clone)]
pub struct MediaWriteCommandRunnerHandle<'a> {
    pub handle: &'a WriteCommandRunnerHandle,
}

impl MediaWriteCommandRunnerHandle<'_> {
    pub async fn set_moderation_request(
        &self,
        account_id: AccountIdInternal,
        request: ModerationRequestContent,
    ) -> Result<(), DatabaseError> {
        self.handle.send_event(|s| MediaWriteCommand::SetModerationRequest {
            s,
            account_id,
            request,
        })
        .await
    }

    pub async fn get_moderation_list_and_create_if_necessary(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Vec<Moderation>, DatabaseError> {
        self.handle.send_event(|s| MediaWriteCommand::GetModerationListAndCreateNewIfNecessary {
            s,
            account_id,
        })
        .await
    }

    pub async fn update_moderation(
        &self,
        moderator_id: AccountIdInternal,
        moderation_request_owner: AccountIdInternal,
        result: HandleModerationRequest,
    ) -> Result<(), DatabaseError> {
        self.handle.send_event(|s| MediaWriteCommand::UpdateModeration {
            s,
            moderator_id,
            moderation_request_owner,
            result,
        })
        .await
    }

    pub async fn save_to_slot(
        &self,
        account_id: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    ) -> Result<(), DatabaseError> {
        self.handle.send_event(|s| MediaWriteCommand::SaveToSlot {
            s,
            account_id,
            content_id,
            slot,
        })
        .await
    }
}


impl WriteCommandRunner {
    pub async fn handle_media_cmd(&self, cmd: MediaWriteCommand) {
        match cmd {
            MediaWriteCommand::SetModerationRequest {
                s,
                account_id,
                request,
            } => self
                .write()
                .set_moderation_request(account_id, request)
                .await
                .send(s),
            MediaWriteCommand::GetModerationListAndCreateNewIfNecessary { s, account_id } => self
                .write()
                .moderation_get_list_and_create_new_if_necessary(account_id)
                .await
                .send(s),
            MediaWriteCommand::SaveToSlot {
                s,
                account_id,
                content_id,
                slot,
            } => self
                .write()
                .save_to_slot(account_id, content_id, slot)
                .await
                .send(s),
            MediaWriteCommand::UpdateModeration {
                s,
                moderator_id,
                moderation_request_owner,
                result,
            } => self
                .write()
                .update_moderation(moderator_id, moderation_request_owner, result)
                .await
                .send(s),
        }
    }
}