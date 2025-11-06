# eventsource2graphite

This tool listens to an EventSource stream and sends the data to a Graphite instance.

## Building

To build the project, run the following command:

```bash
cargo build --release
```

## Installation

After building the project, you can install the binary by copying it to a directory in your `PATH`. For example:

```bash
sudo cp target/release/eventsource2graphite /usr/local/bin/
```

### From source (via cargo)

You can also install the binary directly using `cargo install`:

```bash
cargo install --locked --path .
```

This will install the binary into `~/.cargo/bin/`.

## Systemd Service

To run `eventsource2graphite` as a systemd service, you can use the provided unit file.

1.  Copy the service file to the systemd directory:

    ```bash
    sudo cp eventsource2graphite.service /etc/systemd/system/
    ```

2.  Reload the systemd daemon:

    ```bash
    sudo systemctl daemon-reload
    ```

3.  Enable and start the service:

    ```bash
    sudo systemctl enable --now eventsource2graphite.service
    ```
