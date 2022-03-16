FROM python:3.8.12-buster

WORKDIR /opt/Sigma2Splunk

COPY . .

ENV PATH="/root/.cargo/bin:${PATH}"

RUN apt-get update -y &&\
    apt-get upgrade -y &&\
    apt-get install -y curl &&\
    pip install -r sigma/sigma/requirements.txt &&\
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y &&\
    cargo build --release

ENTRYPOINT ["/opt/Sigma2Splunk/target/release/sigma2splunk"]
