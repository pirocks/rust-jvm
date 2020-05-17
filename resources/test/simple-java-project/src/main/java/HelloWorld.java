public class HelloWorld {
    public static void main(String[] args) {
        System.out.println(System.getProperty("java.vm.name"));
        System.out.println(System.getProperty("java.home"));
        System.out.println(System.getProperty("java.vendor"));
        System.out.println(System.getProperty("java.version"));
        System.out.println(System.getProperty("java.specification.vendor"));
        System.out.println(System.getProperty("java.vm.info"));
        System.out.println(System.getProperty("java.class.path"));
        System.out.println(System.getProperty("sun.boot.class.path"));
        System.out.println("I need a more creative hello world string.");
    }
}
