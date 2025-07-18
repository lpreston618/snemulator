use std::io::Write;

fn hexdump8_str(bytes: &[u8], startval: usize) -> String {
    let mut result = "\n".to_string();

    let mut index = startval;
    for chunk in bytes.chunks(16) {
        let l = chunk.len();
        
        result.push_str(format!("{:08X}: ", index).as_str());

        for b in chunk.iter() {
            result.push_str(format!("{b:02X} ").as_str());
        }

        result.push_str(format!("{:>width$} ", "|", width = (16 - l) * 3 + 1).as_str());
        for b in chunk.iter() {
            match b {
                32..=126 => result.push(*b as char),
                _ => result.push('.'),
            }
        }
        result.push('\n');
        index += 16;
    }

    result
}

/// Prints out a slice of bytes in hex and ASCII format, side by side. When
/// startval is specified, indeces beginning at the startval will be printed
/// before each line.
pub fn hexdump8_at(bytes: &[u8], startval: usize) {
    print!("{}", hexdump8_str(bytes, startval));
}

/// Prints out a slice of words (2 byte values) in hex and ASCII format, side by
/// side. When startval is specified, indeces beginning at the startval will be 
/// printed before each line.
pub fn hexdump16_at(bytes: &[u16], startval: usize) {
    let mut index = startval;
    println!("Dumping {} words...", bytes.len());
    for chunk in bytes.chunks(16) {
        let l = chunk.len();
        print!("${:04X}: ", index);
        for b in chunk.iter() {
            print!("{b:04X} ");
        }

        print!("{:>width$} ", "|", width = (16 - l) * 3 + 1);
        for b in chunk.iter() {
            match b {
                32..=126 => print!("{}{}", (((*b) >> 8) as u8) as char, ((*b) as u8) as char),
                _ => print!("."),
            }
        }
        println!();
        index += 16;
    }
}

/// Prints out a slice of bytes in hex and ASCII format, side by side. When
/// startval is specified, indeces beginning at $0000 will be printed before 
/// each line.
pub fn hexdump8(bytes: &[u8]) {
    hexdump8_at(bytes, 0);
}

/// Prints out a slice of words (2 byte values) in hex and ASCII format, side by 
/// side. When startval is specified, indeces beginning $0000 will be printed 
/// before each line.
pub fn hexdump16(bytes: &[u16]) {
    hexdump16_at(bytes, 0);
}

pub fn hexdump8_to_file(bytes: &[u8], filepath: &str) {
    println!("Dumping {} bytes to file '{}'", bytes.len(), filepath);

    let path = std::path::Path::new(filepath);
    let mut outf = std::fs::File::create(path).unwrap();

    outf.write(hexdump8_str(bytes, 0).as_bytes()).unwrap();

    println!("Done.")
}