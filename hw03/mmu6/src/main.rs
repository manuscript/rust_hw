fn main() {
    println!("Hello, world!");
}


#[derive(Debug, PartialEq, Eq)]
pub enum MemoryError {
    PageNotPresent,     // если флаг P = 0
    WriteProtected,     // при попытке записи, если флаг W = 0
    PrivilegeViolation, // если CPL=Ring3, а флаг U = 0
    InvalidAddress,     // если переданный адрес > 63 (выходит за 6 бит)
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeLevel {
    Ring0, // kernel-space
    Ring3, // user-space
}


#[allow(dead_code)]
pub struct ToyMMU {
    page_table: [u8; 8],
    current_privilege: PrivilegeLevel,
}

/*
impl ToyMMU {
    pub fn new(page_table: [u8; 8], privilege: PrivilegeLevel) -> Self {
        Self {
            page_table,
            current_privilege: privilege,
        }
    }

    /// Метод перевода Виртуального адреса (VA) в Физический адрес (PA)
    ///
    /// # Аргументы
    /// * `va` - 6-битный виртуальный адрес
    /// * `is_write` - флаг, указывающий, является ли операция записью (true) или чтением (false)
    pub fn translate(&self, va: u8, is_write: bool) -> Result<u8, MemoryError> {
        // TODO: Реализовать логику трансляции и проверок флагов
        unimplemented!()
    }
}*/


impl ToyMMU {
    pub fn new(page_table: [u8; 8], privilege: PrivilegeLevel) -> Self {
        Self {
            page_table,
            current_privilege: privilege,
        }
    }

    /// Метод перевода Виртуального адреса (VA) в Физический адрес (PA)
    ///
    /// # Аргументы
    /// * `va` - 6-битный виртуальный адрес
    /// * `is_write` - флаг, указывающий, является ли операция записью (true) или чтением (false)
    pub fn translate(&self, va: u8, is_write: bool) -> Result<u8, MemoryError> {
        // Шаг 1. Проверить адрес
        if va > 63 {
            return Err(MemoryError::InvalidAddress);
        }

        // Шаг 2. Получить номер виртуальной страницы (старшие 3 бита)
        let vpn = va >> 3; // Сдвиг вправо на 3 бита

        // Шаг 3. Получить смещение внутри страницы (младшие 3 бита)
        let offset = va & 0b00000111; // Маска для младших 3 бит

        // Шаг 4. Найти запись в таблице страниц
        let pte = self.page_table[vpn as usize];

        // Шаг 5. Проверить флаг Present (бит 0)
        if pte & 0b00000001 == 0 {
            return Err(MemoryError::PageNotPresent);
        }

        // Шаг 6. Проверить права доступа
        // Флаг User (бит 2)
        let is_user_page = pte & 0b00000100 != 0;
        // Флаг Writable (бит 1)
        let is_writable = pte & 0b00000010 != 0;

        // Проверка привилегий: если CPL=Ring3, а страница не для пользователя (U=0)
        if self.current_privilege == PrivilegeLevel::Ring3 && !is_user_page {
            return Err(MemoryError::PrivilegeViolation);
        }

        // Проверка прав на запись: если операция записи, но страница только для чтения (W=0)
        if is_write && !is_writable {
            return Err(MemoryError::WriteProtected);
        }

        // Шаг 7. Получить номер физической страницы (биты 5–3)
        let phys_page_num = (pte >> 3) & 0b00000111; // Сдвиг на 3 бита и маска для 3 бит

        // Шаг 8. Собрать физический адрес: (физическая страница << 3) | смещение
        let pa = (phys_page_num << 3) | offset;

        Ok(pa)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // Индекс 0: Физическая стр 2, Флаги: U=1, W=1, P=1 (0b010_111 = 23) -> Доступно всем для всего
    // Индекс 1: Физическая стр 5, Флаги: U=0, W=1, P=1 (0b101_011 = 43) -> Только для Ядра (Ring0)
    // Индекс 2: Физическая стр 1, Флаги: U=1, W=0, P=1 (0b001_101 = 13) -> Только для чтения
    // Индекс 3: Физическая стр 0, Флаги: U=0, W=0, P=0 (0b000_000 = 0)  -> Страница отсутствует
    fn setup_test_table() -> [u8; 8] {
        [23, 43, 13, 0, 0, 0, 0, 0]
    }

    #[test]
    fn test_successful_translation() {
        let mmu = ToyMMU::new(setup_test_table(), PrivilegeLevel::Ring3);
        // VA = 3 (0b000_011): Стр 0, Смещение 3.
        // Физическая стр в таблице = 2 (0b010). Физический адрес должен быть: 0b010_011 = 19
        assert_eq!(mmu.translate(3, false), Ok(19));
    }

    #[test]
    fn test_invalid_address() {
        let mmu = ToyMMU::new(setup_test_table(), PrivilegeLevel::Ring3);
        assert_eq!(mmu.translate(64, false), Err(MemoryError::InvalidAddress));
        assert_eq!(mmu.translate(100, false), Err(MemoryError::InvalidAddress));
    }

    #[test]
    fn test_page_not_present() {
        let mmu = ToyMMU::new(setup_test_table(), PrivilegeLevel::Ring3);
        // VA = 24 (0b011_000): Стр 3, Смещение 0. У стр 3 флаг P = 0
        assert_eq!(mmu.translate(24, false), Err(MemoryError::PageNotPresent));
    }

    #[test]
    fn test_write_protection() {
        let mmu = ToyMMU::new(setup_test_table(), PrivilegeLevel::Ring3);
        // VA = 16 (0b010_000): Стр 2, Смещение 0. У стр 2 флаг W = 0 (Только чтение)
        assert!(mmu.translate(16, false).is_ok()); // Читать можно
        assert_eq!(mmu.translate(16, true), Err(MemoryError::WriteProtected)); // Писать нельзя
    }

    #[test]
    fn test_privilege_violation_in_ring3() {
        let mmu = ToyMMU::new(setup_test_table(), PrivilegeLevel::Ring3);
        // VA = 8 (0b001_000): Стр 1, Смещение 0. У стр 1 флаг U = 0 (Ядро)
        assert_eq!(
            mmu.translate(8, false),
            Err(MemoryError::PrivilegeViolation)
        );
    }

    #[test]
    fn test_kernel_can_access_everything() {
        let mmu = ToyMMU::new(setup_test_table(), PrivilegeLevel::Ring0);
        // В Ring0 ядро должно успешно читать страницу, защищенную флагом U=0
        // VA = 11 (0b001_011): Стр 1, Смещение 3.
        // Физическая стр в таблице = 5 (0b101). Физический адрес: 0b101_011 = 43
        assert_eq!(mmu.translate(11, false), Ok(43));
    }
}
