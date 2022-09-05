use rbot;

fn main() {
    let rbot = rbot::Rbot::default();
    rbot.say(&format!("{} here", rbot.name));
    loop {
        let msg = rbot.hear();
        println!("{}", msg);
        let res = rbot.respond(&msg);
        match res {
            Ok(_) => {}
            Err(_) => rbot.say("Pardon?"),
        }
    }
}
