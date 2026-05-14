use std::env;

/// Сортирует вектор строк в лексикографическом порядке с помощью insertion sort
fn insertion_sort(strings: &mut Vec<String>) {
    for i in 1..strings.len() {
        let current = strings[i].clone();
        let mut j = i;

        // Сдвигаем элементы, которые больше текущего, вправо
        while j > 0 && strings[j - 1] > current {
            strings[j] = strings[j - 1].clone();
            j -= 1;
        }

        // Вставляем текущий элемент на правильную позицию
        strings[j] = current;
    }
}

fn main() {
    // Получаем аргументы командной строки и пропускаем первый (имя программы)
    let mut args: Vec<String> = env::args().skip(1).collect();
    
    // Сортируем вручную с помощью нашей функции
    insertion_sort(&mut args);
    
    // Выводим каждый аргумент на отдельной строке
    for arg in args {
        println!("{}", arg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_vector() {
        let mut empty: Vec<String> = vec![];
        insertion_sort(&mut empty);
        assert_eq!(empty, Vec::<String>::new());
    }

    #[test]
    fn test_single_element() {
        let mut single = vec!["hello".to_string()];
        insertion_sort(&mut single);
        assert_eq!(single, vec!["hello".to_string()]);
    }

    #[test]
    fn test_already_sorted() {
        let mut sorted = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string()
        ];
        let expected = sorted.clone();
        insertion_sort(&mut sorted);
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_reverse_sorted() {
        let mut reverse = vec![
            "c".to_string(),
            "b".to_string(),
            "a".to_string()
        ];
        let expected = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string()
        ];
        insertion_sort(&mut reverse);
        assert_eq!(reverse, expected);
    }

    #[test]
    fn test_mixed_case() {
        let mut mixed = vec![
            "Zebra".to_string(),
            "apple".to_string(),
            "Banana".to_string()
        ];
        let expected = vec![
            "Banana".to_string(),
            "Zebra".to_string(),
            "apple".to_string()
        ];
        insertion_sort(&mut mixed);
        assert_eq!(mixed, expected);
    }

    #[test]
    fn test_with_punctuation() {
        let mut with_punct = vec![
            "hello,".to_string(),
            "world!".to_string(),
            "test".to_string()
        ];
        let expected = vec![
            "hello,".to_string(),
            "test".to_string(),
            "world!".to_string()
        ];
        insertion_sort(&mut with_punct);
        assert_eq!(with_punct, expected);
    }

    #[test]
    fn test_duplicate_elements() {
        let mut duplicates = vec![
            "a".to_string(),
            "c".to_string(),
            "a".to_string(),
            "b".to_string()
        ];
        let expected = vec![
            "a".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string()
        ];
        insertion_sort(&mut duplicates);
        assert_eq!(duplicates, expected);
    }

    #[test]
    fn test_long_strings() {
        let mut long_strings = vec![
            "this is a long string".to_string(),
            "short".to_string(),
            "another long string here".to_string()
        ];
        let expected = vec![
            "another long string here".to_string(),
            "short".to_string(),
            "this is a long string".to_string()
        ];
        insertion_sort(&mut long_strings);
        assert_eq!(long_strings, expected);
    }
}
