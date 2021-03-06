use std::ptr;
use std::io::Write;
use JsonValue;
use number::Number;
use object::Object;
use std::io;

use util::print_dec;

const QU: u8 = b'"';
const BS: u8 = b'\\';
const BB: u8 = b'b';
const TT: u8 = b't';
const NN: u8 = b'n';
const FF: u8 = b'f';
const RR: u8 = b'r';
const UU: u8 = b'u';
const __: u8 = 0;

// Look up table for characters that need escaping in a product string
static ESCAPED: [u8; 256] = [
// 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
  UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
  UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
  __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
  __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];

#[cfg(test)]
mod gen_test {
    use codegen::DumpGenerator;
    use codegen::Generator;
    use std::borrow::Borrow;
    use JsonValue;
    use ::parse;

    // can't do an equality check on the output bytes here since quotes and escape characters are added

    #[test]
    fn should_not_panic_on_bad_bytes() {
        // found from fuzzing the json stringify function
        let mut all = [255,255,255,255,255,255,255,255,255,0,217,216,255,255,255,255,255,255,255,255,249,217,255,255,144,255,255,1,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,0,255,0,217,23,23,23,23,23,23,23,23,23,23,23,0,0,22,0,22,0,1,0,0,0,0,0,0,21,27,22,0,1,0,0,0,0,0,0,22,14,0,210,38,221,0,0,0,0,14,0,16,0,4,16,29,29,29,29,29,29,29,29,29,29,0,0,0,5,14,0,0,0,29,29,29,29,29,29,29,29,29,29,29,29,29,0,0,29,29,29,29,144,0,0,8,0,0,0,0,250,190,255,0,0,0,0,0,0,0,0,22,0,1,0,0,0,0,14,0,0,0,0,14,22,14,0,14,0,14,14,14,0,0,0,27,27,27,27,27,22,0,14,0,0,0,0,0,0,0,14,0,0,0,5,14,0,0,0,29,29,29,29,29,29,29,29,29,29,29,29,29,29,29,29,29,29,29,29,12,0,22,14,14,14,14,0,0,0,0,0,0,14,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,14,14,0,0,0,0,88,88,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,14,0,0,0,0,0,14,0,0,0,0,0,0,0,14,0,0,14,14,0,0,0,0,0,0,0,0,0,0,0,21,27,0,14,0,21,27,22,25,1,0,0,0,0,0,0,0,0,0,0,22,0,0,0,0,0,0,0,0,5,14,0,0,0,0,0,0,5,14,0,0,0,0,14,14,255,14,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,253,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,42,255,255,255,255,255,255,255,255,255,255,255,8,145];
        let input = String::from_utf8_lossy(&all);

        let mut generator = DumpGenerator::new();
        generator.write_string(&input);
    }

    #[test]
    fn should_panic_on_bad_bytes() {
        let mut all = "`�^S^R�^]^?^@^@E BOONE TRL ";
        let mut generator = DumpGenerator::new();
        generator.write_string_complex(all, 3);
    }

    #[test]
    fn should_not_panic_on_bad_bytes_2() {
        let mut all = [0, 12, 128, 88, 64, 99].to_vec();
        let input = String::from_utf8_lossy(&all);

        let mut generator = DumpGenerator::new();
        generator.write_string(&input);
    }

    #[test]
    fn should_not_panic_on_bad_bytes_3() {
        let data = b"\x48\x48\x48\x57\x03\xE8\x48\x48\xE8\x03\x8F\x48\x29\x48\x48";
        let s = unsafe {
            String::from_utf8_unchecked(data.to_vec())
        };
        let mut generator = DumpGenerator::new();
        generator.write_string(&s);
    }

    #[test]
    fn should_encode_special_characters() {
        let val = "🤓🥳,🤗,😧,😧";
        let mut data = JsonValue::new_object();
        data["foo"] = val.into();

        let encoded = data.dump();
        assert_eq!(encoded, "{\"foo\":\"🤓🥳,🤗,😧,😧\"}");
    }

    #[test]
    fn should_encode_special_characters_newline() {
        let val = "🤓🥳,🤗,😧,😧 \n foo 🤓🥳,🤗,😧,😧";
        let mut data = JsonValue::new_object();
        data["foo"] = val.into();

        let encoded = data.dump();
        let decoded = parse(&encoded).unwrap();

        assert_eq!(decoded, data, "json values eq");
        assert_eq!(encoded, "{\"foo\":\"🤓🥳,🤗,😧,😧 \\n foo 🤓🥳,🤗,😧,😧\"}", "json strings eq");
    }


}

