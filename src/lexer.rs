
pub struct Lexer {
    input: Vec<u8>, // &'a str
    position: usize,
    read_position: usize,
    ch: u8,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        let l = Lexer{input: input.as_bytes().to_vec(), position: 0, read_position: 0, ch: 0};
        return l;
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = 0;
        } else {
            self.ch = self.input[self.position];
        }

        self.position = self.read_position;
        self.read_position += 1;

    }
}