FROM ubuntu:20.04

ENV TZ=Etc/UTC \
    APP_USER=appuser \
    APP=/usr/src/app

COPY tests/deployment/static/* ${APP}/static/
COPY target/release/deployment-tests ${APP}/

RUN addgroup $APP_USER && useradd -g $APP_USER $APP_USER && \
    chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER

WORKDIR ${APP}

EXPOSE 80

CMD ["./deployment-tests"]
