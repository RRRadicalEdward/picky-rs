use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

#[allow(dead_code)]
#[allow(unused)]
pub mod key;

pub trait SshParser {
    type Error;

    fn decode(stream: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized;
    fn encode(&self, stream: impl Write) -> Result<(), Self::Error>;
}

pub(crate) struct Minpt(pub(crate) Vec<u8>);
pub(crate) struct SshString(pub(crate) String);

impl SshParser for Minpt {
    type Error = io::Error;

    fn decode(mut stream: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let size = stream.read_u32::<BigEndian>()? as usize;
        let mut buffer = vec![0; size];
        stream.read_exact(&mut buffer)?;

        if buffer[0] == 0 {
            buffer.remove(0);
        }

        Ok(Minpt(buffer))
    }

    fn encode(&self, mut stream: impl Write) -> Result<(), Self::Error> {
        let size = self.0.len();
        stream.write_u32::<BigEndian>(size as u32)?;
        stream.write_all(&self.0)
    }
}

impl SshParser for SshString {
    type Error = io::Error;

    fn decode(mut stream: impl Read) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let size = stream.read_u32::<BigEndian>()? as usize;
        let mut buffer = vec![0; size];
        stream.read_exact(&mut buffer)?;

        Ok(SshString(String::from_utf8_lossy(&buffer).to_string()))
    }

    fn encode(&self, mut stream: impl Write) -> Result<(), Self::Error> {
        let size = self.0.len();
        stream.write_u32::<BigEndian>(size as u32)?;
        stream.write_all(self.0.as_bytes())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ssh::key::SshPublicKey;
    use std::io::Cursor;

    #[test]
    fn mpint_decoding() {
        let mpint: Minpt = SshParser::decode(Cursor::new(vec![
            0x00, 0x00, 0x00, 0x08, 0x09, 0xa3, 0x78, 0xf9, 0xb2, 0xe3, 0x32, 0xa7,
        ]))
        .unwrap();
        assert_eq!(mpint.0, vec![0x09, 0xa3, 0x78, 0xf9, 0xb2, 0xe3, 0x32, 0xa7]);

        let mpint: Minpt = SshParser::decode(Cursor::new(vec![0x00, 0x00, 0x00, 0x02, 0x00, 0x80])).unwrap();
        assert_eq!(mpint.0, vec![0x00, 0x80]);

        let mpint: Minpt = SshParser::decode(Cursor::new(vec![0x00, 0x00, 0x00, 0x02, 0xed, 0xcc])).unwrap();
        assert_eq!(mpint.0, vec![0xed, 0xcc]);
    }

    #[test]
    fn mpint_encoding() {
        let mpint = Minpt(vec![0x09, 0xa3, 0x78, 0xf9, 0xb2, 0xe3, 0x32, 0xa7]);
        let mut cursor = Cursor::new(Vec::new());
        mpint.encode(&mut cursor).unwrap();

        assert_eq!(
            cursor.into_inner(),
            vec![0x00, 0x00, 0x00, 0x08, 0x09, 0xa3, 0x78, 0xf9, 0xb2, 0xe3, 0x32, 0xa7],
        );

        let mpint = Minpt(vec![0x80]);
        let mut cursor = Cursor::new(Vec::new());
        mpint.encode(&mut cursor).unwrap();

        assert_eq!(cursor.into_inner(), vec![0x00, 0x00, 0x00, 0x01, 0x80]);
    }
}
