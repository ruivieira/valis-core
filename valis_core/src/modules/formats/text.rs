use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub fn replace_matching_line<P: AsRef<Path>>(
    file_path: P,
    pattern: &str,
    replacement: &str,
) -> io::Result<()> {
    let file_path = file_path.as_ref();
    let temp_file_path = file_path.with_extension("temp");

    let input_file = File::open(file_path)?;
    let reader = BufReader::new(input_file);

    let mut output_file = File::create(&temp_file_path)?;

    for line in reader.lines() {
        let line = line?;
        let stripped_line = line.trim();

        if stripped_line == pattern {
            writeln!(output_file, "{}", replacement)?;
        } else {
            writeln!(output_file, "{}", line)?;
        }
    }

    fs::remove_file(file_path)?;
    fs::rename(temp_file_path, file_path)?;

    Ok(())
}
