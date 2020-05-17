import java.lang.reflect.Method;

public class MethodIntrospectionReflectionDemo {

    static boolean barCalled = false;

    public void foo() {

    }

    public void bar() {
        barCalled = true;
    }

    public static boolean methodsContains(String name, Method[] methods) {
        boolean success = false;
        for (Method method : methods) {
            success |= method.getName().equals(name);
        }
        return success;
    }

    public static void main(String[] args) throws IllegalAccessException, InstantiationException {
        final Method[] methods = MethodIntrospectionReflectionDemo.class.getDeclaredMethods();
        MethodIntrospectionReflectionDemo.class.newInstance().bar();
        if (methodsContains("foo", methods) && methodsContains("bar", methods) && barCalled) {
            System.out.println("success");
            System.exit(0);
        }
        System.out.println("Not success");
        System.exit(-1);
    }
}
