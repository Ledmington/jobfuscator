use binary_reader::bit_reader::BitReader;
use binary_writer::bit_writer::BitWriter;

/*
 * Reference: https://www.rfc-editor.org/rfc/rfc1951.txt.
 */

pub fn decompress(input: &[u8]) -> Vec<u8> {
    let mut reader = BitReader::new(input);
    let mut writer = BitWriter::new();

    loop {
        {
            let pos = reader.get_byte_position();
            println!(
                "0x{:02x} 0x{:02x}",
                reader.read_u8().unwrap(),
                reader.read_u8().unwrap()
            );

            reader.set_byte_position(pos);
        }

        let bfinal: bool = reader.read_bit().unwrap();
        println!("BFINAL = {}", bfinal);

        let btype_0: bool = reader.read_bit().unwrap();
        let btype_1: bool = reader.read_bit().unwrap();
        let btype: u8 = (btype_0 as u8) | ((btype_1 as u8) << 1);
        assert!(btype <= 3);

        println!(
            "BTYPE = {}{}",
            if btype_1 { "1" } else { "0" },
            if btype_0 { "1" } else { "0" }
        );

        match btype {
            0b00 => {
                // The block is uncompressed
                reader.align_to_byte();

                let len: u16 = reader.read_u16().unwrap();
                {
                    let nlen: u16 = reader.read_u16().unwrap();
                    assert_eq!(
                        nlen, !len,
                        "Expected NLEN (0x{:04x}) to be one's complement of LEN (0x{:04x}) but it wasn't.",
                        len, nlen
                    );
                }

                // follows LEN bytes of literal data...
                writer.write_u8_vec(&reader.read_u8_vec(len.into()).unwrap());
            }
            0b01 => {
                // Static Huffman
                println!("Static Huffman");
            }
            0b10 => {
                // Dynamic Huffman
                println!("Dynamic Huffman");
            }
            0b11 => {
                panic!("BTYPE=11 is reserved.");
            }
            _ => unreachable!(),
        }

        if bfinal {
            break;
        }
    }

    writer.array()
}
