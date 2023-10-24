use std::net::UdpSocket;
use std::thread::spawn;
use voip::VoIP;

#[rustfmt::skip]
fn main() {
    let s = spawn(|| {
        if let Ok(socket) = UdpSocket::bind(format!("localhost:1234")) {
            let v = VoIP::new();
            v.start_microphone();
            loop {
                let data = v.send();
                socket.send_to(&data, format!("localhost:12345")).unwrap();
            }
        }
    });
    
    let r = spawn(|| {
        if let Ok(socket) = UdpSocket::bind(format!("localhost:12345")) {
            let v = VoIP::new();
            v.start_stereo();
            loop {
                let mut buf = [0u8; 65507]; // MAX UDP packet but you only need 3840 for this task!
                let (n, _) = socket.recv_from(&mut buf).unwrap();
                v.recv(&buf[..n]);
            }
        }
    });

    s.join().unwrap();
    r.join().unwrap();
}
