use deadpool_postgres::Client;

use crate::{
    errors::MyError,
    models::{DivisionAdmin, Team, TeamDivAssociation},
};

pub async fn get_admins_for_div_id(
    client: &Client,
    divisionid: i64,
) -> Result<Vec<DivisionAdmin>, MyError> {
}

pub async fn get_teams_for_div_id(
    client: &Client,
    divisionid: i64,
) -> Result<Vec<TeamDivAssociation>, MyError> {
}
