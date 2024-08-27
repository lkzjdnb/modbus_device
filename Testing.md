# Testing
## Running tests
> :warning: **Make sure to setup the testing environment [Testing environment](#Testing-Environment)**

Run the tests with cargo : 
```bash
cargo test
```

## Testing Environment
For integration testing the project needs access to an test server docker image. This program can be found at https://github.com/lkzjdnb/modbus_test_server. And needs to be tagged as `modbus-test-server:version`.

Clone the server : 
```bash
git clone https://github.com/lkzjdnb/modbus_test_server.git
cd modbus_test_server
```

Build the image : 
```bash
docker build -t modbus-test-server:1 .
```
