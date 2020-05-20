#The beginnings of a JVM implementation in Rust

Title says it all pretty much. 

###What can it do? 
 - Initialize a VM(with properties and streams correctly initialized)
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
 

### What can it partially do?
 - JNI Interface
 - JVMTI Interface
 - java.lang.Unsafe implementation 
 - Access Control with `AccessController.doPrivileged`
 - Threads

### What can't it do (yet)? 
- JIT 
- Garbage Collection with finalizers
- Network/Sockets and similar complex IO
- Execute `invokedynamic` instructions
- Expose openjdk compatible command line arguments
- Pass arguments to the Java program in question
- ThreadGroups(except the main thread group)