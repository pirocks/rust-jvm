# The beginnings of a JVM implementation in Rust

The title says it all pretty much.

## How do I run it?

This VM depends on `libjava.so` and `libnio.so`, as well as libraries depended on by those shared libraries(except
for `libjvm.so`, which is provided by the libjvm crate). These libraries provide native implementations of standard
library methods, since if you want to interact with the outside world in java you eventually need to use native code.
You can find the aforementioned shared libraries in pretty much any jdk8 distribution.

This project uses rust nightly and as of now (`c46b8d437bf152782ea17a9fdb575dc7177006eb`) compiles
with `rustc 1.45.0-nightly (a74d1862d 2020-05-14)`. To compile you will also need external header files
specifically, `jni.h`, `jvmti.h`, and `jvm.h`. These can also be found in a JDK8 distribution near you, except `jvm.h`
which needs to be obtained from the OpenJDK source.

To run useful programs a copy of `rt.jar` is required. This jar contains essential class definitions such as the
definition of `java.lang.Object`, and similar classes. If you want to run something which depends on `SecureRandom` you
will need additional jar files typically located in `jre/lib/ext` of a JDK8 distribution.

Putting it all together:

```shell script
rustup run nightly bash # get a shell setup for rust nightly
export JVM_H=/home/francis/build/openjdk-jdk8u/jdk/src/share/javavm/export/
export JNI_H=/home/francis/Desktop/jdk8u232-b09/include/
export JVM_MD_H=/home/francis/Clion/rust-jvm/jvmti-jni-bindings/ 
# Only some platforms/builds seem to define the jvm_md.h header, in which case set the above to an appropriate path.
export JNI_MD_H=/home/francis/Desktop/jdk8u232-b09/include/linux/
cargo run -- --main SecureRandomDemo  \ 
  --libjava /home/francis/build/openjdk-jdk8u/build/linux-x86_64-normal-server-release/jdk/lib/amd64/libjava.so  \
# You do not need to use a version of libjava.so from a standard openjdk build, and can instead use libjava.so from a distribution of jdk8. I do this because I want debug symbols.
  --args args for java program go here  \
  --classpath /home/francis/Clion/rust-jvm/resources/test \ #resources/test contains SecureRandomDemo.class. You can change this as needed 
   /home/francis/Desktop/jdk8u232-b09/jre/lib/ /home/francis/Desktop/jdk8u232-b09/jre/lib/ext/ # for rt.jar and ext/
# keep in mind the SecureRandomDemo can be quiet slow, both because this VM is slow, and because it may block if your system lacks entropy
```

Alternatively, you can use the provided Dockerfile, as so:

```shell script
docker build -t rust_jvm_test 
docker run rust_jvm_test --main SecureRandomDemo --libjava /jdk8u252-b09/jre/lib/amd64/libjava.so --args args for java program go here --classpath resources/test /jdk8u252-b09/jre/lib/ /jdk8u252-b09/jre/lib/ext/
```

See resources/test for more Demo classes. You only need to change "--main SecureRandomDemo" to run them.

### What can it do?

- Initialize a VM(with properties and the `System.in`/`System.out` streams correctly initialized)
- Hello World
- Verify Bytecode
- Basic IO as well as some NIO
- Some reflection and introspection
- Basic Class Loading
- Float/Double Arithmetic
- JVMTI/JNI/Class Loading tracing
- Load classes from JARs
- Monitor operations
- Configure JVM properties
- String Internment
- Secure Random
- Pass arguments to the Java program in question

### What can it partially do?

- JNI Interface
- JVMTI Interface
- sun.misc.Unsafe implementation
- Access Control with `AccessController.doPrivileged`
- Threads

### What can't it do (yet)?

- JIT
- Garbage Collection with finalizers
- Network/Sockets and similar complex IO
- Execute `invokedynamic` instructions
- Expose openjdk compatible command line arguments
- ThreadGroups(except the main thread group)
- Support for anything other than x86_64 linux
