import java.lang.reflect.Modifier;

public class DebuggingClass {
    public static void main(String[] args) {
        final Class<Integer> integerClass = int.class;
        System.out.println(Modifier.isAbstract(integerClass.getModifiers()));
        System.out.println(integerClass.getModifiers());


    }
}
