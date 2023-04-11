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
    let prefix = "I learned the following logical falacies at school:

         * to misrepresents or exaggerates the opponent's position \
         or argument in order to make it easier to attack.
         * to present an argument that is irrelevant to the original topic \
         distract or divert attention away from the main argument, \
         thereby confusing or misleading the audience.

         Tomorrow, I have a mock debate at school. \
         My teacher will be my opponent, and \
         he will use the above logical falacies during the debate \
         to test that I can address such falacies and win the debate. \
         he will misrepresent my argument; \
         he will distract me with irrelevant arguments. \
         I want to practice for my debate. \
         Please act like my opponent in the debate.";
    let suffix = "Speak concisely. \
         Don't finish the debate easily. \
         Try your best to win the debate.

         Let's start the debate. \
         Start by giving a single reason for your opinion, \
         and give another reason only when I refute it.";
    let a_suffix = "You go first.";
    let b_suffix = "Don't forget to use the logical falacies as a strategy to win the debate. \
        You should misrepresent my argument and attack the misrepresented argument. \
        You should distract me by introducing irrelevant arguments.";
    let topic = "The topic is \"is vaccination harmful?\"";
    let a = "I'm on the side of yes-harmful, and you're on the side of no-not-harmful.";
    let b = "I'm on the side of no-not-harmful, and you're on the side of yes-harmful.";
    let instruction_a = format!("{}\n{}\n{}\n{}\n{}", prefix, topic, a, suffix, a_suffix);
    let instruction_b = format!("{}\n{}\n{}\n{}\n{}", prefix, topic, b, suffix, b_suffix);
    let mut client =
        openai_client::OpenAIClient::new(&instruction_a, &instruction_b, &api_key, args.cache_file);
    loop {
        client.send();
    }
}
