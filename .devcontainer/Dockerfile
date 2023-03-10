FROM rust:1.66

WORKDIR /app

COPY . .

RUN apt-get update
# install relevant toolchains
RUN apt-get install vim -y --force-yes
RUN apt-get install git -y --force-yes


# configure git
RUN git config --global user.name "Jacky Chen"
RUN git config --global user.email "330213905@qq.com"

