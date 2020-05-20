import java.security.SecureRandom;

public class SecureRandomDemo {
    public static void main(String[] args) {
        final SecureRandom sr = new SecureRandom();
        byte[] bytes = new byte[1];
        sr.nextBytes(bytes);
        final int res = bytes[0];
        System.out.println("A random number:" + res);
    }
}

