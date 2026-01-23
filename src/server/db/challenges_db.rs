use diesel::{AsChangeset, Connection, ExpressionMethods, Insertable, MysqlConnection, QueryDsl, Queryable, RunQueryDsl, Selectable};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::result::Error as DieselError;

pub struct ChallengesDb;

#[derive(Queryable, Selectable, Insertable, PartialEq, AsChangeset, Debug)]
#[diesel(table_name = crate::server::db::schema::challenges)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Challenge {
    pub id: u64,
    pub challenge: Vec<u8>,
    pub solution: Vec<u8>,
    pub user_id: u64,
    pub nonce: Vec<u8>,
}

impl ChallengesDb {
    pub fn create_challenge(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        user_id: u64,
        challenge_blob: Vec<u8>,
        solution_blob: Vec<u8>,
        nonce: Vec<u8>
    ) -> Result<usize, DieselError> {
        use crate::server::db::schema::challenges;

        let row = Challenge {
            id: 0,
            challenge: challenge_blob,
            solution: solution_blob,
            user_id,
            nonce
        };

        conn.transaction(|conn| {
            diesel::insert_into(challenges::table)
                .values(&row)
                .execute(conn)
        })
    }

    pub fn find_challenge_by_id(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        challenge_id: u64,
    ) -> Result<Challenge, DieselError> {
        use crate::server::db::schema::challenges::dsl::*;

        challenges.filter(id.eq(challenge_id)).first::<Challenge>(conn)
    }

    pub fn find_challenges_by_user_id(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        uid: u64,
    ) -> Result<Vec<Challenge>, DieselError> {
        use crate::server::db::schema::challenges::dsl::*;

        challenges.filter(user_id.eq(uid)).load::<Challenge>(conn)
    }

    pub fn update_challenge(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        row: &Challenge,
    ) -> Result<usize, DieselError> {
        use crate::server::db::schema::challenges::dsl::*;

        diesel::update(challenges.filter(id.eq(row.id)))
            .set(row)
            .execute(conn)
    }

    pub fn delete_challenge(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        challenge_id: u64,
    ) -> Result<usize, DieselError> {
        use crate::server::db::schema::challenges::dsl::*;

        diesel::delete(challenges.filter(id.eq(challenge_id))).execute(conn)
    }

    pub fn delete_challenges_for_user(
        conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
        uid: u64,
    ) -> Result<usize, DieselError> {
        use crate::server::db::schema::challenges::dsl::*;

        diesel::delete(challenges.filter(user_id.eq(uid))).execute(conn)
    }
}