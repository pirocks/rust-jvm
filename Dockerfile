FROM rust
RUN rustup toolchain install nightly-2023-10-24-x86_64-unknown-linux-gnu
RUN rustup default nightly-2023-10-24-unknown-linux-gnu
RUN git clone https://github.com/AdoptOpenJDK/openjdk-jdk8u.git --depth 1
RUN wget https://github.com/AdoptOpenJDK/openjdk8-binaries/releases/download/jdk8u252-b09/OpenJDK8U-jdk_x64_linux_hotspot_8u252b09.tar.gz
RUN tar xf OpenJDK8U-jdk_x64_linux_hotspot_8u252b09.tar.gz
RUN apt-get update
RUN apt-get install -y cmake pkg-config libssl-dev git clang
RUN mkdir rust-jvm
ADD . ./rust-jvm
ENV JVM_H openjdk-jdk8u/jdk/src/share/javavm/export/
ENV JNI_H openjdk-jdk8u/jdk/src/share/javavm/export/
ENV JVM_MD_H rust-jvm/jvmti-jni-bindings/
ENV JNI_MD_H openjdk-jdk8u/jdk/src/solaris/javavm/export/
WORKDIR rust-jvm
RUN cargo build
ENV LD_LIBRARY_PATH /rust-jvm/target/debug/deps/
ENTRYPOINT ["target/debug/java"]