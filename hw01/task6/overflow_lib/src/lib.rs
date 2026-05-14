/// Сложение u8 с проверкой переполнения.
/// Возвращает Some(result), если нет переполнения, иначе None.
pub fn add_u8_checked(a: u8, b: u8) -> Option<u8> {
    let sum = a as u16 + b as u16; // Используем u16 для проверки переполнения
    if sum > 255 {
        None
    } else {
        Some(sum as u8)
    }
}

/// Сложение u8 с «заворачиванием» при переполнении.
/// При переполнении результат «заворачивается» по кругу (mod 256).
pub fn add_u8_wrapping(a: u8, b: u8) -> u8 {
    // При переполнении берём остаток от деления на 256 (т. е. младшие 8 бит)
    ((a as u16 + b as u16) % 256) as u8
}

/// Сложение u8 с насыщением при переполнении.
/// Если результат превышает 255, возвращается 255.
pub fn add_u8_saturating(a: u8, b: u8) -> u8 {
    let sum = a as u16 + b as u16;
    if sum > 255 {
        255
    } else {
        sum as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsigned_overflow_modes() {
        // Проверка checked mode
        assert_eq!(add_u8_checked(255, 1), None);
        assert_eq!(add_u8_checked(10, 20), Some(30));

        // Проверка wrapping mode
        assert_eq!(add_u8_wrapping(255, 1), 0);
        assert_eq!(add_u8_wrapping(10, 20), 30);

        // Проверка saturating mode
        assert_eq!(add_u8_saturating(255, 1), 255);
        assert_eq!(add_u8_saturating(10, 20), 30);
    }

    #[test]
    fn edge_cases() {
        // Граничные случаи для checked
        assert_eq!(add_u8_checked(0, 0), Some(0));
        assert_eq!(add_u8_checked(255, 0), Some(255));

        // Граничные случаи для wrapping
        assert_eq!(add_u8_wrapping(0, 0), 0);
        assert_eq!(add_u8_wrapping(255, 0), 255);

        // Граничные случаи для saturating
        assert_eq!(add_u8_saturating(0, 0), 0);
        assert_eq!(add_u8_saturating(255, 0), 255);
    }

    #[test]
    fn saturating_large_values() {
        assert_eq!(add_u8_saturating(200, 100), 255); // Переполнение
        assert_eq!(add_u8_saturating(150, 100), 250); // Без переполнения
    }

    #[test]
    fn wrapping_around_boundary() {
        assert_eq!(add_u8_wrapping(254, 2), 0);  // 256 % 256 = 0
        assert_eq!(add_u8_wrapping(253, 3), 0);  // 256 % 256 = 0
        assert_eq!(add_u8_wrapping(250, 10), 4); // 260 % 256 = 4
    }
}
