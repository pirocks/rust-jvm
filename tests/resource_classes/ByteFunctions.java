public class ByteFunctions {
    public static void main(String[] args) {
        new ByteFunctions().compile_loop();
    }

    void compile_loop() {
        for (int i = 0; i < 1_000; i++) {
            loop();
        }
    }

    void loop() {
        for (int i = -512; i < 512; i++) {
            boolean should_be_equal = i <= Byte.MAX_VALUE && i >= Byte.MIN_VALUE;
            takes_byte(i, (byte) i, should_be_equal);
        }
    }

    void takes_byte(int a, byte b, boolean should_be_equal) {
        if (a == b && a*2 == b*2 && a-2 == b-2 && !should_be_equal) {
            throw new AssertionError();
        }
        if (a != b && a*2 != b*2 && a-2 != b-2 && should_be_equal) {
            throw new AssertionError();
        }
    }
}