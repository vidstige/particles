Write succint, to the point code
Avoid unnecessary early outs and other complications
After a refactor, recursively follow up with cleanup to avoid code structure we would not create from scratch.

Committing:
Each commit should only contain a single step or concern.
Each commit should pass: `cargo test`
Prefix refactoring commit messages with `refactor: `
