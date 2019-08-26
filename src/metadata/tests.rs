use super::Metadata;

#[test]
fn sample_metadata_can_be_deserialized() {
    let file =
        std::fs::File::open("src/metadata/sample-metadata.json").expect("Could not open test file");
    let _metadata: Metadata = serde_json::from_reader(file).expect("Could not parse test metadata");
}

#[test]
fn only_version_1_is_supported() {
    let result: Result<Metadata, _> = serde_json::from_str(
        r#"
{
    "version": 1,
    "packages": []
    "target_directory": ""
}
    "#,
    );
    result.unwrap_err();
}
