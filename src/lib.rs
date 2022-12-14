extern crate audrey;
extern crate dasp_interpolate;
extern crate dasp_signal;
extern crate deepspeech;

use std::fs::File;
use std::io;
use std::path::Path;
use std::process::Command;
use std::{env::args, error::Error};

use audrey::read::Reader;
use dasp_interpolate::linear::Linear;
use dasp_signal::{from_iter, interpolate::Converter, Signal};
use deepspeech::Model;

use ears;
use tts_rust::{languages::Languages, GTTSClient};

// The model has been trained on this specific
// sample rate.
const SAMPLE_RATE: u32 = 16_000;

pub struct Rbot {
    pub name: String,
    tts: Box<dyn TTS>,
    stt: Box<dyn STT>,
    response: Box<dyn Fn(&Rbot, &str) -> Result<(), UnderstandingError>>,
}

impl Rbot {
    pub fn default() -> Self {
        Self {
            name: String::from("ash"),
            tts: Box::new(GTTSClient {
                volume: 1.0,
                language: Languages::English,
            }),
            stt: Box::new(STTDeepSpeech),
            response: Box::new(Rbot::default_respond),
        }
    }

    pub fn new(
        name: String,
        tts: Box<dyn TTS>,
        stt: Box<dyn STT>,
        response: Box<dyn Fn(&Rbot, &str) -> Result<(), UnderstandingError>>,
    ) -> Self {
        Self {
            name,
            tts,
            stt,
            response,
        }
    }
}

pub struct UnderstandingError;

impl std::fmt::Display for UnderstandingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Rbot couldn't understand, please try again.")
    }
}

impl std::fmt::Debug for UnderstandingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{ file: {}, line: {}  }}", file!(), line!())
    }
}

pub trait TTS {
    fn say(&self, msg: &str);
}

pub trait STT {
    fn hear(&self) -> String;
}

impl Rbot {
    fn default_respond(&self, msg: &str) -> Result<(), UnderstandingError> {
        let words: Vec<_> = msg
            .trim()
            .split(' ')
            .filter(|x| !x.is_empty() && x.is_ascii())
            .map(|x| x.to_lowercase())
            .collect();

        if words.len() == 1 && words[0].contains(&self.name) {
            self.say("Yes?");
            return Ok(());
        }

        for (i, word) in words.iter().enumerate() {
            let has = |checked_word| word.contains(checked_word);
            if has("cancel") || has("stop") {
                self.say("Ok, no worries");
                return Ok(());
            }
            if has("leave") || has("by") || has("bye") || has("buy") || has("quit") {
                self.say("see ya");
                panic!("");
            }

            if has("hi") || has("hello") || has("hey") {
                self.say("Hi");
            }

            if has("note") {
                self.say("Opening your notes");
                self.cmd("obsidian");
            }

            if has("command")
                || has("comand")
                || has("shell")
                || has("fish")
                || has("bash")
                || has("terminal")
                || has("shel")
                || has("shal")
            {
                self.cmd("konsole --workdir ~");
            }

            if has("search") || has("serch") || has("sirche") || has("sirch") {
                let request: String = (&words[i + 1..].join(" ")).to_owned();
                let mut query = "https://search.brave.com/search?q=".to_string();
                query.push_str(&request);
                let cmd = format!("firefox \"{}\"", query);
                println!("{}", cmd);
                self.cmd(&cmd);
            }
        }
        Ok(())
    }

    pub fn cmd(&self, cmd: &str) {
        let bash_cmd = Command::new("bash").arg("-c").arg(cmd).output();
        match bash_cmd {
            Ok(_) => {}
            Err(_) => {
                println!("Failed to output command.");
            }
        }
    }
    pub fn say(&self, msg: &str) {
        self.tts.say(msg);
    }

    pub fn hear(&self) -> String {
        return self.stt.hear();
    }

    pub fn respond(&self, msg: &str) -> Result<(), UnderstandingError> {
        (self.response)(self, msg)
    }
}

impl TTS for GTTSClient {
    fn say(&self, msg: &str) {
        self.display_and_speak(msg);
    }
}

pub struct STTDeepSpeech;
impl STT for STTDeepSpeech {
    fn hear(&self) -> String {
        let context = ears::init_in().unwrap();
        let mut recorder = ears::Recorder::new(context);
        recorder.start();
        std::io::stdin().read_line(&mut String::new());
        recorder.stop();
        recorder.save_to_file("mic_input");
        let audio_file_path = "mic_input.wav";
        let model_dir_str = "speech_model";
        let dir_path = Path::new(model_dir_str);
        let mut graph_name: Box<Path> = dir_path.join("output_graph.pb").into_boxed_path();
        let mut scorer_name: Option<Box<Path>> = None;
        // search for model in model directory
        for file in dir_path
            .read_dir()
            .expect("Specified model dir is not a dir")
        {
            if let Ok(f) = file {
                let file_path = f.path();
                if file_path.is_file() {
                    if let Some(ext) = file_path.extension() {
                        if ext == "pb" || ext == "pbmm" || ext == "tflite" {
                            graph_name = file_path.into_boxed_path();
                        } else if ext == "scorer" {
                            scorer_name = Some(file_path.into_boxed_path());
                        }
                    }
                }
            }
        }
        let mut m = Model::load_from_files(&graph_name).unwrap();
        // enable external scorer if found in the model folder
        if let Some(scorer) = scorer_name {
            m.enable_external_scorer(&scorer).unwrap();
        }

        let audio_file = File::open(audio_file_path).unwrap();
        let mut reader = Reader::new(audio_file).unwrap();
        let desc = reader.description();
        assert_eq!(
            1,
            desc.channel_count(),
            "The channel count is required to be one, at least for now"
        );

        // Obtain the buffer of samples
        let audio_buf: Vec<_> = if desc.sample_rate() == SAMPLE_RATE {
            reader.samples().map(|s| s.unwrap()).collect()
        } else {
            // We need to interpolate to the target sample rate
            let interpolator = Linear::new([0i16], [0]);
            let conv = Converter::from_hz_to_hz(
                from_iter(reader.samples::<i16>().map(|s| [s.unwrap()])),
                interpolator,
                desc.sample_rate() as f64,
                SAMPLE_RATE as f64,
            );
            conv.until_exhausted().map(|v| v[0]).collect()
        };

        // Run the speech to text algorithm
        let result = m.speech_to_text(&audio_buf).unwrap();

        result
    }
}
