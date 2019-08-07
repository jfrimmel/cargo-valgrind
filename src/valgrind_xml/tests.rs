use super::{Error, Frame, Kind, Output, Resources};

#[test]
fn sample_output() {
    let xml: Output =
        serde_xml_rs::from_reader(std::fs::File::open("src/valgrind_xml/vg.xml").unwrap()).unwrap();

    let errors = xml.errors.unwrap();
    assert_eq!(errors.len(), 8);
    assert_eq!(errors[0].kind, Kind::LeakDefinitelyLost);
    assert_eq!(errors[0].unique, 0x0);
    assert_eq!(
        errors[0].resources,
        Resources {
            bytes: 15,
            blocks: 1,
        }
    );
    assert_eq!(
        &errors[0].stack_trace.frames[..2],
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

    assert_eq!(errors[1].kind, Kind::LeakStillReachable);
    assert_eq!(errors[1].unique, 0x1);
    assert_eq!(
        errors[1].resources,
        Resources {
            bytes: 24,
            blocks: 1,
        }
    );
}

#[test]
fn unique_ids_have_to_be_in_hex_with_prefix() {
    let result: Error = serde_xml_rs::from_str(
        r"<error>\
           <unique>0xDEAD1234</unique>\
           <tid>1</tid>\
           <kind>Leak_DefinitelyLost</kind>\
           <xwhat>\
           <text>...</text>\
             <leakedbytes>15</leakedbytes>\
             <leakedblocks>1</leakedblocks>\
           </xwhat>\
           <stack>\
             <frame>\
               <ip>0x483AD7B</ip>\
             </frame>\
           </stack>\
         </error>",
    )
    .unwrap();
    assert_eq!(result.unique, 0xDEAD_1234);
}

#[test]
fn missing_hex_prefix_is_an_error() {
    let result: Result<Error, _> = serde_xml_rs::from_str(
        r"<error>\
           <unique>0DEADBEEF</unique>\
           <tid>1</tid>\
           <kind>Leak_DefinitelyLost</kind>\
           <xwhat>\
           <text>...</text>\
             <leakedbytes>15</leakedbytes>\
             <leakedblocks>1</leakedblocks>\
           </xwhat>\
           <stack>\
             <frame>\
               <ip>0x483AD7B</ip>\
             </frame>\
           </stack>\
         </error>",
    );
    assert!(result.is_err());

    let result: Result<Error, _> = serde_xml_rs::from_str(
        r"<error>\
           <unique>xDEADBEEF</unique>\
           <tid>1</tid>\
           <kind>Leak_DefinitelyLost</kind>\
           <xwhat>\
           <text>...</text>\
             <leakedbytes>15</leakedbytes>\
             <leakedblocks>1</leakedblocks>\
           </xwhat>\
           <stack>\
             <frame>\
               <ip>0x483AD7B</ip>\
             </frame>\
           </stack>\
         </error>",
    );
    assert!(result.is_err());

    let result: Result<Error, _> = serde_xml_rs::from_str(
        r"<error>\
           <unique>DEADBEEF</unique>\
           <tid>1</tid>\
           <kind>Leak_DefinitelyLost</kind>\
           <xwhat>\
           <text>...</text>\
             <leakedbytes>15</leakedbytes>\
             <leakedblocks>1</leakedblocks>\
           </xwhat>\
           <stack>\
             <frame>\
               <ip>0x483AD7B</ip>\
             </frame>\
           </stack>\
         </error>",
    );
    assert!(result.is_err());
}

#[test]
fn invalid_hex_digits_are_an_error() {
    let result: Result<Error, _> = serde_xml_rs::from_str(
        r"<error>\
           <unique>0xhello</unique>\
           <tid>1</tid>\
           <kind>Leak_DefinitelyLost</kind>\
           <xwhat>\
           <text>...</text>\
             <leakedbytes>15</leakedbytes>\
             <leakedblocks>1</leakedblocks>\
           </xwhat>\
           <stack>\
             <frame>\
               <ip>0x483AD7B</ip>\
             </frame>\
           </stack>\
         </error>",
    );
    assert!(result.is_err());
}

#[test]
fn hex_and_prefix_case_is_ignored() {
    let result: Error = serde_xml_rs::from_str(
        r"<error>\
           <unique>0XdEaDbEeF</unique>\
           <tid>1</tid>\
           <kind>Leak_DefinitelyLost</kind>\
           <xwhat>\
           <text>...</text>\
             <leakedbytes>15</leakedbytes>\
             <leakedblocks>1</leakedblocks>\
           </xwhat>\
           <stack>\
             <frame>\
               <ip>0x483AD7B</ip>\
             </frame>\
           </stack>\
         </error>",
    )
    .unwrap();
    assert_eq!(result.unique, 0xDEAD_BEEF);
}

#[test]
fn unique_id_is_64bit() {
    let result: Error = serde_xml_rs::from_str(
        r"<error>\
           <unique>0x123456789ABCDEF0</unique>\
           <tid>1</tid>\
           <kind>Leak_DefinitelyLost</kind>\
           <xwhat>\
           <text>...</text>\
             <leakedbytes>15</leakedbytes>\
             <leakedblocks>1</leakedblocks>\
           </xwhat>\
           <stack>\
             <frame>\
               <ip>0x483AD7B</ip>\
             </frame>\
           </stack>\
         </error>",
    )
    .unwrap();
    assert_eq!(result.unique, 0x1234_5678_9ABC_DEF0);
}
