use diesel::{
    AsChangeset, Connection, Insertable, MysqlConnection, Queryable,
    RunQueryDsl, Selectable, QueryDsl, ExpressionMethods,
};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::result::Error as DieselError;

pub struct UsersDb;

#[derive(Queryable, Selectable, Insertable, PartialEq, AsChangeset, Debug)]
#[diesel(table_name = crate::server::db::schema::users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub id: u64,
    pub login: String,
    pub name: String,
    pub password_hash: String
}

impl UsersDb {
    pub fn create_user(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        login: String,
        name: String,
       hash: String
    ) -> Result<usize, DieselError> {
        use crate::server::db::schema::users;

        let user = User {
            id: 0,
            login,
            name,
            password_hash: hash
        };

        conn.transaction(|conn| {
            diesel::insert_into(users::table)
                .values(&user)
                .execute(conn)
        })
    }

    pub fn find_user_by_login(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        user_login: &str,
    ) -> Result<User, DieselError> {
        use crate::server::db::schema::users::dsl::*;

        users
            .filter(login.eq(user_login))
            .first::<User>(conn)
    }

    pub fn is_user_exists(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        user_login: &str,
    ) -> Result<bool, DieselError> {
        use crate::server::db::schema::users::dsl::*;

        match users.filter(login.eq(user_login)).first::<User>(conn) {
            Ok(_) => Ok(true),
            Err(DieselError::NotFound) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn find_user_by_id(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        user_id: u64,
    ) -> Result<User, DieselError> {
        use crate::server::db::schema::users::dsl::*;

        users
            .filter(id.eq(user_id))
            .first::<User>(conn)
    }

    pub fn update_user(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        user: &User,
    ) -> Result<usize, DieselError> {
        use crate::server::db::schema::users::dsl::*;

        diesel::update(users.filter(id.eq(user.id)))
            .set(user)
            .execute(conn)
    }

    pub fn delete_user(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        user_id: u64,
    ) -> Result<usize, DieselError> {
        use crate::server::db::schema::users::dsl::*;

        diesel::delete(users.filter(id.eq(user_id)))
            .execute(conn)
    }
}
