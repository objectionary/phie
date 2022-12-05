use anyhow::Result;
use phie::xmir::{xmir_from_file, XMIR};
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn read_xmir_success() -> Result<()> {
    let tmp = TempDir::new()?;
    let name = "test.xmir";
    create_test_xmir(&tmp, &name).unwrap();
    let xmr = xmir_from_file(tmp.path().join(name).to_str().unwrap());
    assert_eq!(xmr.objects[0].os[2].name, "@");
    Ok(())
}

fn create_test_xmir(tmp: &TempDir, name: &str) -> Result<()> {
    File::create(tmp.path().join(name))?
        .write_all(
            "
            <objects>
              <o abstract=\"\" line=\"34\" name=\"rust\" pos=\"0\">
                <o line=\"34\" name=\"code\" pos=\"1\"/>
                <o line=\"34\" name=\"params\" pos=\"6\"/>
                <o base=\"array\" data=\"array\" line=\"35\" name=\"@\" pos=\"2\">
                <o base=\"code\" line=\"36\" pos=\"4\"/>
                <o base=\"params\" line=\"37\" pos=\"4\"/>
                </o>
              </o>
            </objects>
        "
            .as_bytes(),
        )
        .unwrap();
    Ok(())
}
