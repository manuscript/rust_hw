/// Преобразует картинку 8×8 из строк в массив из 8 байтов
pub fn parse_bitmap_8x8(lines: [&str; 8]) -> [u8; 8] {
    let mut bytes = [0u8; 8];

    for (row_idx, line) in lines.iter().enumerate() {
        let mut byte = 0u8;

        for (char_idx, ch) in line.chars().take(8).enumerate() {
            // Символ с индексом i соответствует биту 7-i (старший бит слева)
            let bit_pos = 7 - char_idx;

            if ch == '#' {
                // Устанавливаем бит в позиции bit_pos
                byte |= 1 << bit_pos;
            }
            // Если ch == '.', бит остаётся 0 — ничего делать не нужно
        }

        bytes[row_idx] = byte;
    }

    bytes
}

/// Преобразует массив из 8 байтов в картинку 8×8
pub fn render_bitmap_8x8(bytes: [u8; 8]) -> [String; 8] {
    let mut lines = [String::new(); 8];

    for (row_idx, &byte) in bytes.iter().enumerate() {
        let mut line = String::with_capacity(8);

        for bit_pos in (0..8).rev() {
            // Проверяем, установлен ли бит в позиции bit_pos
            let is_set = (byte >> bit_pos) & 1 == 1;

            if is_set {
                line.push('#');
            } else {
                line.push('.');
            }
        }

        lines[row_idx] = line;
    }

    lines
}

/// Инвертирует все биты в каждом байте (меняет 0↔1, .↔#)
pub fn invert_bitmap_8x8(bytes: [u8; 8]) -> [u8; 8] {
    let mut inverted = [0u8; 8];

    for i in 0..8 {
        // Инверсия всех битов в байте
        inverted[i] = !bytes[i];
    }

    inverted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bitmap() {
        let image = [
            "..####..",
            ".#....#.",
            "#.#..#.#",
            "#..##..#",
            "#......#",
            "#.#..#.#",
            ".#....#.",
            "..####..",
        ];

        let bytes = parse_bitmap_8x8(image);

        assert_eq!(bytes, [
            0b0011_1100, // 0x3C
            0b0100_0010, // 0x42
            0b1010_0101, // 0xA5
            0b1001_1001, // 0x99
            0b1000_0001, // 0x81
            0b1010_0101, // 0xA5
            0b0100_0010, // 0x42
            0b0011_1100, // 0x3C
        ]);
    }

    #[test]
    fn test_render_bitmap() {
        let bytes = [
            0b0011_1100,
            0b0100_0010,
            0b1010_0101,
            0b1001_1001,
            0b1000_0001,
            0b1010_0101,
            0b0100_0010,
            0b0011_1100,
        ];

        let rendered = render_bitmap_8x8(bytes);

        let expected = [
            "..####..".to_string(),
            ".#....#.".to_string(),
            "#.#..#.#".to_string(),
            "#..##..#".to_string(),
            "#......#".to_string(),
            "#.#..#.#".to_string(),
            ".#....#.".to_string(),
            "..####..".to_string(),
        ];

        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_invert_bitmap() {
        let original = [
            0b0011_1100,
            0b0100_0010,
            0b1010_0101,
            0b1001_1001,
            0b1000_0001,
            0b1010_0101,
            0b0100_0010,
            0b0011_1100,
        ];

        let inverted = invert_bitmap_8x8(original);

        let expected = [
            0b1100_0011, // ##....##
            0b1011_1101, // #.####.#
            0b0101_1010, // .#.##.#.
            0b0110_0110, // .##..##.
            0b0111_1110, // .######.
            0b0101_1010, // .#.##.#.
            0b1011_1101, // #.####.#
            0b1100_0011, // ##....##
        ];

        assert_eq!(inverted, expected);
    }

    #[test]
    fn test_round_trip() {
        let image = [
            "..####..",
            ".#....#.",
            "#.#..#.#",
            "#..##..#",
            "#......#",
            "#.#..#.#",
            ".#....#.",
            "..####..",
        ];

        let bytes = parse_bitmap_8x8(image);
        let rendered = render_bitmap_8x8(bytes);

        // Проверяем, что после преобразования и рендера картинка осталась той же
        for i in 0..8 {
            assert_eq!(image[i], rendered[i]);
        }
    }
}

fn main() {
    let image = [
        "..####..",
        ".#....#.",
        "#.#..#.#",
        "#..##..#",
        "#......#",
        "#.#..#.#",
        ".#....#.",
        "..####..",
    ];

    let bytes = parse_bitmap_8x8(image);

    println!("Bytes:");
    for byte in bytes {
        println!("{byte:08b}  0x{byte:02X}");
    }
    println!();

    println!("Rendered:");
    for line in render_bitmap_8x8(bytes) {
        println!("{line}");
    }
    println!();

    println!("Inverted:");
    for line in render_bitmap_8x8(invert_bitmap_8x8(bytes)) {
        println!("{line}");
    }
}
