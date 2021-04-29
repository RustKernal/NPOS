pub trait Driver {
    fn init();
    fn read_u8 (&mut self, port:usize) -> Result<u8,  u8>;
    fn read_u16(&mut self, port:usize) -> Result<u16, u8>;
    fn read_u32(&mut self, port:usize) -> Result<u32, u8>;
    fn read_u64(&mut self, port:usize) -> Result<u64, u8>;

    fn write_u8 (&mut self, port:usize, value:u8);
    fn write_u16(&mut self, port:usize, value:u8);
    fn write_u32(&mut self, port:usize, value:u8);
    fn write_u64(&mut self, port:usize, value:u8);

    fn get_name(&self) -> &str;

    fn close();
}