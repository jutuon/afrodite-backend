use base64::Engine;
use diesel::{deserialize::FromSql, sql_types::Binary, sqlite::Sqlite};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};



/// Used with database
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Eq, Hash, PartialEq, Copy)]
pub struct AccountIdInternal {
    pub account_id: uuid::Uuid,
    pub account_row_id: i64,
}

impl AccountIdInternal {
    pub fn as_uuid(&self) -> uuid::Uuid {
        self.account_id
    }

    pub fn row_id(&self) -> i64 {
        self.account_row_id
    }

    pub fn as_light(&self) -> AccountIdLight {
        AccountIdLight {
            account_id: self.account_id,
        }
    }
}

impl From<AccountIdInternal> for uuid::Uuid {
    fn from(value: AccountIdInternal) -> Self {
        value.account_id
    }
}


/// AccountId which is internally Uuid object.
/// Consumes less memory.
#[derive(
    Debug,
    Serialize,
    Deserialize,
    ToSchema,
    Clone,
    Eq,
    Hash,
    PartialEq,
    IntoParams,
    Copy,
    diesel::FromSqlRow,
)]
pub struct AccountIdLight {
    pub account_id: uuid::Uuid,
}

impl AccountIdLight {
    pub fn new(account_id: uuid::Uuid) -> Self {
        Self { account_id }
    }

    pub fn as_uuid(&self) -> uuid::Uuid {
        self.account_id
    }

    pub fn to_string(&self) -> String {
        self.account_id.hyphenated().to_string()
    }
}

impl From<AccountIdLight> for uuid::Uuid {
    fn from(value: AccountIdLight) -> Self {
        value.account_id
    }
}




#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct LoginResult {
    pub account: AuthPair,

    /// If None profile microservice is disabled.
    pub profile: Option<AuthPair>,

    /// If None media microservice is disabled.
    pub media: Option<AuthPair>,
}


/// This is just a random string.
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct ApiKey {
    /// API token which server generates.
    api_key: String,
}

impl ApiKey {
    pub fn generate_new() -> Self {
        Self {
            api_key: uuid::Uuid::new_v4().simple().to_string(),
        }
    }

    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub fn into_string(self) -> String {
        self.api_key
    }

    pub fn as_str(&self) -> &str {
        &self.api_key
    }
}

/// This is just a really long random number which is Base64 encoded.
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct RefreshToken {
    token: String,
}

impl RefreshToken {
    pub fn generate_new_with_bytes() -> (Self, Vec<u8>) {
        let mut token = Vec::new();

        // TODO: use longer refresh token
        for _ in 1..=2 {
            token.extend(uuid::Uuid::new_v4().to_bytes_le())
        }

        (Self::from_bytes(&token), token)
    }

    pub fn generate_new() -> Self {
        let (token, _bytes) = Self::generate_new_with_bytes();
        token
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            token: base64::engine::general_purpose::STANDARD.encode(data),
        }
    }

    /// String must be base64 encoded
    /// TODO: add checks?
    pub fn from_string(token: String) -> Self {
        Self { token }
    }

    /// Base64 string
    pub fn into_string(self) -> String {
        self.token
    }

    /// Base64 string
    pub fn as_str(&self) -> &str {
        &self.token
    }

    pub fn bytes(&self) -> Result<Vec<u8>, base64::DecodeError> {
        base64::engine::general_purpose::STANDARD.decode(&self.token)
    }
}

/// AccessToken and RefreshToken
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct AuthPair {
    pub refresh: RefreshToken,
    pub access: ApiKey,
}

impl AuthPair {
    pub fn new(refresh: RefreshToken, access: ApiKey) -> Self {
        Self { refresh, access }
    }
}



#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct Account {
    state: AccountState,
    capablities: Capabilities,
}

impl Account {
    pub fn new() -> Self {
        Self {
            state: AccountState::InitialSetup,
            capablities: Default::default(),
        }
    }

    pub fn new_from(state: AccountState, capablities: Capabilities) -> Self {
        Self { state, capablities }
    }

    pub fn state(&self) -> AccountState {
        self.state
    }

    pub fn capablities(&self) -> &Capabilities {
        &self.capablities
    }

    pub fn complete_setup(&mut self) {
        if self.state == AccountState::InitialSetup {
            self.state = AccountState::Normal;
        }
    }

    // pub fn complete_first_moderation(&mut self) {
    //     if self.state == AccountState::WaitingFirstModeration {
    //         self.state = AccountState::Normal;
    //     }
    // }

    pub fn add_admin_capablities(&mut self) {
        self.capablities.admin_moderate_images = true;
        self.capablities.admin_server_maintentance_view_info = true;
        self.capablities.admin_server_maintentance_update_software = true;
        // TOOD: Other capablities as well?
    }
}

impl Default for Account {
    fn default() -> Self {
        Self {
            state: AccountState::InitialSetup,
            capablities: Capabilities::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub enum AccountState {
    InitialSetup,
    /// Basically normal state, but profile is not really set public
    /// even if the capability is set.
    //WaitingFirstModeration, TODO
    Normal,
    Banned,
    PendingDeletion,
}

macro_rules! define_capablities {
    ($( $(#[doc = $text:literal ])? $name:ident , )* ) => {

        #[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default, PartialEq, Eq)]
        pub struct Capabilities {
            $(
                $(#[doc = $text])?
                #[serde(default, skip_serializing_if = "std::ops::Not::not")] // Skips false
                pub $name: bool,
            )*
        }

    };
}

define_capablities!(
    admin_modify_capablities,
    admin_setup_possible,
    admin_moderate_profiles,
    admin_moderate_images,
    /// View public and private profiles.
    admin_view_all_profiles,
    admin_view_private_info,
    admin_view_profile_history,
    admin_ban_profile,
    admin_server_maintentance_view_info,
    admin_server_maintentance_update_software,
    banned_edit_profile,
    /// View public profiles
    view_public_profiles,
);

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default, PartialEq, Eq)]
pub struct AccountSetup {
    name: String,
    email: String,
}

impl AccountSetup {
    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }

    pub fn email(&self) -> &str {
        &self.email
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, IntoParams)]
pub struct BooleanSetting {
    pub value: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct DeleteStatus {
    delete_date: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct SignInWithLoginInfo {
    pub apple_token: Option<String>,
    pub google_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SignInWithInfo {
    pub google_account_id: Option<GoogleAccountId>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type, PartialEq)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct GoogleAccountId(pub String);

impl FromSql<Binary, Sqlite> for AccountIdLight {
    fn from_sql(
        bytes: <Sqlite as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let bytes = <Vec<u8> as FromSql<Binary, Sqlite>>::from_sql(bytes)?;
        let uuid = uuid::Uuid::from_slice(&bytes)?;
        Ok(AccountIdLight::new(uuid))
    }
}