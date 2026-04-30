use binary_reader::bit_reader::BitReader;
use binary_writer::bit_writer::BitWriter;

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
