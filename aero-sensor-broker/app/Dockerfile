FROM registry.suse.com/bci/bci-micro:15.6 as micro
FROM registry.suse.com/bci/rust:1.79 as builder

# Install build dependencies
RUN zypper --non-interactive --gpg-auto-import-keys install --no-recommends libopenssl-3-devel libgudev-1_0-devel

# Install runtime dependencies in the custom root
COPY --from=micro / /chroot/
COPY scratch-zypp.conf /
ENV ZYPP_CONF=/scratch-zypp.conf
RUN zypper --non-interactive --installroot /chroot -n --gpg-auto-import-keys install --no-recommends libopenssl-3-devel libgudev-1_0-devel && \
    zypper --non-interactive --installroot /chroot clean -a && \
    rm -rf /chroot/var/log/

RUN USER=root cargo new --bin broker
WORKDIR /broker

# Capture dependencies
COPY Cargo.toml Cargo.lock /broker

WORKDIR /broker

COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release && \
    rm ./src/*.rs && \
    DEP_PATH=`echo "./target/release/deps/aero-sensor-broker*" | tr - _` && \
    rm $DEP_PATH
COPY ./src ./src


# Compile the application
RUN cargo build --release

FROM micro

# Final stage: Use the micro base for the actual runtime
WORKDIR /

# Copy the prepared chroot environment from the runtime stage
COPY --from=builder /chroot/ /

# Copy the built executable from the build stage
COPY --from=builder /broker/target/release/aero-sensor-broker /aero-sensor-broker

ENV RUST_LOG=debug
CMD ["/aero-sensor-broker"]

