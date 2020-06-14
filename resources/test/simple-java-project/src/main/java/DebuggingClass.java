import com.sun.jdi.*;
import com.sun.jdi.connect.IllegalConnectorArgumentsException;
import com.sun.tools.jdi.GenericAttachingConnector;
import com.sun.tools.jdi.SocketAttachingConnector;

import java.io.IOException;
import java.util.List;

public class DebuggingClass {
    public static void main(String[] args) throws IOException, IllegalConnectorArgumentsException, IncompatibleThreadStateException {
        final GenericAttachingConnector connector = new SocketAttachingConnector();
        final VirtualMachine attached = connector.attach("localhost:5005", connector.defaultArguments());
        final List<ThreadReference> threads = attached.allThreads();
        for (ReferenceType aClass : attached.allClasses()) {
            if (!aClass.name().equals("int") &&
                    !aClass.name().equals("boolean") &&
                    !aClass.name().equals("byte") &&
                    !aClass.name().equals("short") &&
                    !aClass.name().equals("float") &&
                    !aClass.name().equals("double") &&
                    !aClass.name().equals("long") &&
                    !aClass.name().equals("char")) {
                for (Method method : aClass.allMethods()) {
                    System.out.println(method.name());
                }
            }
            System.out.println(aClass.name());
        }
        for (ThreadReference thread : threads) {
            thread.interrupt();
            for (StackFrame frame : thread.frames()) {
                System.out.println(frame.location().lineNumber());
            }
            System.out.println(thread.name());
        }


    }
}
