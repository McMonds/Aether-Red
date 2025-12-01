use aether_traits::ContentGenerator;
use async_trait::async_trait;
use anyhow::Result;
use rand::Rng;
use uuid::Uuid;

pub struct SpintaxGenerator;

impl SpintaxGenerator {
    /// Resolves {Hi|Hello|Hey} style spintax recursively
    fn resolve_spintax(&self, text: &str) -> String {
        // Simple non-recursive implementation for demo. 
        // Real impl would use a parser or regex loop.
        // For now, we manually handle one level of braces.
        let mut result = String::new();
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
                let mut options_str = String::new();
                while let Some(inner) = chars.next() {
                    if inner == '}' { break; }
                    options_str.push(inner);
                }
                let options: Vec<&str> = options_str.split('|').collect();
                if !options.is_empty() {
                    let idx = rand::thread_rng().gen_range(0..options.len());
                    result.push_str(options[idx]);
                }
            } else {
                result.push(c);
            }
        }
        result
    }

    /// Injects Zero-Width Spaces (U+200B) randomly
    fn inject_zwsp(&self, text: &str) -> String {
        let mut result = String::new();
        let mut rng = rand::thread_rng();
        
        for c in text.chars() {
            result.push(c);
            // 10% chance to inject ZWSP after a character
            if rng.gen_bool(0.1) {
                result.push('\u{200B}');
            }
        }
        result
    }
}

#[async_trait]
impl ContentGenerator for SpintaxGenerator {
    async fn generate_content(&self, context: &str) -> Result<(String, String)> {
        // 1. Resolve Spintax
        let raw_subject = "{Important|Urgent|Notice}: {Update|News} for you";
        let subject = self.resolve_spintax(raw_subject);
        
        // 2. Inject ZWSP (Invisible Layer)
        let obfuscated_subject = self.inject_zwsp(&subject);

        // 3. Resolve Body Spintax
        let raw_body = format!("{{Hi|Hello}} Friend,\n\n{}", context);
        let body = self.resolve_spintax(&raw_body);

        // 4. Inject HTML Comment (UUID Layer)
        let unique_id = Uuid::new_v4();
        let final_body = format!("{} \n\n<!-- UUID: {} -->", body, unique_id);

        Ok((obfuscated_subject, final_body))
    }
}
