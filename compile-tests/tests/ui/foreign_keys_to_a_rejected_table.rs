//! A rejected `server` table is referenced by `channel`, twice, and by `member`. Each of the
//! three foreign keys reports that `server` has to be fixed first, and nothing else does: not
//! the cascades of `server`, and not `message`, which references the accepted `channel`.
//!
//! The errors after the first are those three, one per `#[foreign_key]`.

::spacetimedsl::spacetimedsl!();

pub mod server {
    #[spacetimedsl::dsl(method(update = false, delete = true))]
    #[spacetimedb::table(accessor = server, public)]
    pub struct Server {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[referenced_by(path = crate::channel, table = channel)]
        #[referenced_by(path = crate::member, table = member)]
        id: u64,
    }
}

pub mod channel {
    #[spacetimedsl::dsl(plural_name = channels, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = channel, public)]
    pub struct Channel {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[referenced_by(path = crate::message, table = message)]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::server::ServerId)]
        #[foreign_key(path = crate::server, table = server, column = id, on_delete = Delete)]
        server_id: u64,

        #[index(btree)]
        #[use_wrapper(crate::server::ServerId)]
        #[foreign_key(path = crate::server, table = server, column = id, on_delete = Delete)]
        backup_server_id: u64,
    }
}

pub mod member {
    #[spacetimedsl::dsl(plural_name = members, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = member, public)]
    pub struct Member {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::server::ServerId)]
        #[foreign_key(path = crate::server, table = server, column = id, on_delete = Delete)]
        server_id: u64,
    }
}

pub mod message {
    #[spacetimedsl::dsl(plural_name = messages, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = message, public)]
    pub struct Message {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::channel::ChannelId)]
        #[foreign_key(path = crate::channel, table = channel, column = id, on_delete = Delete)]
        channel_id: u64,
    }
}

fn main() {}
