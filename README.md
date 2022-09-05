# RBOT

rbot is a light framework for creating AI assistants in rust. It provides meager default functionality using Google Text-to-Speech and Mozilla's DeepSpeech.

Default functionality of the bot is very light, only recognizing its name, quitting, searching things on the web etc. This is more of a framework/library for others to set up their own bots.

The DeepSpeech functionality is very much lacking as is. If you want to improve it, download the latest model scorer from that repo and place it into the speech_models directory, this will make the bot a bit more language-correct. Some examples are included to get you started. I'm new to rust so any feeback or improvements I could make are welcome.
