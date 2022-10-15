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
        final byte[] bytes = new byte[10];
        bytes[5] = (byte) -120;
        bytes[6] = (byte) -250;
        final byte zero = bytes[0];
        final byte five = bytes[5];
        final byte six = bytes[6];
        if(zero != 0 || five != -120 || six != 6){
            throw new AssertionError();
        }

        final boolean[] bools = new boolean[10];
        bools[0] = true;
        if(!bools[0] || bools[1]){
            throw new AssertionError();
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