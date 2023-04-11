use clap::Parser;
use gptebate::*;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    api_key_file: Option<String>,
    #[arg(short, long)]
    cache_file: Option<String>,
}

fn main() {
    let args = Args::parse();
    let api_key = args.api_key_file.unwrap_or(".openai_api_key".to_string());
    let prefix = "I want to practice for my school debate contest. \
         Act like my opponent in the debate.";
    let suffix = "You should persuade me and refute my opinion. \
         Don't give all the reasons for your opinion at once. \
         Give a single reason at a time, \
         and give another reason only when I refute it. \
         Also, speak concisely. \
         Don't finish the debate easily. \
         Try your best to win the debate.

         As this is practice, I want you to try to make me embarrassed, \
         so that I can prepare for various situations in the real debate. \
         Use not only logical evidence but also the following \"bad\" debate skills:

         * Argumentum ad hominem (argument against the person): \
         attacking the person making an argument instead of addressing the argument itself.
         * Straw man: to misrepresents or exaggerates the opponent's position \
         or argument in order to make it easier to attack.
         * Red herring: an argument is presented that is irrelevant to the original topic. \
         It's used to distract or divert attention away from the main argument \
         by introducing an irrelevant argument, thereby confusing or misleading the audience.
         * Tu quoque (you too): to deflect criticism or accusations by pointing out \
         the hypocrisy or inconsistency of the accuser, rather than addressing the \
         substance of the criticism itself.
         * Hasty generalization: a conclusion is drawn based on insufficient or \
         unrepresentative evidence. It involves making a sweeping generalization \
         or assumption about a group, based on a limited sample or a small amount of evidence.
         * Inappropriate authority: an argument is presented with an appeal to \
         an authority who lacks expertise or credibility in the relevant field or subject matter.

         Now, let's start the debate.";
    let a_suffix = "You go first.";
    let topic = "The topic is \"is Earth flat or round?\"";
    let a = "I'm on the side of flat-Earth, and you're on the side of round-Earth.";
    let b = "I'm on the side of round-Earth, and you're on the side of flat-Earth.";
    let instruction_a = format!("{}\n{}\n{}\n{}\n{}", prefix, topic, a, suffix, a_suffix);
    let instruction_b = format!("{}\n{}\n{}\n{}", prefix, topic, b, suffix);
    let mut client =
        openai_client::OpenAIClient::new(&instruction_a, &instruction_b, &api_key, args.cache_file);
    loop {
        client.send();
    }
}
