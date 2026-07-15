const FREEZE_COMMANDS: [u8; 3] = [0x1A, 0x2A, 0x3A];
const RASTER_CMD: u8 = 0x0A;

const STATUS_RQM: u8 = 0x80;

const fn q15_mul(a: i16, b: i16) -> i16 {
    ((a as i32 * b as i32) >> 15) as i16
}

fn sin_q15(angle: i16) -> i16 {
    let rad = angle as f64 * std::f64::consts::PI / 32768.0;
    (rad.sin() * 32767.0).round().clamp(-32768.0, 32767.0) as i16
}

fn cos_q15(angle: i16) -> i16 {
    let rad = angle as f64 * std::f64::consts::PI / 32768.0;
    (rad.cos() * 32767.0).round().clamp(-32768.0, 32767.0) as i16
}

// Bytes <-> i16 params, little-endian, matching the DR register's 16-bit halves.
fn read_i16s(input: &[u8]) -> Vec<i16> {
    input.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect()
}
fn write_i16s(values: &[i16]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn q(v: i16) -> f64 { v as f64 / 32768.0 }
fn unq(v: f64) -> i16 { (v * 32768.0).round().clamp(-32768.0, 32767.0) as i16 }
fn to_rad(a: i16) -> f64 { a as f64 * std::f64::consts::PI / 32768.0 }

/// A command's expected input length (in bytes) and its handler.
struct CommandSpec {
    input_len: usize,
    exec: fn(&mut Dsp1State, &[i16]) -> Vec<i16>,
}

type Mat3 = [[i16; 3]; 3];

pub struct ProjectionState {
    centre: [f64; 3],   // centre of projection (viewpoint), global coords
    screen: [f64; 3],   // G: centre of the screen, global coords
    normal: [f64; 3],   // unit vector: screen -> viewpoint
    horiz: [f64; 3],    // unit vector: screen right
    vert: [f64; 3],     // unit vector: screen up
    les: f64,           // base point -> screen distance
    v_offset: f64,      // screen's vertical offset from the horizon plane
    sin_aas: f64, cos_aas: f64,
    sin_azs: f64, cos_azs: f64, // clipped zenith angle
}

#[derive(Default)]
pub struct Dsp1State {
    pub matrix_a: Mat3,
    pub matrix_b: Mat3,
    pub matrix_c: Mat3,
    pub projection: Option<ProjectionState>,
}

impl Dsp1State {
    pub fn attitude_a(&mut self, s: i16, rz: i16, ry: i16, rx: i16) { self.matrix_a = build_attitude(s, rz, ry, rx); }
    pub fn attitude_b(&mut self, s: i16, rz: i16, ry: i16, rx: i16) { self.matrix_b = build_attitude(s, rz, ry, rx); }
    pub fn attitude_c(&mut self, s: i16, rz: i16, ry: i16, rx: i16) { self.matrix_c = build_attitude(s, rz, ry, rx); }

    pub fn objective_a(&self, x: i16, y: i16, z: i16) -> (i16, i16, i16) { mat3_apply(&self.matrix_a, x, y, z) }
    pub fn objective_b(&self, x: i16, y: i16, z: i16) -> (i16, i16, i16) { mat3_apply(&self.matrix_b, x, y, z) }
    pub fn objective_c(&self, x: i16, y: i16, z: i16) -> (i16, i16, i16) { mat3_apply(&self.matrix_c, x, y, z) }

    pub fn subjective_a(&self, f: i16, l: i16, u: i16) -> (i16, i16, i16) { mat3_apply_transposed(&self.matrix_a, f, l, u) }
    pub fn subjective_b(&self, f: i16, l: i16, u: i16) -> (i16, i16, i16) { mat3_apply_transposed(&self.matrix_b, f, l, u) }
    pub fn subjective_c(&self, f: i16, l: i16, u: i16) -> (i16, i16, i16) { mat3_apply_transposed(&self.matrix_c, f, l, u) }

    pub fn scalar_a(&self, x: i16, y: i16, z: i16) -> i16 { scalar(&self.matrix_a, x, y, z) }
    pub fn scalar_b(&self, x: i16, y: i16, z: i16) -> i16 { scalar(&self.matrix_b, x, y, z) }
    pub fn scalar_c(&self, x: i16, y: i16, z: i16) -> i16 { scalar(&self.matrix_c, x, y, z) }

    /// Uses the documented formula directly (from the bsnes comment's prose derivation,
    /// not the fixed-point table chain) — see the "Now the discussion" note in your source
    /// re: the Y/X rotation-order inconsistency between gyrate and attitude.
    pub fn gyrate(&self, az: i16, ax: i16, ay: i16, u: i16, f: i16, l: i16) -> (i16, i16, i16) {
        let to_rad = |a: i16| a as f64 * std::f64::consts::PI / 32768.0;
        let from_rad = |r: f64| (r * 32768.0 / std::f64::consts::PI).round() as i16;
        let (ax_r, ay_r) = (to_rad(ax), to_rad(ay));
        let (u_r, f_r, l_r) = (to_rad(u), to_rad(f), to_rad(l));

        let rx = ax_r + (u_r * ay_r.sin() + f_r * ay_r.cos());
        let ry = ay_r + l_r - ax_r.tan() * (u_r * ay_r.cos() + f_r * ay_r.sin());
        let rz = (u_r * ay_r.cos() - f_r * ay_r.sin()) / ax_r.cos();

        (from_rad(az as f64 * std::f64::consts::PI / 32768.0 + rz), from_rad(ax_r + rx - ax_r), from_rad(ay_r + ry - ay_r))
    }

    pub fn inverse(&self, coefficient: i16, exponent: i16) -> (i16, i16) {
        if coefficient == 0 {
            return (0x7fff, 0x002f);
        }
        let c = coefficient as f64 / 32768.0;
        let inv = 1.0 / (2.0 * c.abs());
        let i_coeff = (inv * 32768.0).round().clamp(-32768.0, 32767.0) as i16;
        (i_coeff * coefficient.signum(), 1 - exponent)
    }

    pub fn parameter(&mut self, fx: i16, fy: i16, fz: i16, lfe: i16, les: i16, aas: i16, azs: i16) -> (i16, i16, i16, i16) {
        let aas_r = to_rad(aas);
        let mut azs_r = to_rad(azs);

        // The real chip clips the zenith angle via a hardware lookup table so the
        // horizon plane never fully degenerates. This is an approximation of that
        // behavior (avoids a singularity at azs_r == 0), not a match to its exact
        // clip curve — verify against real hardware if precise clipping matters.
        const MIN_AZS: f64 = 0.05;
        if azs_r.abs() < MIN_AZS {
            azs_r = MIN_AZS * if azs_r < 0.0 { -1.0 } else { 1.0 };
        }

        let (sin_aas, cos_aas) = (aas_r.sin(), aas_r.cos());
        let (sin_azs, cos_azs) = (azs_r.sin(), azs_r.cos());

        let normal = [-sin_azs * sin_aas, sin_azs * cos_aas, cos_azs];
        let horiz = [cos_aas, sin_aas, 0.0];
        let vert = [-cos_azs * sin_aas, cos_azs * cos_aas, -sin_azs];

        let f = [q(fx), q(fy), q(fz)];
        let (lfe_f, les_f) = (q(lfe), q(les));

        let centre = [f[0] + lfe_f * normal[0], f[1] + lfe_f * normal[1], f[2] + lfe_f * normal[2]];
        let screen = [centre[0] - les_f * normal[0], centre[1] - les_f * normal[1], centre[2] - les_f * normal[2]];

        // ground separation between screen centre and the viewpoint's ground projection
        let c = centre[2] * azs_r.tan();
        let cx = centre[0] + c * sin_aas;
        let cy = centre[1] - c * cos_aas;

        let v_offset = les_f * cos_azs;
        let vva = -(v_offset / sin_azs);

        self.projection = Some(ProjectionState {
            centre, screen, normal, horiz, vert, les: les_f, v_offset,
            sin_aas, cos_aas, sin_azs, cos_azs,
        });

        (0, unq(vva), unq(cx), unq(cy)) // Vof left at 0: see clip-correction note below
    }

    /// Scale/rotation matrix for objects lying on raster line `vs`.
    fn raster_matrix(&self, vs: i16) -> Option<(f64, f64, f64, f64)> {
        let p = self.projection.as_ref()?;
        let voff_line = q(vs) * p.sin_azs + p.v_offset;
        if voff_line.abs() < 1e-6 { return None; } // degenerate line

        let scale = p.centre[2] / voff_line;
        let scale_v = scale / p.cos_azs;
        Some((scale * p.cos_aas, -scale_v * p.sin_aas, scale * p.sin_aas, scale_v * p.cos_aas)) // (A, B, C, D)
    }

    pub fn raster(&self, vs: i16) -> (i16, i16, i16, i16) {
        match self.raster_matrix(vs) {
            Some((a, b, c, d)) => (unq(a), unq(b), unq(c), unq(d)),
            None => (0, 0, 0, 0),
        }
    }

    pub fn target(&self, h: i16, v: i16) -> (i16, i16) {
        let p = self.projection.as_ref().unwrap();
        let (a, b, c, d) = self.raster_matrix(v).unwrap_or((0.0, 0.0, 0.0, 0.0));
        let (h, v) = (h as f64, v as f64); // raw screen-offset units, not Q1.15 — verify scale empirically
        let x = p.centre[0] + a * h + b * v;
        let y = p.centre[1] - c * h + d * v;
        (unq(x), unq(y))
    }

    pub fn project(&self, x: i16, y: i16, z: i16) -> (i16, i16, i16) {
        let p = self.projection.as_ref().unwrap();
        let point = [q(x), q(y), q(z)];
        let rel = [point[0] - p.screen[0], point[1] - p.screen[1], point[2] - p.screen[2]];

        let dot = |a: &[f64; 3], b: &[f64; 3]| a[0]*b[0] + a[1]*b[1] + a[2]*b[2];
        let p_dot_n = dot(&rel, &p.normal);

        let denom = p.les - p_dot_n;
        if denom.abs() < 1e-6 {
            return (32767, 32767, 32767); // point at/behind the screen plane; clamp rather than divide by ~0
        }
        let scale = p.les / denom;

        let h = dot(&rel, &p.horiz) * scale;
        let v = dot(&rel, &p.vert) * scale;
        let m = scale * 256.0; // M==0x0100 (256) means 1:1, per the source comment

        (unq(h), unq(v), unq(m).max(0))
    }
}

pub struct Dsp1 {
    state: Dsp1State,
    active_opcode: Option<u8>,
    active_spec: Option<CommandSpec>,
    input: Vec<u8>,
    output: Vec<u8>,
    output_pos: usize,
    frozen: bool,
    raster_line: i16,
}

impl Dsp1 {
    pub fn new() -> Self {
        Self {
            state: Dsp1State::default(),
            active_opcode: None,
            active_spec: None,
            input: Vec::new(),
            output: Vec::new(),
            output_pos: 0,
            frozen: false,
            raster_line: 0,
        }
    }

    /// CPU write to the Command/Data register.
    pub fn write(&mut self, value: u8) {
        // log::debug!("Write to DSP-1 Data w/ {value:02X}");

        if self.frozen {
            return; // Op1A: chip locks up entirely, matching the real hardware quirk
        }

        if self.active_opcode.is_none() {
            if FREEZE_COMMANDS.contains(&value) {
                self.frozen = true;
                return;
            }
            self.active_opcode = Some(value);
            self.active_spec = Some(lookup_command(value));
            self.input.clear();
            self.output.clear();
            self.output_pos = 0;

            // Commands that take no input (this only happens for opcodes not
            // present in lookup_command's match, which fall back to the 0-length
            // no-op spec) have nothing further to wait for - run them immediately
            // rather than sitting open and silently swallowing every future write
            // as bogus "input" for a command that can never reach its input_len.
            if self.active_spec.as_ref().unwrap().input_len == 0 {
                self.run_active_command(&[]);
            }
            return;
        }

        let spec = self.active_spec.as_ref().unwrap();
        self.input.push(value);
        if self.input.len() == spec.input_len {
            let params = read_i16s(&self.input);
            if self.active_opcode == Some(RASTER_CMD) {
                self.raster_line = params[0]; // Vs, only consulted on the *first* line
            }
            self.run_active_command(&params);
        }
    }

    /// Runs the active command's exec function and stores its output. If the
    /// command produces no output bytes, there's no read() call that will ever
    /// come along to drain it and return us to "ready for a new opcode" - reset
    /// here instead. (RASTER_CMD's output is never empty, so this doesn't
    /// interfere with its self-advancing behavior in read().)
    fn run_active_command(&mut self, params: &[i16]) {
        let spec = self.active_spec.as_ref().unwrap();
        let result = (spec.exec)(&mut self.state, params);
        self.output = write_i16s(&result);
        self.output_pos = 0;

        if self.output.is_empty() {
            self.active_opcode = None;
            self.active_spec = None;
        }
    }

    /// CPU read from the Command/Data register.
    pub fn read(&mut self) -> u8 {
        // log::debug!("Read from DSP-1 Data");

        if self.output_pos >= self.output.len() {
            return 0;
        }

        let byte = self.output[self.output_pos];
        self.output_pos += 1;

        if self.output_pos == self.output.len() {
            if self.active_opcode == Some(RASTER_CMD) {
                let last_word = i16::from_le_bytes([
                    self.output[self.output.len() - 2],
                    self.output[self.output.len() - 1],
                ]);
                // Raster streams continuously (built for HDMA): once the CPU
                // has read a full (An,Bn,Cn,Dn) block, the chip auto-advances
                // to the next line and keeps producing results without a new
                // command write, stopping only when Dn comes back as the
                // out-of-range sentinel 0x8000 (i16::MIN).
                if last_word != i16::MIN {
                    self.raster_line = self.raster_line.wrapping_add(1);
                    let (a, b, c, d) = self.state.raster(self.raster_line);
                    self.output = write_i16s(&[a, b, c, d]);
                    self.output_pos = 0;
                    return byte;
                }
            }
            self.active_opcode = None;
            self.active_spec = None;
        }
        byte
    }

    /// CPU read from the Status register.
    pub fn status(&self) -> u8 {
        // log::debug!("Read from DSP-1 Status");

        let has_pending_output = self.output_pos < self.output.len();
        let awaiting_input = self.active_spec.as_ref()
            .map(|s| self.input.len() < s.input_len)
            .unwrap_or(false);

        if has_pending_output || awaiting_input || self.active_spec.is_none() {
            STATUS_RQM
        } else {
            0
        }
    }
}

fn lookup_command(opcode: u8) -> CommandSpec {
    match opcode {
        0x00 => CommandSpec { input_len: 4, exec: |_s, p| vec![multiply(p[0], p[1])] },
        0x20 => CommandSpec { input_len: 4, exec: |_s, p| vec![multiply(p[0], p[1]).wrapping_add(1)] },

        0x01 => CommandSpec { input_len: 8, exec: |s, p| { s.attitude_a(p[0], p[1], p[2], p[3]); vec![] } },
        0x11 => CommandSpec { input_len: 8, exec: |s, p| { s.attitude_b(p[0], p[1], p[2], p[3]); vec![] } },
        0x21 => CommandSpec { input_len: 8, exec: |s, p| { s.attitude_c(p[0], p[1], p[2], p[3]); vec![] } },

        0x02 => CommandSpec {
            input_len: 14,
            exec: |s, p| { let (vof, vva, cx, cy) = s.parameter(p[0], p[1], p[2], p[3], p[4], p[5], p[6]); vec![vof, vva, cx, cy] },
        },

        0x03 => CommandSpec { input_len: 6, exec: |s, p| { let (x, y, z) = s.subjective_a(p[0], p[1], p[2]); vec![x, y, z] } },
        0x13 => CommandSpec { input_len: 6, exec: |s, p| { let (x, y, z) = s.subjective_b(p[0], p[1], p[2]); vec![x, y, z] } },
        0x23 => CommandSpec { input_len: 6, exec: |s, p| { let (x, y, z) = s.subjective_c(p[0], p[1], p[2]); vec![x, y, z] } },

        0x04 => CommandSpec { input_len: 4, exec: |_s, p| { let (y, x) = triangle(p[0], p[1]); vec![y, x] } },

        0x06 => CommandSpec { input_len: 6, exec: |s, p| { let (h, v, m) = s.project(p[0], p[1], p[2]); vec![h, v, m] } },

        0x08 => CommandSpec {
            input_len: 6,
            exec: |_s, p| { let r = radius(p[0], p[1], p[2]); vec![r as i16, (r >> 16) as i16] },
        },

        0x0A => CommandSpec { input_len: 2, exec: |s, p| { let (a, b, c, d) = s.raster(p[0]); vec![a, b, c, d] } },

        0x0B => CommandSpec { input_len: 6, exec: |s, p| vec![s.scalar_a(p[0], p[1], p[2])] },
        0x1B => CommandSpec { input_len: 6, exec: |s, p| vec![s.scalar_b(p[0], p[1], p[2])] },
        0x2B => CommandSpec { input_len: 6, exec: |s, p| vec![s.scalar_c(p[0], p[1], p[2])] },

        0x0C => CommandSpec { input_len: 6, exec: |_s, p| { let (x2, y2) = rotate(p[0], p[1], p[2]); vec![x2, y2] } },

        0x0D => CommandSpec { input_len: 6, exec: |s, p| { let (f, l, u) = s.objective_a(p[0], p[1], p[2]); vec![f, l, u] } },
        0x1D => CommandSpec { input_len: 6, exec: |s, p| { let (f, l, u) = s.objective_b(p[0], p[1], p[2]); vec![f, l, u] } },
        0x2D => CommandSpec { input_len: 6, exec: |s, p| { let (f, l, u) = s.objective_c(p[0], p[1], p[2]); vec![f, l, u] } },

        0x0E => CommandSpec { input_len: 4, exec: |s, p| { let (x, y) = s.target(p[0], p[1]); vec![x, y] } },

        0x0F => CommandSpec { input_len: 2, exec: |_s, _p| vec![memory_test()] },

        0x10 => CommandSpec { input_len: 4, exec: |s, p| { let (ic, ie) = s.inverse(p[0], p[1]); vec![ic, ie] } },

        0x14 => CommandSpec {
            input_len: 12,
            exec: |s, p| { let (rz, rx, ry) = s.gyrate(p[0], p[1], p[2], p[3], p[4], p[5]); vec![rz, rx, ry] },
        },

        0x18 => CommandSpec { input_len: 8, exec: |_s, p| vec![range(p[0], p[1], p[2], p[3])] },
        0x38 => CommandSpec { input_len: 8, exec: |_s, p| vec![range(p[0], p[1], p[2], p[3]).wrapping_add(1)] },

        0x1C => CommandSpec {
            input_len: 12,
            exec: |_s, p| { let (x2, y2, z2) = polar(p[0], p[1], p[2], p[3], p[4], p[5]); vec![x2, y2, z2] },
        },

        // NOT a real implementation — see note below. Placeholder only.
        0x1F => CommandSpec { input_len: 2, exec: |_s, _p| vec![0i16; 1024] },

        0x28 => CommandSpec { input_len: 6, exec: |_s, p| vec![distance(p[0], p[1], p[2])] },

        0x2F => CommandSpec { input_len: 2, exec: |_s, _p| vec![memory_size()] },

        _ => {
            log::warn!("DSP-1: unrecognized opcode ${opcode:02X}, treating as no-op");
            CommandSpec { input_len: 0, exec: |_s, _p| vec![] }
        }
    }
}

pub fn multiply(a: i16, b: i16) -> i16 {
    q15_mul(a, b)
}

pub fn triangle(angle: i16, radius: i16) -> (i16, i16) {
    (q15_mul(sin_q15(angle), radius), q15_mul(cos_q15(angle), radius)) // (Y, X)
}

pub fn radius(x: i16, y: i16, z: i16) -> i32 {
    ((x as i32 * x as i32) + (y as i32 * y as i32) + (z as i32 * z as i32)) << 1
}

pub fn range(x: i16, y: i16, z: i16, r: i16) -> i16 {
    (((x as i32 * x as i32) + (y as i32 * y as i32) + (z as i32 * z as i32)
        - (r as i32 * r as i32)) >> 15) as i16
}

pub fn distance(x: i16, y: i16, z: i16) -> i16 {
    let sq = (x as f64).powi(2) + (y as f64).powi(2) + (z as f64).powi(2);
    if sq == 0.0 { return 0; }
    // sq is in Q2.30 (product of two Q1.15 values); take sqrt directly.
    let sq_q30 = sq / (32768.0 * 32768.0);
    (sq_q30.sqrt() * 32768.0).round().clamp(-32768.0, 32767.0) as i16
}

pub fn rotate(angle: i16, x1: i16, y1: i16) -> (i16, i16) {
    let s = sin_q15(angle);
    let c = cos_q15(angle);
    let x2 = q15_mul(y1, s).wrapping_add(q15_mul(x1, c));
    let y2 = q15_mul(y1, c).wrapping_sub(q15_mul(x1, s));
    (x2, y2)
}

pub fn polar(az: i16, ay: i16, ax: i16, x1: i16, y1: i16, z1: i16) -> (i16, i16, i16) {
    let (x, y) = rotate(az, x1, y1);
    let (mut x1, mut y1) = (x, y);
    let (z, x2) = {
        let s = sin_q15(ay); let c = cos_q15(ay);
        (q15_mul(x1, s).wrapping_add(q15_mul(z1, c)), q15_mul(x1, c).wrapping_sub(q15_mul(z1, s)))
    };
    let mut z1 = z;
    let (y, z2) = {
        let s = sin_q15(ax); let c = cos_q15(ax);
        (q15_mul(z1, s).wrapping_add(q15_mul(y1, c)), q15_mul(z1, c).wrapping_sub(q15_mul(y1, s)))
    };
    let _ = (&mut x1, &mut y1, &mut z1); // silence unused warnings from the intermediate swaps above
    (x2, y, z2)
}

// -- Attitude-matrix family (A/B/C share identical math, differ only in which matrix they touch) --

fn build_attitude(s: i16, rz: i16, ry: i16, rx: i16) -> Mat3 {
    let (sin_rz, cos_rz) = (sin_q15(rz), cos_q15(rz));
    let (sin_ry, cos_ry) = (sin_q15(ry), cos_q15(ry));
    let (sin_rx, cos_rx) = (sin_q15(rx), cos_q15(rx));
    let s = s >> 1;

    [
        [
            q15_mul(q15_mul(s, cos_rz), cos_ry),
            q15_mul(q15_mul(s, sin_rz), cos_rx).wrapping_add(q15_mul(q15_mul(q15_mul(s, cos_rz), sin_rx), sin_ry)),
            q15_mul(q15_mul(s, sin_rz), sin_rx).wrapping_sub(q15_mul(q15_mul(q15_mul(s, cos_rz), cos_rx), sin_ry)),
        ],
        [
            -q15_mul(q15_mul(s, sin_rz), cos_ry),
            q15_mul(q15_mul(s, cos_rz), cos_rx).wrapping_sub(q15_mul(q15_mul(q15_mul(s, sin_rz), sin_rx), sin_ry)),
            q15_mul(q15_mul(s, cos_rz), sin_rx).wrapping_add(q15_mul(q15_mul(q15_mul(s, sin_rz), cos_rx), sin_ry)),
        ],
        [
            q15_mul(s, sin_ry),
            -q15_mul(q15_mul(s, sin_rx), cos_ry),
            q15_mul(q15_mul(s, cos_rx), cos_ry),
        ],
    ]
}

fn mat3_apply(m: &Mat3, x: i16, y: i16, z: i16) -> (i16, i16, i16) {
    // "objective": global -> object coords, using columns
    let f = q15_mul(m[0][0], x).wrapping_add(q15_mul(m[1][0], y)).wrapping_add(q15_mul(m[2][0], z));
    let l = q15_mul(m[0][1], x).wrapping_add(q15_mul(m[1][1], y)).wrapping_add(q15_mul(m[2][1], z));
    let u = q15_mul(m[0][2], x).wrapping_add(q15_mul(m[1][2], y)).wrapping_add(q15_mul(m[2][2], z));
    (f, l, u)
}

fn mat3_apply_transposed(m: &Mat3, f: i16, l: i16, u: i16) -> (i16, i16, i16) {
    // "subjective": object -> global coords, using rows
    let x = q15_mul(m[0][0], f).wrapping_add(q15_mul(m[0][1], l)).wrapping_add(q15_mul(m[0][2], u));
    let y = q15_mul(m[1][0], f).wrapping_add(q15_mul(m[1][1], l)).wrapping_add(q15_mul(m[1][2], u));
    let z = q15_mul(m[2][0], f).wrapping_add(q15_mul(m[2][1], l)).wrapping_add(q15_mul(m[2][2], u));
    (x, y, z)
}

fn scalar(m: &Mat3, x: i16, y: i16, z: i16) -> i16 {
    (((x as i32 * m[0][0] as i32) + (y as i32 * m[1][0] as i32) + (z as i32 * m[2][0] as i32)) >> 15) as i16
}

pub fn memory_test() -> i16 { 0x0000 } // always reports OK
pub fn memory_size() -> i16 { 0x0100 } // DSP1/DSP1A revision; use 0x0101 to report as DSP1B