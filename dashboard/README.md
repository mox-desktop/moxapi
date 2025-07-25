# MoxAPI Server Configuration

## Config File Structure

The `hosts.yaml` file should look like this:

```yaml
- hostname: "t851"
  ip: "http://t851:8000"
  api_key_file: "/run/secrets/t851_api_key"
- hostname: "laptop"
  ip: "http://laptop:8000"
  api_key_file: "/run/secrets/laptop_api_key"
- hostname: "laptop-huawei"
  ip: "http://laptop-huawei:8000"
  api_key_file: "/run/secrets/laptop_huawei_api_key"
```

Each `api_key_file` should be a file containing only the API key for that host.

## Config Path Resolution Order

The server will look for the config file in this order:

1. **CLI argument**: `./moxapi-server /path/to/hosts.yaml`
2. **Environment variable**: `MOXAPI_CONFIG=/path/to/hosts.yaml ./moxapi-server`
3. **System path**: `/etc/moxapi/hosts.yaml`
4. **User config**: `~/.config/mox/moxapi/hosts.yaml`

## Docker Usage

To use Docker secrets or bind mounts for config and API keys:

- Mount your config file and secrets into the container:

```sh
docker run \
  -v /path/to/hosts.yaml:/etc/moxapi/hosts.yaml:ro \
  -v /path/to/t851_api_key:/run/secrets/t851_api_key:ro \
  -v /path/to/laptop_api_key:/run/secrets/laptop_api_key:ro \
  -v /path/to/laptop_huawei_api_key:/run/secrets/laptop_huawei_api_key:ro \
  moxapi-server
```

Or use Docker secrets with Swarm/Kubernetes and mount them at the paths referenced in your config. 

## Licensing

This project is dual-licensed:

- **Individuals:** Free for personal, non-commercial use under the GNU AGPL v3.0 (or later). See LICENSE-AGPL for details.
- **Companies/Organizations:** Commercial use requires a paid license. Please contact the author for commercial licensing options.

For questions or to obtain a commercial license, email: <your-email@example.com> 
