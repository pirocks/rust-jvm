package io.github.pirocks;

import org.apache.bcel.classfile.ClassParser;
import org.apache.bcel.util.ClassPath;

import java.io.IOException;

public class CommonsCaller {
    public static void main(String[] args) throws IOException {
        new ClassParser(Double.class.getResourceAsStream("/Main.class"),"Main.class")
                .parse();
    }
}
