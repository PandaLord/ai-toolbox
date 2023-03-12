FROM rust:slim-buster

WORKDIR /app

COPY . .

## configure proxy settings
# test for now
# ENV HTTP_PROXY "http://127.0.0.1:10811"
# ENV HTTPS_PROXY "http://127.0.0.1:10811"

# install relevant toolchains
RUN apt-get update && apt-get upgrade
RUN apt install build-essential
RUN apt install pkg-config -y --force-yes
RUN apt-get install vim -y --force-yes
RUN apt-get install git -y --force-yes


# configure git
RUN git config --global user.name "Jacky Chen"
RUN git config --global user.email "330213905@qq.com"
