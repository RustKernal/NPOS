pub trait Printer {
    fn print_str(&mut self, text:&str);
    fn carriage_return(&mut self);
    fn new_line(&mut self);
    fn tab(&mut self);
}