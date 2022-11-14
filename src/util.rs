use std::path::Path;

pub fn path_to_url<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();

    let mut it = path.components();
    if let Some(comp) = it.next() {
        let mut output = String::from(
            comp.as_os_str()
                .to_str()
                .expect("Invalid UTF in input path"),
        );
        for comp in it {
            output += "/";
            output += comp
                .as_os_str()
                .to_str()
                .expect("Invalid UTF in input path");
        }
        output
    } else {
        String::new()
    }
}
