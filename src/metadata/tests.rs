use super::Metadata;

#[test]
fn sample_metadata_can_be_deserialized() {
    let file = std::fs::File::open("src/metadata/sample-metadata.json").unwrap();
    let _metadata: Metadata = serde_json::from_reader(file).unwrap();
}
