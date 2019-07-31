use super::{Kind, Output, Resources};

#[test]
fn sample_output() {
    let xml: Output =
        serde_xml_rs::from_reader(std::fs::File::open("src/valgrind_xml/vg.xml").unwrap()).unwrap();

    assert_eq!(xml.errors.len(), 8);
    assert_eq!(xml.errors[0].kind, Kind::DefinitelyLost);
    assert_eq!(
        xml.errors[0].resources,
        Resources {
            bytes: 15,
            blocks: 1,
        }
    );
    assert_eq!(xml.errors[1].kind, Kind::StillReachable);
    assert_eq!(
        xml.errors[1].resources,
        Resources {
            bytes: 24,
            blocks: 1,
        }
    );
}
