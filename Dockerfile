FROM rust
RUN rustup toolchain install nightly-2020-05-15-x86_64-unknown-linux-gnu
RUN rustup default nightly-2020-05-15-x86_64-unknown-linux-gnu
RUN git clone https://github.com/AdoptOpenJDK/openjdk-jdk8u.git --depth 1
RUN wget https://github.com/AdoptOpenJDK/openjdk8-binaries/releases/download/jdk8u252-b09/OpenJDK8U-jdk_x64_linux_hotspot_8u252b09.tar.gz
RUN tar xf OpenJDK8U-jdk_x64_linux_hotspot_8u252b09.tar.gz
RUN apt-get update
RUN apt-get install -y cmake pkg-config libssl-dev git clang
RUN mkdir rust-jvm
ADD . ./rust-jvm
ENV JVM_H openjdk-jdk8u/jdk/src/share/javavm/export/
ENV JNI_H jdk8u252-b09/include/
ENV JVM_MD_H rust-jvm/jvmti-jni-bindings/
ENV JNI_MD_H jdk8u252-b09/include/linux/
WORKDIR rust-jvm
RUN cargo build
ENV LD_LIBRARY_PATH /rust-jvm/target/debug/deps/
ENTRYPOINT ["target/debug/java"]
# run with(assuming you built with `docker build -t test`):
# docker run test --main SecureRandomDemo --libjava /jdk8u252-b09/jre/lib/amd64/libjava.so --args args not implemented yet so... --classpath resources/test /jdk8u252-b09/jre/lib/ /jdk8u252-b09/jre/lib/ext/
# See resources/test for more Demo classes. You only need to change "--main SecureRandomDemo" to run them.