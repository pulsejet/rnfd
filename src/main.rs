mod socket;
mod tlv;

fn main() {
    socket::listen_unix("/tmp/rnfd.sock");
}
