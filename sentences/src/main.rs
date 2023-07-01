//use dotenvy::dotenv;
use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
    set_key,
};
use std::{
    env,
    io::{stdin, stdout, Write},
};

#[tokio::main]
async fn main() {
    // Make sure you have a file named `.env` with the `OPENAI_KEY` environment variable defined!
    //dotenv().unwrap();
    set_key(env::var("OPENAI_KEY").unwrap());

    let mut messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: r#"You give example sentences in traditional chinese for a given word. You try to make simple sentences using simple words. You use real-life sentences. Try to use words that are a at most the same TOCL level

Use the following format

Chinese Sentence
Chinese Pinyin
English translation (meaning of the character here)

Chinese Sentence
Chinese Pinyin
English translation(meaning of the character here)

Chinese Sentence
Chinese Pinyin
English translation(meaning of the character here)"#.to_string(),
        name: None,
    }];

    loop {
        print!("User: ");
        stdout().flush().unwrap();

        let mut user_message_content = String::new();

        stdin().read_line(&mut user_message_content).unwrap();
        messages.push(ChatCompletionMessage {
            role: ChatCompletionMessageRole::User,
            content: user_message_content,
            name: None,
        });

        let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", messages.clone())
            .create()
            .await
            .unwrap()
            .unwrap();
        let returned_message = chat_completion.choices.first().unwrap().message.clone();

        println!(
            "{:#?}: {}",
            &returned_message.role,
            &returned_message.content.trim()
        );

        messages.push(returned_message);
    }
}

struct Response {
    chinese: String,
    pinyin: String,
    english: String,
}
