pub struct MyVec<T> {
    data: [Option<T>; 32],
    len: usize,
}

impl<T> MyVec<T> {
    pub fn new() -> Self {
        // Инициализируем массив из 32 None.
        // Это безопасно: Option<T> не требует инициализации T.
        MyVec {
            data: std::array::from_fn(|_| None),
            len: 0,
        }
    }

    /// Добавляет элемент. Возвращает Err(value), если достигнут лимит (32).
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len >= 32 {
            return Err(value);
        }
        self.data[self.len] = Some(value);
        self.len += 1;
        Ok(())
    }

    /// Удаляет и возвращает последний элемент.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        self.data[self.len].take()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx >= self.len {
            return None;
        }
        // Гарантированно Some, потому что idx < len
        self.data[idx].as_ref()
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx >= self.len {
            return None;
        }
        self.data[idx].as_mut()
    }

    pub fn iter(&self) -> MyVecIter<'_, T> {
        MyVecIter {
            vec: self,
            pos: 0,
        }
    }

    pub fn clear(&mut self) {
        // Очищаем только занятые элементы, чтобы корректно вызвать drop у T.
        while let Some(Some(_)) = self.data.get_mut(self.len.wrapping_sub(1)) {
            // Более простой и безопасный способ:
            break;
        }
        // Правильный вариант: пробегаем по всем занятым слотам и забираем значения.
        for i in 0..self.len {
            let _ = self.data[i].take();
        }
        self.len = 0;
    }
}

pub struct MyVecIter<'a, T> {
    vec: &'a MyVec<T>,
    pos: usize,
}

impl<'a, T> Iterator for MyVecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.vec.len {
            return None;
        }
        let item = self.vec.data[self.pos].as_ref().unwrap(); // безопасно: pos < len
        self.pos += 1;
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn push_pop_basic() {
        let mut v: MyVec<i32> = MyVec::new();
        v.push(1).unwrap();
        v.push(2).unwrap();
        v.push(3).unwrap();
        assert_eq!(v.len(), 3);
        assert_eq!(v.pop(), Some(3));
        assert_eq!(v.pop(), Some(2));
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn iter_borrows() {
        let mut v: MyVec<String> = MyVec::new();
        v.push("a".into()).unwrap();
        v.push("b".into()).unwrap();
        let collected: Vec<&String> = v.iter().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0], "a");
        assert_eq!(collected[1], "b");
    }

    #[test]
    fn capacity_overflow() {
        let mut v: MyVec<i32> = MyVec::new();
        for i in 0..32 {
            v.push(i).unwrap();
        }
        assert!(v.push(99).is_err());
        assert_eq!(v.len(), 32);
    }

    #[test]
    fn drop_called() {
        // Добавляем #[derive(Debug)], чтобы удовлетворить требование unwrap()
        #[derive(Debug)]
        struct Dropped(Rc<Cell<i32>>);

        impl Drop for Dropped {
            fn drop(&mut self) {
                let count = self.0.get();
                self.0.set(count + 1);
            }
        }

        let counter = Rc::new(Cell::new(0));

        {
            let mut v: MyVec<Dropped> = MyVec::new();
            v.push(Dropped(counter.clone())).unwrap();
            v.push(Dropped(counter.clone())).unwrap();
            // При clear() должны быть вызваны 2 drop
            v.clear();
            assert_eq!(counter.get(), 2);
        }

        {
            let mut v: MyVec<Dropped> = MyVec::new();
            v.push(Dropped(counter.clone())).unwrap();
            v.push(Dropped(counter.clone())).unwrap();
            // При выходе из области видимости (drop самого MyVec) должны быть вызваны ещё 2 drop
        }

        assert_eq!(counter.get(), 4);
    }

    #[test]
    fn get_and_get_mut() {
        let mut v: MyVec<i32> = MyVec::new();
        v.push(10).unwrap();
        v.push(20).unwrap();

        assert_eq!(v.get(0), Some(&10));
        assert_eq!(v.get(2), None);

        if let Some(x) = v.get_mut(1) {
            *x = 99;
        }
        assert_eq!(v.get(1), Some(&99));
    }

    #[test]
    fn clear_and_reuse() {
        let mut v: MyVec<i32> = MyVec::new();
        for i in 0..10 {
            v.push(i).unwrap();
        }
        v.clear();
        assert_eq!(v.len(), 0);
        assert!(v.is_empty());

        // После clear можно снова заполнять
        v.push(100).unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v.pop(), Some(100));
    }
}
