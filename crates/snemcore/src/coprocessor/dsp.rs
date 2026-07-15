const STATUS_RQM: u8 = 0x80;

/// A command's expected input length (in bytes) and its handler.
struct CommandSpec {
    input_len: usize,
    exec: fn(&[u8]) -> Vec<u8>,
}

pub struct Dsp1 {
    /// Command byte (once known) for the in-progress transaction.
    active: Option<CommandSpec>,
    /// Raw bytes received so far for the active command.
    input: Vec<u8>,
    /// Raw bytes staged for the CPU to read back.
    output: Vec<u8>,
    output_pos: usize,
}

impl Dsp1 {
    pub fn new() -> Self {
        Self { active: None, input: Vec::new(), output: Vec::new(), output_pos: 0 }
    }

    /// CPU write to the Command/Data register.
    pub fn write(&mut self, value: u8) {
        if self.active.is_none() {
            // First byte of a new transaction selects the command.
            self.active = Some(lookup_command(value));
            self.input.clear();
            self.output.clear();
            self.output_pos = 0;
            return;
        }

        let spec = self.active.as_ref().unwrap();
        self.input.push(value);

        if self.input.len() == spec.input_len {
            self.output = (spec.exec)(&self.input);
            self.output_pos = 0;
            // Command consumed; next write starts a fresh transaction once
            // all output has been read (see `read`).
        }
    }

    /// CPU read from the Command/Data register.
    pub fn read(&mut self) -> u8 {
        if self.output_pos < self.output.len() {
            let byte = self.output[self.output_pos];
            self.output_pos += 1;
            if self.output_pos == self.output.len() {
                self.active = None; // transaction complete, ready for next command
            }
            byte
        } else {
            0
        }
    }

    /// CPU read from the Status register.
    pub fn status(&self) -> u8 {
        let has_pending_output = self.output_pos < self.output.len();
        let awaiting_input = self.active.as_ref()
            .map(|s| self.input.len() < s.input_len)
            .unwrap_or(false);
        if has_pending_output || awaiting_input || self.active.is_none() {
            STATUS_RQM
        } else {
            0
        }
    }
}

fn lookup_command(opcode: u8) -> CommandSpec {
    match opcode {
        0x00 | 0x20 => CommandSpec { input_len: 4, exec: cmd_multiply }, // 16-bit Multiplication

        0x10 => todo_command(), // Inverse Calculation
        0x01 => todo_command(), // Set Attitude A
        0x11 => todo_command(), // Set Attitude B
        0x21 => todo_command(), // Set Attitude C
        0x02 => todo_command(), // Projection Parameter Setting
        0x03 => todo_command(), // Object -> Global Coordinate A
        0x13 => todo_command(), // Object -> Global Coordinate B
        0x23 => todo_command(), // Object -> Global Coordinate C
        0x04 => todo_command(), // Trigonometric Calculation
        0x14 => todo_command(), // 3D Angle Rotation
        0x06 => todo_command(), // Object Projection Calculation
        0x08 => todo_command(), // Vector Size Calculation
        0x18 => todo_command(), // Vector Size Comparison
        0x28 => todo_command(), // Vector Absolute Value (bugged pre-DSP1B)
        0x38 => todo_command(), // Vector Size Comparison
        0x0A => todo_command(), // Raster Data Calculation (streaming, see note below)
        0x0B => todo_command(), // Inner Product, Attitude A
        0x1B => todo_command(), // Inner Product, Attitude B
        0x2B => todo_command(), // Inner Product, Attitude C
        0x0C => todo_command(), // 2D Coordinate Rotation
        0x1C => todo_command(), // 3D Coordinate Rotation
        0x0D => todo_command(), // Global -> Object Coordinate A
        0x1D => todo_command(), // Global -> Object Coordinate B
        0x2D => todo_command(), // Global -> Object Coordinate C
        0x0E => todo_command(), // Selected Screen Point Coordinate Calc
        0x0F => todo_command(), // Test: Memory Test
        0x1F => todo_command(), // Test: Transfer Data ROM
        0x2F => todo_command(), // Test: ROM Version

        _ => todo_command(),
    }
}

