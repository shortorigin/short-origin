use contracts::WeatherFeatureSliceV1;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherFactorRow {
    pub region_id: String,
    pub feature_name: String,
    pub value: f64,
    pub lead_hours: u16,
    pub qc_flag_count: usize,
    pub raw_artifact_hashes: Vec<String>,
}

pub fn weather_factor_rows(slices: &[WeatherFeatureSliceV1]) -> Vec<WeatherFactorRow> {
    slices
        .iter()
        .flat_map(|slice| {
            slice.features.iter().map(move |feature| WeatherFactorRow {
                region_id: slice.region_id.clone(),
                feature_name: format!("{:?}", feature.feature),
                value: feature.value,
                lead_hours: slice.lead_hours,
                qc_flag_count: feature.qc_flags.len(),
                raw_artifact_hashes: slice
                    .provenance
                    .iter()
                    .map(|record| record.raw_artifact_hash.clone())
                    .collect(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::weather_factor_rows;

    #[test]
    fn weather_factor_rows_keep_lineage() {
        let snapshot: sdk_rs::WeatherPlatformSnapshotV1 =
            serde_json::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../testing/fixtures/weather/run-2026-03-10/platform_snapshot.json"
            )))
            .expect("weather snapshot fixture");
        let rows = weather_factor_rows(&snapshot.feature_slices);

        assert_eq!(rows.len(), 4);
        assert!(rows.iter().any(|row| {
            row.raw_artifact_hashes
                .contains(&"sha-hrrr-001".to_string())
        }));
        assert!(
            rows.iter()
                .any(|row| row.feature_name.contains("WindSpeed10m"))
        );
    }
}
