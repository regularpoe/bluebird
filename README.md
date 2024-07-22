# bluebird

Compare your local CI file variables with ones defined in your GitLab project.

## Prerequsites

- clone repo
- build it with `release`, `cargo build --release`
- move it to `$path` or run it from directory

## Usage

- define `.env` file like:

```env
url=<your gitlab URL>
project_id=<project id>
token=<token>
```

- run the checker

```bash
bluebird -f .gitlab-ci.yml
```
