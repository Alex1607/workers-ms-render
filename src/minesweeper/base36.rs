const BASE: i64 = 36;
const CHARACTERS: &str = "0123456789abcdefghijklmnopqrstuvwxyz";

pub fn decode(number: &str) -> i64 {
    let mut result: i64 = 0;
    let length = number.len();
    let chars: Vec<char> = CHARACTERS.chars().collect();

    for i in 0..length {
        let digit = chars
            .iter()
            .position(|&c| {
                c == number
                    .chars()
                    .nth(length - i - 1)
                    .expect("Error while decoding base36")
            })
            .expect("Error while decoding base36") as i64;
        result += BASE.pow(i as u32) * digit;
    }

    result
}
