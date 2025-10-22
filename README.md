# authelia-webfinger

Simple tool for exposing static Authelia users for use with the WebFinger protocol.  Useful for DIY Tailscale setups
that utilize your own domain for authentication instead of Google or some other OIDC provider.

## Usage

After building release, create a new systemd unit file like the following:

```toml
[Unit]
Description=Authelia Webfinger provider
Requires=network-online.target
After=network-online.target

[Service]
Restart=on-failure

ExecStart=/usr/local/bin/authelia_webfinger --conf /opt/authelia/data/authelia/config/users_database.yml --auth-url https://authelia.host

[Install]
WantedBy=multi-user.target
```

Any user that is in the `users_database.yml` file that has an `email` attribute will be available by making a WebFinger
request to http://localhost/webfinger.  You can use something like nginx to proxy via the standard url:

```
server {
    server_name authelia.host;
    listen 80;

    location /.well-known/ {
        proxy_set_header HOST $http_host;
        proxy_pass http://localhost:8081/;
    }
}
```
