/// Prints out a slice of bytes in hex and ASCII format, side by side. When
/// startval is specified, indeces beginning at the startval will be printed
/// before each line. If startval is unspecified, indeces start at 0.
pub fn hexdump(bytes: &[u8], startval: usize) {
    let mut index = startval;
    println!();
    for chunk in bytes.chunks(8) {
        print!("{:08X}", index);
        for b in chunk.iter() {
            print!("{b:02X}");
        }
        print!(" | ");
        for b in chunk.iter() {
            match b {
                32..=126 => print!("{}", b as char),
                _ => print!("."),
            }
        }
        println!();
        index += 8;
    }
}
