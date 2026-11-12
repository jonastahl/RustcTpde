use std::os::fd::FromRawFd;
use std::fs::File;
use std::io::Read;

extern "C" {
    fn pipe(pipedes: *mut i32) -> i32;
    fn dup(fildes: i32) -> i32;
    fn dup2(fildes: i32, fildes2: i32) -> i32;
    fn close(fildes: i32) -> i32;
}

extern "Rust" {
    fn print();
}

fn main() {
    unsafe {
        // 1. Sichere den originalen stdout File-Deskriptor (FD 1)
        let original_stdout = dup(1);

        // 2. Erstelle eine Pipe [read_fd, write_fd]
        let mut pipe_fd = [0; 2];
        assert_eq!(pipe(pipe_fd.as_mut_ptr()), 0);

        // 3. Biege stdout (FD 1) auf das Schreib-Ende der Pipe um
        dup2(pipe_fd[1], 1);

        // 4. Führe die Funktion aus (schreibt nun in die Pipe)
        print();

        // 5. Stelle den originalen stdout wieder her & schließe das Schreib-Ende
        dup2(original_stdout, 1);
        close(pipe_fd[1]);
        close(original_stdout);

        // 6. Lese das Lese-Ende der Pipe aus
        let mut pipe_read = File::from_raw_fd(pipe_fd[0]);
        let mut output = String::new();
        pipe_read.read_to_string(&mut output).unwrap();

        assert_eq!(output, "Hello, world!\n");
    }
}