use std::io;
mod editor;
mod terminal;

use editor::Editor;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).map(|s| s.as_str()).unwrap_or("untitled.txt");

    let mut editor = Editor::new(filename);
    let result = editor.run();

    editor.cleanup()?;
    
    result
}
