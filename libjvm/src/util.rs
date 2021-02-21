use nix::errno::Errno::EINTR;

pub fn retry_on_eintr(to_retry: impl Fn() -> i32) -> i32 {
    loop {
        let err = to_retry();
        if nix::errno::errno() != EINTR || err != -1 {
            return err;
        }
    }
}
