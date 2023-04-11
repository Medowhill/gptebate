use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    fs::{self, File},
};

use async_openai::{types::*, Client};
use tokio::runtime::{Builder, Runtime};

type CacheKey = Vec<(String, String)>;

struct Cache {
    cache_file: Option<String>,
    map: RefCell<BTreeMap<CacheKey, String>>,
    updated: Cell<bool>,
}

impl Cache {
    fn new(cache_file: Option<String>) -> Self {
        let v: Vec<(CacheKey, String)> = if let Some(cache_file) = &cache_file {
            if let Ok(cache_file) = File::open(cache_file) {
                serde_json::from_reader(cache_file).unwrap()
            } else {
                vec![]
            }
        } else {
            vec![]
        };
        let map = v.into_iter().collect();
        Self {
            cache_file,
            map: RefCell::new(map),
            updated: Cell::new(false),
        }
    }

    fn get(&self, key: &CacheKey) -> Option<String> {
        self.map.borrow().get(key).cloned()
    }

    fn insert(&self, key: CacheKey, value: String) {
        if self.cache_file.is_some() {
            self.updated.set(true);
            self.map.borrow_mut().insert(key, value);
        }
    }

    fn save(&self) {
        if let Some(cache_file) = &self.cache_file {
            if self.updated.get() {
                let mut file = File::create(cache_file).unwrap();
                let v: Vec<_> = self
                    .map
                    .borrow()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                serde_json::to_writer(&mut file, &v).unwrap();
                self.updated.set(false);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Person {
    A,
    B,
}

#[derive(Debug)]
struct Message {
    person: Person,
    text: String,
}

pub struct OpenAIClient<'a> {
    inner: Client,
    runtime: Runtime,
    cache: Cache,

    instruction_a: &'a str,
    instruction_b: &'a str,
    dialogue: Vec<Message>,
}

const MODEL: &str = "gpt-3.5-turbo";

impl<'a> OpenAIClient<'a> {
    pub fn new(
        instruction_a: &'a str,
        instruction_b: &'a str,
        api_key_file: &str,
        cache_file: Option<String>,
    ) -> Self {
        let api_key = fs::read_to_string(api_key_file).unwrap().trim().to_string();
        let inner = Client::new().with_api_key(api_key);
        let runtime = Builder::new_current_thread().enable_all().build().unwrap();
        let cache = Cache::new(cache_file);
        Self {
            inner,
            runtime,
            cache,

            instruction_a,
            instruction_b,
            dialogue: vec![],
        }
    }

    pub fn send(&mut self) {
        let person = if self
            .dialogue
            .last()
            .map(|m| m.person == Person::B)
            .unwrap_or(true)
        {
            Person::A
        } else {
            Person::B
        };
        let text = self.send_to(&self.dialogue, person);
        let message = Message { person, text };
        println!("{:?}:\n{}\n\n", message.person, message.text);
        self.dialogue.push(message);
    }

    fn send_to(&self, msgs: &[Message], person: Person) -> String {
        let m1 = system("You are a helpful assitant.");
        let m2 = user(if person == Person::A {
            self.instruction_a
        } else {
            self.instruction_b
        });
        let mut v = vec![m1, m2];
        for m in msgs {
            let m = if m.person != person {
                user(&m.text)
            } else {
                assistant(&m.text)
            };
            v.push(m);
        }
        self.send_request(v)
    }

    fn send_request(&self, msgs: Vec<ChatCompletionRequestMessage>) -> String {
        let key = msgs
            .iter()
            .map(|ChatCompletionRequestMessage { role, content, .. }| {
                (role_to_str(role).to_string(), content.clone())
            })
            .collect::<Vec<_>>();
        if let Some(result) = self.cache.get(&key) {
            return result;
        }
        let tokens = num_tokens(&msgs);
        let mut request = CreateChatCompletionRequestArgs::default();
        request
            .model(MODEL)
            .messages(msgs)
            .max_tokens(4095 - tokens)
            .temperature(0f32);
        let request = request.build().unwrap();
        let response = self
            .runtime
            .block_on(self.inner.chat().create(request))
            .unwrap();
        assert_eq!(tokens as u32, response.usage.unwrap().prompt_tokens);
        let result = response.choices[0].message.content.clone();
        self.cache.insert(key, result.clone());
        self.cache.save();
        result
    }
}

fn role_to_str(role: &Role) -> &'static str {
    match role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
    }
}

fn num_tokens(msgs: &[ChatCompletionRequestMessage]) -> u16 {
    let bpe = tiktoken_rs::cl100k_base().unwrap();
    let count = |s: &str| bpe.encode_with_special_tokens(s).len() as u16;
    let mut num_tokens = 3;
    for msg in msgs {
        let role = role_to_str(&msg.role);
        num_tokens += 4 + count(role) + count(&msg.content);
    }
    num_tokens
}

fn system(s: &str) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessage {
        role: Role::System,
        content: s.to_string(),
        name: None,
    }
}

fn user(s: &str) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessage {
        role: Role::User,
        content: s.to_string(),
        name: None,
    }
}

fn assistant(s: &str) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessage {
        role: Role::Assistant,
        content: s.to_string(),
        name: None,
    }
}
