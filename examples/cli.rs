use rbot;

struct TTSCLI;
impl rbot::TTS for TTSCLI {
    fn say(&self, msg: &str) {
        println!("{}", msg);
    }
}

struct STTCLI;
impl rbot::STT for STTCLI {
    fn hear(&self) -> String {
        let mut string = String::new();
        std::io::stdin()
            .read_line(&mut string)
            .expect("Could not read standard input");
        string
    }
}

fn main() {
    // create rbot
    let ash = rbot::Rbot::new(
        String::from("ash"),
        Box::new(TTSCLI),
        Box::new(STTCLI),
        Box::new(
            |ash: &rbot::Rbot, msg: &str| -> Result<(), rbot::UnderstandingError> {
                ash.say("I don't care");
                Ok(())
            },
        ),
    );

    // run rbot
    ash.say("What do you want?");
    loop{
        let msg = ash.hear();
        match ash.respond(&msg){
            Err(_) => {println!("Huh?")},
            _ => {}
        }
    }
}
