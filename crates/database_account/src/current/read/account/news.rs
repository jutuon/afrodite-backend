use database::{define_current_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::{alias, prelude::*};
use error_stack::Result;
use model::{AccountId, AccountIdInternal, NewsId, NewsItem, NewsItemInternal, NewsItemSimple, NewsLocale, NewsSyncVersion, NewsTranslationInternal, PageItemCountForNewPublicNews, PublicationId, RequireNewsLocale, UnreadNewsCount};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountNews, CurrentSyncReadAccountNews);

impl<C: ConnectionProvider> CurrentSyncReadAccountNews<C> {
    pub fn news_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<NewsSyncVersion, DieselDatabaseError> {
        use crate::schema::account_state::dsl::*;

        account_state
            .filter(account_id.eq(id.as_db_id()))
            .select(news_sync_version)
            .first(self.conn())
            .into_db_error(id)
    }

    pub fn unread_news_count(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<UnreadNewsCount, DieselDatabaseError> {
        use crate::schema::account_state::dsl::*;

        account_state
            .filter(account_id.eq(id.as_db_id()))
            .select(unread_news_count)
            .first(self.conn())
            .into_db_error(id)
    }

    /// News ordering goes from recent to older starting
    /// from `id_value`.
    pub fn paged_news(
        &mut self,
        id_value: PublicationId,
        previous_id_value: Option<PublicationId>,
        page: i64,
        locale_value: NewsLocale,
        include_private_news: bool,
    ) -> Result<(Vec<NewsItemSimple>, PageItemCountForNewPublicNews), DieselDatabaseError> {
        use crate::schema::{news, news_translations};

        let private_rows: Vec<(NewsItemInternal, Option<String>)> = if page == 0 && include_private_news {
            news::table
                .left_outer_join(
                    news_translations::table.on(
                        news::id.eq(news_translations::news_id).and(
                            news_translations::locale.eq(locale_value.locale.clone()).or(news_translations::locale.eq(NewsLocale::ENGLISH))
                        )
                    ),
                )
                .filter(news::publication_id.is_null())
                .select((
                    NewsItemInternal::as_select(),
                    news_translations::title.nullable(),
                ))
                .order(news::id.desc())
                .load(self.conn())
                .into_db_error(())?
        } else {
            vec![]
        };

        const PAGE_SIZE: i64 = 25;

        let public_rows: Vec<(NewsItemInternal, Option<String>, PublicationId)> = news::table
            .left_outer_join(
                news_translations::table.on(
                    news::id.eq(news_translations::news_id).and(
                        news_translations::locale.eq(locale_value.locale).or(news_translations::locale.eq(NewsLocale::ENGLISH))
                    )
                ),
            )
            .filter(news::publication_id.is_not_null())
            .filter(news::publication_id.le(id_value))
            .select((
                NewsItemInternal::as_select(),
                news_translations::title.nullable(),
                news::publication_id.assume_not_null(),
            ))
            .order(news::publication_id.desc())
            .limit(PAGE_SIZE)
            .offset(PAGE_SIZE.saturating_mul(page))
            .load(self.conn())
            .into_db_error(())?;

        let mut new_items_count = 0;

        let items = private_rows
            .into_iter()
            .map(|r| {
                NewsItemSimple {
                    id: r.0.id,
                    title: r.1,
                    time: r.0.first_publication_unix_time,
                    private: r.0.publication_id.is_none()
                }
            })
            .chain(
                public_rows
                    .into_iter()
                    .map(|r| {
                        if let Some(previous_id_value) = previous_id_value {
                            if r.2.id > previous_id_value.id {
                                new_items_count += 1;
                            }
                        }
                        NewsItemSimple {
                            id: r.0.id,
                            title: r.1,
                            time: r.0.first_publication_unix_time,
                            private: r.0.publication_id.is_none()
                        }
                    })
            )
            .collect();

        Ok((items, PageItemCountForNewPublicNews { c: new_items_count }))
    }

    pub fn news_item(
        &mut self,
        news_id_value: NewsId,
        locale_value: NewsLocale,
        require_locale: RequireNewsLocale,
    ) -> Result<Option<NewsItem>, DieselDatabaseError> {
        use crate::schema::{account_id, news, news_translations};

        let (creator_aid, editor_aid) = alias!(account_id as creator_aid, account_id as editor_aid);

        let value: Option<(NewsTranslationInternal, Option<AccountId>, Option<AccountId>)> = news::table
            .inner_join(
                news_translations::table.on(
                    news::id.eq(news_translations::news_id).and(
                        news_translations::locale.eq(locale_value.locale.clone()).or(news_translations::locale.eq(NewsLocale::ENGLISH))
                    )
                ),
            )
            .left_outer_join(
                creator_aid.on(
                    news_translations::account_id_creator.assume_not_null().eq(creator_aid.field(account_id::id))
                ),
            )
            .left_outer_join(
                editor_aid.on(
                    news_translations::account_id_editor.assume_not_null().eq(editor_aid.field(account_id::id))
                ),
            )
            .filter(news::id.eq(news_id_value))
            .select((
                NewsTranslationInternal::as_select(),
                creator_aid.field(account_id::uuid).nullable(),
                editor_aid.field(account_id::uuid).nullable()
            ))
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        let (internal, creator, editor) = if let Some(value) = value {
            value
        } else {
            return Ok(None);
        };

        if require_locale.require_locale && internal.locale != locale_value.locale {
            return Ok(None);
        }

        let news_item = NewsItem {
            title: internal.title,
            body: internal.body,
            locale: internal.locale,
            creation_time: internal.creation_unix_time,
            edit_unix_time: internal.edit_unix_time.map(|x| x.ut),
            version: Some(internal.version_number),
            aid_creator: creator,
            aid_editor: editor,
        };

        Ok(Some(news_item))
    }

    pub fn is_public(
        &mut self,
        news_id_value: NewsId,
    ) -> Result<bool, DieselDatabaseError> {
        use crate::schema::news;

        let value = news::table
            .filter(news::id.eq(news_id_value))
            .select(NewsItemInternal::as_select())
            .first(self.conn())
            .into_db_error(())?;

        Ok(value.publication_id.is_some())
    }

    pub fn publication_id_at_news_iterator_reset(
        &mut self,
        id_value: AccountIdInternal,
    ) -> Result<Option<PublicationId>, DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        account_state
            .filter(account_id.eq(id_value.as_db_id()))
            .select(publication_id_at_news_iterator_reset)
            .first(self.conn())
            .into_db_error(())
    }
}
