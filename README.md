# transformrs

transformrs is an interface for AI API providers.

For examples, see [`examples/`](https://github.com/transformrs/transformrs/tree/main/examples).

Provider | Chat* | Text to Image | Text to Speech
--: | --- | --- | ---
Cerebras | x |
ElevenLabs | | | x
DeepInfra | x | x | x
Google | x |  | [x](#google-cloud-api)
Groq | x |
Hyperbolic | x | x | x
OpenAI | x | | [x](https://platform.openai.com/docs/guides/text-to-speech)
Other** | x
SambaNova | x |
TogetherAI | x |


\*Chat supports streaming and image input.

\*\*Other OpenAI-compatible providers can be used via `Provider::Other(base_url)`.

## Users 

Projects that use this library:

- [trf](https://github.com/transformrs/trf) - A command line tool to interact with AI providers.
- [trv](https://github.com/transformrs/trv) - A command line tool to create videos from slides and text.

## Why was this Project Created?

I was looking into making a command line tool that could summarize PDF files.
Then I noticed that I probably needed to use a cloud provider.
However, then I would be requiring myself and users to use the same cloud provider.
This library is avoids that.
It provides the interface to multiple cloud providers, so that users can pick their favourite.

## Cloud versus Local

I think running AI locally is a nice idea that is unlikely to take off.
Most people will want to use the best AI models that are available.
However, this is unreasonably expensive and slow.
In most cases, running AI in the cloud will be orders of magnitude cheaper and faster.
For example, running DeepSeek R1 requires a $2000 server while the server still only does around 2-3 tokens per second.
This means that responses will take multiple minutes to complete.
Conversely, running the same model in the cloud will cost only a few cents per million tokens and is much faster.
Assuming you would use about 10 000 tokens per day, the cost would still only be around $4 per year.
Prices are also falling with around [80% per year](https://huijzer.xyz/posts/ai-learning-rate/).
So if you take into account the cost of the server and the cost of having to wait for the response, the cloud is several orders of magnitude cheaper.

That's why I expect that most people will run AI in the cloud.
Luckily, there are nowadays many providers.
This library aims to make it easy to easily built on top of these providers.

## Rust

Since we're building on top of HTTP via cloud providers, we do not necessarily need Python for running AI.
We can use Rust which in my opinion is better suited for this task.
Rust code usually has fewer bugs, produces smaller binaries, is easier to distribute, has better WebAssembly support, and is faster.

## Core Utilities

What I hope is that we will see many more "core utilities" like `cat`, `ls`, and `grep` built on top of AI.
As pointed out above, it is unlikely that these utilities will run the models locally.
Instead, it's more likely that they will be built on top of the cloud providers.
One example of this is [llm](https://github.com/simonw/llm) by Simon Willison.
Examples I'd looking forward to are PDF summarizers, PDF to text, text to speech, and more.

## Why the name transformrs?

Essentially AI is about transforming data, so I called this library `transformrs` as "Transformations in Rust".
It's also a play on the word "transformers" which is an important algorithm in AI.

## Google Cloud API

This is a difficult one to get the key working.
On the Cloud Text-to-Speech API page, click to enable the API.
In the API key page (API & Services -> Credentials), ensure that the key has access (is restricted to) the Cloud Text-to-Speech API (for text to speech) and the Generative Language API (for chat completions).

This is using the "old" Cloud Text-to-Speech API instead of the "new" Gemini 2.0 API.
Gemini 2.0 can do higher quality text to speech, but it currently seems not very stable yet.
It for example doesn't follow the prompt very closely.
It may for example say "This is the text you asked for" instead of just reading the text.

The API is documented at [cloud.google.com](https://cloud.google.com/text-to-speech/docs/reference/rest/v1beta1/text/synthesize).
