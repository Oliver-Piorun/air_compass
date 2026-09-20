use sqlx::{Pool, Postgres};

use crate::observation::Observation;

pub async fn store(
    postgres_pool: &Pool<Postgres>,
    observation: &Observation,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO observations (
            temperature,
            relative_humidity,
            absolute_humidity,
            outdoor_temperature,
            outdoor_relative_humidity,
            outdoor_absolute_humidity
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        observation.temperature,
        observation.relative_humidity,
        observation.absolute_humidity,
        observation.outdoor_temperature,
        observation.outdoor_relative_humidity,
        observation.outdoor_absolute_humidity
    )
    .execute(postgres_pool)
    .await?;

    Ok(())
}
