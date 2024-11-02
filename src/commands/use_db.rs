use crate::database::session::DatabaseSession;

pub fn handle_use(session: &mut DatabaseSession, db_name: &str) {
    session.use_database(db_name);
}
