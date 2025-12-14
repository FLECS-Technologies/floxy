# ==============================================================================
# entrypoint: As we go distroless, we no longer have a shell to execute custom
#   entrypoints. Add a small Rust helper that takes care of container
#   initialization and starts our actual binary.
# ==============================================================================
FROM --platform=${BUILDPLATFORM} rust:alpine AS entrypoint

ARG BUILD_TYPE=debug
ARG TARGETPLATFORM

ARG GIT_SHA
ENV GIT_SHA=$GIT_SHA

COPY entrypoint/ /src/entrypoint
COPY docker/scripts/ /tmp/scripts

WORKDIR /src/entrypoint

RUN /tmp/scripts/build.sh

# ==============================================================================
# staging: Merges our custom entrypoint with the nginx base image to build an
#   intermediate image for validation.
# ==============================================================================
FROM alpine:3 AS staging-release

# rootfs
COPY --from=flecspublic.azurecr.io/docker/nginx:alpine-openssl / /staging/
COPY docker/fs/ /staging/

# executables
COPY --from=entrypoint /entrypoint /staging/entrypoint

# remove default configuration
RUN rm /staging/etc/nginx/conf.d/default.conf
# ==============================================================================
FROM alpine:3 AS staging-debug

# rootfs
COPY --from=flecspublic.azurecr.io/docker/nginx:alpine-openssl-debug / /staging/
COPY docker/fs/ /staging/

# executables
COPY --from=entrypoint /entrypoint /staging/entrypoint

# remove default configuration
RUN rm /staging/etc/nginx/conf.d/default.conf

# ==============================================================================
# build-validation: Performs some basic sanity checks before finalizing the
#   image. For example, we validate our binary was compiled for the correct
#   architecture and stripped in accordance with the build type.
# ==============================================================================
FROM alpine:3 AS validation-release

ARG TARGETPLATFORM
ENV BUILD_TYPE=release

COPY --from=staging-release /staging /final

RUN apk add --no-cache \
    file

COPY docker/scripts/ /tmp/scripts

RUN /tmp/scripts/validate.sh
# ==============================================================================
FROM alpine:3 AS validation-debug

ARG TARGETPLATFORM
ENV BUILD_TYPE=debug

COPY --from=staging-debug /staging /final

RUN apk add --no-cache \
    file

COPY docker/scripts/ /tmp/scripts

RUN /tmp/scripts/validate.sh

# ==============================================================================
# final: Copies the rootfs from nginx-validation and finalizes the build by
#   setting our entrypoint.
# ==============================================================================
FROM scratch AS release

ENV PATH=/bin:/usr/sbin:/usr/bin

COPY --from=validation-release /final /

ENTRYPOINT ["/entrypoint"]
# ==============================================================================
FROM scratch AS debug

ENV PATH=/bin:/usr/sbin:/usr/bin

COPY --from=validation-debug /final /

ENTRYPOINT ["/entrypoint"]
