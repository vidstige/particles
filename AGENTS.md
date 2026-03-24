Write succint, to the point code
Avoid unnecessary early outs and other complications
After a refactor, recursively follow up with cleanup to avoid code structure we would not create from scratch.
When a function is only used inside one file it should be placed before the function that calls it.

Committing:
Each commit should only contain a single step or concern.
Each commit should pass: `cargo test` and `cargo fmt --check`
Prefix refactoring commit messages with `refactor: `
