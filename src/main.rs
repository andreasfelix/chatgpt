use {
    clap::Parser,
    color_eyre::eyre::{self, Context, ContextCompat},
    dialoguer::{theme::ColorfulTheme, BasicHistory, Input, Password},
    directories::ProjectDirs,
    figment::{
        providers::{Env, Format, Toml},
        Figment,
    },
    indicatif::ProgressBar,
    reqwest::{blocking::Client, StatusCode},
    serde::{Deserialize, Serialize},
    std::{fs, path::Path},
};

// TODOS
// * implement streaming
// * exit with ctrl + d
// * clear with ctrl + l
// * persistent history
// * better ui

#[derive(Debug, Deserialize)]
struct Config {
    openai_api_key: Option<String>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Initial prompt
    prompt: Option<String>,

    /// Set new openai api key
    #[arg(short, long)]
    set_openai_api_key: bool,

    /// Delete config file
    #[arg(short, long)]
    delete_config: bool,
}

fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let project_dir = ProjectDirs::from("com.github", "andreasfelix", "chatgpt")
        .context("failed to get project dir")?;

    let config_dir = project_dir.config_dir();
    fs::create_dir_all(config_dir).context("failed to create config dir")?;

    let config_path = project_dir.config_dir().join("config.toml");

    if args.set_openai_api_key {
        ask_for_openai_api_key(&config_path)?;
        return Ok(());
    }

    if args.delete_config {
        fs::remove_file(&config_path).context("failed to delete config file")?;
        println!("info: deleted config {:?}", &config_path);
        return Ok(());
    }

    let config: Config = Figment::new()
        // todo: handle invalid config
        .merge(Toml::file(&config_path))
        .merge(Env::prefixed("CHATGPT_"))
        .extract()?;

    let openai_key = match config.openai_api_key {
        Some(openai_key) => openai_key,
        None => ask_for_openai_api_key(&config_path)?,
    };
    let mut history = BasicHistory::new().max_entries(8).no_duplicates(true);

    let mut messages: Vec<GptMessage> = vec![GptMessage {
        role: GptRole::User,
        content: if let Some(prompt) = args.prompt {
            prompt
        } else {
            get_user_input(&mut history)
        },
    }];

    // TODO: remove code duplication below
    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(std::time::Duration::from_millis(120));
    spinner.set_message("computing ...");
    let message = generate_text(&messages, &openai_key)?;
    spinner.finish_and_clear();
    println!(
        "{} · {}",
        console::style("🤖chatgpt").bold(),
        &message.content
    );
    messages.push(message);

    loop {
        messages.push(GptMessage {
            role: GptRole::User,
            content: get_user_input(&mut history),
        });
        let spinner = ProgressBar::new_spinner();
        spinner.enable_steady_tick(std::time::Duration::from_millis(120));
        spinner.set_message("computing ...");
        let message = generate_text(&messages, &openai_key)?;
        spinner.finish_and_clear();
        println!(
            "{} · {}",
            console::style("🤖chatgpt").bold(),
            &message.content
        );
        messages.push(message);
    }
}

fn get_user_input(history: &mut BasicHistory) -> String {
    Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("user")
        .history_with(history)
        .interact_text()
        .unwrap()
}

fn ask_for_openai_api_key(config_path: &Path) -> eyre::Result<String> {
    let openai_key = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("enter your openai api key")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.starts_with("sk-") && input.chars().count() == 51 {
                Ok(())
            } else {
                Err("the openai api key should start with 'sk-' and should be 51 characters")
            }
        })
        .interact()
        .unwrap();

    fs::write(&config_path, format!(r#"openai_api_key = "{openai_key}""#))
        .context("failed to store openai api key in config file")?;
    println!("info: stored openai api key in {config_path:?}");
    Ok(openai_key)
}

/*
 * OPENAI
 */

fn generate_text(messages: &[GptMessage], openai_api_key: &str) -> eyre::Result<GptMessage> {
    let response = Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(openai_api_key)
        .json(&serde_json::json!({
            "messages": messages,
            "model": "gpt-3.5-turbo-0613".to_string(), // there is also gpt-4-0613
        }))
        .send()?;

    let response = match response.status() {
        StatusCode::OK => response.json::<GptResponse>()?,
        StatusCode::UNAUTHORIZED => todo!("handle wrong key"),
        _ => eyre::bail!(
            "openai request failed. response:\n{:?}",
            response.json::<serde_json::Value>()
        ),
    };

    // todo: check if we only get one choice
    Ok(response
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message)
        .context("openai returned empty or more than one response")?)
}

#[derive(Deserialize)]
struct GptResponse {
    choices: Vec<GptChoice>,
}

#[derive(Deserialize)]
struct GptChoice {
    message: GptMessage,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GptRole {
    User,
    Assistant,
    System,
}

#[derive(Deserialize, Serialize)]
pub struct GptMessage {
    pub role: GptRole,
    pub content: String,
}
