import sun.reflect.generics.factory.CoreReflectionFactory;
import sun.reflect.generics.repository.ClassRepository;
import sun.reflect.generics.scope.ClassScope;

import java.lang.reflect.Method;
import java.lang.reflect.Type;
import java.util.*;

public class IntrospectionTests {
    public static void main(String[] args) throws NoSuchMethodException {
        final Class<AbstractMap.SimpleEntry> simpleEntryClass = AbstractMap.SimpleEntry.class;
        final Method setValue = simpleEntryClass.getMethod("setValue", Object.class);
        if(!Arrays.toString(setValue.getGenericParameterTypes()).equals("[V]")){
            throw new AssertionError("1");
        }
        if(!setValue.getGenericReturnType().toString().equals("V")){
            throw new AssertionError("2");
        }
        if(!Arrays.toString(AbstractMap.class.getGenericInterfaces()).equals("[java.util.Map<K, V>]")){
            throw new AssertionError("3");
        }
        if(!Arrays.toString(HashMap.class.getGenericInterfaces()).equals("[java.util.Map<K, V>, interface java.lang.Cloneable, interface java.io.Serializable]")){
            throw new AssertionError("4");
        }
        if(!Arrays.toString(HashMap.class.getDeclaredClasses()).equals("[class java.util.HashMap$TreeNode, class java.util.HashMap$EntrySpliterator, class java.util.HashMap$ValueSpliterator, class java.util.HashMap$KeySpliterator, class java.util.HashMap$HashMapSpliterator, class java.util.HashMap$EntryIterator, class java.util.HashMap$ValueIterator, class java.util.HashMap$KeyIterator, class java.util.HashMap$HashIterator, class java.util.HashMap$EntrySet, class java.util.HashMap$Values, class java.util.HashMap$KeySet, class java.util.HashMap$Node]")){
            throw new AssertionError("5");
        }
        if(!Arrays.toString(AbstractMap.SimpleEntry.class.getDeclaredFields()).equals("[private static final long java.util.AbstractMap$SimpleEntry.serialVersionUID, private final java.lang.Object java.util.AbstractMap$SimpleEntry.key, private java.lang.Object java.util.AbstractMap$SimpleEntry.value]")){
            throw new AssertionError("6");
        }
        if(!AbstractMap.SimpleEntry.class.getDeclaredFields()[2].toGenericString().equals("private V java.util.AbstractMap$SimpleEntry.value")){
            throw new AssertionError("7");
        }

        for (Class<?> declaredClass : HashMap.class.getDeclaredClasses()) {
            if(declaredClass.getName().equals("java.util.HashMap$Values")){
                final ClassRepository classRepository = ClassRepository.make("Ljava/util/AbstractCollection<TV;>;", CoreReflectionFactory.make(declaredClass, ClassScope.make(declaredClass)));
                System.out.println(classRepository);
                System.out.println(Arrays.toString(classRepository.getSuperInterfaces()));
                System.out.println(classRepository.getSuperclass());
                System.out.println(declaredClass);
                final Type genericSuperclass = declaredClass.getGenericSuperclass();
                System.out.println(genericSuperclass.getClass());
                if(!genericSuperclass.toString().equals("java.util.AbstractCollection<V>")){
                    throw new AssertionError("8");
                }
            }
        }
        if(!HashMap.class.getGenericSuperclass().toString().equals("java.util.AbstractMap<K, V>")){
            throw new AssertionError("9");
        }
    }
}