pub trait Generator {
    type T: Write;

    #[inline(never)]
    fn write_string_complex(&mut self, string: &str, mut start: usize) -> io::Result<()> {
        let bytes = string.as_bytes();
        try!(self.write(&bytes[ .. start]));

        for (index, ch) in string.bytes().enumerate().skip(start) {
            let escape = ESCAPED[ch as usize];
            if escape > 0 {
                try!(self.write(&bytes[start .. index]));
                try!(self.write(&[b'\\', escape]));
                start = index + 1;
            }
            if escape == b'u' {
                try!(write!(self.get_writer(), "{:04x}", ch));
            }
        }
        try!(self.write(&bytes[start ..]));
        self.write_char(b'"')
    }

    fn get_writer(&mut self) -> &mut Self::T;

    #[inline(always)]
    fn write(&mut self, slice: &[u8]) -> io::Result<()> {
        self.get_writer().write_all(slice)
    }

    #[inline(always)]
    fn write_char(&mut self, ch: u8) -> io::Result<()> {
        self.get_writer().write_all(&[ch])
    }

    fn write_min(&mut self, slice: &[u8], min: u8) -> io::Result<()>;

    #[inline(always)]
    fn new_line(&mut self) -> io::Result<()> { Ok(()) }

    #[inline(always)]
    fn indent(&mut self) {}

    #[inline(always)]
    fn dedent(&mut self) {}

    #[inline(always)]
    fn write_string(&mut self, string: &str) -> io::Result<()> {
        try!(self.write_char(b'"'));

        for (index, ch) in string.bytes().enumerate() {
            if ESCAPED[ch as usize] > 0 {
                return self.write_string_complex(string, index)
            }
        }

        try!(self.write(string.as_bytes()));
        self.write_char(b'"')
    }

    #[inline(always)]
    fn write_number(&mut self, num: &Number) -> io::Result<()> {
        if num.is_nan() {
            return self.write(b"null");
        }
        let (positive, mantissa, exponent) = num.as_parts();
        unsafe {
            print_dec::write(
                self.get_writer(),
                positive,
                mantissa,
                exponent
            )
        }
    }

    #[inline(always)]
    fn write_object(&mut self, object: &Object) -> io::Result<()> {
        try!(self.write_char(b'{'));
        let mut iter = object.iter();

        if let Some((key, value)) = iter.next() {
            self.indent();
            try!(self.new_line());
            try!(self.write_string(key));
            try!(self.write_min(b": ", b':'));
            try!(self.write_json(value));
        } else {
            try!(self.write_char(b'}'));
            return Ok(());
        }

        for (key, value) in iter {
            try!(self.write_char(b','));
            try!(self.new_line());
            try!(self.write_string(key));
            try!(self.write_min(b": ", b':'));
            try!(self.write_json(value));
        }

        self.dedent();
        try!(self.new_line());
        self.write_char(b'}')
    }

    fn write_json(&mut self, json: &JsonValue) -> io::Result<()> {
        match *json {
            JsonValue::Null               => self.write(b"null"),
            JsonValue::Short(ref short)   => self.write_string(short.as_str()),
            JsonValue::String(ref string) => self.write_string(string),
            JsonValue::Number(ref number) => self.write_number(number),
            JsonValue::Boolean(true)      => self.write(b"true"),
            JsonValue::Boolean(false)     => self.write(b"false"),
            JsonValue::Array(ref array)   => {
                try!(self.write_char(b'['));
                let mut iter = array.iter();

                if let Some(item) = iter.next() {
                    self.indent();
                    try!(self.new_line());
                    try!(self.write_json(item));
                } else {
                    try!(self.write_char(b']'));
                    return Ok(());
                }

                for item in iter {
                    try!(self.write_char(b','));
                    try!(self.new_line());
                    try!(self.write_json(item));
                }

                self.dedent();
                try!(self.new_line());
                self.write_char(b']')
            },
            JsonValue::Object(ref object) => {
                self.write_object(object)
            }
        }
    }
}

pub struct DumpGenerator {
    code: Vec<u8>,
}

impl DumpGenerator {
    pub fn new() -> Self {
        DumpGenerator {
            code: Vec::with_capacity(1024),
        }
    }

