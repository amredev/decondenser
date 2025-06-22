// /// Unescape the string by replacing the escape sequences with their actual characters.
// #[must_use = "this is a pure function; ignoring its result is definitely a bug"]
// pub fn unescape(params: UnescapeParams<'_>) -> UnescapeOutput {
//     let UnescapeParams { input, config } = params;
//     let config = config.unwrap_or_default();

//     let mut output = String::with_capacity(input.len());

//     let mut cursor = input.chars();

//     while let Some(char) = cursor.next() {
//         if char != config.escape_char {
//             output.push(char);
//             continue;
//         }

//         let Some(next) = cursor.next() else {
//             // If the escape character is at the end of the string, we can't
//             // unescape it. Just leave it as is.
//             output.push(config.escape_char);
//             break;
//         };

//         let replacement = config.mapping.iter().find(|&&(src, _)| src == char);

//         if let Some(&(_, dest)) = replacement {
//             output.push(dest);
//         } else {
//             // If the character is not in the mapping, keep it as is
//             output.extend([config.escape_char, char]);
//         }
//     }

//     UnescapeOutput { output }
// }
