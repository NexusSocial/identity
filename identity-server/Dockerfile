# We use linux/amd64 because we don't actually care which arch yet.
FROM --platform=linux/amd64 gcr.io/distroless/static-debian12:nonroot@sha256:69830f29ed7545c762777507426a412f97dad3d8d32bae3e74ad3fb6160917ea AS distroless
COPY artifacts/identity-server-linux-aarch64 /artifacts/linux/arm64/identity-server
COPY artifacts/identity-server-linux-x86_64 /artifacts/linux/amd64/identity-server
COPY artifacts/identity-server-windows-x86_64.exe /artifacts/windows/amd64/identity-server

FROM scratch
ARG TARGETPLATFORM

COPY --from=distroless /etc/passwd /etc/passwd
COPY --from=distroless /etc/group /etc/group
USER nonroot:nonroot
ENV USER=nonroot

ENV XDG_CACHE_HOME=/var/.cache
# Only here to create the .cache folder
COPY --chmod=644 --chown=nonroot:nonroot .empty /var/.cache/.empty
VOLUME ["/var"]
WORKDIR ["/var"]

ENV XDG_CONFIG_HOME=/etc/cfg
COPY --chmod=644 --chown=nonroot:nonroot ./default-config.toml /etc/cfg/config.toml
VOLUME ["/etc/cfg"]

# Bring in the actual binary we will run
COPY --from=distroless --chmod=544 --chown=nonroot:nonroot /artifacts/$TARGETPLATFORM/identity-server /opt/identity-server

EXPOSE 8443/tcp
EXPOSE 8443/udp

ENV RUST_BACKTRACE=1
ENTRYPOINT ["/opt/identity-server"]
CMD ["serve", "--config", "/etc/cfg/config.toml"]
