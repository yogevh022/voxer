pub fn print_bar(progress: f32, bar_width: usize) {
    let filled = (progress * bar_width as f32) as usize;
    let empty = bar_width - filled;
    print!(
        "\r[{}{}] {:>3}%",
        "=".repeat(filled),
        " ".repeat(empty),
        (progress * 100.0) as usize,
    );
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
}
