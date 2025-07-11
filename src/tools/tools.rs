/// Prints out a slice of bytes in hex and ASCII format, side by side. When
/// startval is specified, indeces beginning at the startval will be printed
/// before each line. If startval is unspecified, indeces start at 0.
pub fn hexdump8_at(bytes: &[u8], startval: usize) {
    let mut index = startval;
    println!();
    for chunk in bytes.chunks(16) {
        let l = chunk.len();
        print!("{:08X}: ", index);
        for b in chunk.iter() {
            print!("{b:02X} ");
        }

        print!("{:>width$} ", "|", width = (16 - l) * 3 + 1);
        for b in chunk.iter() {
            match b {
                32..=126 => print!("{}", *b as char),
                _ => print!("."),
            }
        }
        println!();
        index += 8;
    }
}

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
/// startval is specified, indeces beginning at the startval will be printed
/// before each line. If startval is unspecified, indeces start at 0.
pub fn hexdump8(bytes: &[u8]) {
    hexdump8_at(bytes, 0);
}

pub fn hexdump16(bytes: &[u16]) {
    hexdump16_at(bytes, 0);
}