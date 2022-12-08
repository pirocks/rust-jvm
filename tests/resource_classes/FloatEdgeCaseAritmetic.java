public class FloatEdgeCaseAritmetic {
    public static void main(String[] args) {
        for (int i = 0; i < 1_000; i++) {
            loop();
        }
    }

    static strictfp void loop() {
        assert_(Float.MAX_VALUE - Float.MAX_VALUE == 0.0);
        assert_(Float.MIN_VALUE/2 == 0.0);
        assert_(Float.MIN_VALUE/2.0 != 0.0);
        assert_(Float.MIN_VALUE/2.0f == 0.0);
        assert_(0 == Double.MAX_VALUE - Double.MAX_VALUE);
        assert_(Double.MIN_VALUE/2 == 0.0);
        assert_(Double.MIN_VALUE/2.0 == 0.0);
        assert_(Double.MIN_VALUE/2.0f == 0.0);
        assert_((double)1.0f == 1.0);
        assert_((double)Float.NaN != Double.NaN);
        assert_(Double.isNaN((double)Float.NaN));
    }

    static void assert_(boolean success){
        if(!success){
            throw new AssertionError();
        }
    }
}
