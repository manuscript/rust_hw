use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

pub const BUFFER_SIZE: usize = 64 * 1024;

// -----------------------------------------------------------------------------
// MyBufReader
// -----------------------------------------------------------------------------

#[allow(dead_code)]
pub struct MyBufReader {
    file: File,
    buffer: Vec<u8>,
    pos: usize,  // текущая позиция в буфере
    capacity: usize,  // сколько данных фактически в буфере
}

impl MyBufReader {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::open(path)?;
        let buffer = vec![0; BUFFER_SIZE];

        Ok(MyBufReader {
            file,
            buffer,
            pos: 0,
            capacity: 0,
        })
    }

    pub fn read_byte(&mut self) -> io::Result<Option<u8>> {
        // Если мы в конце буфера, читаем новый блок
        if self.pos >= self.capacity {
            self.pos = 0;
            self.capacity = self.file.read(&mut self.buffer)?;

            // Если ничего не прочитали — файл закончился
            if self.capacity == 0 {
                return Ok(None);
            }
        }

        // Берём байт из буфера и сдвигаем позицию
        let byte = self.buffer[self.pos];
        self.pos += 1;
        Ok(Some(byte))
    }
}


// -----------------------------------------------------------------------------
// MyBufWriter
// -----------------------------------------------------------------------------

pub struct MyBufWriter {
    file: File,
    buffer: Vec<u8>,
    pos: usize,
}

impl MyBufWriter {
    pub fn create(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::create(path)?;
        let buffer = vec![0; BUFFER_SIZE];

        Ok(MyBufWriter {
            file,
            buffer,
            pos: 0,
        })
    }

    pub fn write_buffered(&mut self, data: &[u8]) -> io::Result<()> {
        for &byte in data {
            // Если буфер заполнен — записываем его в файл
            if self.pos >= BUFFER_SIZE {
                self.flush()?;
            }
            // Добавляем байт в буфер
            self.buffer[self.pos] = byte;
            self.pos += 1;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if self.pos > 0 {
            // Записываем всё, что есть в буфере
            self.file.write_all(&self.buffer[..self.pos])?;
            self.pos = 0;  // сбрасываем позицию
        }
        self.file.flush()
    }

    pub fn close(mut self) -> io::Result<()> {
        self.flush()
    }
}

impl Drop for MyBufWriter {
    fn drop(&mut self) {
        // Ошибку из Drop вернуть нельзя.
        // Поэтому в реальном коде лучше явно вызывать close() или flush().
        let _ = self.flush();
    }
}

// -----------------------------------------------------------------------------
// Медленная версия
// -----------------------------------------------------------------------------

pub fn copy_slow(input: impl AsRef<Path>, output: impl AsRef<Path>) -> io::Result<u64> {
    let mut input = File::open(input)?;
    let mut output = File::create(output)?;

    let mut copied = 0;
    let mut byte = [0u8; 1];

    loop {
        let n = input.read(&mut byte)?;
        if n == 0 {
            break;
        }

        output.write_all(&byte[..n])?;
        copied += n as u64;
    }

    output.flush()?;

    Ok(copied)
}

// -----------------------------------------------------------------------------
// Быстрая версия
// -----------------------------------------------------------------------------
// copy_fast специально тоже использует побайтный API.
// Разница должна быть не в коде копирования, а в реализации MyBufReader и MyBufWriter
// эту функцию не нужно менять, она должна работать с любыми реализациями MyBufReader и MyBufWriter,
// которые вы сделаете
pub fn copy_fast(input: impl AsRef<Path>, output: impl AsRef<Path>) -> io::Result<u64> {
    let mut reader = MyBufReader::open(input)?;
    let mut writer = MyBufWriter::create(output)?;

    let mut copied = 0;

    while let Some(byte) = reader.read_byte()? {
        writer.write_buffered(&[byte])?;
        copied += 1;
    }

    writer.close()?;

    Ok(copied)
}

pub const RECORD_SIZE: usize = 10;

pub fn make_record(index: usize) -> [u8; RECORD_SIZE] {
    let mut record = [0u8; RECORD_SIZE];

    (0..RECORD_SIZE).for_each(|i| {
        record[i] = ((index + i) % 251) as u8;
    });

    record
}

pub fn generate_input_file(path: impl AsRef<Path>, records: usize) -> io::Result<()> {
    let mut file = File::create(path)?;

    for i in 0..records {
        let record = make_record(i);
        file.write_all(&record)?;
    }

    file.flush()?;

    Ok(())
}
