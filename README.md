# Sigma2Splunk
Match sigma rules against splunk


# Install dependencies

Sigma:

pip install -r sigma/sigma/requirements.txt --use-wheel --no-index --find-links sigma/sigma/wheels

docker run -v `pwd`/rules:/rules sigma2splunk --splunk https://192.168.10.42:8089 --username analyst --password analyst\! --index botsv2 --threads 8 --earliest 1h /rules/sysmon

# Run

```
USAGE:
    sigma2splunk [OPTIONS] --splunk <splunk> --username <username> --password <password> <RULES>

ARGS:
    <RULES>    

OPTIONS:
    -s, --splunk <splunk>        Splunk management url (eg: https://splunk.fak:8089)
    -u, --username <username>    Splunk username
    -p, --password <password>    Splunk password
    -i, --index <index>          Splunk index to use for queries [default: main]
    -t, --threads <threads>      Number of parallel requests [default: 4]
    -e, --earliest <earliest>    Add `earliest=` to your searches [default: 1y]
    -h, --help                   Print help information
    -V, --version                Print version information
```

For example:

```
./sigma2splunk --splunk https://localhost:8089 --username analyst --password analyst\! --index botsv2 --threads 4 --earliest 4y ./rules/builtin/
```