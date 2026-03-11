use contracts::{DataRegisterEntryV1, WeatherFeatureSliceV1};

pub fn weather_feature_register_entries(
    slices: &[WeatherFeatureSliceV1],
) -> Vec<DataRegisterEntryV1> {
    slices
        .iter()
        .flat_map(|slice| {
            slice
                .features
                .iter()
                .map(move |feature| DataRegisterEntryV1 {
                    series_name: format!("{:?}", feature.feature),
                    country_area: slice.region_id.clone(),
                    source: slice.provenance.first().map_or_else(
                        || "weather".to_string(),
                        |record| record.source_dataset.clone(),
                    ),
                    frequency: if slice.lead_hours == 0 {
                        "Observation".to_string()
                    } else {
                        "Forecast".to_string()
                    },
                    last_obs: slice.valid_time.to_rfc3339(),
                    units: feature.units.clone(),
                    transform: "direct".to_string(),
                    lag: format!("T+{}h", slice.lead_hours),
                    quality_flag: feature
                        .qc_flags
                        .iter()
                        .map(|flag| format!("{flag:?}"))
                        .collect::<Vec<_>>()
                        .join(", "),
                    notes: slice
                        .provenance
                        .iter()
                        .map(|record| {
                            format!("{} {}", record.source_object_ref, record.raw_artifact_hash)
                        })
                        .collect::<Vec<_>>()
                        .join(" | "),
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use contracts::WeatherFeatureSliceV1;

    use super::weather_feature_register_entries;

    fn load_slices() -> Vec<WeatherFeatureSliceV1> {
        let snapshot: sdk_rs::WeatherPlatformSnapshotV1 =
            serde_json::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../testing/fixtures/weather/run-2026-03-10/platform_snapshot.json"
            )))
            .expect("weather snapshot fixture");
        snapshot.feature_slices
    }

    #[test]
    fn weather_feature_register_entries_preserve_qc_and_provenance() {
        let entries = weather_feature_register_entries(&load_slices());

        assert_eq!(entries.len(), 4);
        assert!(
            entries
                .iter()
                .any(|entry| entry.quality_flag.contains("Estimated"))
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry.notes.contains("sha-hrrr-001"))
        );
    }
}
