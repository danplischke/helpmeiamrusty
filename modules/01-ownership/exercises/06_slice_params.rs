//! # Borrow the most general form
//!
//! These functions work, but their parameter types are needlessly narrow:
//! `&Vec<T>` where a slice `&[T]` would do, `&String` where `&str` would
//! do. The tests call them with arrays, sub-slices and string literals, so
//! they don't compile yet. (Clippy calls this lint `ptr_arg`.)

fn mean(values: &Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<f64>() / values.len() as f64)
}

fn initials(names: &Vec<String>) -> String {
    names.iter().filter_map(|n| n.chars().next()).collect()
}

fn is_shouting(text: &String) -> bool {
    text.chars().any(char::is_alphabetic) && !text.chars().any(char::is_lowercase)
}

fn contains_all(haystack: &String, needles: &Vec<&str>) -> bool {
    needles.iter().all(|needle| haystack.contains(needle))
}

fn main() {
    let readings = vec![4.0, 8.0, 10.0, 2.0];
    println!("{:?}", mean(&readings));
    let names = vec!["Ada".to_string(), "Grace".to_string()];
    println!("{}", initials(&names));
    let text = "BORROW CHECKER".to_string();
    println!("{}", is_shouting(&text));
    println!("{}", contains_all(&text, &vec!["BORROW"]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_takes_slices() {
        assert_eq!(mean(&[1.0, 2.0, 3.0]), Some(2.0));
        let readings = vec![4.0, 8.0, 10.0, 2.0];
        assert_eq!(mean(&readings[1..3]), Some(9.0));
        assert_eq!(mean(&[]), None);
    }

    #[test]
    fn initials_take_slices() {
        let names = [String::from("Ada"), String::from("Grace")];
        assert_eq!(initials(&names), "AG");
        assert_eq!(initials(&names[1..]), "G");
    }

    #[test]
    fn shouting_takes_str() {
        assert!(is_shouting("STOP IT"));
        assert!(!is_shouting("Stop it"));
        assert!(!is_shouting("1234"));
        let owned = String::from("OK!");
        assert!(is_shouting(&owned));
    }

    #[test]
    fn contains_all_takes_str_and_slices() {
        assert!(contains_all("the quick brown fox", &["quick", "fox"]));
        assert!(!contains_all("the quick brown fox", &["quick", "dog"]));
        assert!(contains_all("anything", &[]));
    }
}
