# examples/

This directory contains examples of how to use the `transformrs` crate.

To run an example, use `cargo run --example <example-name>`.
For example,

```bash
$ cargo run --example chat
```

## API Keys

For the examples to work, the right API key has to be set.

To do so, either add an `.env` file in the root of the repository or set the right environment variables.
For example,

```bash
DEEPINFRA_KEY=<YOUR API KEY>
```

Names for other providers are similar.
For example, `OPENAI_KEY` for OpenAI and `GOOGLE_KEY` for Google.
When using another provider, set the `Provider` to the right one in the example code.
