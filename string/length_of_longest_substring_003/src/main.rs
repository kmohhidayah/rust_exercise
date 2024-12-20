struct Solution;

impl Solution {
    pub fn length_of_longest_substring(s: String) -> i32 {
        use std::collections::HashSet;
        let chars: Vec<char> = s.chars().collect();
        let mut set = HashSet::new();
        let mut max_len = 0;
        let mut i = 0;
        let mut j = 0;

        while j < chars.len() {
            if !set.contains(&chars[j]) {
                set.insert(chars[j]);
                j += 1;
                max_len = max_len.max(set.len());
            } else {
                set.remove(&chars[i]);
                i += 1;
            }
        }

        max_len as i32
    }
}

fn main() {
    // Test cases
    let test_cases = vec![
        "abcabcbb".to_string(),
        "bbbbb".to_string(),
        "pwwkew".to_string(),
        "".to_string(),
    ];

    for test in test_cases {
        println!(
            "Input: \"{}\", Output: {}",
            test,
            Solution::length_of_longest_substring(test.clone())
        );
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            Solution::length_of_longest_substring("abcabcbb".to_string()),
            3
        );
        assert_eq!(
            Solution::length_of_longest_substring("bbbbb".to_string()),
            1
        );
        assert_eq!(
            Solution::length_of_longest_substring("pwwkew".to_string()),
            3
        );
        assert_eq!(Solution::length_of_longest_substring("".to_string()), 0);
    }
}
