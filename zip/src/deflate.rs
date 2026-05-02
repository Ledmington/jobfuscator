use binary_reader::bit_reader::BitReader;
use binary_writer::bit_writer::BitWriter;

/*
 * Reference: https://www.rfc-editor.org/rfc/rfc1951.txt.
 */

pub fn decompress(input: &Vec<u8>) -> Vec<u8> {
    let mut reader = BitReader::new(input);
    let mut writer = BitWriter::new();

    loop {
        let bfinal: bool = reader.read_bit().unwrap();
        if bfinal {
            break;
        }

        let btype_0: bool = reader.read_bit().unwrap();
        let btype_1: bool = reader.read_bit().unwrap();
        let btype: u8 = ((btype_0 as u8) << 1) | (btype_1 as u8);
        assert!(btype <= 3);

        match btype {
            0b00 => {
                // The block is uncompressed
                let len: u16 = reader.read_u16().unwrap();
                {
                    let nlen: u16 = reader.read_u16().unwrap();
                    assert!(
                        !len == nlen,
                        "Expected NLEN (0x{:04x}) to be one's complement of LEN (0x{:04x}) but it wasn't.",
                        len,
                        nlen
                    );
                }

                // follows LEN bytes of literal data...
            }
            0b01 => {
                // Static Huffman
            }
            0b10 => {
                // Dynamic Huffman
            }
            0b11 => {
                panic!("BTYPE=11 is reserved.");
            }
            _ => unreachable!(),
        }
    }

    return writer.array();
}
