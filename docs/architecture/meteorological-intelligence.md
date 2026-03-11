# Meteorological Intelligence

## Business Placement

`meteorological_intelligence` is a domain-owned weather capability. It publishes weather products
for UI, analytics, and future decision-support consumers without embedding those consumer concerns
inside the ingest path.

## Product Ladder

1. `raw_source_asset`: immutable source bytes and acquisition provenance.
2. `normalized_weather_product`: source-aligned forecast, radar, satellite, and observation
   products with native identifiers preserved.
3. `weather_view_product`: UI-oriented layer manifests for fast visualization.
4. `weather_feature_product`: model-ready feature slices for analytical consumers.

## Serving Model

- Bulk weather artifacts remain in open formats such as Zarr or Parquet/GeoParquet.
- Metadata, freshness, and lineage are represented by governed records and canonical events.
- The standalone weather app reads platform-managed weather snapshots.
- Analytical services consume contract types directly and remain read-only in v1.
