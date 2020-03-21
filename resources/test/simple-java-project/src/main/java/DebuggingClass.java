import java.security.AccessController;

import java.security.AccessController;
import java.security.PrivilegedAction;

public class DebuggingClass {
    public static void main(String[] args) {
        AccessController.doPrivileged(new PrivilegedAction<Object>() {
            @Override
            public Object run() {
                try {
                    DebuggingClass.class.getClassLoader().loadClass("java.lang.invoke.DirectMethodHandle$Lazy").newInstance();
                } catch (InstantiationException | IllegalAccessException | ClassNotFoundException e) {
                    e.printStackTrace();
                }
                return new Object();
            }
        });


    }
}
