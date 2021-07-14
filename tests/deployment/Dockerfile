FROM rust:latest as Builder
WORKDIR /usr/src/dummyapp
COPY . .
RUN cargo build --release
RUN rm src/*.rs

FROM ubuntu
ARG APP=/usr/src/app
ENV TZ=Etc/UTC \
    APP_USER=appuser
RUN addgroup $APP_USER 
RUN useradd -g $APP_USER $APP_USER
COPY --from=Builder /usr/src/dummyapp/target/release/hello-legionlabs ${APP}/
COPY --from=Builder /usr/src/dummyapp/static/* ${APP}/static/
RUN chown -R $APP_USER:$APP_USER ${APP}
USER $APP_USER
WORKDIR ${APP}
EXPOSE 80
CMD ["./hello-legionlabs"]