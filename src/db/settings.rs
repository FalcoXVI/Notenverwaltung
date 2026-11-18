use rusqlite::{OptionalExtension, params};

use super::{Database, DbResult};
use crate::calc::TrafficThresholds;
use crate::domain::fraction;

const KEY_GREEN_MAX: &str = "traffic_green_max";
const KEY_YELLOW_MAX: &str = "traffic_yellow_max";

impl Database {
    /// Loads the traffic light limits; missing values fall back to the
    /// defaults (3.0 and 4.0).
    pub fn load_traffic_thresholds(&self) -> DbResult<TrafficThresholds> {
        let defaults = TrafficThresholds::default();
        // Limits are stored in hundredths of a grade (3.5 = 350).
        let read = |key: &str| -> DbResult<Option<i64>> {
            Ok(self
                .conn
                .query_row("SELECT value FROM setting WHERE key = ?1", [key], |row| row.get(0))
                .optional()?)
        };
        Ok(TrafficThresholds {
            green_max: read(KEY_GREEN_MAX)?.map_or(defaults.green_max, |v| fraction(v, 100)),
            yellow_max: read(KEY_YELLOW_MAX)?.map_or(defaults.yellow_max, |v| fraction(v, 100)),
        })
    }

    /// Stores the traffic light limits (rounded to hundredths).
    pub fn save_traffic_thresholds(&mut self, thresholds: &TrafficThresholds) -> DbResult<()> {
        let tx = self.conn.transaction()?;
        let values = [
            (KEY_GREEN_MAX, thresholds.green_max),
            (KEY_YELLOW_MAX, thresholds.yellow_max),
        ];
        for (key, value) in values {
            let hundredths = (value * 100).round().to_integer();
            tx.execute(
                "INSERT INTO setting (key, value) VALUES (?1, ?2)
                 ON CONFLICT (key) DO UPDATE SET value = excluded.value",
                params![key, hundredths],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}
