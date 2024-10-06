# We use linux/amd64 because we don't actually care which arch yet.
FROM --platform=linux/amd64 gcr.io/distroless/static-debian12:nonroot@sha256:69830f29ed7545c762777507426a412f97dad3d8d32bae3e74ad3fb6160917ea AS distroless
COPY artifacts/identity-server-linux-aarch64 /artifacts/linux/arm64/identity-server
COPY artifacts/identity-server-linux-x86_64 /artifacts/linux/amd64/identity-server
COPY artifacts/identity-server-windows-x86_64.exe /artifacts/windows/amd64/identity-server

FROM scratch
ARG TARGETPLATFORM

COPY --from=distroless /etc/passwd /etc/passwd
COPY --from=distroless /etc/group /etc/group
USER nonroot
ENV USER=nonroot

ENV XDG_CACHE_HOME=/home/nonroot/.cache
VOLUME ["/home/nonroot/.cache"]

# Bring in the actual binary we will run
COPY --from=distroless --chmod=544 --chown=nonroot:nonroot /artifacts/$TARGETPLATFORM/identity-server /opt/identity-server
ENTRYPOINT ["/opt/identity-server"]
VOLUME ["/var/db"]
WORKDIR ["/var/db"]

EXPOSE 443/tcp
EXPOSE 80/tcp

