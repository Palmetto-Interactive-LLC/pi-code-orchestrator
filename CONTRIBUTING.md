# Contributing

See [How to develop and contribute](docs/how-to/develop-and-contribute.md) for build instructions, testing, project layout, and code conventions.

Quick start:

```bash
cargo test
cargo clippy
cargo fmt
```

Documentation follows the [Diátaxis](https://diataxis.fr/) framework. When adding docs:

- **Tutorial** — learning-oriented, one guided path (`docs/tutorial/`)
- **How-to** — task-oriented, problem in the title (`docs/how-to/`)
- **Reference** — facts only, no steps (`docs/reference/`)
- **Explanation** — context and why (`docs/explanation/`)

Do not mix types in a single page. Link between them instead.
