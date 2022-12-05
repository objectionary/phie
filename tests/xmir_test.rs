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
    // assert_eq!(xmr.objects[0].os[2].name, "@");
    Ok(())
}

fn create_test_xmir(tmp: &TempDir, name: &str) -> Result<()> {
    File::create(tmp.path().join(name))?
        .write_all(
            "
  <objects>
    <o abstract=\"\" line=\"30\" name=\"heap\" pos=\"0\">
      <o line=\"30\" name=\"size\" pos=\"1\"/>
      <o abstract=\"\" atom=\"pointer\" line=\"32\" name=\"malloc\" pos=\"2\">
        <o line=\"35\" name=\"s\" pos=\"3\"/>
      </o>
      <o abstract=\"\" atom=\"bool\" line=\"37\" name=\"free\" pos=\"2\">
        <o line=\"41\" name=\"p\" pos=\"3\"/>
      </o>
      <o abstract=\"\" line=\"43\" name=\"pointer\" pos=\"2\">
        <o line=\"44\" name=\"address\" pos=\"3\"/>
        <o line=\"44\" name=\"length\" pos=\"11\"/>
        <o base=\"address\" line=\"45\" name=\"@\" pos=\"4\"/>
        <o abstract=\"\" line=\"47\" name=\"add\" pos=\"4\">
          <o line=\"48\" name=\"x\" pos=\"5\"/>
          <o base=\"&amp;\" line=\"49\" pos=\"6\"/>
          <o base=\".^\" line=\"49\" method=\"\" pos=\"7\"/>
          <o base=\".pointer\" line=\"49\" method=\"\" name=\"@\" pos=\"9\">
            <o base=\"address\" line=\"50\" pos=\"8\"/>
            <o base=\".plus\" line=\"50\" method=\"\" pos=\"15\">
              <o base=\"length\" line=\"51\" pos=\"10\"/>
              <o base=\".times\" line=\"51\" method=\"\" pos=\"16\">
                <o base=\"x\" line=\"51\" pos=\"23\"/>
              </o>
            </o>
            <o base=\"length\" line=\"52\" pos=\"8\"/>
          </o>
        </o>
        <o abstract=\"\" line=\"54\" name=\"sub\" pos=\"4\">
          <o line=\"55\" name=\"x\" pos=\"5\"/>
          <o base=\"&amp;\" line=\"56\" pos=\"6\"/>
          <o base=\".add\" line=\"56\" method=\"\" name=\"@\" pos=\"7\">
            <o base=\"x\" line=\"57\" pos=\"8\"/>
            <o base=\".times\" line=\"57\" method=\"\" pos=\"9\">
              <o base=\"int\" data=\"bytes\" line=\"57\" pos=\"16\">FF FF FF FF FF FF FF FF</o>
            </o>
          </o>
        </o>
        <o abstract=\"\" atom=\"?\" line=\"59\" name=\"block\" pos=\"4\">
          <o line=\"60\" name=\"len\" pos=\"5\"/>
          <o line=\"60\" name=\"inverse\" pos=\"9\"/>
        </o>
      </o>
    </o>
  </objects>
        "
            .as_bytes(),
        )
        .unwrap();
    Ok(())
}
