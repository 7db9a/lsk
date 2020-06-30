# ls-key

[![asciicast](https://asciinema.org/a/L9jaxLPtp2AFs4AOSU3KJ7Tss.png)](https://asciinema.org/a/L9jaxLPtp2AFs4AOSU3KJ7Tss)

## Development

### Testing

At the moment, some tests must be ran on host while others in docker. ls-key's tests simulates keyboard input and I can't figure out how to do that in docker.

**Run tests for host or docker**

Stage code changes, if any. It's very important that you do this.

`git add -u`

Run the following scripts, which uncomment #[ignore] for either host or docker tests.

`./unignore_host_tests` or `./unignore_host_tests`

Run tests on host.

`cargo test -- --test-threads=1 --nocapture`

Run tests on docker, using dev script.

`./dev.sh test rust-lib`

***Undo any unstaged changes (those are the 'unignore' script)***

`git restore .`

#### Special cases

For an unknown reason, these test only run if 'asked' to explicitly.

`./dev.sh test rust-lib list`

One or more tests rely on exact terminal size.

`./unignore_host_term_size_dependent`