    pub fn consume(self) -> String {
        // Original strings were unicode, numbers are all ASCII,
        // therefore this is safe.
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

impl Generator for DumpGenerator {
    type T = Vec<u8>;

    fn write(&mut self, slice: &[u8]) -> io::Result<()> {
        extend_from_slice(&mut self.code, slice);
        Ok(())
    }

    #[inline(always)]
    fn write_char(&mut self, ch: u8) -> io::Result<()> {
        self.code.push(ch);
        Ok(())
    }

    #[inline(always)]
    fn get_writer(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    #[inline(always)]
    fn write_min(&mut self, _: &[u8], min: u8) -> io::Result<()> {
        self.code.push(min);
        Ok(())
    }
}

pub struct PrettyGenerator {
    code: Vec<u8>,
    dent: u16,
    spaces_per_indent: u16,
}

impl PrettyGenerator {
    pub fn new(spaces: u16) -> Self {
        PrettyGenerator {
            code: Vec::with_capacity(1024),
            dent: 0,
            spaces_per_indent: spaces
        }
    }

    pub fn consume(self) -> String {
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

impl Generator for PrettyGenerator {
    type T = Vec<u8>;

    #[inline(always)]
    fn write(&mut self, slice: &[u8]) -> io::Result<()> {
        extend_from_slice(&mut self.code, slice);
        Ok(())
    }

    #[inline(always)]
    fn write_char(&mut self, ch: u8) -> io::Result<()> {
        self.code.push(ch);
        Ok(())
    }

    #[inline(always)]
    fn get_writer(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    #[inline(always)]
    fn write_min(&mut self, slice: &[u8], _: u8) -> io::Result<()> {
        extend_from_slice(&mut self.code, slice);
        Ok(())
    }

    fn new_line(&mut self) -> io::Result<()> {
        self.code.push(b'\n');
        for _ in 0..(self.dent * self.spaces_per_indent) {
            self.code.push(b' ');
        }
        Ok(())
    }

    fn indent(&mut self) {
        self.dent += 1;
    }

    fn dedent(&mut self) {
        self.dent -= 1;
    }
}

pub struct WriterGenerator<'a, W: 'a + Write> {
    writer: &'a mut W
}

impl<'a, W> WriterGenerator<'a, W> where W: 'a + Write {
    pub fn new(writer: &'a mut W) -> Self {
        WriterGenerator {
            writer: writer
        }
    }
}

impl<'a, W> Generator for WriterGenerator<'a, W> where W: Write {
    type T = W;

    #[inline(always)]
    fn get_writer(&mut self) -> &mut W {
        &mut self.writer
    }

    #[inline(always)]
    fn write_min(&mut self, _: &[u8], min: u8) -> io::Result<()> {
        self.writer.write_all(&[min])
    }
}


pub struct PrettyWriterGenerator<'a, W: 'a + Write> {
    writer: &'a mut W,
    dent: u16,
    spaces_per_indent: u16,
}

impl<'a, W> PrettyWriterGenerator<'a, W> where W: 'a + Write {
    pub fn new(writer: &'a mut W, spaces: u16) -> Self {
        PrettyWriterGenerator {
            writer: writer,
            dent: 0,
            spaces_per_indent: spaces,
        }
    }
}

impl<'a, W> Generator for PrettyWriterGenerator<'a, W> where W: Write {
    type T = W;

    #[inline(always)]
    fn get_writer(&mut self) -> &mut W {
        &mut self.writer
    }

    #[inline(always)]
    fn write_min(&mut self, slice: &[u8], _: u8) -> io::Result<()> {
        self.writer.write_all(slice)
    }

    fn new_line(&mut self) -> io::Result<()> {
        try!(self.write_char(b'\n'));
        for _ in 0..(self.dent * self.spaces_per_indent) {
            try!(self.write_char(b' '));
        }
        Ok(())
    }

    fn indent(&mut self) {
        self.dent += 1;
    }

    fn dedent(&mut self) {
        self.dent -= 1;
    }
}

// From: https://github.com/dtolnay/fastwrite/blob/master/src/lib.rs#L68
//
// LLVM is not able to lower `Vec::extend_from_slice` into a memcpy, so this
// helps eke out that last bit of performance.
#[inline]
fn extend_from_slice(dst: &mut Vec<u8>, src: &[u8]) {
    let dst_len = dst.len();
    let src_len = src.len();

    dst.reserve(src_len);

    unsafe {
        // We would have failed if `reserve` overflowed
        dst.set_len(dst_len + src_len);

        ptr::copy_nonoverlapping(
            src.as_ptr(),
            dst.as_mut_ptr().offset(dst_len as isize),
            src_len);
    }
}
