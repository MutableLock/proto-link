use chrono::{NaiveDateTime, Utc};
use diesel::{AsChangeset, Connection, ExpressionMethods, Insertable, MysqlConnection, QueryDsl, Queryable, RunQueryDsl, Selectable};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::result::Error as DieselError;
// ... existing code ...
use rand::rngs::OsRng;
use rand::RngCore;

pub struct TokensDb;

#[derive(Queryable, Selectable, Insertable, PartialEq, AsChangeset, Debug)]
#[diesel(table_name = crate::server::db::schema::tokens)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct TokenRow {
    pub id: u64,
    pub token: u64,
    pub user_id: u64,
    pub expires_at: NaiveDateTime,
}

impl TokensDb {
    pub fn create_token(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        user_id: u64,
        expires_at_value: NaiveDateTime,
    ) -> Result<u64, DieselError> {
        use crate::server::db::schema::tokens;

        const MAX_ATTEMPTS: usize = 32;

        conn.transaction(|conn| {
            for _ in 0..MAX_ATTEMPTS {
                let mut token_value = rand::rng().next_u64();
                if token_value == 0 {
                    token_value = 1;
                }

                let row = TokenRow {
                    id: 0,
                    token: token_value,
                    user_id,
                    expires_at: expires_at_value,
                };

                match diesel::insert_into(tokens::table).values(&row).execute(conn) {
                    Ok(_) => return Ok(token_value),
                    Err(DieselError::DatabaseError(
                            diesel::result::DatabaseErrorKind::UniqueViolation,
                            _,
                        )) => {
                        // token collision, retry
                        continue;
                    }
                    Err(e) => return Err(e),
                }
            }

        //    let _ = Self::delete_expired(conn, Utc::now().naive_utc());
            Err(DieselError::RollbackTransaction)
        })
    }

    pub fn find_token_by_value(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        token_value: u64,
    ) -> Result<TokenRow, DieselError> {
        use crate::server::db::schema::tokens::dsl::*;

        tokens.filter(token.eq(token_value)).first::<TokenRow>(conn)
    }

    pub fn find_tokens_by_user_id(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        uid: u64,
    ) -> Result<Vec<TokenRow>, DieselError> {
        use crate::server::db::schema::tokens::dsl::*;

        tokens.filter(user_id.eq(uid)).load::<TokenRow>(conn)
    }

    pub fn update_token(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        row: &TokenRow,
    ) -> Result<usize, DieselError> {
        use crate::server::db::schema::tokens::dsl::*;

        diesel::update(tokens.filter(id.eq(row.id)))
            .set(row)
            .execute(conn)
    }

    pub fn delete_token_by_value(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        token_value: u64,
    ) -> Result<usize, DieselError> {
        use crate::server::db::schema::tokens::dsl::*;

        diesel::delete(tokens.filter(token.eq(token_value))).execute(conn)
    }

    pub fn delete_tokens_for_user(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        uid: u64,
    ) -> Result<usize, DieselError> {
        use crate::server::db::schema::tokens::dsl::*;

        diesel::delete(tokens.filter(user_id.eq(uid))).execute(conn)
    }

    pub fn delete_expired(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        now: NaiveDateTime,
    ) -> Result<usize, DieselError> {
        use crate::server::db::schema::tokens::dsl::*;

        diesel::delete(tokens.filter(expires_at.lt(now))).execute(conn)
    }
}