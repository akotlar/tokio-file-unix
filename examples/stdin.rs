extern crate futures;
extern crate tokio_core;
extern crate tokio_file_unix;

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    // convert stdin into a nonblocking file;
    // this is the only part that makes use of tokio_file_unix
    let stdin = std::io::stdin();
    let file = tokio_file_unix::StdFile(stdin.lock());
    let file = tokio_file_unix::File::new_nb(file).unwrap();
    let file = file.into_reader(&handle).unwrap();

    println!("Type something and hit enter!");
    use futures::{Future, future};
    core.run(future::loop_fn((file, Vec::new()), |(file, line)| {
        // read each line
        tokio_core::io::read_until(file, b'\n', line).map(|(file, mut line)| {

            // demonstrate that the event loop isn't blocked by I/O!
            let one_sec = std::time::Duration::new(1, 0);
            handle.spawn(
                tokio_core::reactor::Timeout::new(one_sec, &handle).unwrap()
                .map_err(|_| ())
                .map(|()| println!(" ... timeout works!"))
            );

            if line.ends_with(b"\n") {
                println!("Got: {:?}", std::str::from_utf8(&line));
                line.clear();
                future::Loop::Continue((file, line))
            } else {                    // EOF
                future::Loop::Break(())
            }
        })
    })).unwrap();
}