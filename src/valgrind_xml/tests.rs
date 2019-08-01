use super::{Frame, Kind, Output, Resources};

#[test]
fn sample_output() {
    let xml: Output =
        serde_xml_rs::from_reader(std::fs::File::open("src/valgrind_xml/vg.xml").unwrap()).unwrap();

    assert_eq!(xml.errors.len(), 8);
    assert_eq!(xml.errors[0].kind, Kind::LeakDefinitelyLost);
    assert_eq!(xml.errors[0].unique, 0x0);
    assert_eq!(
        xml.errors[0].resources,
        Resources {
            bytes: 15,
            blocks: 1,
        }
    );
    assert_eq!(
        &xml.errors[0].stack_trace.frames[..2],
        &[
            Frame {
                instruction_pointer: 0x483AD7B,
                object: Some("/usr/lib/valgrind/vgpreload_memcheck-amd64-linux.so".into()),
                directory: Some("/build/valgrind/src/valgrind/coregrind/m_replacemalloc".into()),
                function: Some("realloc".into()),
                file: Some("vg_replace_malloc.c".into()),
                line: Some(826),
            },
            Frame {
                instruction_pointer: 0x12B6F4,
                object: Some("/home/jfrimmel/git/lava.rs/target/debug/examples/creation".into()),
                directory: Some(
                    "/rustc/a53f9df32fbb0b5f4382caaad8f1a46f36ea887c/src/liballoc".into()
                ),
                function: Some("realloc".into()),
                file: Some("alloc.rs".into()),
                line: Some(125),
            },
        ]
    );

    assert_eq!(xml.errors[1].kind, Kind::LeakStillReachable);
    assert_eq!(xml.errors[1].unique, 0x1);
    assert_eq!(
        xml.errors[1].resources,
        Resources {
            bytes: 24,
            blocks: 1,
        }
    );
}
