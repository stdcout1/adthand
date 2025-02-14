use std::fmt::Display;

use utils::Answer;

pub struct Waybar {
    text: String,
    tooltip: String,
    alt: String,
}

// maybe just import serde LOL
impl Display for Waybar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\"text\": \"{}\",\"tooltip\": \"{}\", \"alt\": \"{}\"}}",
            self.text, self.tooltip, self.alt
        )
    }
}

impl Waybar {
    pub fn new(answer: Answer) -> Option<Self> {
        if let Answer::Waybar(name, time, relative_time, all) = answer {
            let mut builder = String::new();
            for (name, time) in all {
                builder.push_str(name);
                builder.push_str(": ");
                builder.push_str(&time);
                builder.push_str(" \\n");
            }

            let result = Waybar {
                text: format!("{} {}", name, relative_time),
                alt: format!("{} at {}", name, time),
                tooltip: builder,
            };
            Some(result)
        } else {
            None
        }
    }
}
