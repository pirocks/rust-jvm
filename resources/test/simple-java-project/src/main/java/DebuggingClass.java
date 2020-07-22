import com.sun.jdi.*;
import com.sun.jdi.connect.IllegalConnectorArgumentsException;
import com.sun.tools.jdi.GenericAttachingConnector;
import com.sun.tools.jdi.SocketAttachingConnector;

import java.io.IOException;
import java.util.List;

public class DebuggingClass {
    public static void main(String[] args) throws IOException, IllegalConnectorArgumentsException {
        final GenericAttachingConnector connector = new SocketAttachingConnector();
        final VirtualMachine attached = connector.attach("localhost:5005", connector.defaultArguments());
        for (ReferenceType aClass : attached.allClasses()) {
           /* if (!aClass.name().equals("int") &&
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
            }*/
            System.out.println(aClass.name());
        }
//        attached.setDebugTraceMode(VirtualMachine.TRACE_ALL);
        attached.suspend();
        final List<ThreadReference> threads = attached.allThreads();
//        for (ThreadReference thread : threads) {
//            if(thread.name().equals("Main")) {
////                thread.interrupt();
//                thread.interrupt();
//                thread.suspend();
//                System.out.println(thread.name());
//                System.out.println(thread.status());
//                System.out.println("Suspended:" + thread.isSuspended());
//                try {
//                    System.out.println(thread.isSuspended());
//                    for (StackFrame frame : thread.frames()) {
//                        try {
//                            for (LocalVariable variable : frame.visibleVariables()) {
//                                System.out.println(variable.name());
//                                System.out.println(frame.getValue(variable));
//                                System.out.println(variable.isArgument());
//                                System.out.println(variable.genericSignature());
//
//                            }
//                        } catch (AbsentInformationException e) {
//                            e.printStackTrace();
//                        }
//                        System.out.println(frame.thisObject());
//                        System.out.println(frame.thread());
//                        try {
//                            System.out.println(frame.location().lineNumber());
//                        } catch (InternalError e) {
//                            e.printStackTrace();
//                        }
//                    }
//                    System.out.println(thread.frameCount());
//                } catch (IncompatibleThreadStateException e) {
//                    e.printStackTrace();
//                }
//
//                System.out.println(thread.name());
//            }
//        }


    }
}