fn todo_command() -> CommandSpec {
    CommandSpec { input_len: 0, exec: |_| Vec::new() }
}

//   00h  16-bit Multiplication
//   10h  Inverse Calculation
//   20h  16-bit Multiplication
//   01h  Set Attitude A
//   11h  Set Attitude B
//   21h  Set Attitude C
//   02h  Projection Parameter Setting
//   03h  Convert from Object to Global Coordinate A
//   13h  Convert from Object to Global Coordinate B
//   23h  Convert from Object to Global Coordinate C
//   04h  Trigonometric Calculation
//   14h  3D Angle Rotation
//   06h  Object Projection Calculation
//   08h  Vector Size Calculation
//   18h  Vector Size Comparison
//   28h  Vector Absolute Value Calculation (bugged) (fixed in DSP1B)
//   38h  Vector Size Comparison
//   0Ah  Raster Data Calculation
//   0Bh  Calculation of Inner Product with the Forward Attitude A and a Vector
//   1Bh  Calculation of Inner Product with the Forward Attitude B and a Vector
//   2Bh  Calculation of Inner Product with the Forward Attitude C and a Vector
//   0Ch  2D Coordinate Rotation
//   1Ch  3D Coordinate Rotation
//   0Dh  Convert from Global to Object Coordinate A
//   1Dh  Convert from Global to Object Coordinate B
//   2Dh  Convert from Global to Object Coordinate C
//   0Eh  Coordinate Calculation of a selected point on the Screen
//   0Fh  Test Memory Test
//   1Fh  Test Transfer DATA ROM
//   2Fh  Test ROM Version (0100h=DSP1/DSP1A, 0101h=DSP1B)

/// Multiply: two signed 1.15 fixed-point inputs -> one signed 1.15 fixed-point
/// product (high 16 bits of the 32-bit multiply, i.e. Q1.15 * Q1.15 -> Q1.15).
fn cmd_multiply(input: &[u8]) -> Vec<u8> {
    let a = i16::from_le_bytes([input[0], input[1]]);
    let b = i16::from_le_bytes([input[2], input[3]]);
    let product = (a as i32) * (b as i32);
    let result = (product >> 15) as i16; // rescale Q2.30 -> Q1.15
    result.to_le_bytes().to_vec()
}

/// Inverse: two bytes for the coefficient (C), two bytes for the exponent (E). Input is C*2^E.
/// Output is coefficient and exponent of number 1/(C*2^E). Code based on bsnes implementation.
fn cmd_inverse(input: &[u8]) -> Vec<u8> {
    let mut coefficient = i16::from_le_bytes([input[0], input[1]]);
    let mut exponent = i16::from_le_bytes([input[2], input[3]]);

    let mut inv_coefficient: i16;
    let mut inv_exponent: i16;
   
    if coefficient == 0x0000 {
        inv_coefficient = 0x7fff;
        inv_exponent = 0x002f;
    } else {
        let mut sign = 1;

        if coefficient < 0 {
            if coefficient < -32767 {
                coefficient = -32767;
            }

            coefficient = -coefficient;
            sign = -1;
        }

        // Step Three: Normalize
        while coefficient < 0x4000 {
            coefficient <<= 1;
            exponent -= 1;
        }

        // Step Four: Special Case
        if coefficient == 0x4000  {
            if sign == 1  {
                inv_coefficient = 0x7fff;
            }
        else  {
            inv_coefficient = -0x4000;
            coefficient -= 1;
        }
        }
        else {
            // Step Five: Initial Guess
            let i: i16 = DataRom[((Coefficient - 0x4000) >> 7) + 0x0065];

            // Step Six: Iterate Newton's Method
            i = (i + (-i * (Coefficient * i >> 15) >> 15)) << 1;
            i = (i + (-i * (Coefficient * i >> 15) >> 15)) << 1;

            iCoefficient = i * Sign;
        }

        iExponent = 1 - Exponent;
    }

   vec![inv_coefficient, inv_exponent]
}