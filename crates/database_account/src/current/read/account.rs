use database::{define_current_read_commands, ConnectionProvider};

define_current_read_commands!(CurrentReadAccount, CurrentSyncReadAccount);

mod data;
mod demo;
mod sign_in_with;
mod email;
mod news;

impl<C: ConnectionProvider> CurrentSyncReadAccount<C> {
    pub fn data(self) -> data::CurrentSyncReadAccountData<C> {
        data::CurrentSyncReadAccountData::new(self.cmds)
    }

    pub fn sign_in_with(self) -> sign_in_with::CurrentSyncReadAccountSignInWith<C> {
        sign_in_with::CurrentSyncReadAccountSignInWith::new(self.cmds)
    }

    pub fn demo_mode(self) -> demo::CurrentSyncReadAccountDemo<C> {
        demo::CurrentSyncReadAccountDemo::new(self.cmds)
    }

    pub fn email(self) -> email::CurrentSyncReadAccountEmail<C> {
        email::CurrentSyncReadAccountEmail::new(self.cmds)
    }

    pub fn news(self) -> news::CurrentSyncReadAccountNews<C> {
        news::CurrentSyncReadAccountNews::new(self.cmds)
    }
}
