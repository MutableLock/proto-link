use diesel::{AsChangeset, Connection, Insertable, MysqlConnection, Queryable, RunQueryDsl, Selectable};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::QueryDsl;
use diesel::ExpressionMethods;
pub struct UsersDb;

#[derive(Queryable, Selectable, Insertable, PartialEq, AsChangeset)]
#[diesel(table_name = crate::db::schema::users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User{
    pub id: u64,
    pub login: String,
    pub name: String,
    pub password_hash_pbkdf2: String,
    pub password_salt: String,
}

impl UsersDb{
    pub fn create_user(conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>, login: String, name: String, pbkdf2: String, salt: String) -> bool {
        use crate::db::schema::users;
        let user = User{
            id: 0,
            login,
            name,
            password_hash_pbkdf2: pbkdf2,
            password_salt: salt,
        };
        conn.transaction(|conn| {diesel::insert_into(users::table).values(user).execute(conn)}).is_ok()
    }

    pub fn find_user_by_login(conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>, login: String) -> Option<User> {
        use crate::db::schema::users::dsl::*;
        users.filter(login.eq(login)).first(conn).ok()
    }

    pub fn is_user_exists(conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>, user_login: &str) -> bool {
        use crate::db::schema::users::dsl::*;
        users.filter(login.eq(user_login)).first::<User>(conn).is_ok()
    }

    pub fn find_user_by_id(conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>, id: u64) -> Option<User> {
        use crate::db::schema::users::dsl::*;
        users.filter(id.eq(id)).first(conn).ok()
    }

    pub fn update_user(conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>, user: User) -> bool {
        use crate::db::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user.id))).set(user).execute(conn).is_ok()
    }

    pub fn delete_user(conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>, user_id: u64) -> bool {
        use crate::db::schema::users::dsl::*;
        diesel::delete(users.filter(id.eq(user_id))).execute(conn).is_ok()
    }
}