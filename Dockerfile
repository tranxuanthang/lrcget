FROM ghcr.io/linuxserver/baseimage-selkies:ubuntunoble

ENV TITLE=LRCGET \
        NO_GAMEPAD=true

RUN apt-get update
RUN curl -L https://github.com/tranxuanthang/lrcget/releases/latest/download/LRCGET_0.9.3_amd64.deb > package.deb && \
        apt-get install -y ./package.deb && rm package.deb && \
        apt-get clean && \
        rm -rf /var/lib/apt/lists/*

# Add LRCGET to start on startup
RUN echo "/usr/bin/LRCGET" > defaults/autostart

EXPOSE 3001

volume /CONFIG
