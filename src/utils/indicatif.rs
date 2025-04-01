use indicatif::{ProgressBar, ProgressStyle};

pub fn pb(value: u64, message: &str, colors: &str) -> ProgressBar {
    let pb = ProgressBar::new(value as u64);
    let template = format!(
        "{{spinner:.green}} {{msg}} [{{elapsed_precise}}] [{{bar:40.{colors}}}] {{pos:>7}}/{{len:7}}",
        colors = colors
    );
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&template)
            .unwrap()
            .progress_chars("#>-")
    );
    pb.set_message(message.to_string());
    pb
}