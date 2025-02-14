# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2025-02-14

### Added

- Return error when it occurs for text to speech ([#15](https://github.com/rikhuijzer/transformrs/pull/15) and [#16](https://github.com/rikhuijzer/transformrs/pull/16)).
- Allow request body to be printed via `tracing` ([#17](https://github.com/rikhuijzer/transformrs/pull/17)).

### Fixed

- Streaming now prints word (parts) instead of only full sentences or paragraphs ([#19](https://github.com/rikhuijzer/transformrs/pull/19)).

## [0.5.0] - 2025-02-12

### Added

- Google text to speech ([#13](https://github.com/rikhuijzer/transformrs/pull/13)).
- OpenAI text to speech.

### Changed

- Internally return `Bytes` instead of `serde_json::Value` for main structs.
- Rename `raw` to `raw_value` and introduce `bytes` methods for main structs.
- Rename `openai` module to `chat`.

### Removed

- `Api` struct was unused and removed.

## [0.4.0] - 2025-02-09

### Added

- Add Hyperbolic text to speech ([#12](https://github.com/rikhuijzer/transformrs/pull/12)).
- Add `examples/`.
- Support arbitrary providers via `Provider::Other(base_url)`.
- Add models interface to request models from a provider.
- Add support for Groq.
- Implement `Display` for `Content`.

## [0.3.0] - 2025-02-08

### Added

- Support Deep Infra image generation.
- Add support for images in chat completions ([#7](https://github.com/rikhuijzer/transformrs/pull/7)).
- Add support for Google Gemini chat completions ([#5](https://github.com/rikhuijzer/transformrs/pull/5)).

### Changed

- `ChatCompletionResponse`, `ImageResponse`, and `SpeechResponse` to allow for unstructured access to the response.
- Handle chat completion errors better (and test them).

## [0.2.1] - 2025-02-06

### Removed

- Accidental `println!`.

## [0.2.0] - 2025-02-06

### Added

- Text to image ([#4](https://github.com/rikhuijzer/transformrs/pull/4)).
- Text to speech ([#3](https://github.com/rikhuijzer/transformrs/pull/3)).

### Changed

- Rename `Provider::url` to `Provider::domain`.
- Rename `read_keys` to `load_keys`.

## [0.1.0] - 2025-02-02

Initial release.

[0.5.0]: https://github.com/rikhuijzer/transformrs/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/rikhuijzer/transformrs/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/rikhuijzer/transformrs/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/rikhuijzer/transformrs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/rikhuijzer/transformrs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/rikhuijzer/transformrs/releases/tag/v0.1.0
