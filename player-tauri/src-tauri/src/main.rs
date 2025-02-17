fn main() {
    // input
    //     .read_exact(&mut buf)
    //     .expect("Episode info was not received");
    // dbg!(buf);

    // let contentLength = u32::from_le_bytes(buf);
    // dbg!(contentLength);
    // let mut content = String::with_capacity(contentLength as usize);
    // input.read_line(&mut content);

    // dbg!(&content);

    //let output = [buf[0], buf[1], buf[2], 0, 1, 0];
    //io::stdout().write_all(&output).expect("msg");

    // for arg in args() {
    //     episode_info.push_str(&arg);
    //     episode_info.push(' ');
    // }
    //dbg!(&episode_info);

    aniplayer_lib::run();
}
pub fn get_info() {}
