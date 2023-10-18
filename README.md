<div align="center">

# 🤖 ChatGPT CLI

### A simple CLI for ChatGPT implemented in Rust 🦀.

</div>

## Installation

Install `chatgpt` cli with Nix:

```sh
nix profile install github:andreasfelix/chatgpt
```

Or, add it to your flake:

```nix
chatgpt = {
    url = "github:andreasfelix/chatgpt";
    inputs.nixpkgs.follows = "nixpkgs";
};
```

## Usage

If you pass an argument it will be used as initial prompt:

```sh
chatgpt "What is a lepton?"
```

**Output**

```
🤖 chatgpt · A lepton is a type of elementary particle that belongs to the family of fundamental particles in the Standard Model of particle physics. Leptons are the building blocks ...
```

## Options

```
Usage: chatgpt [OPTIONS] [PROMPT]

Arguments:
  [PROMPT]  Initial prompt

Options:
  -s, --set-openai-api-key  Set new openai api key
  -d, --delete-config       Delete config file
  -h, --help                Print help
  -V, --version             Print version
```

## License

[MIT License](https://github.com/andreasfelix/chatgpt/blob/main/LICENSE)
