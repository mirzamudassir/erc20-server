 ### Setup:
- setup a local postgres database with an account that has superuser permissions.
- add `.envrc` with `export DATABASE_URL=postgres://user:password@localhost/comn
- `cargo build`
- `target/debug/cargo-broker`
- By default, the broker will start at `localhost:5800`
- run `curl 'http://localhost:5800/key?addr=1'` to validate that it's up and running. It should return an empty json array

### Test:

- Ensure the steps from Setup are complete and `cargo build` is working.
- Run `cargo test -- --show-output --test-threads=1`
- `--show-output` helps when debugging a failing test or adding new.
- a specific test can be passed as parameter to reduce run time `cargo test <test_name> -- --show-output --test-threads=1` i.e. Run `cargo test get_crate_item -- --show-output --test-threads=1`
- `--test-threads=1` disables multithreading and is neccessary since we only have one backend database and connection pool.
